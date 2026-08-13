use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::RuntimeError;
use crate::storage::LocalStore;

const LOG_SCHEMA_VERSION: u32 = 1;
const RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticFileSummary {
    pub name: String,
    pub bytes: u64,
    pub events: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsPreview {
    pub log_directory: String,
    pub retention_days: u8,
    pub total_bytes: u64,
    pub total_events: u64,
    pub files: Vec<DiagnosticFileSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DiagnosticEvent {
    schema_version: u32,
    timestamp: String,
    level: String,
    component: String,
    event: String,
    fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug)]
pub struct DiagnosticLogger {
    directory: PathBuf,
    component: String,
}

impl DiagnosticLogger {
    /// Create a logger using the current user's platform log directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the user's storage paths cannot be resolved.
    pub fn for_current_user(component: &str) -> Result<Self, RuntimeError> {
        let store = LocalStore::for_current_user()?;
        Ok(Self::new(&store.paths().logs_directory, component))
    }

    #[must_use]
    pub fn new(directory: &Path, component: &str) -> Self {
        Self {
            directory: directory.to_owned(),
            component: safe_identifier(component),
        }
    }

    /// Append a sanitized structured event and prune logs older than seven days.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory or log file cannot be written.
    pub fn record(
        &self,
        level: &str,
        event: &str,
        fields: BTreeMap<String, Value>,
    ) -> Result<(), RuntimeError> {
        fs::create_dir_all(&self.directory)?;
        secure_directory(&self.directory)?;
        prune_directory(&self.directory, SystemTime::now())?;

        let now = Utc::now();
        let path = self.directory.join(format!(
            "{}-{}.jsonl",
            self.component,
            now.format("%Y-%m-%d")
        ));
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        secure_file(&path)?;
        let entry = DiagnosticEvent {
            schema_version: LOG_SCHEMA_VERSION,
            timestamp: now.to_rfc3339_opts(SecondsFormat::Secs, true),
            level: safe_identifier(level),
            component: self.component.clone(),
            event: safe_identifier(event),
            fields: sanitize_fields(fields),
        };
        let mut line = serde_json::to_vec(&entry)?;
        line.push(b'\n');
        file.write_all(&line)?;
        Ok(())
    }

    pub fn info(&self, event: &str) {
        let _ = self.record("info", event, BTreeMap::new());
    }
}

/// Summarize the structured files that would be exported.
///
/// # Errors
///
/// Returns an error when the log directory cannot be read.
pub fn preview(store: &LocalStore) -> Result<DiagnosticsPreview, RuntimeError> {
    preview_directory(&store.paths().logs_directory)
}

/// Export sanitized logs and a manifest into a ZIP archive in Downloads.
///
/// # Errors
///
/// Returns an error when logs cannot be read or the archive cannot be written.
pub fn export(store: &LocalStore) -> Result<PathBuf, RuntimeError> {
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        RuntimeError::InvalidInput("the current user has no HOME directory".to_owned())
    })?;
    let downloads = home.join("Downloads");
    let destination = if downloads.is_dir() { downloads } else { home };
    let path = destination.join(format!(
        "WarpgateSH-diagnostics-{}.zip",
        Utc::now().format("%Y%m%d-%H%M%S-%3f")
    ));
    export_to(store, &path)?;
    Ok(path)
}

/// Write a sanitized diagnostics archive to an explicit destination.
///
/// # Errors
///
/// Returns an error when logs cannot be read or the archive cannot be written.
pub fn export_to(store: &LocalStore, destination: &Path) -> Result<(), RuntimeError> {
    let preview = preview(store)?;
    let mut archive_manifest = preview.clone();
    "local WarpgateSH log directory".clone_into(&mut archive_manifest.log_directory);
    let file = File::create(destination)?;
    secure_file(destination)?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    archive
        .start_file("manifest.json", options)
        .map_err(|error| zip_error(&error))?;
    archive.write_all(&serde_json::to_vec_pretty(&archive_manifest)?)?;

    for summary in &preview.files {
        let source = store.paths().logs_directory.join(&summary.name);
        archive
            .start_file(format!("logs/{}", summary.name), options)
            .map_err(|error| zip_error(&error))?;
        let reader = BufReader::new(File::open(source)?);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let Ok(mut value) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            sanitize_value(None, &mut value);
            serde_json::to_writer(&mut archive, &value)?;
            archive.write_all(b"\n")?;
        }
    }
    archive.finish().map_err(|error| zip_error(&error))?;
    Ok(())
}

fn preview_directory(directory: &Path) -> Result<DiagnosticsPreview, RuntimeError> {
    if !directory.exists() {
        return Ok(DiagnosticsPreview {
            log_directory: directory.display().to_string(),
            retention_days: 7,
            total_bytes: 0,
            total_events: 0,
            files: Vec::new(),
        });
    }
    prune_directory(directory, SystemTime::now())?;
    let mut files = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !is_structured_log(&path) {
            continue;
        }
        let events = BufReader::new(File::open(&path)?)
            .lines()
            .filter(|line| {
                line.as_ref()
                    .is_ok_and(|line| serde_json::from_str::<Value>(line).is_ok())
            })
            .count() as u64;
        files.push(DiagnosticFileSummary {
            name: entry.file_name().to_string_lossy().into_owned(),
            bytes: entry.metadata()?.len(),
            events,
        });
    }
    files.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(DiagnosticsPreview {
        log_directory: directory.display().to_string(),
        retention_days: 7,
        total_bytes: files.iter().map(|file| file.bytes).sum(),
        total_events: files.iter().map(|file| file.events).sum(),
        files,
    })
}

fn prune_directory(directory: &Path, now: SystemTime) -> Result<(), RuntimeError> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !is_structured_log(&path) {
            continue;
        }
        let modified = entry.metadata()?.modified()?;
        if now.duration_since(modified).unwrap_or_default() > RETENTION {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn is_structured_log(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "jsonl")
        && path.file_name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name.starts_with("agent-") || name.starts_with("companion-")
        })
}

fn sanitize_fields(fields: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    fields
        .into_iter()
        .map(|(key, mut value)| {
            sanitize_value(Some(&key), &mut value);
            (key, value)
        })
        .collect()
}

fn sanitize_value(key: Option<&str>, value: &mut Value) {
    if key.is_some_and(is_sensitive_key) {
        *value = json!("[REDACTED]");
        return;
    }
    match value {
        Value::String(text) => *text = redact_text(text),
        Value::Array(values) => {
            for value in values {
                sanitize_value(None, value);
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                sanitize_value(Some(key), value);
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized: String = key
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    [
        "token",
        "password",
        "secret",
        "authorization",
        "privatekey",
        "credential",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn redact_text(text: &str) -> String {
    let mut result = text.to_owned();
    for marker in [
        "bearer ",
        "token=",
        "password=",
        "secret=",
        "authorization=",
    ] {
        let mut cursor = 0;
        loop {
            let lower = result.to_ascii_lowercase();
            let Some(offset) = lower[cursor..].find(marker) else {
                break;
            };
            let start = cursor + offset;
            let value_start = start + marker.len();
            let value_end = result[value_start..]
                .find(char::is_whitespace)
                .map_or(result.len(), |offset| value_start + offset);
            result.replace_range(value_start..value_end, "[REDACTED]");
            cursor = value_start + "[REDACTED]".len();
        }
    }
    result
}

fn safe_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
        .take(64)
        .collect()
}

fn zip_error(error: &zip::result::ZipError) -> RuntimeError {
    RuntimeError::Command(format!("could not create diagnostics archive: {error}"))
}

#[cfg(unix)]
fn secure_directory(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn secure_directory(_path: &Path) -> Result<(), RuntimeError> {
    Ok(())
}

#[cfg(unix)]
fn secure_file(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn secure_file(_path: &Path) -> Result<(), RuntimeError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use warpgatesh_core::paths::WarpgatePaths;

    #[test]
    fn writes_structured_sanitized_events() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let logger = DiagnosticLogger::new(directory.path(), "agent");
        let mut fields = BTreeMap::new();
        fields.insert("token".to_owned(), json!("api-secret"));
        fields.insert("apiToken".to_owned(), json!("another-secret"));
        fields.insert(
            "message".to_owned(),
            json!("authorization=api-secret failed"),
        );
        logger
            .record("error", "sync.failed", fields)
            .expect("write event");

        let path = fs::read_dir(directory.path())
            .expect("log directory")
            .next()
            .expect("log entry")
            .expect("read log entry")
            .path();
        let text = fs::read_to_string(path).expect("log text");
        assert!(text.contains("sync.failed"));
        assert!(!text.contains("api-secret"));
        assert!(!text.contains("another-secret"));
        assert!(text.contains("[REDACTED]"));
    }

    #[test]
    fn exports_only_sanitized_structured_logs() {
        let home = tempfile::tempdir().expect("temporary home");
        let store = LocalStore::new(WarpgatePaths::for_home(home.path()));
        let logger = DiagnosticLogger::new(&store.paths().logs_directory, "companion");
        let mut fields = BTreeMap::new();
        fields.insert("password".to_owned(), json!("do-not-export"));
        logger
            .record("info", "app.started", fields)
            .expect("write event");
        let destination = home.path().join("diagnostics.zip");
        export_to(&store, &destination).expect("export archive");

        let file = File::open(destination).expect("archive");
        let mut archive = zip::ZipArchive::new(file).expect("open archive");
        let mut manifest = String::new();
        std::io::Read::read_to_string(
            &mut archive.by_name("manifest.json").expect("manifest"),
            &mut manifest,
        )
        .expect("read manifest");
        assert!(!manifest.contains(&home.path().display().to_string()));
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut archive.by_index(1).expect("log entry"), &mut contents)
            .expect("read log entry");
        assert!(!contents.contains("do-not-export"));
        assert!(contents.contains("[REDACTED]"));
    }

    #[test]
    fn removes_structured_logs_after_seven_days() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let logger = DiagnosticLogger::new(directory.path(), "agent");
        logger.info("agent.started");
        let future = SystemTime::now() + RETENTION + Duration::from_secs(1);

        prune_directory(directory.path(), future).expect("prune logs");

        assert_eq!(
            fs::read_dir(directory.path())
                .expect("log directory")
                .count(),
            0
        );
    }

    #[test]
    fn ignores_unrelated_json_lines_files() {
        let directory = tempfile::tempdir().expect("temporary directory");
        fs::write(directory.path().join("application-data.jsonl"), "{}\n").expect("unrelated file");

        let preview = preview_directory(directory.path()).expect("preview");

        assert!(preview.files.is_empty());
    }
}
