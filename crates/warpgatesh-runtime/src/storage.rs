use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use warpgatesh_core::paths::WarpgatePaths;
use warpgatesh_core::profiles::ProfileCatalog;

use crate::RuntimeError;

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const PREFERENCES_SCHEMA_VERSION: u32 = 1;
pub const AGENT_STATUS_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Preferences {
    pub schema_version: u32,
    pub sync_interval_seconds: u64,
    pub launch_companion_at_login: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: PREFERENCES_SCHEMA_VERSION,
            sync_interval_seconds: 5 * 60,
            launch_companion_at_login: false,
        }
    }
}

impl Preferences {
    /// Validate values shared by the agent and companion.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the schema or synchronization interval is invalid.
    pub fn validate(&self) -> Result<(), RuntimeError> {
        if self.schema_version != PREFERENCES_SCHEMA_VERSION {
            return Err(RuntimeError::Incompatible(format!(
                "unsupported preferences schema version {}",
                self.schema_version
            )));
        }
        if !(60..=86_400).contains(&self.sync_interval_seconds) {
            return Err(RuntimeError::InvalidInput(
                "the synchronization interval must be between 60 and 86400 seconds".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorKind {
    Unauthorized,
    ApiUnreachable,
    HostKey,
    Incompatible,
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentStatus {
    pub schema_version: u32,
    pub last_attempt_epoch_seconds: u64,
    pub last_success_epoch_seconds: Option<u64>,
    pub last_error_kind: Option<AgentErrorKind>,
    pub last_error_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Snapshot {
    pub schema_version: u32,
    pub synchronized_at_epoch_seconds: u64,
    pub targets: Vec<SyncedTarget>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SyncedTarget {
    pub profile: String,
    pub target_id: String,
    pub name: String,
    pub short_alias: Option<String>,
    pub qualified_alias: String,
}

#[derive(Clone, Debug)]
pub struct LocalStore {
    paths: WarpgatePaths,
}

impl LocalStore {
    #[must_use]
    pub const fn new(paths: WarpgatePaths) -> Self {
        Self { paths }
    }

    /// Resolve storage paths from the current user's home directory.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] if no home directory is available.
    pub fn for_current_user() -> Result<Self, RuntimeError> {
        let home = std::env::var_os("HOME").ok_or_else(|| {
            RuntimeError::InvalidInput("the current user has no HOME directory".to_owned())
        })?;
        Ok(Self::new(WarpgatePaths::for_home(Path::new(&home))))
    }

    #[must_use]
    pub const fn paths(&self) -> &WarpgatePaths {
        &self.paths
    }

    /// Load the non-secret profile catalog.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the file cannot be read or validated.
    pub fn load_profiles(&self) -> Result<ProfileCatalog, RuntimeError> {
        if !self.paths.profiles.exists() {
            return Ok(ProfileCatalog::default());
        }
        let catalog: ProfileCatalog = serde_json::from_slice(&fs::read(&self.paths.profiles)?)?;
        catalog.validate()?;
        Ok(catalog)
    }

    /// Persist the non-secret profile catalog atomically.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when serialization or writing fails.
    pub fn save_profiles(&self, catalog: &ProfileCatalog) -> Result<(), RuntimeError> {
        catalog.validate()?;
        atomic_write_json(&self.paths.profiles, catalog)
    }

    /// Load persisted user preferences, or defaults on first launch.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when persisted preferences cannot be read or validated.
    pub fn load_preferences(&self) -> Result<Preferences, RuntimeError> {
        if !self.paths.preferences.exists() {
            return Ok(Preferences::default());
        }
        let preferences: Preferences = serde_json::from_slice(&fs::read(&self.paths.preferences)?)?;
        preferences.validate()?;
        Ok(preferences)
    }

    /// Persist validated preferences atomically.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when preferences are invalid or cannot be written.
    pub fn save_preferences(&self, preferences: &Preferences) -> Result<(), RuntimeError> {
        preferences.validate()?;
        atomic_write_json(&self.paths.preferences, preferences)
    }

    /// Load the latest health report written by the background agent.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the report cannot be read or has an unsupported schema.
    pub fn load_agent_status(&self) -> Result<Option<AgentStatus>, RuntimeError> {
        if !self.paths.agent_status.exists() {
            return Ok(None);
        }
        let status: AgentStatus = serde_json::from_slice(&fs::read(&self.paths.agent_status)?)?;
        if status.schema_version != AGENT_STATUS_SCHEMA_VERSION {
            return Err(RuntimeError::Incompatible(format!(
                "unsupported agent status schema version {}",
                status.schema_version
            )));
        }
        Ok(Some(status))
    }

    /// Persist the latest background-agent health report atomically.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the report cannot be written.
    pub fn save_agent_status(&self, status: &AgentStatus) -> Result<(), RuntimeError> {
        atomic_write_json(&self.paths.agent_status, status)
    }

    /// Load the last complete synchronization snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the file cannot be read or decoded.
    pub fn load_snapshot(&self) -> Result<Option<Snapshot>, RuntimeError> {
        if !self.paths.snapshot.exists() {
            return Ok(None);
        }
        let snapshot: Snapshot = serde_json::from_slice(&fs::read(&self.paths.snapshot)?)?;
        if snapshot.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(RuntimeError::Incompatible(format!(
                "unsupported snapshot schema version {}",
                snapshot.schema_version
            )));
        }
        Ok(Some(snapshot))
    }

    /// Persist a complete synchronization snapshot atomically.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when serialization or writing fails.
    pub fn save_snapshot(&self, snapshot: &Snapshot) -> Result<(), RuntimeError> {
        atomic_write_json(&self.paths.snapshot, snapshot)
    }
}

/// Write bytes by replacing the destination atomically on the same filesystem.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the parent or file cannot be written.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    let parent = path.parent().ok_or_else(|| {
        RuntimeError::InvalidInput(format!("{} has no parent directory", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    set_private_directory_permissions(parent)?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = temporary_path(path, nonce);
    let result = write_and_replace(&temporary, path, bytes);
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn atomic_write_json(path: &Path, value: &impl Serialize) -> Result<(), RuntimeError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

fn temporary_path(path: &Path, nonce: u128) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("warpgatesh");
    path.with_file_name(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()))
}

fn write_and_replace(
    temporary: &Path,
    destination: &Path,
    bytes: &[u8],
) -> Result<(), RuntimeError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, destination)?;
    Ok(())
}

fn set_private_directory_permissions(path: &Path) -> Result<(), RuntimeError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use warpgatesh_core::profiles::Profile;

    use super::*;

    fn store(directory: &TempDir) -> LocalStore {
        LocalStore::new(WarpgatePaths::for_home(directory.path()))
    }

    #[test]
    fn missing_profile_file_is_an_empty_catalog() {
        let directory = TempDir::new().expect("temporary directory");
        assert_eq!(
            store(&directory).load_profiles().expect("empty catalog"),
            ProfileCatalog::default()
        );
    }

    #[test]
    fn profile_catalog_round_trips_without_a_token() {
        let directory = TempDir::new().expect("temporary directory");
        let store = store(&directory);
        let mut catalog = ProfileCatalog::default();
        catalog
            .upsert(Profile {
                name: "lab".to_owned(),
                base_url: "https://warpgate.example/".to_owned(),
                username: "gregory".to_owned(),
                warpgate_version: Some("0.27.0".to_owned()),
                ssh_host: "ssh.warpgate.example".to_owned(),
                ssh_port: 2222,
            })
            .expect("valid profile");

        store.save_profiles(&catalog).expect("save catalog");
        assert_eq!(store.load_profiles().expect("load catalog"), catalog);
        let persisted = fs::read_to_string(&store.paths().profiles).expect("profile file");
        assert!(!persisted.contains("token"));
    }

    #[test]
    fn preferences_default_and_round_trip() {
        let directory = TempDir::new().expect("temporary directory");
        let store = store(&directory);
        assert_eq!(
            store.load_preferences().expect("defaults"),
            Preferences::default()
        );

        let preferences = Preferences {
            sync_interval_seconds: 900,
            launch_companion_at_login: true,
            ..Preferences::default()
        };
        store
            .save_preferences(&preferences)
            .expect("save preferences");
        assert_eq!(
            store.load_preferences().expect("load preferences"),
            preferences
        );
    }
}
