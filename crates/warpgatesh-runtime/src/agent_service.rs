use std::path::Path;

use warpgatesh_core::paths::WarpgatePaths;

use crate::RuntimeError;

/// Ensure the platform's per-user background service is installed and running.
///
/// Returns `true` when the service definition was installed, updated, or started.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the executable is missing or the platform service manager
/// rejects the operation.
pub fn ensure_installed(
    paths: &WarpgatePaths,
    agent_executable: &Path,
) -> Result<bool, RuntimeError> {
    #[cfg(target_os = "macos")]
    {
        return crate::launchd::ensure_installed(paths, agent_executable);
    }

    #[cfg(target_os = "linux")]
    {
        return crate::systemd::ensure_installed(paths, agent_executable);
    }

    #[allow(unreachable_code)]
    Err(RuntimeError::Command(
        "persistent background agents are not supported on this platform".to_owned(),
    ))
}

/// Report whether the platform's per-user background service is active.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the service manager cannot be queried.
pub fn is_loaded() -> Result<bool, RuntimeError> {
    #[cfg(target_os = "macos")]
    {
        return crate::launchd::is_loaded();
    }

    #[cfg(target_os = "linux")]
    {
        return crate::systemd::is_loaded();
    }

    #[allow(unreachable_code)]
    Ok(false)
}

/// Stop and unregister the platform's per-user background service.
///
/// Returns `true` when a service definition or active service was removed.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the service manager rejects the operation.
pub fn uninstall(paths: &WarpgatePaths) -> Result<bool, RuntimeError> {
    #[cfg(target_os = "macos")]
    {
        return crate::launchd::uninstall(paths);
    }

    #[cfg(target_os = "linux")]
    {
        return crate::systemd::uninstall(paths);
    }

    #[allow(unreachable_code)]
    Ok(false)
}
