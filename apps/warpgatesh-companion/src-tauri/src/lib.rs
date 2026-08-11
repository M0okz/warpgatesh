mod commands;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the `WarpgateSH` desktop companion.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or its event loop exits with an error.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            let show = MenuItem::with_id(app, "show", "Afficher WarpgateSH", true, None::<&str>)?;
            let sync = MenuItem::with_id(app, "sync", "Synchroniser maintenant", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quitter WarpgateSH", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &show,
                    &sync,
                    &PredefinedMenuItem::separator(app)?,
                    &quit,
                ],
            )?;
            let mut tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("WarpgateSH")
                .show_menu_on_left_click(false);
            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }
            #[cfg(target_os = "macos")]
            {
                tray = tray.icon_as_template(true);
            }
            tray.build(app)?;
            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "sync" => commands::synchronize_from_tray(),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|app, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(app);
            }
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_companion_state,
            commands::sync_now,
            commands::save_preferences,
            commands::open_token_page_for,
            commands::inspect_profile,
            commands::add_profile,
            commands::renew_profile_token,
            commands::remove_profile,
            commands::open_target,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the WarpgateSH companion");

    app.run(|app, event| {
        #[cfg(target_os = "macos")]
        if matches!(
            event,
            tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            }
        ) {
            show_main_window(app);
        }
    });
}
