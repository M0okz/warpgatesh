//! Private JSON-lines transport over inherited pipes. No listening TCP or Unix socket.
use std::io::{BufRead, Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;

use crate::relaunch::NativeSession;
use crate::{commands, installation, updates};

const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_IN_FLIGHT: usize = 16;

#[derive(Deserialize)]
struct Request {
    id: u64,
    #[serde(flatten)]
    operation: Operation,
}

#[derive(Deserialize)]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
enum Operation {
    Initialize,
    GetCompanionState,
    SyncNow,
    PreviewDiagnostics,
    ExportDiagnostics,
    SavePreferences {
        preferences: commands::CompanionPreferences,
    },
    OpenTokenPageFor {
        base_url: String,
    },
    InspectProfile {
        request: commands::ProfileRequest,
    },
    AddProfile {
        request: commands::ProfileRequest,
    },
    RenewProfileToken {
        name: String,
        token: String,
    },
    RotateProfileToken {
        name: String,
        session_cookie: Option<String>,
    },
    SetTokenAutoRenew {
        name: String,
        enabled: bool,
    },
    RemoveProfile {
        name: String,
    },
    OpenTarget {
        alias: String,
    },
    InstallCommandLineTool,
    UninstallWarpgatesh {
        request: commands::UninstallRequest,
    },
    CheckForUpdates,
    InstallUpdate,
}

pub(crate) fn run(mut context: tauri::Context<tauri::Wry>) {
    let parent_pid = std::env::args()
        .skip_while(|arg| arg != "--native-parent")
        .nth(1)
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|pid| *pid > 1)
        .expect("native backend requires its parent process id");
    context.config_mut().app.windows.clear();
    let application = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(move |app| {
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Prohibited)?;
            app.manage(NativeSession { parent_pid });
            app.manage(updates::UpdateManager::new().map_err(std::io::Error::other)?);
            let handle = app.handle().clone();
            std::thread::spawn(move || serve(&handle));
            Ok(())
        })
        .build(context)
        .expect("initialize native backend");
    application.run(|_, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            write_message(&json!({"event": "exit"}));
        }
    });
}

fn serve(app: &AppHandle) {
    let mut input = std::io::stdin().lock();
    let active = Arc::new(AtomicUsize::new(0));
    loop {
        let mut bytes = Vec::new();
        let read = (&mut input)
            .take((MAX_REQUEST_BYTES + 1) as u64)
            .read_until(b'\n', &mut bytes);
        match read {
            Ok(0) | Err(_) => break,
            Ok(_) if bytes.len() > MAX_REQUEST_BYTES => break,
            Ok(_) => {}
        }
        let Ok(request) = serde_json::from_slice::<Request>(&bytes) else {
            let id = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|value| value.get("id").and_then(Value::as_u64))
                .unwrap_or(0);
            write_message(&json!({"id": id, "error": "Requête locale invalide."}));
            continue;
        };
        if active.fetch_add(1, Ordering::AcqRel) >= MAX_IN_FLIGHT {
            active.fetch_sub(1, Ordering::AcqRel);
            write_message(&json!({"id": request.id, "error": "Trop de demandes simultanées."}));
            continue;
        }
        let handle = app.clone();
        let active = Arc::clone(&active);
        tauri::async_runtime::spawn(async move {
            let result = dispatch(handle, request.operation).await;
            let message = match result {
                Ok(result) => json!({"id": request.id, "result": result}),
                Err(error) => json!({"id": request.id, "error": error}),
            };
            write_message(&message);
            active.fetch_sub(1, Ordering::AcqRel);
        });
    }
    app.exit(0);
}

fn write_message(message: &Value) {
    let mut output = std::io::stdout().lock();
    if serde_json::to_writer(&mut output, message).is_ok() {
        let _ = output.write_all(b"\n");
        let _ = output.flush();
    }
}

async fn dispatch(app: AppHandle, operation: Operation) -> Result<Value, String> {
    match operation {
        Operation::Initialize => {
            installation::ensure_bundled_agent().map_err(|error| error.to_string())?;
            updates::start_background_checks(app);
            Ok(Value::Null)
        }
        Operation::GetCompanionState => serialize(commands::get_companion_state(app).await?),
        Operation::SyncNow => serialize(commands::sync_now().await?),
        Operation::PreviewDiagnostics => serialize(commands::preview_diagnostics().await?),
        Operation::ExportDiagnostics => serialize(commands::export_diagnostics().await?),
        Operation::SavePreferences { preferences } => {
            commands::save_preferences(app, preferences).await?;
            Ok(Value::Null)
        }
        Operation::OpenTokenPageFor { base_url } => {
            commands::open_token_page_for(base_url).await?;
            Ok(Value::Null)
        }
        Operation::InspectProfile { request } => {
            serialize(commands::inspect_profile(request).await?)
        }
        Operation::AddProfile { request } => {
            if request
                .expected_fingerprints
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err("Vérifiez les empreintes SSH avant d’enregistrer ce profil.".to_owned());
            }
            commands::add_profile(request).await?;
            Ok(Value::Null)
        }
        Operation::RenewProfileToken { name, token } => {
            commands::renew_profile_token(name, token).await?;
            Ok(Value::Null)
        }
        Operation::RotateProfileToken {
            name,
            session_cookie,
        } => serialize(commands::rotate_profile_token(name, session_cookie).await?),
        Operation::SetTokenAutoRenew { name, enabled } => {
            commands::set_token_auto_renew(name, enabled).await?;
            Ok(Value::Null)
        }
        Operation::RemoveProfile { name } => {
            commands::remove_profile(name).await?;
            Ok(Value::Null)
        }
        Operation::OpenTarget { alias } => {
            commands::open_target(alias).await?;
            Ok(Value::Null)
        }
        Operation::InstallCommandLineTool => {
            serialize(commands::install_command_line_tool().await?)
        }
        Operation::UninstallWarpgatesh { request } => {
            commands::uninstall_warpgatesh(app, request).await?;
            Ok(Value::Null)
        }
        Operation::CheckForUpdates => serialize(updates::check_for_updates(app).await?),
        Operation::InstallUpdate => {
            updates::install_update(app).await?;
            Ok(Value::Null)
        }
    }
}

fn serialize(value: impl serde::Serialize) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|_| "Réponse locale invalide.".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_commands_and_accepts_native_contract() {
        assert!(
            serde_json::from_str::<Request>(r#"{"id":1,"command":"delete_everything"}"#).is_err()
        );
        assert!(
            serde_json::from_str::<Request>(r#"{"id":2,"command":"get_companion_state"}"#).is_ok()
        );
        assert!(
            serde_json::from_str::<Request>(
                r#"{"id":3,"command":"open_target","args":{"alias":"example.lab"}}"#
            )
            .is_ok()
        );
    }
}
