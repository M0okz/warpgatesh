//! Explicit opt-in for new companion installations; existing agents are preserved.
use crate::installation;
use serde::{Deserialize, Serialize};
use warpgatesh_runtime::storage::{LocalStore, atomic_write};
use warpgatesh_runtime::{RuntimeError, launchd};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSetup {
    pub needs_decision: bool,
    pub enabled: bool,
    pub installed: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Decision {
    Enabled,
    Deferred,
}

fn decision(store: &LocalStore) -> Result<Option<Decision>, RuntimeError> {
    match std::fs::read(store.paths().application_support.join("agent-setup.json")) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn evaluate(enabled: bool, registered: bool, decision: Option<Decision>) -> AgentSetup {
    AgentSetup {
        enabled,
        installed: enabled || registered,
        needs_decision: !enabled && !registered && decision.is_none(),
    }
}

fn active(store: &LocalStore) -> Result<bool, RuntimeError> {
    Ok(
        (store.paths().launch_agent.exists() && launchd::is_loaded()?)
            || warpgatesh_runtime::ipc::request_with_read_timeout(
                &store.paths().agent_socket,
                "status",
                std::time::Duration::from_millis(150),
            )
            .is_ok_and(|response| response.split_whitespace().any(|field| field == "running")),
    )
}

pub fn status(store: &LocalStore) -> Result<AgentSetup, RuntimeError> {
    Ok(evaluate(
        active(store)?,
        store.paths().launch_agent.exists(),
        decision(store)?,
    ))
}

pub fn initialize() -> Result<AgentSetup, RuntimeError> {
    let store = LocalStore::for_current_user()?;
    initialize_for(&store)
}

fn initialize_for(store: &LocalStore) -> Result<AgentSetup, RuntimeError> {
    // Never create a new service during startup. Existing active installations
    // still receive property-list upgrades, including their app association.
    if store.paths().launch_agent.exists() && launchd::is_loaded()? {
        installation::ensure_bundled_agent()?;
    }
    if (store.paths().launch_agent.exists() || active(store)?) && decision(store)?.is_none() {
        atomic_write(
            &store.paths().application_support.join("agent-setup.json"),
            &serde_json::to_vec(&Decision::Enabled)?,
        )?;
    }
    status(store)
}

pub fn set_enabled(enabled: bool) -> Result<AgentSetup, RuntimeError> {
    let store = LocalStore::for_current_user()?;
    if enabled {
        if cfg!(debug_assertions) {
            return Err(RuntimeError::Command("L’installation de l’agent nécessite l’application distribuée. Utilisez le mode démo pour tester cette fenêtre.".to_owned()));
        }
        installation::ensure_bundled_agent()?;
    } else {
        launchd::uninstall(store.paths())?;
    }
    atomic_write(
        &store.paths().application_support.join("agent-setup.json"),
        &serde_json::to_vec(if enabled {
            &Decision::Enabled
        } else {
            &Decision::Deferred
        })?,
    )?;
    status(&store)
}

/// Use foreground operations only when no service is installed or loaded.
/// A stopped or blocked existing agent remains an error, not a silent fallback.
pub fn manual_mode(store: &LocalStore) -> Result<bool, RuntimeError> {
    Ok(!active(store)? && !store.paths().launch_agent.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_new_undecided_installations_are_prompted() {
        assert!(evaluate(false, false, None).needs_decision);
        assert!(!evaluate(true, true, None).needs_decision);
        assert!(!evaluate(true, false, None).needs_decision);
        assert!(!evaluate(false, true, None).needs_decision);
        assert!(!evaluate(false, false, Some(Decision::Deferred)).needs_decision);
        assert!(!evaluate(false, false, Some(Decision::Enabled)).needs_decision);
    }
    #[test]
    fn later_survives_relaunch_without_creating_an_agent() {
        let home = tempfile::TempDir::new().expect("temporary home");
        let store = LocalStore::new(warpgatesh_core::paths::WarpgatePaths::for_home(home.path()));
        assert_eq!(decision(&store).expect("new install"), None);
        assert!(initialize_for(&store).expect("first launch").needs_decision);
        assert!(!store.paths().launch_agent.exists());
        atomic_write(
            &store.paths().application_support.join("agent-setup.json"),
            &serde_json::to_vec(&Decision::Deferred).expect("decision"),
        )
        .expect("save choice");
        assert_eq!(
            decision(&store).expect("relaunch"),
            Some(Decision::Deferred)
        );
        assert!(
            !initialize_for(&store)
                .expect("deferred relaunch")
                .needs_decision
        );
        assert!(!store.paths().launch_agent.exists());
    }
}
