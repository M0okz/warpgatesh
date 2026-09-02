use std::path::Path;

#[cfg(target_os = "linux")]
use std::{fs, process::Command};
#[cfg(target_os = "linux")]
use warpgatesh_core::paths::WarpgatePaths;

use crate::RuntimeError;
#[cfg(target_os = "linux")]
use crate::storage::atomic_write;

#[cfg(target_os = "linux")]
const UNIT: &str = "warpgatesh-agent.service";

#[cfg(target_os = "linux")]
pub fn ensure_installed(
    paths: &WarpgatePaths,
    agent_executable: &Path,
) -> Result<bool, RuntimeError> {
    if !agent_executable.is_file() {
        return Err(RuntimeError::Command(format!(
            "agent executable not found at {}",
            agent_executable.display()
        )));
    }

    let unit = render_unit(agent_executable)?;
    let changed = fs::read(&paths.systemd_user_service).ok().as_deref() != Some(unit.as_bytes());
    let active = is_loaded()?;

    if changed {
        atomic_write(&paths.systemd_user_service, unit.as_bytes())?;
        run_systemctl(&["daemon-reload"])?;
    }

    run_systemctl(&["enable", "--now", UNIT])?;
    if changed && active {
        run_systemctl(&["restart", UNIT])?;
    }

    Ok(changed || !active)
}

#[cfg(target_os = "linux")]
pub fn is_loaded() -> Result<bool, RuntimeError> {
    let output = Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", UNIT])
        .output()?;
    Ok(output.status.success())
}

#[cfg(target_os = "linux")]
pub fn uninstall(paths: &WarpgatePaths) -> Result<bool, RuntimeError> {
    let active = is_loaded()?;
    let installed = paths.systemd_user_service.exists();

    if active || installed {
        let output = Command::new("systemctl")
            .args(["--user", "disable", "--now", UNIT])
            .output()?;
        if !output.status.success() && active {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(RuntimeError::Command(format!(
                "systemctl --user failed to disable {UNIT}: {message}"
            )));
        }
    }

    let removed = match fs::remove_file(&paths.systemd_user_service) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    if removed {
        run_systemctl(&["daemon-reload"])?;
    }
    match fs::remove_file(&paths.agent_socket) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(active || removed)
}

#[cfg(target_os = "linux")]
fn run_systemctl(arguments: &[&str]) -> Result<(), RuntimeError> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(arguments)
        .output()?;
    if output.status.success() {
        return Ok(());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(RuntimeError::Command(if message.is_empty() {
        format!("systemctl --user failed with {}", output.status)
    } else {
        format!("systemctl --user failed: {message}")
    }))
}

fn render_unit(agent_executable: &Path) -> Result<String, RuntimeError> {
    let executable = systemd_path(agent_executable)?;
    Ok(format!(
        r#"[Unit]
Description=WarpgateSH background synchronization agent
Documentation=https://github.com/M0okz/warpgatesh
After=graphical-session.target

[Service]
Type=simple
ExecStart="{executable}"
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
"#
    ))
}

fn systemd_path(path: &Path) -> Result<String, RuntimeError> {
    let text = path.to_str().ok_or_else(|| {
        RuntimeError::InvalidInput(format!("path is not valid UTF-8: {}", path.display()))
    })?;
    if text.chars().any(char::is_control) {
        return Err(RuntimeError::InvalidInput(
            "systemd service paths cannot contain control characters".to_owned(),
        ));
    }

    Ok(text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('%', "%%"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_restartable_user_service() {
        let unit = render_unit(Path::new("/home/tester/WarpgateSH Tools/warpgatesh-agent"))
            .expect("systemd unit");

        assert!(unit.contains("ExecStart=\"/home/tester/WarpgateSH Tools/warpgatesh-agent\""));
        assert!(unit.contains("Restart=on-failure"));
        assert!(unit.contains("WantedBy=default.target"));
    }

    #[test]
    fn escapes_systemd_specifiers_in_executable_paths() {
        let unit =
            render_unit(Path::new("/home/tester/100%/warpgatesh-agent")).expect("systemd unit");

        assert!(unit.contains("/home/tester/100%%/warpgatesh-agent"));
    }
}
