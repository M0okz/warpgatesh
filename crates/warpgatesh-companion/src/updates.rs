//! Update installation belongs to the native Sparkle controller, never this backend.
use crate::CompanionContext;
use serde::Serialize;
use std::path::Path;
const INSTALLED_APPLICATION: &str = "/Applications/WarpgateSH.app";
const HOMEBREW_CASK_ROOTS: [&str; 2] = [
    "/opt/homebrew/Caskroom/warpgatesh",
    "/usr/local/Caskroom/warpgatesh",
];
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdatePhase {
    Idle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannel {
    Direct,
    Homebrew,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub phase: UpdatePhase,
    pub channel: UpdateChannel,
    pub current_version: String,
    pub available_version: Option<String>,
    pub notes: Option<String>,
    pub checked_at_epoch_seconds: Option<u64>,
    pub progress_percent: Option<u8>,
    pub message: Option<String>,
}

#[derive(Clone)]
pub struct UpdateManager {
    state: UpdateStatus,
}
impl UpdateManager {
    pub fn new() -> Self {
        Self {
            state: Self::initial_status(),
        }
    }
    pub fn snapshot(&self) -> UpdateStatus {
        self.state.clone()
    }
    fn initial_status() -> UpdateStatus {
        UpdateStatus {
            phase: UpdatePhase::Idle,
            channel: detect_update_channel(),
            current_version: env!("CARGO_PKG_VERSION").to_owned(),
            available_version: None,
            notes: None,
            checked_at_epoch_seconds: None,
            progress_percent: None,
            message: None,
        }
    }
}
pub fn check_for_updates(_app: CompanionContext) -> Result<UpdateStatus, String> {
    Err("Updates are managed by the native macOS application.".to_owned())
}
pub fn install_update(_app: CompanionContext) -> Result<(), String> {
    Err("Updates require confirmation in the native macOS application.".to_owned())
}
pub fn status(app: &CompanionContext) -> UpdateStatus {
    app.updates.snapshot()
}
fn detect_update_channel() -> UpdateChannel {
    if cfg!(debug_assertions) {
        return UpdateChannel::Unsupported;
    }
    let Ok(executable) = std::env::current_exe() else {
        return UpdateChannel::Unsupported;
    };
    let homebrew_cask_roots = HOMEBREW_CASK_ROOTS.map(Path::new);
    detect_update_channel_from(
        &executable,
        Path::new(INSTALLED_APPLICATION),
        &homebrew_cask_roots,
    )
}

fn detect_update_channel_from(
    executable: &Path,
    installed_application: &Path,
    homebrew_cask_roots: &[&Path],
) -> UpdateChannel {
    let inside_applications = executable
        .ancestors()
        .any(|path| path == installed_application);
    if !inside_applications {
        return UpdateChannel::Unsupported;
    }

    let homebrew_managed = homebrew_cask_roots.iter().any(|root| root.is_dir());
    if homebrew_managed {
        UpdateChannel::Homebrew
    } else {
        UpdateChannel::Direct
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn refuses_integrated_updates_outside_the_installed_application() {
        assert_eq!(
            detect_update_channel_from(
                Path::new("/private/tmp/WarpgateSH.app/Contents/MacOS/warpgatesh-companion"),
                Path::new("/Applications/WarpgateSH.app"),
                &[Path::new("/opt/homebrew/Caskroom/warpgatesh")],
            ),
            UpdateChannel::Unsupported
        );
    }

    #[test]
    fn recognizes_a_dmg_install_even_when_the_cli_links_to_the_bundle() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let application = directory.path().join("Applications/WarpgateSH.app");
        let executable = application.join("Contents/MacOS/warpgatesh-companion");
        let bundled_cli = application.join("Contents/MacOS/warpgatesh");
        let cli_link = directory.path().join("usr-local-bin-warpgatesh");
        fs::create_dir_all(executable.parent().expect("executable parent"))
            .expect("application executable directory");
        fs::write(&bundled_cli, b"cli").expect("bundled CLI");
        std::os::unix::fs::symlink(&bundled_cli, &cli_link).expect("managed CLI link");

        assert_eq!(
            detect_update_channel_from(&executable, &application, &[]),
            UpdateChannel::Direct
        );
        assert_eq!(
            fs::canonicalize(cli_link).expect("canonical CLI link"),
            fs::canonicalize(bundled_cli).expect("canonical bundled CLI")
        );
    }

    #[test]
    fn recognizes_an_installed_homebrew_cask() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let application = directory.path().join("Applications/WarpgateSH.app");
        let executable = application.join("Contents/MacOS/warpgatesh-companion");
        let cask_root = directory.path().join("Caskroom/warpgatesh");
        fs::create_dir_all(executable.parent().expect("executable parent"))
            .expect("application executable directory");
        fs::create_dir_all(&cask_root).expect("Homebrew cask root");

        assert_eq!(
            detect_update_channel_from(&executable, &application, &[&cask_root]),
            UpdateChannel::Homebrew
        );
    }
}
