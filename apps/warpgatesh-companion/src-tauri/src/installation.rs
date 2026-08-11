use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::launchd;
use warpgatesh_runtime::storage::LocalStore;

const CLI_LINK: &str = "/usr/local/bin/warpgatesh";
const HOMEBREW_CLI: &str = "/opt/homebrew/bin/warpgatesh";
const ADMIN_INSTALL_SCRIPT: &str = r#"on run argv
  set sourcePath to item 1 of argv
  set targetPath to item 2 of argv
  set targetDirectory to item 3 of argv
  set installCommand to "/bin/mkdir -p " & quoted form of targetDirectory & " && /bin/ln -s " & quoted form of sourcePath & " " & quoted form of targetPath
  do shell script installCommand with administrator privileges
end run"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliInstallation {
    Managed(PathBuf),
    External(PathBuf),
    Missing(PathBuf),
    Conflict(PathBuf),
}

impl CliInstallation {
    pub fn status(&self) -> &'static str {
        match self {
            Self::Managed(_) => "managed",
            Self::External(_) => "external",
            Self::Missing(_) => "missing",
            Self::Conflict(_) => "conflict",
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Managed(path)
            | Self::External(path)
            | Self::Missing(path)
            | Self::Conflict(path) => path,
        }
    }
}

pub fn ensure_bundled_agent() -> Result<bool, RuntimeError> {
    if cfg!(debug_assertions) {
        return Ok(false);
    }
    let executable = bundled_executable("warpgatesh-agent")?;
    if executable.starts_with("/Volumes") {
        return Err(RuntimeError::Command(
            "Déplacez WarpgateSH dans Applications avant d’activer son agent.".to_owned(),
        ));
    }
    let store = LocalStore::for_current_user()?;
    launchd::ensure_installed(store.paths(), &executable)
}

pub fn cli_installation() -> Result<CliInstallation, RuntimeError> {
    let bundled = bundled_executable("warpgatesh")?;
    let target = Path::new(CLI_LINK);
    let candidates = executable_candidates(target);
    detect_cli_installation(&bundled, target, &candidates)
}

pub fn install_cli() -> Result<CliInstallation, RuntimeError> {
    let bundled = bundled_executable("warpgatesh")?;
    if bundled.starts_with("/Volumes") {
        return Err(RuntimeError::Command(
            "Déplacez WarpgateSH dans Applications avant d’installer sa CLI.".to_owned(),
        ));
    }
    let target = Path::new(CLI_LINK);
    let candidates = executable_candidates(target);
    match detect_cli_installation(&bundled, target, &candidates)? {
        installed @ (CliInstallation::Managed(_) | CliInstallation::External(_)) => {
            return Ok(installed);
        }
        CliInstallation::Conflict(path) => {
            return Err(RuntimeError::Command(format!(
                "La commande {} existe déjà et n’appartient pas à WarpgateSH.",
                path.display()
            )));
        }
        CliInstallation::Missing(_) => {}
    }

    match install_link(&bundled, target) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            install_link_with_administrator(&bundled, target)?;
        }
        Err(error) => return Err(error.into()),
    }

    match detect_cli_installation(&bundled, target, &candidates)? {
        installed @ CliInstallation::Managed(_) => Ok(installed),
        _ => Err(RuntimeError::Command(
            "macOS n’a pas installé la commande warpgatesh.".to_owned(),
        )),
    }
}

fn bundled_executable(name: &str) -> Result<PathBuf, RuntimeError> {
    bundled_executable_from(&std::env::current_exe()?, name)
}

fn bundled_executable_from(current: &Path, name: &str) -> Result<PathBuf, RuntimeError> {
    let parent = current.parent().ok_or_else(|| {
        RuntimeError::Command("Le bundle WarpgateSH n’a pas de dossier exécutable.".to_owned())
    })?;
    let executable = parent.join(name);
    if is_executable(&executable)? {
        Ok(executable)
    } else {
        Err(RuntimeError::Command(format!(
            "Le bundle WarpgateSH ne contient pas {}.",
            executable.display()
        )))
    }
}

fn executable_candidates(target: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from(HOMEBREW_CLI)];
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(
            std::env::split_paths(&path)
                .map(|directory| directory.join("warpgatesh"))
                .filter(|candidate| candidate != target),
        );
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

fn detect_cli_installation(
    bundled: &Path,
    target: &Path,
    candidates: &[PathBuf],
) -> Result<CliInstallation, RuntimeError> {
    if !is_executable(bundled)? {
        return Err(RuntimeError::Command(format!(
            "La CLI embarquée est introuvable à {}.",
            bundled.display()
        )));
    }

    if path_entry_exists(target)? {
        return if same_file(target, bundled) {
            Ok(CliInstallation::Managed(target.to_path_buf()))
        } else {
            Ok(CliInstallation::Conflict(target.to_path_buf()))
        };
    }

    for candidate in candidates {
        if is_executable(candidate)? {
            return Ok(CliInstallation::External(candidate.clone()));
        }
    }

    Ok(CliInstallation::Missing(target.to_path_buf()))
}

fn is_executable(path: &Path) -> Result<bool, RuntimeError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn path_entry_exists(path: &Path) -> Result<bool, RuntimeError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn same_file(left: &Path, right: &Path) -> bool {
    let Ok(left) = fs::canonicalize(left) else {
        return false;
    };
    let Ok(right) = fs::canonicalize(right) else {
        return false;
    };
    left == right
}

fn install_link(source: &Path, target: &Path) -> std::io::Result<()> {
    let parent = target.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "the CLI link has no parent directory",
        )
    })?;
    fs::create_dir_all(parent)?;
    std::os::unix::fs::symlink(source, target)
}

fn install_link_with_administrator(source: &Path, target: &Path) -> Result<(), RuntimeError> {
    let parent = target.parent().ok_or_else(|| {
        RuntimeError::InvalidInput("Le lien CLI n’a pas de dossier parent.".to_owned())
    })?;
    let output = Command::new("/usr/bin/osascript")
        .args(["-e", ADMIN_INSTALL_SCRIPT, "--"])
        .arg(source)
        .arg(target)
        .arg(parent)
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        let details = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(RuntimeError::Command(if details.is_empty() {
            "L’installation de la CLI a été annulée ou refusée par macOS.".to_owned()
        } else {
            format!("L’installation de la CLI a échoué : {details}")
        }))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn executable(path: &Path) {
        fs::write(path, b"test executable").expect("write executable");
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("permissions");
    }

    #[test]
    fn finds_a_bundled_sibling_executable() {
        let directory = TempDir::new().expect("temporary directory");
        let current = directory.path().join("warpgatesh-companion");
        let cli = directory.path().join("warpgatesh");
        executable(&current);
        executable(&cli);

        assert_eq!(
            bundled_executable_from(&current, "warpgatesh").expect("bundled CLI"),
            cli
        );
    }

    #[test]
    fn detects_and_installs_a_managed_cli_link() {
        let directory = TempDir::new().expect("temporary directory");
        let bundled = directory.path().join("bundle/warpgatesh");
        let target = directory.path().join("bin/warpgatesh");
        fs::create_dir_all(bundled.parent().expect("bundle parent")).expect("bundle directory");
        executable(&bundled);

        assert_eq!(
            detect_cli_installation(&bundled, &target, &[]).expect("missing status"),
            CliInstallation::Missing(target.clone())
        );
        install_link(&bundled, &target).expect("install link");
        assert_eq!(
            detect_cli_installation(&bundled, &target, &[]).expect("managed status"),
            CliInstallation::Managed(target)
        );
    }

    #[test]
    fn never_overwrites_an_existing_command() {
        let directory = TempDir::new().expect("temporary directory");
        let bundled = directory.path().join("bundle/warpgatesh");
        let target = directory.path().join("bin/warpgatesh");
        fs::create_dir_all(bundled.parent().expect("bundle parent")).expect("bundle directory");
        fs::create_dir_all(target.parent().expect("target parent")).expect("target directory");
        executable(&bundled);
        executable(&target);

        assert_eq!(
            detect_cli_installation(&bundled, &target, &[]).expect("conflict status"),
            CliInstallation::Conflict(target)
        );
    }

    #[test]
    fn treats_a_broken_cli_link_as_a_conflict() {
        let directory = TempDir::new().expect("temporary directory");
        let bundled = directory.path().join("bundle/warpgatesh");
        let target = directory.path().join("bin/warpgatesh");
        fs::create_dir_all(bundled.parent().expect("bundle parent")).expect("bundle directory");
        fs::create_dir_all(target.parent().expect("target parent")).expect("target directory");
        executable(&bundled);
        std::os::unix::fs::symlink("missing-warpgatesh", &target).expect("broken link");

        assert_eq!(
            detect_cli_installation(&bundled, &target, &[]).expect("conflict status"),
            CliInstallation::Conflict(target)
        );
    }

    #[test]
    fn accepts_an_existing_external_installation() {
        let directory = TempDir::new().expect("temporary directory");
        let bundled = directory.path().join("bundle/warpgatesh");
        let target = directory.path().join("usr-local/warpgatesh");
        let external = directory.path().join("homebrew/warpgatesh");
        fs::create_dir_all(bundled.parent().expect("bundle parent")).expect("bundle directory");
        fs::create_dir_all(external.parent().expect("external parent"))
            .expect("external directory");
        executable(&bundled);
        executable(&external);

        assert_eq!(
            detect_cli_installation(&bundled, &target, std::slice::from_ref(&external))
                .expect("external status"),
            CliInstallation::External(external)
        );
    }
}
