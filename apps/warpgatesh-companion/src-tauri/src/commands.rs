use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use warpgatesh_core::aliases::is_valid_profile_name;
use warpgatesh_core::profiles::Profile;
use warpgatesh_runtime::api::ApiClient;
use warpgatesh_runtime::configuration::ConfigurationMutation;
use warpgatesh_runtime::diagnostics::{self, DiagnosticLogger, DiagnosticsPreview};
use warpgatesh_runtime::ipc;
use warpgatesh_runtime::ssh::{open_token_page, scan_host_keys};
use warpgatesh_runtime::storage::{AgentErrorKind, LocalStore, Preferences};

use crate::{installation, updates};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionState {
    agent_running: bool,
    agent_synchronizing: bool,
    profiles: Vec<CompanionProfile>,
    targets: Vec<CompanionTarget>,
    last_sync_age_seconds: Option<u64>,
    preferences: CompanionPreferences,
    terminal_integration: TerminalIntegration,
    update: updates::UpdateStatus,
    alerts: Vec<CompanionAlert>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalIntegration {
    status: String,
    path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompanionProfile {
    name: String,
    username: String,
    base_url: String,
    warpgate_version: Option<String>,
    ssh_host: String,
    ssh_port: u16,
    is_default: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompanionTarget {
    alias: String,
    qualified_alias: String,
    name: String,
    profile: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionPreferences {
    sync_interval_seconds: u64,
    launch_companion_at_login: bool,
    default_profile: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompanionAlert {
    id: String,
    kind: String,
    title: String,
    message: String,
    action: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRequest {
    name: String,
    base_url: String,
    token: String,
    ssh_host: Option<String>,
    ssh_port: Option<u16>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallRequest {
    delete_user_data: bool,
    confirmation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInspection {
    normalized_base_url: String,
    username: String,
    warpgate_version: Option<String>,
    ssh_host: String,
    ssh_port: u16,
    fingerprints: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsExport {
    path: String,
}

#[tauri::command]
pub async fn get_companion_state(app: AppHandle) -> Result<CompanionState, String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    let launches_at_login = app.autolaunch().is_enabled().map_err(display_error)?;
    let terminal = load_terminal_integration()?;
    let update = updates::status(&app);
    load_state(&store, launches_at_login, terminal, update)
}

#[tauri::command]
pub async fn install_command_line_tool() -> Result<TerminalIntegration, String> {
    installation::install_cli()
        .map(|status| terminal_integration(&status))
        .map_err(display_error)
}

#[tauri::command]
pub async fn uninstall_warpgatesh(app: AppHandle, request: UninstallRequest) -> Result<(), String> {
    if request.confirmation.trim() != "DÉSINSTALLER" {
        return Err("Saisissez DÉSINSTALLER pour confirmer.".to_owned());
    }

    if app.autolaunch().is_enabled().map_err(display_error)? {
        app.autolaunch().disable().map_err(display_error)?;
    }
    let store = LocalStore::for_current_user().map_err(display_error)?;
    installation::uninstall_components(&store).map_err(display_error)?;
    if request.delete_user_data {
        installation::delete_user_data(&store).map_err(display_error)?;
    }
    installation::move_application_to_trash().map_err(display_error)?;

    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        handle.exit(0);
    });
    Ok(())
}

#[tauri::command]
pub async fn sync_now() -> Result<String, String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    DiagnosticLogger::new(&store.paths().logs_directory, "companion").info("sync.requested");
    request_sync(&store)
}

#[tauri::command]
pub async fn preview_diagnostics() -> Result<DiagnosticsPreview, String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    diagnostics::preview(&store).map_err(display_error)
}

#[tauri::command]
pub async fn export_diagnostics() -> Result<DiagnosticsExport, String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    let logger = DiagnosticLogger::new(&store.paths().logs_directory, "companion");
    logger.info("diagnostics.export-requested");
    let path = diagnostics::export(&store).map_err(display_error)?;
    let _ = Command::new("/usr/bin/open").arg("-R").arg(&path).spawn();
    logger.info("diagnostics.exported");
    Ok(DiagnosticsExport {
        path: path.display().to_string(),
    })
}

#[tauri::command]
pub async fn save_preferences(
    app: AppHandle,
    preferences: CompanionPreferences,
) -> Result<(), String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    let catalog = store.load_profiles().map_err(display_error)?;
    if let Some(default) = &preferences.default_profile {
        if catalog.find(default).is_none() {
            return Err(format!("Le profil « {default} » n’existe pas."));
        }
    }

    if preferences.launch_companion_at_login {
        app.autolaunch().enable().map_err(display_error)?;
    } else {
        app.autolaunch().disable().map_err(display_error)?;
    }
    let persisted = Preferences {
        sync_interval_seconds: preferences.sync_interval_seconds,
        launch_companion_at_login: preferences.launch_companion_at_login,
        ..Preferences::default()
    };
    request_configuration(
        &store,
        &ConfigurationMutation::SavePreferences {
            preferences: persisted,
            default_profile: preferences.default_profile,
        },
    )?;
    Ok(())
}

#[tauri::command]
pub async fn open_token_page_for(base_url: String) -> Result<(), String> {
    let client = ApiClient::new(&base_url).map_err(display_error)?;
    let page = client.token_page_url().map_err(display_error)?;
    open_token_page(page.as_str()).map_err(display_error)
}

#[tauri::command]
pub async fn inspect_profile(request: ProfileRequest) -> Result<ProfileInspection, String> {
    validate_profile_request(&request)?;
    let client = ApiClient::new(request.base_url.trim()).map_err(display_error)?;
    let metadata = client
        .validate(request.token.trim())
        .map_err(display_error)?;
    let ssh_host = request
        .ssh_host
        .as_deref()
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .unwrap_or(&metadata.ssh_host)
        .to_owned();
    let ssh_port = request.ssh_port.unwrap_or(metadata.ssh_port);
    let host_keys = scan_host_keys(&ssh_host, ssh_port).map_err(display_error)?;
    Ok(ProfileInspection {
        normalized_base_url: client.base_url().as_str().to_owned(),
        username: metadata.username,
        warpgate_version: metadata.version,
        ssh_host,
        ssh_port,
        fingerprints: host_keys.fingerprints,
    })
}

#[tauri::command]
pub async fn add_profile(request: ProfileRequest) -> Result<(), String> {
    validate_profile_request(&request)?;
    let client = ApiClient::new(request.base_url.trim()).map_err(display_error)?;
    let metadata = client
        .validate(request.token.trim())
        .map_err(display_error)?;
    let ssh_host = request
        .ssh_host
        .as_deref()
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .unwrap_or(&metadata.ssh_host)
        .to_owned();
    let ssh_port = request.ssh_port.unwrap_or(metadata.ssh_port);
    let host_keys = scan_host_keys(&ssh_host, ssh_port).map_err(display_error)?;
    let store = LocalStore::for_current_user().map_err(display_error)?;
    request_configuration(
        &store,
        &ConfigurationMutation::SaveProfile {
            profile: Profile {
                name: request.name.clone(),
                base_url: client.base_url().as_str().to_owned(),
                username: metadata.username,
                warpgate_version: metadata.version,
                ssh_host,
                ssh_port,
            },
            token: request.token.trim().to_owned(),
            known_hosts: host_keys.known_hosts,
        },
    )?;
    Ok(())
}

#[tauri::command]
pub async fn renew_profile_token(name: String, token: String) -> Result<(), String> {
    if token.trim().is_empty() {
        return Err("Le jeton API est requis.".to_owned());
    }
    let store = LocalStore::for_current_user().map_err(display_error)?;
    let catalog = store.load_profiles().map_err(display_error)?;
    let existing = catalog
        .find(&name)
        .cloned()
        .ok_or_else(|| format!("Le profil « {name} » n’existe pas."))?;
    let client = ApiClient::new(&existing.base_url).map_err(display_error)?;
    let metadata = client.validate(token.trim()).map_err(display_error)?;
    request_configuration(
        &store,
        &ConfigurationMutation::RenewToken {
            name,
            token: token.trim().to_owned(),
            username: metadata.username,
            warpgate_version: metadata.version,
        },
    )?;
    Ok(())
}

#[tauri::command]
pub async fn remove_profile(name: String) -> Result<(), String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    request_configuration(&store, &ConfigurationMutation::RemoveProfile { name })?;
    Ok(())
}

#[tauri::command]
pub async fn open_target(alias: String) -> Result<(), String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    let snapshot = store
        .load_snapshot()
        .map_err(display_error)?
        .ok_or_else(|| "Aucune cible SSH synchronisée n’est disponible.".to_owned())?;
    let known = snapshot.targets.iter().any(|target| {
        target.qualified_alias == alias || target.short_alias.as_deref() == Some(alias.as_str())
    });
    if !known {
        return Err("L’alias SSH sélectionné n’est pas dans l’instantané local.".to_owned());
    }

    #[cfg(target_os = "macos")]
    let status = Command::new("/usr/bin/open")
        .arg(format!("ssh://{alias}"))
        .status()
        .map_err(display_error)?;

    #[cfg(target_os = "linux")]
    let status = Command::new("xdg-open")
        .arg(format!("ssh://{alias}"))
        .status()
        .map_err(display_error)?;

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err(
        "L’ouverture de cibles SSH n’est pas prise en charge sur cette plateforme.".to_owned(),
    );

    if status.success() {
        Ok(())
    } else {
        Err(format!("Le terminal s’est arrêté avec le statut {status}."))
    }
}

fn load_state(
    store: &LocalStore,
    launches_at_login: bool,
    terminal_integration: TerminalIntegration,
    update: updates::UpdateStatus,
) -> Result<CompanionState, String> {
    let catalog = store.load_profiles().map_err(display_error)?;
    let snapshot = store.load_snapshot().map_err(display_error)?;
    let preferences = store.load_preferences().map_err(display_error)?;
    let profiles = catalog
        .profiles
        .iter()
        .map(|profile| CompanionProfile {
            is_default: catalog.default_profile.as_deref() == Some(profile.name.as_str()),
            name: profile.name.clone(),
            username: profile.username.clone(),
            base_url: profile.base_url.clone(),
            warpgate_version: profile.warpgate_version.clone(),
            ssh_host: profile.ssh_host.clone(),
            ssh_port: profile.ssh_port,
        })
        .collect();
    let (targets, last_sync_age_seconds) = snapshot.map_or_else(
        || (Vec::new(), None),
        |snapshot| {
            let targets = snapshot
                .targets
                .into_iter()
                .map(|target| CompanionTarget {
                    alias: target
                        .short_alias
                        .unwrap_or_else(|| target.qualified_alias.clone()),
                    qualified_alias: target.qualified_alias,
                    name: target.name,
                    profile: target.profile,
                })
                .collect();
            let age = epoch_seconds().saturating_sub(snapshot.synchronized_at_epoch_seconds);
            (targets, Some(age))
        },
    );
    let agent_runtime = ipc::request_with_read_timeout(
        &store.paths().agent_socket,
        "status",
        Duration::from_secs(1),
    )
    .map_or(
        AgentRuntimeState {
            running: false,
            synchronizing: false,
        },
        |response| parse_agent_runtime_state(&response),
    );
    let alerts = build_alerts(
        store,
        agent_runtime.running,
        last_sync_age_seconds,
        &preferences,
    )?;

    Ok(CompanionState {
        agent_running: agent_runtime.running,
        agent_synchronizing: agent_runtime.synchronizing,
        profiles,
        targets,
        last_sync_age_seconds,
        preferences: CompanionPreferences {
            sync_interval_seconds: preferences.sync_interval_seconds,
            launch_companion_at_login: launches_at_login,
            default_profile: catalog.default_profile,
        },
        terminal_integration,
        update,
        alerts,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AgentRuntimeState {
    running: bool,
    synchronizing: bool,
}

fn parse_agent_runtime_state(response: &str) -> AgentRuntimeState {
    AgentRuntimeState {
        running: response.split_whitespace().any(|field| field == "running"),
        synchronizing: response
            .split_whitespace()
            .any(|field| field == "state=synchronizing"),
    }
}

fn load_terminal_integration() -> Result<TerminalIntegration, String> {
    installation::cli_installation()
        .map(|status| terminal_integration(&status))
        .map_err(display_error)
}

fn terminal_integration(status: &installation::CliInstallation) -> TerminalIntegration {
    TerminalIntegration {
        status: status.status().to_owned(),
        path: status.path().display().to_string(),
    }
}

fn build_alerts(
    store: &LocalStore,
    agent_running: bool,
    sync_age: Option<u64>,
    preferences: &Preferences,
) -> Result<Vec<CompanionAlert>, String> {
    let mut alerts = Vec::new();
    if !agent_running {
        alerts.push(CompanionAlert {
            id: "agent-down".to_owned(),
            kind: "error".to_owned(),
            title: "Agent arrêté".to_owned(),
            message: "La synchronisation automatique ne fonctionne plus.".to_owned(),
            action: None,
        });
    }
    if let Some(status) = store
        .load_agent_status()
        .map_err(display_error)?
        .filter(|status| status.consecutive_failures >= 2)
    {
        if let (Some(kind), Some(message)) = (status.last_error_kind, status.last_error_message) {
            let (id, title, action) = match kind {
                AgentErrorKind::Unauthorized => ("token", "Jeton refusé", Some("profiles")),
                AgentErrorKind::ApiUnreachable => ("api", "Warpgate injoignable", None),
                AgentErrorKind::HostKey => ("host-key", "Clé SSH à vérifier", Some("profiles")),
                AgentErrorKind::Incompatible => ("api-version", "API incompatible", None),
                AgentErrorKind::Other => ("sync", "Synchronisation en échec", None),
            };
            alerts.push(CompanionAlert {
                id: id.to_owned(),
                kind: "error".to_owned(),
                title: title.to_owned(),
                message,
                action: action.map(str::to_owned),
            });
        }
    }
    let stale_after = preferences
        .sync_interval_seconds
        .saturating_mul(2)
        .saturating_add(60);
    if alerts.is_empty() && sync_age.is_some_and(|age| age > stale_after) {
        alerts.push(CompanionAlert {
            id: "stale".to_owned(),
            kind: "warning".to_owned(),
            title: "Synchronisation en retard".to_owned(),
            message: "L’instantané local est plus ancien que prévu.".to_owned(),
            action: None,
        });
    }
    Ok(alerts)
}

fn validate_profile_request(request: &ProfileRequest) -> Result<(), String> {
    if !is_valid_profile_name(request.name.trim()) {
        return Err(
            "Le nom doit utiliser uniquement des lettres minuscules, chiffres et tirets."
                .to_owned(),
        );
    }
    if request.token.trim().is_empty() {
        return Err("Le jeton API est requis.".to_owned());
    }
    Ok(())
}

fn request_sync(store: &LocalStore) -> Result<String, String> {
    ipc::request_with_retry(&store.paths().agent_socket, "sync", Duration::from_secs(20))
        .map_err(display_error)
}

fn request_configuration(
    store: &LocalStore,
    mutation: &ConfigurationMutation,
) -> Result<String, String> {
    ipc::request_mutation(
        &store.paths().agent_socket,
        mutation,
        Duration::from_secs(20),
    )
    .map_err(display_error)
}

pub(crate) fn synchronize_from_tray() {
    std::thread::spawn(|| {
        if let Ok(store) = LocalStore::for_current_user() {
            let _ = request_sync(&store);
        }
    });
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
    use tempfile::TempDir;
    use warpgatesh_core::paths::WarpgatePaths;
    use warpgatesh_core::profiles::{Profile, ProfileCatalog};
    use warpgatesh_runtime::storage::{
        AgentStatus, SNAPSHOT_SCHEMA_VERSION, Snapshot, SyncedTarget,
    };

    use super::*;

    #[test]
    fn maps_the_local_snapshot_without_exposing_secrets() {
        let home = TempDir::new().expect("temporary home");
        let store = LocalStore::new(WarpgatePaths::for_home(home.path()));
        let mut catalog = ProfileCatalog::default();
        catalog
            .upsert(Profile {
                name: "homeblack".to_owned(),
                base_url: "https://warpgate.example/".to_owned(),
                username: "gregory".to_owned(),
                warpgate_version: Some("0.27.1".to_owned()),
                ssh_host: "10.60.0.17".to_owned(),
                ssh_port: 2222,
            })
            .expect("profile");
        store.save_profiles(&catalog).expect("save profiles");
        store
            .save_snapshot(&Snapshot {
                schema_version: SNAPSHOT_SCHEMA_VERSION,
                synchronized_at_epoch_seconds: epoch_seconds(),
                targets: vec![SyncedTarget {
                    profile: "homeblack".to_owned(),
                    target_id: "target-1".to_owned(),
                    name: "dmz-nextcloud-01".to_owned(),
                    short_alias: None,
                    qualified_alias: "dmz-nextcloud-01.homeblack".to_owned(),
                }],
            })
            .expect("save snapshot");

        let state = load_state(
            &store,
            false,
            TerminalIntegration {
                status: "missing".to_owned(),
                path: "/usr/local/bin/warpgatesh".to_owned(),
            },
            updates::UpdateStatus {
                phase: updates::UpdatePhase::Current,
                channel: updates::UpdateChannel::Direct,
                current_version: env!("CARGO_PKG_VERSION").to_owned(),
                available_version: None,
                notes: None,
                checked_at_epoch_seconds: Some(epoch_seconds()),
                progress_percent: None,
                message: None,
            },
        )
        .expect("companion state");
        assert_eq!(state.profiles.len(), 1);
        assert_eq!(state.targets.len(), 1);
        assert_eq!(state.targets[0].alias, "dmz-nextcloud-01.homeblack");
        assert!(!state.agent_running);
        assert!(!state.agent_synchronizing);
        assert_eq!(state.alerts[0].id, "agent-down");
        assert_eq!(state.terminal_integration.status, "missing");
        assert_eq!(state.update.phase, updates::UpdatePhase::Current);
    }

    #[test]
    fn parses_the_live_agent_synchronization_state() {
        assert_eq!(
            parse_agent_runtime_state("running state=synchronizing next_sync_seconds=0"),
            AgentRuntimeState {
                running: true,
                synchronizing: true,
            }
        );
        assert_eq!(
            parse_agent_runtime_state("running state=idle next_sync_seconds=42"),
            AgentRuntimeState {
                running: true,
                synchronizing: false,
            }
        );
    }

    #[test]
    fn hides_the_first_sync_failure_and_reports_the_second() {
        let home = TempDir::new().expect("temporary home");
        let store = LocalStore::new(WarpgatePaths::for_home(home.path()));
        let preferences = Preferences::default();

        store
            .save_agent_status(&AgentStatus {
                schema_version: warpgatesh_runtime::storage::AGENT_STATUS_SCHEMA_VERSION,
                last_attempt_epoch_seconds: epoch_seconds(),
                last_success_epoch_seconds: Some(epoch_seconds().saturating_sub(60)),
                consecutive_failures: 1,
                last_error_kind: Some(AgentErrorKind::ApiUnreachable),
                last_error_message: Some("temporary failure".to_owned()),
            })
            .expect("first failed attempt");
        assert!(
            build_alerts(&store, true, Some(60), &preferences)
                .expect("alerts after first failure")
                .is_empty()
        );

        store
            .save_agent_status(&AgentStatus {
                schema_version: warpgatesh_runtime::storage::AGENT_STATUS_SCHEMA_VERSION,
                last_attempt_epoch_seconds: epoch_seconds(),
                last_success_epoch_seconds: Some(epoch_seconds().saturating_sub(90)),
                consecutive_failures: 2,
                last_error_kind: Some(AgentErrorKind::ApiUnreachable),
                last_error_message: Some("second failure".to_owned()),
            })
            .expect("second failed attempt");
        let alerts = build_alerts(&store, true, Some(90), &preferences)
            .expect("alerts after second failure");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, "api");
    }
}
