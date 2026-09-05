use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::Manager;

#[path = "../src/relaunch.rs"]
mod relaunch;

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).expect("probe directory"));
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .setup(move |app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            if root.join("installed").exists() {
                let window = app.get_webview_window("main").expect("probe window");
                window.show()?;
                let old_pid = fs::read_to_string(root.join("old-pid"))?;
                let old_exited = !std::process::Command::new("/bin/kill")
                    .args(["-0", old_pid.trim()])
                    .stderr(std::process::Stdio::null())
                    .status()?
                    .success();
                fs::write(
                    root.join("reopened"),
                    serde_json::to_vec(&serde_json::json!({
                        "visible": window.is_visible()?,
                        "oldExited": old_exited,
                        "build": "original build marker"
                    }))?,
                )?;
                app.handle().exit(0);
            } else {
                fs::write(root.join("started"), b"old process initialized")?;
                fs::write(root.join("old-pid"), std::process::id().to_string())?;
                let handle = app.handle().clone();
                let relaunch = relaunch::RelaunchPlan::prepare(&handle)?;
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(500));
                    fs::rename(root.join("Probe.app"), root.join("previous.app")).unwrap();
                    fs::rename(root.join("Replacement.app"), root.join("Probe.app")).unwrap();
                    fs::remove_dir_all(root.join("previous.app")).unwrap();
                    fs::write(root.join("installed"), b"bundle replaced").unwrap();
                    relaunch.restart(&handle).unwrap();
                });
            }
            Ok(())
        })
        .build(tauri::generate_context!(
            "examples/relaunch-probe/tauri.conf.json"
        ))
        .expect("probe app");
    app.run(|_, _| {});
}
