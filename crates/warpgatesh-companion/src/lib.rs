//! Standalone macOS companion services and private pipe transport.
#[cfg(target_os = "macos")]
mod agent_setup;
#[cfg(target_os = "macos")]
mod commands;
#[cfg(target_os = "macos")]
mod companion_login;
#[cfg(target_os = "macos")]
mod installation;
#[cfg(target_os = "macos")]
mod native;
#[cfg(target_os = "macos")]
mod updates;
#[cfg(target_os = "macos")]
pub use native::run;

#[cfg(target_os = "macos")]
#[derive(Clone)]
pub(crate) struct CompanionContext {
    pub updates: updates::UpdateManager,
}
#[cfg(target_os = "macos")]
impl CompanionContext {
    pub fn exit(status: i32) -> ! {
        native::write_message(&serde_json::json!({"event":"exit"}));
        std::process::exit(status)
    }
}
