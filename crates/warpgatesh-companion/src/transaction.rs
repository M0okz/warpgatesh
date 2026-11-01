//! Recoverable bundle replacement. The old app is retained until new backend startup succeeds.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use warpgatesh_runtime::storage::{LocalStore, atomic_write};
#[derive(Serialize, Deserialize)]
struct Pending {
    directory: PathBuf,
    version: String,
}
fn journal() -> Result<PathBuf, String> {
    Ok(LocalStore::for_current_user()
        .map_err(|e| e.to_string())?
        .paths()
        .application_support
        .join("pending-app-update.json"))
}
fn validate(directory: &Path, application: &Path) -> Result<(), String> {
    let parent = application.parent().ok_or("No application parent")?;
    if directory.parent() != Some(parent)
        || !directory
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with(".warpgatesh-update-"))
        || std::fs::symlink_metadata(directory)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("Invalid recovery directory".into());
    }
    Ok(())
}
pub fn replace(
    candidate: &Path,
    application: &Path,
    directory: &Path,
    version: &str,
) -> Result<(), String> {
    let path = journal()?;
    if path.exists() {
        return Err("Une mise à jour attend une récupération : quittez l’app et utilisez warpgatesh-companion --recover-update.".into());
    }
    validate(directory, application)?;
    atomic_write(
        &path,
        &serde_json::to_vec(&Pending {
            directory: directory.into(),
            version: version.into(),
        })
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if let Err(error) = swap(candidate, application, &directory.join("previous.app")) {
        if !directory.join("previous.app").exists() {
            let _ = std::fs::remove_file(path);
        }
        return Err(error);
    }
    Ok(())
}
fn swap(candidate: &Path, application: &Path, backup: &Path) -> Result<(), String> {
    if std::fs::symlink_metadata(backup).is_ok() {
        return Err("Recovery copy already exists".into());
    }
    std::fs::rename(application, backup).map_err(|e| e.to_string())?;
    if let Err(error) = std::fs::rename(candidate, application) {
        std::fs::rename(backup, application).map_err(|e| format!("Recovery required: {e}"))?;
        return Err(error.to_string());
    }
    Ok(())
}
pub fn acknowledge() -> Result<(), String> {
    let installed = Path::new("/Applications/WarpgateSH.app");
    if std::env::current_exe().map_err(|e| e.to_string())?
        != installed.join("Contents/MacOS/warpgatesh-companion")
    {
        return Ok(());
    }
    let path = journal()?;
    if !path.exists() {
        return Ok(());
    }
    let pending: Pending =
        serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    validate(&pending.directory, installed)?;
    if pending.version != env!("CARGO_PKG_VERSION") {
        return Err("Version de récupération inattendue".into());
    }
    std::fs::remove_dir_all(pending.directory).map_err(|e| e.to_string())?;
    std::fs::remove_file(path).map_err(|e| e.to_string())
}
/// Restore the retained application after an interrupted update.
///
/// # Errors
/// Returns an error when the recovery journal is invalid or the bundle cannot be restored.
pub fn recover() -> Result<(), String> {
    let path = journal()?;
    let pending: Pending =
        serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let installed = Path::new("/Applications/WarpgateSH.app");
    validate(&pending.directory, installed)?;
    let backup = pending.directory.join("previous.app");
    if !backup.join("Contents/Info.plist").is_file() {
        return Err("Copie de récupération absente".into());
    }
    let rejected = pending.directory.join("rejected.app");
    std::fs::rename(installed, &rejected).map_err(|e| e.to_string())?;
    if let Err(error) = std::fs::rename(backup, installed) {
        let _ = std::fs::rename(&rejected, installed);
        return Err(error.to_string());
    }
    std::fs::remove_dir_all(pending.directory).map_err(|e| e.to_string())?;
    std::fs::remove_file(path).map_err(|e| e.to_string())?;
    if warpgatesh_runtime::launchd::is_loaded().map_err(|e| e.to_string())? {
        warpgatesh_runtime::launchd::restart().map_err(|e| e.to_string())?;
    }
    std::process::Command::new("/usr/bin/open")
        .arg("-n")
        .arg(installed)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_replacement_restores_original() {
        let d = tempfile::tempdir().unwrap();
        let app = d.path().join("live.app");
        let old = d.path().join("old.app");
        std::fs::create_dir(&app).unwrap();
        std::fs::write(app.join("version"), b"old").unwrap();
        assert!(swap(&d.path().join("missing"), &app, &old).is_err());
        assert_eq!(std::fs::read(app.join("version")).unwrap(), b"old");
        assert!(!old.exists());
    }
    #[test]
    fn successful_replacement_preserves_original_for_acknowledgement() {
        let d = tempfile::tempdir().unwrap();
        let app = d.path().join("live.app");
        let new = d.path().join("new.app");
        let old = d.path().join("old.app");
        std::fs::create_dir(&app).unwrap();
        std::fs::create_dir(&new).unwrap();
        std::fs::write(app.join("version"), b"old").unwrap();
        std::fs::write(new.join("version"), b"new").unwrap();
        swap(&new, &app, &old).unwrap();
        assert_eq!(std::fs::read(app.join("version")).unwrap(), b"new");
        assert_eq!(std::fs::read(old.join("version")).unwrap(), b"old");
    }
}
