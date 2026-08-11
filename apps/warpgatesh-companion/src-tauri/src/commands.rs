use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use warpgatesh_runtime::ipc;
use warpgatesh_runtime::storage::LocalStore;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanionState {
    agent_running: bool,
    profiles: Vec<CompanionProfile>,
    targets: Vec<CompanionTarget>,
    last_sync_age_seconds: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompanionProfile {
    name: String,
    username: String,
    base_url: String,
    is_default: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompanionTarget {
    alias: String,
    qualified_alias: String,
    name: String,
    profile: String,
}

#[tauri::command]
pub async fn get_companion_state() -> Result<CompanionState, String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    load_state(&store)
}

#[tauri::command]
pub async fn sync_now() -> Result<String, String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    ipc::request_with_retry(&store.paths().agent_socket, "sync", Duration::from_secs(10))
        .map_err(display_error)
}

#[tauri::command]
pub async fn open_target(alias: String) -> Result<(), String> {
    let store = LocalStore::for_current_user().map_err(display_error)?;
    let snapshot = store
        .load_snapshot()
        .map_err(display_error)?
        .ok_or_else(|| "No synchronized SSH target is available.".to_owned())?;
    let known = snapshot.targets.iter().any(|target| {
        target.qualified_alias == alias || target.short_alias.as_deref() == Some(alias.as_str())
    });
    if !known {
        return Err("The selected SSH alias is not in the local snapshot.".to_owned());
    }

    #[cfg(target_os = "macos")]
    let status = Command::new("/usr/bin/open")
        .arg(format!("ssh://{alias}"))
        .status()
        .map_err(display_error)?;

    #[cfg(target_os = "linux")]
    let status = Command::new("xdg-open")
        .arg(format!("ssh://{alias}"))
        .status()
        .map_err(display_error)?;

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err("Opening SSH targets is unsupported on this platform.".to_owned());

    if status.success() {
        Ok(())
    } else {
        Err(format!("The terminal launcher exited with {status}."))
    }
}

fn load_state(store: &LocalStore) -> Result<CompanionState, String> {
    let catalog = store.load_profiles().map_err(display_error)?;
    let snapshot = store.load_snapshot().map_err(display_error)?;
    let profiles = catalog
        .profiles
        .into_iter()
        .map(|profile| CompanionProfile {
            is_default: catalog.default_profile.as_deref() == Some(profile.name.as_str()),
            name: profile.name,
            username: profile.username,
            base_url: profile.base_url,
        })
        .collect();
    let (targets, last_sync_age_seconds) = snapshot.map_or_else(
        || (Vec::new(), None),
        |snapshot| {
            let targets = snapshot
                .targets
                .into_iter()
                .map(|target| CompanionTarget {
                    alias: target
                        .short_alias
                        .unwrap_or_else(|| target.qualified_alias.clone()),
                    qualified_alias: target.qualified_alias,
                    name: target.name,
                    profile: target.profile,
                })
                .collect();
            let age = epoch_seconds().saturating_sub(snapshot.synchronized_at_epoch_seconds);
            (targets, Some(age))
        },
    );
    let agent_running = ipc::request(&store.paths().agent_socket, "status").is_ok();

    Ok(CompanionState {
        agent_running,
        profiles,
        targets,
        last_sync_age_seconds,
    })
}

fn epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;
    use warpgatesh_core::paths::WarpgatePaths;
    use warpgatesh_core::profiles::{Profile, ProfileCatalog};
    use warpgatesh_runtime::storage::{SNAPSHOT_SCHEMA_VERSION, Snapshot, SyncedTarget};

    use super::*;

    #[test]
    fn maps_the_local_snapshot_without_exposing_secrets() {
        let home = TempDir::new().expect("temporary home");
        let store = LocalStore::new(WarpgatePaths::for_home(home.path()));
        let mut catalog = ProfileCatalog::default();
        catalog
            .upsert(Profile {
                name: "homeblack".to_owned(),
                base_url: "https://warpgate.example/".to_owned(),
                username: "gregory".to_owned(),
                warpgate_version: Some("0.27.1".to_owned()),
                ssh_host: "10.60.0.17".to_owned(),
                ssh_port: 2222,
            })
            .expect("profile");
        store.save_profiles(&catalog).expect("save profiles");
        store
            .save_snapshot(&Snapshot {
                schema_version: SNAPSHOT_SCHEMA_VERSION,
                synchronized_at_epoch_seconds: epoch_seconds(),
                targets: vec![SyncedTarget {
                    profile: "homeblack".to_owned(),
                    target_id: "target-1".to_owned(),
                    name: "dmz-nextcloud-01".to_owned(),
                    short_alias: None,
                    qualified_alias: "dmz-nextcloud-01.homeblack".to_owned(),
                }],
            })
            .expect("save snapshot");

        let state = load_state(&store).expect("companion state");
        assert_eq!(state.profiles.len(), 1);
        assert_eq!(state.targets.len(), 1);
        assert_eq!(state.targets[0].alias, "dmz-nextcloud-01.homeblack");
        assert!(!state.agent_running);
    }
}
