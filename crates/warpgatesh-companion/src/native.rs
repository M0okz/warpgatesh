//! Private JSON-lines transport over inherited pipes. No listening TCP or Unix socket.
use std::io::{BufRead, Read, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::CompanionContext;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{agent_setup, commands, updates};

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
    SetBackgroundAgent {
        enabled: bool,
    },
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

/// Run the private standalone backend.
///
/// # Panics
/// Panics if the required parent PID, async runtime or local storage is unavailable.
pub fn run() {
    let _parent_pid = std::env::args()
        .skip_while(|a| a != "--native-parent")
        .nth(1)
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|p| *p > 1)
        .expect("native parent id");
    let runtime = tokio::runtime::Runtime::new().expect("async runtime");
    let app = CompanionContext {
        updates: updates::UpdateManager::new(),
    };
    let _guard = runtime.enter();
    serve(&app);
}

fn serve(app: &CompanionContext) {
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
        tokio::spawn(async move {
            let result = dispatch(handle, request.operation).await;
            let message = match result {
                Ok(result) => json!({"id": request.id, "result": result}),
                Err(error) => json!({"id": request.id, "error": error}),
            };
            write_message(&message);
            active.fetch_sub(1, Ordering::AcqRel);
        });
    }
    CompanionContext::exit(0);
}

pub(crate) fn write_message(message: &Value) {
    let mut output = std::io::stdout().lock();
    if serde_json::to_writer(&mut output, message).is_ok() {
        let _ = output.write_all(b"\n");
        let _ = output.flush();
    }
}

async fn dispatch(app: CompanionContext, operation: Operation) -> Result<Value, String> {
    match operation {
        Operation::Initialize => {
            let setup = commands::run_blocking(|| {
                agent_setup::initialize().map_err(|error| error.to_string())
            })
            .await?;
            serialize(setup)
        }
        Operation::SetBackgroundAgent { enabled } => serialize(
            commands::run_blocking(move || {
                agent_setup::set_enabled(enabled).map_err(|error| error.to_string())
            })
            .await?,
        ),
        Operation::GetCompanionState => {
            let state = commands::get_companion_state(app).await?;
            serialize(state)
        }
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
        Operation::CheckForUpdates => serialize(updates::check_for_updates(app)?),
        Operation::InstallUpdate => {
            updates::install_update(app)?;
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
