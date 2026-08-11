#[cfg(any(target_os = "macos", test))]
use std::path::Path;

#[cfg(any(target_os = "macos", test))]
use warpgatesh_core::paths::WarpgatePaths;

#[cfg(any(target_os = "macos", test))]
use crate::RuntimeError;

pub const LABEL: &str = "dev.warpgatesh.agent";

/// Ensure the per-user macOS `LaunchAgent` is registered and running.
///
/// Returns `true` when a new property list was installed.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the agent executable is missing or launchd
/// rejects the registration.
#[cfg(target_os = "macos")]
pub fn ensure_installed(
    paths: &WarpgatePaths,
    agent_executable: &Path,
) -> Result<bool, RuntimeError> {
    use std::fs;

    use crate::storage::atomic_write;

    if !agent_executable.is_file() {
        return Err(RuntimeError::Command(format!(
            "agent executable not found at {}",
            agent_executable.display()
        )));
    }

    let property_list = render_property_list(paths, agent_executable)?;
    let changed = fs::read(&paths.launch_agent).ok().as_deref() != Some(property_list.as_bytes());
    if changed {
        atomic_write(&paths.launch_agent, property_list.as_bytes())?;
    }

    let loaded = is_loaded()?;
    if loaded && changed {
        let service = launchd_service()?;
        run_launchctl(&["bootout", &service])?;
    }
    if !loaded || changed {
        let domain = launchd_domain()?;
        let property_list_path = path_text(&paths.launch_agent)?;
        run_launchctl(&["bootstrap", &domain, property_list_path])?;
    }
    Ok(changed)
}

/// Report whether launchd currently knows the per-user service.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the current user identifier cannot be read.
#[cfg(target_os = "macos")]
pub fn is_loaded() -> Result<bool, RuntimeError> {
    use std::process::Command;

    Ok(Command::new("/bin/launchctl")
        .args(["print", &launchd_service()?])
        .output()?
        .status
        .success())
}

/// Stop and remove the per-user macOS `LaunchAgent`.
///
/// Returns `true` when a loaded service or property list was removed.
///
/// # Errors
///
/// Returns [`RuntimeError`] when launchd rejects the operation or the property
/// list cannot be removed.
#[cfg(target_os = "macos")]
pub fn uninstall(paths: &WarpgatePaths) -> Result<bool, RuntimeError> {
    use std::fs;

    let loaded = is_loaded()?;
    if loaded {
        run_launchctl(&["bootout", &launchd_service()?])?;
    }

    let property_list_removed = match fs::remove_file(&paths.launch_agent) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    match fs::remove_file(&paths.agent_socket) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(loaded || property_list_removed)
}

#[cfg(target_os = "macos")]
fn run_launchctl(arguments: &[&str]) -> Result<(), RuntimeError> {
    use std::process::Command;

    let output = Command::new("/bin/launchctl").args(arguments).output()?;
    if output.status.success() {
        return Ok(());
    }
    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(RuntimeError::Command(if message.is_empty() {
        format!("launchctl failed with {}", output.status)
    } else {
        format!("launchctl failed: {message}")
    }))
}

#[cfg(target_os = "macos")]
fn launchd_domain() -> Result<String, RuntimeError> {
    Ok(format!("gui/{}", user_id()?))
}

#[cfg(target_os = "macos")]
fn launchd_service() -> Result<String, RuntimeError> {
    Ok(format!("{}/{LABEL}", launchd_domain()?))
}

#[cfg(target_os = "macos")]
fn user_id() -> Result<String, RuntimeError> {
    use std::process::Command;

    let output = Command::new("/usr/bin/id").arg("-u").output()?;
    if !output.status.success() {
        return Err(RuntimeError::Command(
            "could not determine the current user id".to_owned(),
        ));
    }
    let identifier = String::from_utf8(output.stdout)
        .map_err(|_| RuntimeError::Command("the current user id is not UTF-8".to_owned()))?;
    Ok(identifier.trim().to_owned())
}

#[cfg(target_os = "macos")]
fn path_text(path: &Path) -> Result<&str, RuntimeError> {
    path.to_str().ok_or_else(|| {
        RuntimeError::InvalidInput(format!("path is not valid UTF-8: {}", path.display()))
    })
}

#[cfg(any(target_os = "macos", test))]
fn render_property_list(
    paths: &WarpgatePaths,
    agent_executable: &Path,
) -> Result<String, RuntimeError> {
    let executable = xml_path(agent_executable)?;
    let stdout = xml_path(&paths.agent_stdout_log)?;
    let stderr = xml_path(&paths.agent_stderr_log)?;
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{executable}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ProcessType</key>
  <string>Background</string>
  <key>ThrottleInterval</key>
  <integer>10</integer>
  <key>StandardOutPath</key>
  <string>{stdout}</string>
  <key>StandardErrorPath</key>
  <string>{stderr}</string>
</dict>
</plist>
"#
    ))
}

#[cfg(any(target_os = "macos", test))]
fn xml_path(path: &Path) -> Result<String, RuntimeError> {
    let text = path.to_str().ok_or_else(|| {
        RuntimeError::InvalidInput(format!("path is not valid UTF-8: {}", path.display()))
    })?;
    if text.contains('\n') || text.contains('\r') {
        return Err(RuntimeError::InvalidInput(
            "launch agent paths cannot contain line breaks".to_owned(),
        ));
    }
    Ok(text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_safe_per_user_launch_agent() {
        let paths = WarpgatePaths::for_home(Path::new("/Users/A & B"));
        let plist = render_property_list(
            &paths,
            Path::new("/Applications/WarpgateSH.app/Contents/MacOS/warpgatesh-agent"),
        )
        .expect("property list");

        assert!(plist.contains("<string>dev.warpgatesh.agent</string>"));
        assert!(plist.contains("/Users/A &amp; B/"));
        assert!(plist.contains("agent.log"));
        assert!(plist.contains("<key>KeepAlive</key>\n  <true/>"));
    }
}
