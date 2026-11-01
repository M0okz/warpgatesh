//! Standalone macOS companion services and private pipe transport.
#[cfg(target_os = "macos")]
mod agent_setup;
#[cfg(target_os = "macos")]
mod commands;
#[cfg(target_os = "macos")]
mod companion_login;
#[cfg(target_os = "macos")]
mod distribution;
#[cfg(target_os = "macos")]
mod installation;
#[cfg(target_os = "macos")]
mod native;
#[cfg(target_os = "macos")]
mod relaunch;
#[cfg(target_os = "macos")]
mod updates;
#[cfg(target_os = "macos")]
pub use native::run;

#[cfg(target_os = "macos")]
#[derive(Clone)]
pub(crate) struct CompanionContext {
    pub parent_pid: u32,
    pub updates: updates::UpdateManager,
    pub background_started: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
#[cfg(target_os = "macos")]
impl CompanionContext {
    pub fn exit(&self, status: i32) -> ! {
        self.background_started
            .store(false, std::sync::atomic::Ordering::Release);
        native::write_message(&serde_json::json!({"event":"exit"}));
        std::process::exit(status)
    }
}

#[cfg(target_os = "macos")]
mod transaction;
#[cfg(target_os = "macos")]
pub use transaction::recover;
