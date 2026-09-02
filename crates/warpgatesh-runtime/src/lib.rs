//! Warpgate API and operating-system integration for `WarpgateSH`.

pub mod agent_service;
pub mod api;
pub mod configuration;
pub mod diagnostics;
pub mod error;
pub mod ipc;
pub mod keychain;
pub mod launchd;
pub mod ssh;
pub mod storage;
pub mod sync;
#[cfg(any(target_os = "linux", test))]
mod systemd;

pub use error::RuntimeError;
