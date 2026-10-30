//! Warpgate API and operating-system integration for `WarpgateSH`.

pub mod agent_service;
pub mod api;
pub mod configuration;
pub(crate) mod configuration_lock;
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
pub mod token_api;
pub mod token_lifecycle;

pub use error::RuntimeError;
