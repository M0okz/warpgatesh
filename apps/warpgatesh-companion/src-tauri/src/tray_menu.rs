use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};
use warpgatesh_runtime::ipc;
use warpgatesh_runtime::storage::LocalStore;

use crate::{commands, updates};

const AGENT_STATUS_TIMEOUT: Duration = Duration::from_millis(500);
const STATUS_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const GITHUB_REPOSITORY_URL: &str = "https://github.com/M0okz/warpgatesh";
const GITHUB_DOCUMENTATION_URL: &str = "https://github.com/M0okz/warpgatesh#readme";
const GITHUB_ISSUES_URL: &str = "https://github.com/M0okz/warpgatesh/issues";

#[derive(Clone, Debug, Eq, PartialEq)]
struct AgentRuntimeStatus {
    running: bool,
    synchronizing: bool,
    next_sync_seconds: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TrayLabels {
    agent: String,
    last_sync: String,
    next_sync: String,
    sync_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UpdateLabels {
    status: String,
    download: String,
    download_enabled: bool,
}

struct HelpMenu {
    submenu: Submenu<tauri::Wry>,
    update_status: MenuItem<tauri::Wry>,
    download_update: MenuItem<tauri::Wry>,
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn show_view(app: &tauri::AppHandle, view: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("warpgatesh:navigate", view);
    }
}

pub(crate) fn install(app: &mut tauri::App) -> tauri::Result<()> {
    let labels = load_labels();
    let agent = MenuItem::with_id(app, "agent-status", &labels.agent, true, None::<&str>)?;
    let last_sync = MenuItem::with_id(
        app,
        "last-sync-status",
        &labels.last_sync,
        false,
        None::<&str>,
    )?;
    let next_sync = MenuItem::with_id(
        app,
        "next-sync-status",
        &labels.next_sync,
        false,
        None::<&str>,
    )?;
    let sync = MenuItem::with_id(
        app,
        "sync",
        "Synchroniser maintenant",
        labels.sync_enabled,
        None::<&str>,
    )?;
    let show = MenuItem::with_id(app, "show", "Ouvrir WarpgateSH", true, None::<&str>)?;
    let profiles = MenuItem::with_id(app, "profiles", "Profils…", true, None::<&str>)?;
    let prefs = MenuItem::with_id(app, "preferences", "Préférences…", true, None::<&str>)?;
    let help = build_help_menu(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter WarpgateSH", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &agent,
            &last_sync,
            &next_sync,
            &PredefinedMenuItem::separator(app)?,
            &sync,
            &PredefinedMenuItem::separator(app)?,
            &show,
            &profiles,
            &prefs,
            &help.submenu,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("WarpgateSH")
        .show_menu_on_left_click(true)
        .icon(tauri::include_image!("icons/tray-icon.png"));
    #[cfg(target_os = "macos")]
    {
        tray = tray.icon_as_template(true);
    }
    tray.build(app)?;

    thread::spawn(move || {
        let mut previous = labels;
        loop {
            thread::sleep(STATUS_REFRESH_INTERVAL);
            let labels = load_labels();
            if labels == previous {
                continue;
            }
            let _ = agent.set_text(&labels.agent);
            let _ = last_sync.set_text(&labels.last_sync);
            let _ = next_sync.set_text(&labels.next_sync);
            let _ = sync.set_enabled(labels.sync_enabled);
            previous = labels;
        }
    });

    let update_app = app.handle().clone();
    thread::spawn(move || {
        let mut previous = update_labels_for(&updates::status(&update_app));
        loop {
            thread::sleep(STATUS_REFRESH_INTERVAL);
            let labels = update_labels_for(&updates::status(&update_app));
            if labels == previous {
                continue;
            }
            let _ = help.update_status.set_text(&labels.status);
            let _ = help.download_update.set_text(&labels.download);
            let _ = help.download_update.set_enabled(labels.download_enabled);
            previous = labels;
        }
    });

    Ok(())
}

fn build_help_menu(app: &tauri::App) -> tauri::Result<HelpMenu> {
    let update_labels = update_labels_for(&updates::status(app.handle()));
    let github = MenuItem::with_id(app, "github", "GitHub", true, None::<&str>)?;
    let documentation =
        MenuItem::with_id(app, "documentation", "Documentation", true, None::<&str>)?;
    let troubleshooting = MenuItem::with_id(
        app,
        "troubleshooting",
        "Dépannage et signalement",
        true,
        None::<&str>,
    )?;
    let installed_version = MenuItem::with_id(
        app,
        "installed-version",
        format!("Version installée : {}", env!("CARGO_PKG_VERSION")),
        false,
        None::<&str>,
    )?;
    let update_status = MenuItem::with_id(
        app,
        "update-status",
        &update_labels.status,
        false,
        None::<&str>,
    )?;
    let download_update = MenuItem::with_id(
        app,
        "download-update",
        &update_labels.download,
        update_labels.download_enabled,
        None::<&str>,
    )?;
    let submenu = Submenu::with_items(
        app,
        "Aide et assistance",
        true,
        &[
            &github,
            &documentation,
            &troubleshooting,
            &PredefinedMenuItem::separator(app)?,
            &installed_version,
            &update_status,
            &download_update,
        ],
    )?;

    Ok(HelpMenu {
        submenu,
        update_status,
        download_update,
    })
}

pub(crate) fn handle_menu_event(app: &tauri::AppHandle, event: &MenuEvent) {
    match event.id().as_ref() {
        "agent-status" | "show" => show_main_window(app),
        "profiles" => show_view(app, "profiles"),
        "preferences" => show_view(app, "preferences"),
        "sync" => commands::synchronize_from_tray(),
        "github" => open_external_url(GITHUB_REPOSITORY_URL),
        "documentation" => open_external_url(GITHUB_DOCUMENTATION_URL),
        "troubleshooting" => open_external_url(GITHUB_ISSUES_URL),
        "download-update" => show_view(app, "updates"),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn update_labels_for(update: &updates::UpdateStatus) -> UpdateLabels {
    use updates::{UpdateChannel, UpdatePhase};

    let default_download = "Rechercher les mises à jour…".to_owned();
    match update.phase {
        UpdatePhase::Idle => UpdateLabels {
            status: "Mises à jour : en attente".to_owned(),
            download: default_download,
            download_enabled: false,
        },
        UpdatePhase::Checking => UpdateLabels {
            status: "Recherche de mise à jour…".to_owned(),
            download: default_download,
            download_enabled: false,
        },
        UpdatePhase::Current => UpdateLabels {
            status: "WarpgateSH est à jour".to_owned(),
            download: default_download,
            download_enabled: false,
        },
        UpdatePhase::Available => {
            let version = update.available_version.as_deref().unwrap_or("nouvelle");
            let download = match update.channel {
                UpdateChannel::Direct => format!("Installer WarpgateSH {version}…"),
                UpdateChannel::Homebrew => "Mettre à jour avec Homebrew…".to_owned(),
                UpdateChannel::Unsupported => format!("Télécharger WarpgateSH {version}…"),
            };
            UpdateLabels {
                status: format!("Mise à jour disponible : {version}"),
                download,
                download_enabled: true,
            }
        }
        UpdatePhase::Downloading => UpdateLabels {
            status: format!(
                "Téléchargement : {} %",
                update.progress_percent.unwrap_or_default()
            ),
            download: "Mise à jour en cours…".to_owned(),
            download_enabled: false,
        },
        UpdatePhase::Installing => UpdateLabels {
            status: "Installation et redémarrage…".to_owned(),
            download: "Mise à jour en cours…".to_owned(),
            download_enabled: false,
        },
        UpdatePhase::Error => UpdateLabels {
            status: "Recherche de mise à jour indisponible".to_owned(),
            download: default_download,
            download_enabled: false,
        },
    }
}

fn open_external_url(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("/usr/bin/open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let _ = Command::new("xdg-open").arg(url).spawn();

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let _ = url;
}

fn load_labels() -> TrayLabels {
    let Ok(store) = LocalStore::for_current_user() else {
        return TrayLabels {
            agent: "○ État de l’agent indisponible".to_owned(),
            last_sync: "Dernière synchro : inconnue".to_owned(),
            next_sync: "Prochaine synchro : inconnue".to_owned(),
            sync_enabled: false,
        };
    };
    let now = epoch_seconds();
    let last_success = store
        .load_agent_status()
        .ok()
        .flatten()
        .and_then(|status| status.last_success_epoch_seconds)
        .or_else(|| {
            store
                .load_snapshot()
                .ok()
                .flatten()
                .map(|snapshot| snapshot.synchronized_at_epoch_seconds)
        });
    let interval = store
        .load_preferences()
        .map_or(5 * 60, |preferences| preferences.sync_interval_seconds);
    let runtime = query_agent(&store);
    labels_for(now, last_success, interval, &runtime)
}

fn query_agent(store: &LocalStore) -> AgentRuntimeStatus {
    match ipc::request_with_read_timeout(
        &store.paths().agent_socket,
        "status",
        AGENT_STATUS_TIMEOUT,
    ) {
        Ok(response) => parse_agent_status(&response),
        Err(_) => AgentRuntimeStatus {
            running: false,
            synchronizing: false,
            next_sync_seconds: None,
        },
    }
}

fn parse_agent_status(response: &str) -> AgentRuntimeStatus {
    let mut status = AgentRuntimeStatus {
        running: response.starts_with("running"),
        synchronizing: false,
        next_sync_seconds: None,
    };
    for field in response.split_whitespace().skip(1) {
        if let Some(value) = field.strip_prefix("state=") {
            status.synchronizing = value == "synchronizing";
        } else if let Some(value) = field.strip_prefix("next_sync_seconds=") {
            status.next_sync_seconds = value.parse().ok();
        }
    }
    status
}

fn labels_for(
    now: u64,
    last_success: Option<u64>,
    interval: u64,
    runtime: &AgentRuntimeStatus,
) -> TrayLabels {
    let age = last_success.map(|timestamp| now.saturating_sub(timestamp));
    let agent = if !runtime.running {
        "🔴 Agent arrêté"
    } else if runtime.synchronizing {
        "🟢 Synchronisation en cours…"
    } else {
        "🟢 Agent actif"
    };
    let last_sync = age.map_or_else(
        || "Dernière synchro : jamais".to_owned(),
        |seconds| format!("Dernière synchro : {}", format_age(seconds)),
    );
    let fallback_remaining = age.map(|seconds| interval.saturating_sub(seconds));
    let next_sync = if !runtime.running {
        "Prochaine synchro : agent arrêté".to_owned()
    } else if runtime.synchronizing {
        "Prochaine synchro : en cours".to_owned()
    } else {
        match runtime.next_sync_seconds.or(fallback_remaining) {
            Some(0) => "Prochaine synchro : imminente".to_owned(),
            Some(seconds) => format!("Prochaine synchro : dans {}", format_duration(seconds)),
            None => "Prochaine synchro : en attente".to_owned(),
        }
    };
    TrayLabels {
        agent: agent.to_owned(),
        last_sync,
        next_sync,
        sync_enabled: runtime.running && !runtime.synchronizing,
    }
}

fn format_age(seconds: u64) -> String {
    if seconds < 5 {
        return "à l’instant".to_owned();
    }
    format!("il y a {}", format_duration(seconds))
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{seconds} s");
    }
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    if minutes < 60 {
        return format!("{minutes} min {remaining_seconds:02} s");
    }
    let hours = minutes / 60;
    let remaining_minutes = minutes % 60;
    format!("{hours} h {remaining_minutes:02} min")
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_detailed_agent_status() {
        assert_eq!(
            parse_agent_status("running state=idle next_sync_seconds=83"),
            AgentRuntimeStatus {
                running: true,
                synchronizing: false,
                next_sync_seconds: Some(83),
            }
        );
        assert_eq!(
            parse_agent_status("running state=synchronizing next_sync_seconds=0"),
            AgentRuntimeStatus {
                running: true,
                synchronizing: true,
                next_sync_seconds: Some(0),
            }
        );
    }

    #[test]
    fn remains_compatible_with_the_previous_agent_response() {
        assert_eq!(
            parse_agent_status("running"),
            AgentRuntimeStatus {
                running: true,
                synchronizing: false,
                next_sync_seconds: None,
            }
        );
    }

    #[test]
    fn builds_live_menu_labels() {
        let labels = labels_for(
            1_000,
            Some(875),
            300,
            &AgentRuntimeStatus {
                running: true,
                synchronizing: false,
                next_sync_seconds: Some(42),
            },
        );

        assert_eq!(labels.agent, "🟢 Agent actif");
        assert_eq!(labels.last_sync, "Dernière synchro : il y a 2 min 05 s");
        assert_eq!(labels.next_sync, "Prochaine synchro : dans 42 s");
        assert!(labels.sync_enabled);
    }

    #[test]
    fn disables_sync_while_the_agent_is_busy() {
        let labels = labels_for(
            1_000,
            None,
            300,
            &AgentRuntimeStatus {
                running: true,
                synchronizing: true,
                next_sync_seconds: Some(0),
            },
        );

        assert_eq!(labels.agent, "🟢 Synchronisation en cours…");
        assert_eq!(labels.next_sync, "Prochaine synchro : en cours");
        assert!(!labels.sync_enabled);
    }

    #[test]
    fn marks_a_stopped_agent_in_red() {
        let labels = labels_for(
            1_000,
            Some(875),
            300,
            &AgentRuntimeStatus {
                running: false,
                synchronizing: false,
                next_sync_seconds: None,
            },
        );

        assert_eq!(labels.agent, "🔴 Agent arrêté");
        assert_eq!(labels.next_sync, "Prochaine synchro : agent arrêté");
        assert!(!labels.sync_enabled);
    }

    #[test]
    fn presents_an_available_direct_update() {
        let labels = update_labels_for(&updates::UpdateStatus {
            phase: updates::UpdatePhase::Available,
            channel: updates::UpdateChannel::Direct,
            current_version: "0.1.6".to_owned(),
            available_version: Some("0.1.7".to_owned()),
            notes: None,
            checked_at_epoch_seconds: Some(1),
            progress_percent: None,
            message: None,
        });

        assert_eq!(labels.status, "Mise à jour disponible : 0.1.7");
        assert_eq!(labels.download, "Installer WarpgateSH 0.1.7…");
        assert!(labels.download_enabled);
    }

    #[test]
    fn reports_download_progress_without_allowing_a_second_install() {
        let labels = update_labels_for(&updates::UpdateStatus {
            phase: updates::UpdatePhase::Downloading,
            channel: updates::UpdateChannel::Direct,
            current_version: "0.1.6".to_owned(),
            available_version: Some("0.1.7".to_owned()),
            notes: None,
            checked_at_epoch_seconds: Some(1),
            progress_percent: Some(64),
            message: None,
        });

        assert_eq!(labels.status, "Téléchargement : 64 %");
        assert!(!labels.download_enabled);
    }
}
