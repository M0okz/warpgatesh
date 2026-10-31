//! Stable app identity for the companion's existing legacy login item.
use plist::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use warpgatesh_runtime::RuntimeError;

const BUNDLE_ID: &str = "dev.warpgatesh.companion";
const LABEL: &str = "WarpgateSH";

fn location() -> Result<(PathBuf, PathBuf), RuntimeError> {
    let store = warpgatesh_runtime::storage::LocalStore::for_current_user()?;
    Ok((
        store
            .paths()
            .launch_agent
            .with_file_name("WarpgateSH.plist"),
        std::env::current_exe()?,
    ))
}

pub fn migrate_existing() -> Result<bool, RuntimeError> {
    // Developer previews must not rewrite the installed application's login item.
    if cfg!(debug_assertions) {
        return Ok(false);
    }
    let (path, executable) = location()?;
    associate_file(&path, &executable)
}

pub fn enable() -> Result<(), RuntimeError> {
    if cfg!(debug_assertions) {
        return Err(RuntimeError::Command(
            "Cette option nécessite l’application distribuée.".to_owned(),
        ));
    }
    let (path, executable) = location()?;
    enable_file(&path, &executable)
}

fn read_owned(path: &Path, executable: &Path) -> Result<Option<Value>, RuntimeError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err(foreign_item());
        }
        Ok(_) => {}
    }
    let expected = executable.to_str().ok_or_else(foreign_item)?;
    let value = Value::from_file(path).map_err(|error| invalid_plist(&error))?;
    let dictionary = value.as_dictionary().ok_or_else(foreign_item)?;
    let arguments = dictionary.get("ProgramArguments").and_then(Value::as_array);
    if dictionary.contains_key("Program")
        || dictionary.get("Label").and_then(Value::as_string) != Some(LABEL)
        || arguments
            .and_then(|args| args.first())
            .and_then(Value::as_string)
            != Some(expected)
    {
        return Err(foreign_item());
    }
    Ok(Some(value))
}

fn association() -> Value {
    Value::Array(vec![Value::String(BUNDLE_ID.to_owned())])
}

fn associate_file(path: &Path, executable: &Path) -> Result<bool, RuntimeError> {
    let Some(mut value) = read_owned(path, executable)? else {
        return Ok(false);
    };
    let dictionary = value.as_dictionary_mut().ok_or_else(foreign_item)?;
    if dictionary.get("AssociatedBundleIdentifiers") == Some(&association()) {
        return Ok(false);
    }
    dictionary.insert("AssociatedBundleIdentifiers".to_owned(), association());
    write_plist(path, &value)?;
    Ok(true)
}

fn enable_file(path: &Path, executable: &Path) -> Result<(), RuntimeError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            associate_file(path, executable)?;
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let executable = executable.to_str().ok_or_else(foreign_item)?;
    let mut dictionary = plist::Dictionary::new();
    dictionary.insert("Label".to_owned(), Value::String(LABEL.to_owned()));
    dictionary.insert(
        "ProgramArguments".to_owned(),
        Value::Array(vec![Value::String(executable.to_owned())]),
    );
    dictionary.insert("RunAtLoad".to_owned(), Value::Boolean(true));
    dictionary.insert("AssociatedBundleIdentifiers".to_owned(), association());
    write_plist(path, &Value::Dictionary(dictionary))
}

fn write_plist(path: &Path, value: &Value) -> Result<(), RuntimeError> {
    let mut bytes = Vec::new();
    value
        .to_writer_xml(&mut bytes)
        .map_err(|error| invalid_plist(&error))?;
    let parent = path.parent().ok_or_else(foreign_item)?;
    fs::create_dir_all(parent)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = parent.join(format!(
        ".warpgatesh-login-{}-{nonce}.tmp",
        std::process::id()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)?;
    let result = (|| {
        file.write_all(&bytes)?;
        if let Ok(metadata) = fs::metadata(path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(Into::into)
}

fn foreign_item() -> RuntimeError {
    RuntimeError::Command(
        "L’élément d’ouverture de session appartient à une autre installation et a été conservé."
            .to_owned(),
    )
}
fn invalid_plist(error: &plist::Error) -> RuntimeError {
    RuntimeError::Command(format!("Invalid companion login item: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    const LEGACY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>WarpgateSH</string>
<key>ProgramArguments</key><array><string>/Applications/WarpgateSH.app/Contents/MacOS/warpgatesh-companion</string><string>--from-autostart</string></array>
<key>RunAtLoad</key><true/>
</dict></plist>"#;
    #[test]
    fn associates_the_real_legacy_autostart_format_without_changing_launch_behavior() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let path = directory.path().join("WarpgateSH.plist");
        std::fs::write(&path, LEGACY).expect("legacy login item");
        let before = plist::Value::from_reader_xml(LEGACY.as_bytes()).expect("legacy plist");
        assert!(
            associate_file(
                &path,
                Path::new("/Applications/WarpgateSH.app/Contents/MacOS/warpgatesh-companion")
            )
            .expect("associate owned login item")
        );
        let mut after = plist::Value::from_file(&path).expect("migrated plist");
        let association = after
            .as_dictionary_mut()
            .expect("dictionary")
            .remove("AssociatedBundleIdentifiers");
        assert_eq!(
            association,
            Some(plist::Value::Array(vec![plist::Value::String(
                "dev.warpgatesh.companion".to_owned()
            )]))
        );
        assert_eq!(after, before, "preserve arguments, label and RunAtLoad");
        assert!(
            !associate_file(
                &path,
                Path::new("/Applications/WarpgateSH.app/Contents/MacOS/warpgatesh-companion")
            )
            .expect("idempotent migration")
        );
    }
    #[test]
    fn new_login_items_are_associated_from_the_first_write() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let path = directory.path().join("WarpgateSH.plist");
        let executable =
            Path::new("/Applications/WarpgateSH.app/Contents/MacOS/warpgatesh-companion");
        assert!(!associate_file(&path, executable).expect("disabled login item"));
        assert!(!path.exists());
        enable_file(&path, executable).expect("explicitly enable login item");
        let value = Value::from_file(&path).expect("enabled plist");
        let dictionary = value.as_dictionary().expect("dictionary");
        assert_eq!(
            dictionary.get("AssociatedBundleIdentifiers"),
            Some(&association())
        );
        assert_eq!(dictionary.get("RunAtLoad"), Some(&Value::Boolean(true)));
        let original = fs::read(&path).expect("file");
        enable_file(&path, executable).expect("already enabled");
        assert_eq!(fs::read(&path).expect("unchanged file"), original);
    }
    #[test]
    fn preserves_other_installations_and_symlink_targets() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let path = directory.path().join("WarpgateSH.plist");
        let foreign = LEGACY.replace(
            "/Applications/WarpgateSH.app/",
            "/Applications/Another.app/",
        );
        fs::write(&path, &foreign).expect("foreign fixture");
        let executable =
            Path::new("/Applications/WarpgateSH.app/Contents/MacOS/warpgatesh-companion");
        assert!(associate_file(&path, executable).is_err());
        assert!(enable_file(&path, executable).is_err());
        assert_eq!(fs::read_to_string(&path).expect("preserved file"), foreign);
        fs::remove_file(&path).expect("remove fixture");
        let target = directory.path().join("linked.plist");
        fs::write(&target, LEGACY).expect("target");
        std::os::unix::fs::symlink(&target, &path).expect("symlink fixture");
        assert!(associate_file(&path, executable).is_err());
        assert_eq!(
            fs::read_to_string(&target).expect("preserved target"),
            LEGACY
        );
    }
    #[test]
    fn association_matches_the_shipping_app() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("Tauri config");
        assert_eq!(config["identifier"].as_str(), Some(BUNDLE_ID));
    }
}
