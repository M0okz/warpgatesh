mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the `WarpgateSH` desktop companion.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or its event loop exits with an error.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_companion_state,
            commands::sync_now,
            commands::open_target
        ])
        .run(tauri::generate_context!())
        .expect("error while running the WarpgateSH companion");
}
