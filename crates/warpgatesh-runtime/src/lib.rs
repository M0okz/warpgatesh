//! Warpgate API and operating-system integration for `WarpgateSH`.

pub mod api;
pub mod error;
pub mod ipc;
pub mod keychain;
pub mod launchd;
pub mod ssh;
pub mod storage;
pub mod sync;

pub use error::RuntimeError;
