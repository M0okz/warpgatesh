use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use semver::Version;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;
use warpgatesh_runtime::launchd;
use warpgatesh_runtime::storage::{LocalStore, atomic_write};

const CACHE_SCHEMA_VERSION: u32 = 1;
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const BACKGROUND_POLL_INTERVAL: Duration = Duration::from_secs(60 * 60);
const UPDATE_EVENT: &str = "warpgatesh:update-state";
const INSTALLED_APPLICATION: &str = "/Applications/WarpgateSH.app";
const HOMEBREW_CASK_ROOTS: [&str; 2] = [
    "/opt/homebrew/Caskroom/warpgatesh",
    "/usr/local/Caskroom/warpgatesh",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdatePhase {
    Idle,
    Checking,
    Current,
    Available,
    Downloading,
    Installing,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannel {
    Direct,
    Homebrew,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub phase: UpdatePhase,
    pub channel: UpdateChannel,
    pub current_version: String,
    pub available_version: Option<String>,
    pub notes: Option<String>,
    pub checked_at_epoch_seconds: Option<u64>,
    pub progress_percent: Option<u8>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct UpdateCache {
    schema_version: u32,
    checked_at_epoch_seconds: u64,
    latest_version: String,
    notes: Option<String>,
}

#[derive(Clone)]
pub struct UpdateManager {
    state: Arc<Mutex<UpdateStatus>>,
    operation_active: Arc<AtomicBool>,
    cache_path: PathBuf,
}

struct OperationGuard(Arc<AtomicBool>);

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl UpdateManager {
    /// Create the application update manager from local, non-secret state.
    ///
    /// # Errors
    ///
    /// Returns an error when the current-user storage paths cannot be resolved.
    pub fn new() -> Result<Self, String> {
        let store = LocalStore::for_current_user().map_err(display_error)?;
        let cache_path = store.paths().application_support.join("update-check.json");
        let current_version = env!("CARGO_PKG_VERSION").to_owned();
        let channel = detect_update_channel();
        let cached = load_cache(&cache_path);
        let state = cached.as_ref().map_or_else(
            || UpdateStatus::idle(current_version.clone(), channel),
            |cache| status_from_cache(&current_version, channel, cache),
        );
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            operation_active: Arc::new(AtomicBool::new(false)),
            cache_path,
        })
    }

    #[must_use]
    pub fn snapshot(&self) -> UpdateStatus {
        lock_recover(&self.state).clone()
    }

    /// Check the signed Tauri update feed.
    ///
    /// A background check is skipped when a successful check is less than six hours old. Passing
    /// `force` is reserved for a user-initiated check and never installs anything.
    pub async fn check(&self, app: &AppHandle, force: bool) -> Result<UpdateStatus, String> {
        let snapshot = self.snapshot();
        if !force && !check_is_due(snapshot.checked_at_epoch_seconds, epoch_seconds()) {
            return Ok(snapshot);
        }
        let Some(_guard) = self.begin_operation() else {
            return Ok(self.snapshot());
        };

        self.publish(
            app,
            UpdateStatus {
                phase: UpdatePhase::Checking,
                progress_percent: None,
                message: None,
                ..self.snapshot()
            },
        );

        let checked_at = epoch_seconds();
        let result = match app.updater().map_err(display_error) {
            Ok(updater) => updater.check().await.map_err(display_error),
            Err(error) => Err(error),
        };

        match result {
            Ok(Some(update)) => {
                let cache = UpdateCache {
                    schema_version: CACHE_SCHEMA_VERSION,
                    checked_at_epoch_seconds: checked_at,
                    latest_version: update.version.clone(),
                    notes: update.body.clone(),
                };
                let cache_warning = save_cache(&self.cache_path, &cache)
                    .err()
                    .map(|error| format!("La version a été vérifiée, mais le cache local n’a pas pu être enregistré : {error}"));
                let status = UpdateStatus {
                    phase: UpdatePhase::Available,
                    channel: self.snapshot().channel,
                    current_version: update.current_version,
                    available_version: Some(update.version),
                    notes: update.body,
                    checked_at_epoch_seconds: Some(checked_at),
                    progress_percent: None,
                    message: cache_warning,
                };
                self.publish(app, status.clone());
                Ok(status)
            }
            Ok(None) => {
                let current_version = env!("CARGO_PKG_VERSION").to_owned();
                let cache = UpdateCache {
                    schema_version: CACHE_SCHEMA_VERSION,
                    checked_at_epoch_seconds: checked_at,
                    latest_version: current_version.clone(),
                    notes: None,
                };
                let cache_warning = save_cache(&self.cache_path, &cache)
                    .err()
                    .map(|error| format!("La version a été vérifiée, mais le cache local n’a pas pu être enregistré : {error}"));
                let status = UpdateStatus {
                    phase: UpdatePhase::Current,
                    channel: self.snapshot().channel,
                    current_version,
                    available_version: None,
                    notes: None,
                    checked_at_epoch_seconds: Some(checked_at),
                    progress_percent: None,
                    message: cache_warning,
                };
                self.publish(app, status.clone());
                Ok(status)
            }
            Err(error) => {
                let previous = self.snapshot();
                let status = UpdateStatus {
                    phase: UpdatePhase::Error,
                    progress_percent: None,
                    message: Some(format!("La vérification a échoué : {error}")),
                    ..previous
                };
                self.publish(app, status.clone());
                Err(status.message.clone().unwrap_or(error))
            }
        }
    }

    /// Download, verify, and install an update after explicit confirmation by the caller.
    ///
    /// # Errors
    ///
    /// Returns an error when no update is available, this installation is externally managed, the
    /// signature is invalid, the archive cannot be installed, or the background agent cannot be
    /// restarted from the updated bundle.
    pub async fn install(&self, app: &AppHandle) -> Result<(), String> {
        match self.snapshot().channel {
            UpdateChannel::Direct => {}
            UpdateChannel::Homebrew => {
                return Err(
                    "Cette installation est gérée par Homebrew. Utilisez « brew upgrade --cask warpgatesh »."
                        .to_owned(),
                );
            }
            UpdateChannel::Unsupported => {
                return Err(
                    "Installez WarpgateSH dans Applications avant d’utiliser la mise à jour intégrée."
                        .to_owned(),
                );
            }
        }
        let Some(_guard) = self.begin_operation() else {
            return Err("Une opération de mise à jour est déjà en cours.".to_owned());
        };

        let updater = app
            .updater()
            .map_err(|error| self.fail(app, format!("Mise à jour indisponible : {error}")))?;
        let Some(update) = updater
            .check()
            .await
            .map_err(|error| self.fail(app, format!("Vérification impossible : {error}")))?
        else {
            let current = UpdateStatus {
                phase: UpdatePhase::Current,
                available_version: None,
                notes: None,
                checked_at_epoch_seconds: Some(epoch_seconds()),
                progress_percent: None,
                message: None,
                ..self.snapshot()
            };
            self.publish(app, current);
            return Err("WarpgateSH est déjà à jour.".to_owned());
        };

        let available_version = update.version.clone();
        let notes = update.body.clone();
        self.publish(
            app,
            UpdateStatus {
                phase: UpdatePhase::Downloading,
                available_version: Some(available_version.clone()),
                notes: notes.clone(),
                progress_percent: Some(0),
                message: None,
                ..self.snapshot()
            },
        );

        let manager = self.clone();
        let progress_app = app.clone();
        let mut downloaded = 0_u64;
        let mut published_progress = 0_u8;
        let bytes = update
            .download(
                move |chunk_length, total_length| {
                    downloaded = downloaded.saturating_add(chunk_length as u64);
                    let Some(total) = total_length.filter(|total| *total > 0) else {
                        return;
                    };
                    let progress = ((downloaded.saturating_mul(100) / total).min(100)) as u8;
                    if progress == published_progress {
                        return;
                    }
                    published_progress = progress;
                    manager.publish_progress(&progress_app, progress);
                },
                || {},
            )
            .await
            .map_err(|error| self.fail(app, format!("Téléchargement refusé : {error}")))?;

        self.publish(
            app,
            UpdateStatus {
                phase: UpdatePhase::Installing,
                available_version: Some(available_version),
                notes,
                progress_percent: Some(100),
                message: None,
                ..self.snapshot()
            },
        );
        update
            .install(bytes)
            .map_err(|error| self.fail(app, format!("Installation impossible : {error}")))?;

        launchd::restart().map_err(|error| {
            self.fail(
                app,
                format!("Mise à jour installée, mais l’agent n’a pas redémarré : {error}"),
            )
        })?;
        app.request_restart();
        Ok(())
    }

    fn begin_operation(&self) -> Option<OperationGuard> {
        self.operation_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| OperationGuard(Arc::clone(&self.operation_active)))
    }

    fn publish(&self, app: &AppHandle, status: UpdateStatus) {
        *lock_recover(&self.state) = status.clone();
        let _ = app.emit(UPDATE_EVENT, status);
    }

    fn publish_progress(&self, app: &AppHandle, progress: u8) {
        let mut status = self.snapshot();
        status.phase = UpdatePhase::Downloading;
        status.progress_percent = Some(progress);
        self.publish(app, status);
    }

    fn fail(&self, app: &AppHandle, message: String) -> String {
        let status = UpdateStatus {
            phase: UpdatePhase::Error,
            progress_percent: None,
            message: Some(message.clone()),
            ..self.snapshot()
        };
        self.publish(app, status);
        message
    }
}

impl UpdateStatus {
    fn idle(current_version: String, channel: UpdateChannel) -> Self {
        Self {
            phase: UpdatePhase::Idle,
            channel,
            current_version,
            available_version: None,
            notes: None,
            checked_at_epoch_seconds: None,
            progress_percent: None,
            message: None,
        }
    }
}

pub fn start_background_checks(app: AppHandle) {
    let manager = app.state::<UpdateManager>().inner().clone();
    thread::spawn(move || {
        loop {
            let _ = tauri::async_runtime::block_on(manager.check(&app, false));
            thread::sleep(BACKGROUND_POLL_INTERVAL);
        }
    });
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) -> Result<UpdateStatus, String> {
    let manager = app.state::<UpdateManager>().inner().clone();
    manager.check(&app, true).await
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<UpdateManager>().inner().clone();
    manager.install(&app).await
}

#[must_use]
pub fn status(app: &AppHandle) -> UpdateStatus {
    app.state::<UpdateManager>().snapshot()
}

fn status_from_cache(
    current_version: &str,
    channel: UpdateChannel,
    cache: &UpdateCache,
) -> UpdateStatus {
    let available = newer_than(&cache.latest_version, current_version);
    UpdateStatus {
        phase: if available {
            UpdatePhase::Available
        } else {
            UpdatePhase::Current
        },
        channel,
        current_version: current_version.to_owned(),
        available_version: available.then(|| cache.latest_version.clone()),
        notes: available.then(|| cache.notes.clone()).flatten(),
        checked_at_epoch_seconds: Some(cache.checked_at_epoch_seconds),
        progress_percent: None,
        message: None,
    }
}

fn newer_than(candidate: &str, current: &str) -> bool {
    Version::parse(candidate)
        .and_then(|candidate| Version::parse(current).map(|current| candidate > current))
        .unwrap_or(false)
}

fn check_is_due(checked_at: Option<u64>, now: u64) -> bool {
    checked_at.is_none_or(|checked_at| now.saturating_sub(checked_at) >= CHECK_INTERVAL.as_secs())
}

fn load_cache(path: &Path) -> Option<UpdateCache> {
    let cache: UpdateCache = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    (cache.schema_version == CACHE_SCHEMA_VERSION).then_some(cache)
}

fn save_cache(path: &Path, cache: &UpdateCache) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(cache).map_err(display_error)?;
    atomic_write(path, &bytes).map_err(display_error)
}

fn detect_update_channel() -> UpdateChannel {
    if cfg!(debug_assertions) {
        return UpdateChannel::Unsupported;
    }
    let Ok(executable) = std::env::current_exe() else {
        return UpdateChannel::Unsupported;
    };
    let homebrew_cask_roots = HOMEBREW_CASK_ROOTS.map(Path::new);
    detect_update_channel_from(
        &executable,
        Path::new(INSTALLED_APPLICATION),
        &homebrew_cask_roots,
    )
}

fn detect_update_channel_from(
    executable: &Path,
    installed_application: &Path,
    homebrew_cask_roots: &[&Path],
) -> UpdateChannel {
    let inside_applications = executable
        .ancestors()
        .any(|path| path == installed_application);
    if !inside_applications {
        return UpdateChannel::Unsupported;
    }

    let homebrew_managed = homebrew_cask_roots.iter().any(|root| root.is_dir());
    if homebrew_managed {
        UpdateChannel::Homebrew
    } else {
        UpdateChannel::Direct
    }
}

fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_automatically_every_six_hours_unless_forced_by_the_caller() {
        assert!(check_is_due(None, 100_000));
        assert!(!check_is_due(Some(100_000), 100_000 + 21_599));
        assert!(check_is_due(Some(100_000), 100_000 + 21_600));
    }

    #[test]
    fn restores_an_available_release_from_the_update_cache() {
        let status = status_from_cache(
            "0.1.7",
            UpdateChannel::Direct,
            &UpdateCache {
                schema_version: CACHE_SCHEMA_VERSION,
                checked_at_epoch_seconds: 42,
                latest_version: "0.1.8".to_owned(),
                notes: Some("Nouveautés".to_owned()),
            },
        );

        assert_eq!(status.phase, UpdatePhase::Available);
        assert_eq!(status.available_version.as_deref(), Some("0.1.8"));
        assert_eq!(status.notes.as_deref(), Some("Nouveautés"));
    }

    #[test]
    fn refuses_integrated_updates_outside_the_installed_application() {
        assert_eq!(
            detect_update_channel_from(
                Path::new("/private/tmp/WarpgateSH.app/Contents/MacOS/warpgatesh-companion"),
                Path::new("/Applications/WarpgateSH.app"),
                &[Path::new("/opt/homebrew/Caskroom/warpgatesh")],
            ),
            UpdateChannel::Unsupported
        );
    }

    #[test]
    fn recognizes_a_dmg_install_even_when_the_cli_links_to_the_bundle() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let application = directory.path().join("Applications/WarpgateSH.app");
        let executable = application.join("Contents/MacOS/warpgatesh-companion");
        let bundled_cli = application.join("Contents/MacOS/warpgatesh");
        let cli_link = directory.path().join("usr-local-bin-warpgatesh");
        fs::create_dir_all(executable.parent().expect("executable parent"))
            .expect("application executable directory");
        fs::write(&bundled_cli, b"cli").expect("bundled CLI");
        std::os::unix::fs::symlink(&bundled_cli, &cli_link).expect("managed CLI link");

        assert_eq!(
            detect_update_channel_from(&executable, &application, &[]),
            UpdateChannel::Direct
        );
        assert_eq!(
            fs::canonicalize(cli_link).expect("canonical CLI link"),
            fs::canonicalize(bundled_cli).expect("canonical bundled CLI")
        );
    }

    #[test]
    fn recognizes_an_installed_homebrew_cask() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let application = directory.path().join("Applications/WarpgateSH.app");
        let executable = application.join("Contents/MacOS/warpgatesh-companion");
        let cask_root = directory.path().join("Caskroom/warpgatesh");
        fs::create_dir_all(executable.parent().expect("executable parent"))
            .expect("application executable directory");
        fs::create_dir_all(&cask_root).expect("Homebrew cask root");

        assert_eq!(
            detect_update_channel_from(&executable, &application, &[&cask_root]),
            UpdateChannel::Homebrew
        );
    }

    #[test]
    fn never_offers_a_cached_older_release_as_an_update() {
        assert!(!newer_than("0.1.7", "0.1.8"));
        assert!(newer_than("0.1.9", "0.1.8"));
        assert!(!newer_than("nightly", "0.1.8"));
    }
}
