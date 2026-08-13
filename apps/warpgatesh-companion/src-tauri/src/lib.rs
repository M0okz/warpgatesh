mod commands;
mod installation;
mod tray_menu;
mod updates;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use warpgatesh_runtime::diagnostics::DiagnosticLogger;

fn handle_second_instance(app: &tauri::AppHandle, _arguments: Vec<String>, _cwd: String) {
    tray_menu::show_main_window(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the `WarpgateSH` desktop companion.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or its event loop exits with an error.
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(handle_second_instance))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let diagnostics =
                DiagnosticLogger::for_current_user("companion").map_err(std::io::Error::other)?;
            diagnostics.info("process.started");
            app.manage(updates::UpdateManager::new().map_err(std::io::Error::other)?);
            #[cfg(target_os = "macos")]
            {
                app.handle()
                    .set_activation_policy(tauri::ActivationPolicy::Accessory)?;
                if let Err(error) = installation::ensure_bundled_agent() {
                    let mut fields = std::collections::BTreeMap::new();
                    fields.insert("message".to_owned(), serde_json::json!(error.to_string()));
                    let _ = diagnostics.record("error", "agent.install-failed", fields);
                    eprintln!("warpgatesh-companion: could not install the agent: {error}");
                }
            }

            tray_menu::install(app)?;
            updates::start_background_checks(app.handle().clone());
            Ok(())
        })
        .on_menu_event(|app, event| tray_menu::handle_menu_event(app, &event))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_companion_state,
            commands::sync_now,
            commands::preview_diagnostics,
            commands::export_diagnostics,
            commands::save_preferences,
            commands::open_token_page_for,
            commands::inspect_profile,
            commands::add_profile,
            commands::renew_profile_token,
            commands::remove_profile,
            commands::open_target,
            commands::install_command_line_tool,
            commands::uninstall_warpgatesh,
            updates::check_for_updates,
            updates::install_update,
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
            tray_menu::show_main_window(app);
        }
    });
}
