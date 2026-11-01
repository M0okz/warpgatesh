//! Update wire compatibility without a GUI framework.
use base64::Engine;
use serde::Deserialize;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::time::Duration;

const MAX_ARCHIVE: u64 = 256 * 1024 * 1024;
const MAX_EXPANDED: u64 = 1024 * 1024 * 1024;
const FEED: &str = "https://github.com/M0okz/warpgatesh/releases/latest/download/latest.json";
const IDENTIFIER: &str = "dev.warpgatesh.companion";
const KEY: &str = include_str!("../../../apps/warpgatesh-macos/Bundle/updater.pub");

pub struct Updater {
    client: reqwest::Client,
}
#[derive(Deserialize)]
struct Manifest {
    version: String,
    notes: Option<String>,
    platforms: std::collections::BTreeMap<String, Artifact>,
}
#[derive(Deserialize)]
struct Artifact {
    url: String,
    signature: String,
}
pub struct Update {
    pub version: String,
    pub current_version: String,
    pub body: Option<String>,
    artifact: Artifact,
}
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("WarpgateSH/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| e.to_string())
}
impl Updater {
    pub fn new() -> Result<Self, String> {
        Ok(Self { client: client()? })
    }
    pub async fn check(&self) -> Result<Option<Update>, String> {
        let response = self
            .client
            .get(FEED)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let bytes = bounded_response(response, 1024 * 1024).await?;
        let mut manifest: Manifest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let version = semver::Version::parse(&manifest.version).map_err(|e| e.to_string())?;
        if version
            <= semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?
        {
            return Ok(None);
        }
        let target = if cfg!(target_arch = "aarch64") {
            "darwin-aarch64"
        } else {
            "darwin-x86_64"
        };
        let artifact = manifest
            .platforms
            .remove(target)
            .ok_or("Update platform missing")?;
        if !artifact.url.starts_with("https://") || artifact.signature.is_empty() {
            return Err("Invalid update artifact".into());
        }
        Ok(Some(Update {
            version: manifest.version,
            current_version: env!("CARGO_PKG_VERSION").into(),
            body: manifest.notes,
            artifact,
        }))
    }
}
async fn bounded_response(mut response: reqwest::Response, limit: u64) -> Result<Vec<u8>, String> {
    if response.content_length().is_some_and(|s| s > limit) {
        return Err("Response too large".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() as u64 + chunk.len() as u64 > limit {
            return Err("Response too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
impl Update {
    pub async fn download(
        &self,
        mut progress: impl FnMut(usize, Option<u64>),
        complete: impl FnOnce(),
    ) -> Result<Vec<u8>, String> {
        let mut response = client()?
            .get(&self.artifact.url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let total = response.content_length();
        if total.is_some_and(|length| length > MAX_ARCHIVE) {
            return Err("Update archive too large".into());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            if bytes.len() as u64 + chunk.len() as u64 > MAX_ARCHIVE {
                return Err("Update archive too large".into());
            }
            progress(chunk.len(), total);
            bytes.extend_from_slice(&chunk);
        }
        verify(&bytes, &self.artifact.signature, KEY)?;
        complete();
        Ok(bytes)
    }
    pub fn install(self, bytes: &[u8]) -> Result<(), String> {
        verify(bytes, &self.artifact.signature, KEY)?;
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let application = executable
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or("Missing app bundle")?;
        install_archive(bytes, application, &self.version)
    }
}
fn verify(bytes: &[u8], signature: &str, key: &str) -> Result<(), String> {
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(signature.trim())
        .map_err(|e| e.to_string())?;
    let signature = minisign_verify::Signature::decode(
        std::str::from_utf8(&decoded).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    minisign_verify::PublicKey::decode(key)
        .map_err(|e| e.to_string())?
        .verify(bytes, &signature, false)
        .map_err(|e| e.to_string())
}
fn extract(bytes: &[u8], destination: &Path) -> Result<(), String> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder.take(MAX_EXPANDED + 1));
    let mut total = 0u64;
    let mut count = 0;
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        count += 1;
        total = total
            .checked_add(entry.size())
            .ok_or("Expanded archive too large")?;
        if count > 20_000 || total > MAX_EXPANDED {
            return Err("Expanded archive too large".into());
        }
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        if !safe_path(&path) {
            return Err("Unsafe archive path".into());
        }
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            return Err("Unsupported archive entry".into());
        }
        if !entry.unpack_in(destination).map_err(|e| e.to_string())? {
            return Err("Unsafe archive entry".into());
        }
    }
    if !destination
        .join("WarpgateSH.app/Contents/Info.plist")
        .is_file()
    {
        return Err("Missing app bundle".into());
    }
    Ok(())
}
fn safe_path(path: &Path) -> bool {
    use std::path::Component;
    let mut components = path.components();
    matches!(components.next(), Some(Component::Normal(n)) if n == "WarpgateSH.app")
        && components.all(|c| matches!(c, Component::Normal(_)))
}
fn checked(args: &[&std::ffi::OsStr], command: &str) -> Result<(), String> {
    let result = std::process::Command::new(command)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if result.status.success() {
        Ok(())
    } else {
        Err(format!("Validation failed: {command}"))
    }
}
fn install_archive(bytes: &[u8], application: &Path, version: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    if application != Path::new("/Applications/WarpgateSH.app") {
        return Err("Update requires the installed application".into());
    }
    let store =
        warpgatesh_runtime::storage::LocalStore::for_current_user().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&store.paths().application_support).map_err(|e| e.to_string())?;
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(store.paths().application_support.join("app-update.lock"))
        .map_err(|e| e.to_string())?;
    fs2::FileExt::try_lock_exclusive(&lock).map_err(|_| "Another update is already running")?;
    let staging = tempfile::Builder::new()
        .prefix(".warpgatesh-update-")
        .tempdir_in(application.parent().ok_or("No parent")?)
        .map_err(|e| e.to_string())?;
    extract(bytes, staging.path())?;
    let candidate = staging.path().join("WarpgateSH.app");
    let info = plist::Value::from_file(candidate.join("Contents/Info.plist"))
        .map_err(|e| e.to_string())?;
    let info = info.as_dictionary().ok_or("Invalid bundle metadata")?;
    for (name, value) in [
        ("CFBundleIdentifier", IDENTIFIER),
        ("CFBundleExecutable", "warpgatesh-ui"),
        ("CFBundleShortVersionString", version),
    ] {
        if info.get(name).and_then(plist::Value::as_string) != Some(value) {
            return Err(format!("Unexpected {name}"));
        }
    }
    let requirement = format!(
        "anchor apple generic and identifier \"{IDENTIFIER}\" and certificate leaf[subject.OU] = \"5FAF9ZA2X9\""
    );
    checked(
        &[
            OsStr::new("--verify"),
            OsStr::new("--deep"),
            OsStr::new("--strict"),
            OsStr::new("-R"),
            OsStr::new(&requirement),
            candidate.as_os_str(),
        ],
        "/usr/bin/codesign",
    )?;
    checked(
        &[
            OsStr::new("--assess"),
            OsStr::new("--type"),
            OsStr::new("execute"),
            candidate.as_os_str(),
        ],
        "/usr/sbin/spctl",
    )?;
    let directory = staging.keep();
    let result = crate::transaction::replace(&candidate, application, &directory, version);
    if result.is_err() && !directory.join("previous.app").exists() {
        let _ = std::fs::remove_dir_all(directory);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_paths_outside_the_single_bundle() {
        for value in [
            "/Applications/WarpgateSH.app",
            "../WarpgateSH.app",
            "WarpgateSH.app/../outside",
            "other.app/Contents",
        ] {
            assert!(!safe_path(Path::new(value)));
        }
        assert!(safe_path(Path::new(
            "WarpgateSH.app/Contents/MacOS/warpgatesh-ui"
        )));
    }
    #[test]
    fn extraction_rejects_links_without_writing_outside_staging() {
        let mut bytes = Vec::new();
        {
            let gzip = flate2::write::GzEncoder::new(&mut bytes, flate2::Compression::default());
            let mut archive = tar::Builder::new(gzip);
            let mut header = tar::Header::new_gnu();
            header.set_path("WarpgateSH.app/escape").unwrap();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_link_name("../../outside").unwrap();
            header.set_size(0);
            header.set_mode(0o777);
            header.set_cksum();
            archive.append(&header, std::io::empty()).unwrap();
            archive.into_inner().unwrap().finish().unwrap();
        }
        let directory = tempfile::tempdir().unwrap();
        assert!(extract(&bytes, directory.path()).is_err());
        assert!(!directory.path().join("WarpgateSH.app/escape").exists());
    }
    #[test]
    fn rejects_invalid_signatures_before_extracting() {
        assert!(verify(b"invalid", "invalid", KEY).is_err());
    }
}
