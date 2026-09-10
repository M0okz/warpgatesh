use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Listener, Manager};
use warpgatesh_runtime::ipc;
use warpgatesh_runtime::storage::LocalStore;

use crate::{commands, updates};

const AGENT_STATUS_TIMEOUT: Duration = Duration::from_millis(500);
const STATUS_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
const ANIMATION_INTERVAL: Duration = Duration::from_millis(80);
const GITHUB_REPOSITORY_URL: &str = "https://github.com/M0okz/warpgatesh";
const GITHUB_DOCUMENTATION_URL: &str = "https://github.com/M0okz/warpgatesh#readme";
const GITHUB_ISSUES_URL: &str = "https://github.com/M0okz/warpgatesh/issues";

#[derive(Clone, Debug, Eq, PartialEq)]
struct AgentRuntimeStatus {
    running: bool,
    synchronizing: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TrayLabels {
    agent: String,
    last_sync: String,
    synchronizing: bool,
    sync: String,
    sync_enabled: bool,
}

struct SyncRequests(mpsc::SyncSender<()>);

impl TrayLabels {
    fn set_synchronizing(&mut self) {
        "🟢 Synchronisation en cours…".clone_into(&mut self.agent);
        self.synchronizing = true;
        "Synchronisation en cours…".clone_into(&mut self.sync);
        self.sync_enabled = false;
    }
}

const SPINNER_FRAMES: usize = 12;

#[derive(Debug, Eq, PartialEq)]
enum SyncIcon {
    Idle,
    Spinner(usize),
}

#[derive(Default)]
struct SyncAnimation {
    next_frame: usize,
    active: bool,
}

impl SyncAnimation {
    fn advance(&mut self, synchronizing: bool) -> Option<SyncIcon> {
        if synchronizing {
            self.active = true;
            let frame = self.next_frame;
            self.next_frame = (frame + 1) % SPINNER_FRAMES;
            Some(SyncIcon::Spinner(frame))
        } else if self.active {
            self.active = false;
            self.next_frame = 0;
            Some(SyncIcon::Idle)
        } else {
            None
        }
    }
}

fn spinner_frames() -> Vec<tauri::image::Image<'static>> {
    // Template masks stay legible in both menu-bar appearances. Generate these
    // small frames once; no decoding or drawing is needed on animation ticks.
    const SIZE: u32 = 44;
    (0..12_u32)
        .map(|frame| {
            let mut rgba = Vec::new();
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let mut alpha = 0;
                    for dot in 0..12_u32 {
                        let angle = f64::from(dot) * std::f64::consts::TAU / 12.0;
                        let dx = f64::from(x) + 0.5 - (22.0 + 14.0 * angle.sin());
                        let dy = f64::from(y) + 0.5 - (22.0 - 14.0 * angle.cos());
                        if dx * dx + dy * dy <= 6.25 {
                            let age = (frame + 12 - dot) % 12;
                            alpha = u8::try_from(255 - age * 18).unwrap_or(0);
                        }
                    }
                    rgba.extend_from_slice(&[0, 0, 0, alpha]);
                }
            }
            tauri::image::Image::new_owned(rgba, SIZE, SIZE)
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UpdateLabels {
    status: String,
    download: String,
    download_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpdateMenuAction {
    Check,
    Open,
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
    let sync = MenuItem::with_id(app, "sync", &labels.sync, labels.sync_enabled, None::<&str>)?;
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
    let tray = tray.build(app)?;
    let animation_sender = start_icon_animation(tray, labels.synchronizing);
    let sender = start_status_refresh(agent, last_sync, sync, labels, animation_sender);
    app.manage(SyncRequests(sender));

    let update_app = app.handle().clone();
    app.listen(updates::UPDATE_EVENT, move |_| {
        let labels = update_labels_for(&updates::status(&update_app));
        let _ = help.update_status.set_text(&labels.status);
        let _ = help.download_update.set_text(&labels.download);
        let _ = help.download_update.set_enabled(labels.download_enabled);
    });

    Ok(())
}

fn start_icon_animation(tray: tauri::tray::TrayIcon, synchronizing: bool) -> mpsc::Sender<bool> {
    let (animation_sender, animation_receiver) = mpsc::channel();
    let _ = animation_sender.send(synchronizing);
    thread::spawn(move || {
        let frames = spinner_frames();
        let mut animation = SyncAnimation::default();
        loop {
            // Sleep until a state change when idle; only wake at frame rate
            // while synchronization is actually running.
            let synchronizing = if animation.active {
                match animation_receiver.recv_timeout(ANIMATION_INTERVAL) {
                    Ok(active) => active,
                    Err(mpsc::RecvTimeoutError::Timeout) => true,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            } else {
                let Ok(active) = animation_receiver.recv() else {
                    break;
                };
                active
            };
            if let Some(frame) = animation.advance(synchronizing) {
                let icon = match frame {
                    SyncIcon::Spinner(index) => frames[index].clone(),
                    SyncIcon::Idle => tauri::include_image!("icons/tray-icon.png"),
                };
                let _ = tray.set_icon_with_as_template(Some(icon), true);
            }
        }
    });

    animation_sender
}

fn start_status_refresh(
    agent: MenuItem<tauri::Wry>,
    last_sync: MenuItem<tauri::Wry>,
    sync: MenuItem<tauri::Wry>,
    labels: TrayLabels,
    animation_sender: mpsc::Sender<bool>,
) -> mpsc::SyncSender<()> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let apply = |labels: &TrayLabels| {
            let _ = animation_sender.send(labels.synchronizing);
            let _ = agent.set_text(&labels.agent);
            let _ = last_sync.set_text(&labels.last_sync);
            let _ = sync.set_text(&labels.sync);
            let _ = sync.set_enabled(labels.sync_enabled);
        };
        let mut previous = labels;
        let mut request_failed = false;
        loop {
            match receiver.recv_timeout(STATUS_REFRESH_INTERVAL) {
                Ok(()) if previous.sync_enabled => {
                    let mut busy = previous.clone();
                    busy.set_synchronizing();
                    apply(&busy);
                    request_failed = commands::synchronize_from_tray().is_err();
                    previous = busy;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                _ => {}
            }
            let mut labels = load_labels();
            if labels.synchronizing {
                request_failed = false;
            } else if request_failed {
                "🔴 Synchronisation impossible".clone_into(&mut labels.agent);
            }
            if labels != previous {
                apply(&labels);
                previous = labels;
            }
        }
    });
    sender
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
        "sync" => {
            let _ = app.state::<SyncRequests>().0.try_send(());
        }
        "github" => open_external_url(GITHUB_REPOSITORY_URL),
        "documentation" => open_external_url(GITHUB_DOCUMENTATION_URL),
        "troubleshooting" => open_external_url(GITHUB_ISSUES_URL),
        "download-update" => handle_update_menu_action(app),
        "quit" => app.exit(0),
        _ => {}
    }
}

fn handle_update_menu_action(app: &tauri::AppHandle) {
    match update_menu_action_for(&updates::status(app)) {
        Some(UpdateMenuAction::Check) => {
            show_view(app, "updates");
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = updates::check_for_updates(app).await;
            });
        }
        Some(UpdateMenuAction::Open) => show_view(app, "updates"),
        None => {}
    }
}

fn update_menu_action_for(update: &updates::UpdateStatus) -> Option<UpdateMenuAction> {
    use updates::UpdatePhase;

    match update.phase {
        UpdatePhase::Idle | UpdatePhase::Current | UpdatePhase::Error => {
            Some(UpdateMenuAction::Check)
        }
        UpdatePhase::Available => Some(UpdateMenuAction::Open),
        UpdatePhase::Checking | UpdatePhase::Downloading | UpdatePhase::Installing => None,
    }
}

fn update_labels_for(update: &updates::UpdateStatus) -> UpdateLabels {
    use updates::{UpdateChannel, UpdatePhase};

    let default_download = "Rechercher les mises à jour…".to_owned();
    let download_enabled = update_menu_action_for(update).is_some();
    match update.phase {
        UpdatePhase::Idle => UpdateLabels {
            status: "Mises à jour : en attente".to_owned(),
            download: default_download,
            download_enabled,
        },
        UpdatePhase::Checking => UpdateLabels {
            status: "Recherche de mise à jour…".to_owned(),
            download: default_download,
            download_enabled: false,
        },
        UpdatePhase::Current => UpdateLabels {
            status: "WarpgateSH est à jour".to_owned(),
            download: default_download,
            download_enabled,
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
                download_enabled,
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
            download_enabled,
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
            synchronizing: false,
            sync: "Synchroniser maintenant".to_owned(),
            sync_enabled: false,
        };
    };
    // Query first, then read the committed success timestamp so completion
    // cannot leave the menu displaying the previous snapshot for another tick.
    let runtime = query_agent(&store);
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
    labels_for(now, last_success, &runtime)
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
        },
    }
}

fn parse_agent_status(response: &str) -> AgentRuntimeStatus {
    let mut status = AgentRuntimeStatus {
        running: response.starts_with("running"),
        synchronizing: false,
    };
    for field in response.split_whitespace().skip(1) {
        if let Some(value) = field.strip_prefix("state=") {
            status.synchronizing = value == "synchronizing";
        }
    }
    status
}

fn labels_for(now: u64, last_success: Option<u64>, runtime: &AgentRuntimeStatus) -> TrayLabels {
    let age = last_success.map(|timestamp| now.saturating_sub(timestamp));
    let agent = if runtime.running {
        "🟢 Agent actif"
    } else {
        "🔴 Agent arrêté"
    };
    let last_sync = age.map_or_else(
        || "Dernière synchro : jamais".to_owned(),
        |seconds| format!("Dernière synchro : {}", format_age(seconds)),
    );
    let mut labels = TrayLabels {
        agent: agent.to_owned(),
        last_sync,
        synchronizing: false,
        sync: "Synchroniser maintenant".to_owned(),
        sync_enabled: runtime.running,
    };
    if runtime.running && runtime.synchronizing {
        labels.set_synchronizing();
    }
    labels
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
            }
        );
        assert_eq!(
            parse_agent_status("running state=synchronizing next_sync_seconds=0"),
            AgentRuntimeStatus {
                running: true,
                synchronizing: true,
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
            }
        );
    }

    #[test]
    fn builds_live_menu_labels() {
        let labels = labels_for(
            1_000,
            Some(875),
            &AgentRuntimeStatus {
                running: true,
                synchronizing: false,
            },
        );

        assert_eq!(labels.agent, "🟢 Agent actif");
        assert_eq!(labels.last_sync, "Dernière synchro : il y a 2 min 05 s");
        assert_eq!(labels.sync, "Synchroniser maintenant");
        assert!(!labels.synchronizing);
        assert!(labels.sync_enabled);
    }

    #[test]
    fn disables_sync_while_the_agent_is_busy() {
        let labels = labels_for(
            1_000,
            None,
            &AgentRuntimeStatus {
                running: true,
                synchronizing: true,
            },
        );

        assert_eq!(labels.agent, "🟢 Synchronisation en cours…");
        assert_eq!(labels.sync, "Synchronisation en cours…");
        assert_eq!(labels.last_sync, "Dernière synchro : jamais");
        assert!(labels.synchronizing);
        assert!(!labels.sync_enabled);
    }

    #[test]
    fn marks_a_stopped_agent_in_red() {
        let labels = labels_for(
            1_000,
            Some(875),
            &AgentRuntimeStatus {
                running: false,
                synchronizing: false,
            },
        );

        assert_eq!(labels.agent, "🔴 Agent arrêté");
        assert!(!labels.synchronizing);
        assert!(!labels.sync_enabled);
    }

    #[test]
    fn refreshes_the_last_success_after_completion_and_keeps_it_on_failure() {
        let busy = parse_agent_status("running state=synchronizing");
        let idle = parse_agent_status("running state=idle");
        assert_eq!(
            labels_for(1_000, Some(875), &busy).last_sync,
            "Dernière synchro : il y a 2 min 05 s"
        );
        let completed = labels_for(1_001, Some(1_001), &idle);
        assert_eq!(completed.last_sync, "Dernière synchro : à l’instant");
        assert!(completed.sync_enabled);
        assert!(!completed.synchronizing);
        assert_eq!(
            labels_for(1_001, Some(875), &idle).last_sync,
            "Dernière synchro : il y a 2 min 06 s"
        );
    }

    #[test]
    fn animates_only_while_busy_and_restores_the_idle_icon() {
        let mut animation = SyncAnimation::default();
        assert_eq!(animation.advance(false), None);
        for frame in 0..SPINNER_FRAMES {
            assert_eq!(animation.advance(true), Some(SyncIcon::Spinner(frame)));
        }
        assert_eq!(animation.advance(true), Some(SyncIcon::Spinner(0)));
        assert_eq!(animation.advance(false), Some(SyncIcon::Idle));
        assert_eq!(animation.advance(false), None);
        assert_eq!(animation.advance(true), Some(SyncIcon::Spinner(0)));
    }

    #[test]
    fn spinner_frames_are_distinct_transparent_template_masks() {
        let frames = spinner_frames();
        assert_eq!(frames.len(), SPINNER_FRAMES);
        for (index, frame) in frames.iter().enumerate() {
            assert_eq!((frame.width(), frame.height()), (44, 44));
            assert_eq!(frame.rgba().len(), 44 * 44 * 4);
            assert!(
                frame
                    .rgba()
                    .chunks_exact(4)
                    .all(|pixel| pixel[..3] == [0, 0, 0])
            );
            assert!(frame.rgba().chunks_exact(4).any(|pixel| pixel[3] == 255));
            assert_eq!(frame.rgba()[3], 0);
            assert_ne!(frame.rgba(), frames[(index + 1) % SPINNER_FRAMES].rgba());
        }
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
    fn allows_a_manual_update_check_when_the_app_is_current() {
        let status = updates::UpdateStatus {
            phase: updates::UpdatePhase::Current,
            channel: updates::UpdateChannel::Direct,
            current_version: env!("CARGO_PKG_VERSION").to_owned(),
            available_version: None,
            notes: None,
            checked_at_epoch_seconds: Some(1),
            progress_percent: None,
            message: None,
        };
        let labels = update_labels_for(&status);

        assert_eq!(labels.download, "Rechercher les mises à jour…");
        assert!(labels.download_enabled);
        assert_eq!(
            update_menu_action_for(&status),
            Some(UpdateMenuAction::Check)
        );
    }

    #[test]
    fn reports_download_progress_without_allowing_a_second_install() {
        let status = updates::UpdateStatus {
            phase: updates::UpdatePhase::Downloading,
            channel: updates::UpdateChannel::Direct,
            current_version: "0.1.6".to_owned(),
            available_version: Some("0.1.7".to_owned()),
            notes: None,
            checked_at_epoch_seconds: Some(1),
            progress_percent: Some(64),
            message: None,
        };
        let labels = update_labels_for(&status);

        assert_eq!(labels.status, "Téléchargement : 64 %");
        assert!(!labels.download_enabled);
        assert_eq!(update_menu_action_for(&status), None);
    }
}
