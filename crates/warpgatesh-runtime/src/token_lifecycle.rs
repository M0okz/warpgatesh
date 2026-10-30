//! Durable token rotation. Secrets, pending transitions and retired credentials live only in
//! the native secret store; the JSON status cache contains no credentials.
use crate::{
    RuntimeError,
    api::ApiClient,
    configuration::{ConfigurationMutation, LocalConfiguration},
    configuration_lock::ConfigurationLock,
    keychain::TokenStore,
    storage::{LocalStore, atomic_write},
    token_api::{ApiAuthorization, IssuedToken, TokenApi},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use warpgatesh_core::profiles::Profile;

const DEFAULT_LIFETIME: u64 = 30 * 86_400;
const RETIREMENT_GRACE: u64 = 300;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedTokenStatus {
    pub managed: bool,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub renewal_failures: u32,
    #[serde(default)]
    pub next_attempt_epoch_seconds: Option<u64>,
    pub automatic_renewal: bool,
    pub expires_at_epoch_seconds: Option<u64>,
    pub renew_after_epoch_seconds: Option<u64>,
}

// Never derive Debug for records containing secrets.
#[derive(Clone, Deserialize, Serialize)]
struct Enrollment {
    profile: Profile,
    known_hosts: String,
}
#[derive(Clone, Deserialize, Serialize)]
struct Transition {
    authorization: ApiAuthorization,
    previous_secret: Option<String>,
    label: String,
    requested_expiry: u64,
    candidate: Option<IssuedToken>,
    enrollment: Option<Enrollment>,
    #[serde(default)]
    cancelled: bool,
}
#[derive(Clone, Deserialize, Serialize)]
struct Retirement {
    id: String,
    not_before: u64,
}
#[derive(Clone, Deserialize, Serialize)]
struct Record {
    schema_version: u32,
    base_url: String,
    username: String,
    automatic: bool,
    active: Option<IssuedToken>,
    transition: Option<Transition>,
    retirements: Vec<Retirement>,
}

fn credential_key(name: &str) -> String {
    format!("managed-token::{name}")
}
fn cache_path(store: &LocalStore) -> std::path::PathBuf {
    store.paths().application_support.join("token-status.json")
}

/// Read non-secret token management state for the interface.
/// # Errors
/// Returns an error for an invalid cache; no secret-store read is performed here.
pub fn statuses(store: &LocalStore) -> Result<BTreeMap<String, ManagedTokenStatus>, RuntimeError> {
    match fs::read(cache_path(store)) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(e.into()),
    }
}
fn update_cache(
    store: &LocalStore,
    name: &str,
    status: Option<ManagedTokenStatus>,
) -> Result<(), RuntimeError> {
    let mut cache = statuses(store)?;
    if let Some(status) = status {
        cache.insert(name.to_owned(), status);
    } else {
        cache.remove(name);
    }
    atomic_write(&cache_path(store), &serde_json::to_vec_pretty(&cache)?)
}
/// Forget local management metadata when removing a profile or all user data.
/// # Errors
/// Returns an error if secret storage or the status cache cannot be updated.
pub fn clear_management<T: TokenStore>(
    store: &LocalStore,
    tokens: &T,
    name: &str,
) -> Result<(), RuntimeError> {
    tokens.delete(&credential_key(name))?;
    update_cache(store, name, None)
}
fn read_record<T: TokenStore>(tokens: &T, name: &str) -> Result<Option<Record>, RuntimeError> {
    tokens
        .get_optional(&credential_key(name))?
        .map(|json| {
            let record: Record = serde_json::from_str(&json).map_err(|_| {
                RuntimeError::Keychain("invalid managed credential state".to_owned())
            })?;
            if record.schema_version != 1 {
                return Err(RuntimeError::Incompatible(
                    "unsupported managed credential schema".to_owned(),
                ));
            }
            Ok(record)
        })
        .transpose()
}
fn status(record: &Record) -> Result<ManagedTokenStatus, RuntimeError> {
    let expiry = record
        .active
        .as_ref()
        .map(|t| t.token.expiry_epoch())
        .transpose()?;
    let renew = record
        .active
        .as_ref()
        .map(|token| {
            let expiry = token.token.expiry_epoch()?;
            let created = chrono::DateTime::parse_from_rfc3339(&token.token.created)
                .ok()
                .and_then(|d| u64::try_from(d.timestamp()).ok())
                .unwrap_or(expiry.saturating_sub(DEFAULT_LIFETIME));
            let lifetime = expiry.saturating_sub(created);
            let lead = (lifetime / 5).clamp(120, 7 * 86_400).min(lifetime / 2);
            Ok::<u64, RuntimeError>(expiry.saturating_sub(lead))
        })
        .transpose()?;
    Ok(ManagedTokenStatus {
        managed: record.active.is_some(),
        last_error: None,
        renewal_failures: 0,
        next_attempt_epoch_seconds: None,
        automatic_renewal: record.automatic,
        expires_at_epoch_seconds: expiry,
        renew_after_epoch_seconds: renew,
    })
}
fn persist<T: TokenStore>(
    store: &LocalStore,
    tokens: &T,
    name: &str,
    record: &Record,
) -> Result<(), RuntimeError> {
    tokens.set(&credential_key(name), &serde_json::to_string(record)?)?;
    let mut value = status(record)?;
    if value.automatic_renewal {
        if let Some(previous) = statuses(store)?.get(name) {
            if previous.expires_at_epoch_seconds == value.expires_at_epoch_seconds
                || record.transition.is_some()
            {
                value.last_error.clone_from(&previous.last_error);
                value.renewal_failures = previous.renewal_failures;
                value.next_attempt_epoch_seconds = previous.next_attempt_epoch_seconds;
            }
        }
    }
    update_cache(store, name, Some(value))
}
fn unique_label(name: &str) -> Result<String, RuntimeError> {
    let mut bytes = [0u8; 16];
    fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let suffix: String = bytes
        .iter()
        .flat_map(|b| [b >> 4, b & 15])
        .map(|b| char::from(b"0123456789abcdef"[usize::from(b)]))
        .collect();
    Ok(format!("WarpgateSH/{name}/{suffix}"))
}
fn identity_matches<A: TokenApi>(
    api: &A,
    auth: &ApiAuthorization,
    username: &str,
) -> Result<(), RuntimeError> {
    if api.identity(auth)?.username != username {
        return Err(RuntimeError::InvalidInput(
            "Reconnect with the same Warpgate identity as this profile.".to_owned(),
        ));
    }
    Ok(())
}

/// Create or rotate an application-managed credential, retaining the usable old credential until
/// the replacement has been validated and journaled in the native secret store.
/// # Errors
/// Returns an error on authentication, network, secret-store or configuration failures.
pub fn rotate<T: TokenStore, A: TokenApi>(
    store: &LocalStore,
    tokens: &T,
    api: &A,
    name: &str,
    session: Option<&str>,
    now: u64,
) -> Result<ManagedTokenStatus, RuntimeError> {
    let _guard = ConfigurationLock::acquire(store)?;
    let profile = store
        .load_profiles()?
        .find(name)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidInput("unknown profile".to_owned()))?;
    rotate_locked(store, tokens, api, &profile, session, now, None)
}

/// Enroll a token only after the caller has verified the SSH fingerprints and the user saved.
/// # Errors
/// Returns an error without revoking any imported credential when enrollment fails.
pub fn enroll<T: TokenStore, A: TokenApi>(
    store: &LocalStore,
    tokens: &T,
    api: &A,
    profile: &Profile,
    known_hosts: String,
    session: &str,
    now: u64,
) -> Result<ManagedTokenStatus, RuntimeError> {
    let _guard = ConfigurationLock::acquire(store)?;
    if let Some(existing) = store.load_profiles()?.find(&profile.name) {
        if existing.base_url != profile.base_url || existing.username != profile.username {
            return Err(RuntimeError::InvalidInput(
                "use a different local profile name for another identity or instance".to_owned(),
            ));
        }
    }
    let enrollment = Enrollment {
        profile: profile.clone(),
        known_hosts,
    };
    rotate_locked(
        store,
        tokens,
        api,
        profile,
        Some(session),
        now,
        Some(enrollment),
    )
}

fn rotate_locked<T: TokenStore, A: TokenApi>(
    store: &LocalStore,
    tokens: &T,
    api: &A,
    profile: &Profile,
    session: Option<&str>,
    now: u64,
    enrollment: Option<Enrollment>,
) -> Result<ManagedTokenStatus, RuntimeError> {
    let name = &profile.name;
    let primary = tokens.get_optional(name)?;
    let mut record = read_record(tokens, name)?.unwrap_or(Record {
        schema_version: 1,
        base_url: profile.base_url.clone(),
        username: profile.username.clone(),
        automatic: true,
        active: None,
        transition: None,
        retirements: Vec::new(),
    });
    if record.base_url != profile.base_url || record.username != profile.username {
        return Err(RuntimeError::InvalidInput(
            "managed credential belongs to another profile identity".to_owned(),
        ));
    }
    if record.transition.is_some() {
        if let Some(cookie) = session {
            let authorization = ApiAuthorization::Session(cookie.to_owned());
            identity_matches(api, &authorization, &profile.username)?;
            let lifetime = api
                .maximum_duration(&authorization)?
                .unwrap_or(DEFAULT_LIFETIME)
                .min(DEFAULT_LIFETIME);
            if lifetime < 360 {
                return Err(RuntimeError::InvalidInput(
                    "the server token lifetime is too short for automatic renewal".to_owned(),
                ));
            }
            if let Some(transition) = &mut record.transition {
                if let Some(candidate) = &transition.candidate {
                    if matches!(
                        api.identity(&ApiAuthorization::Token(candidate.secret.clone())),
                        Err(RuntimeError::Unauthorized)
                    ) {
                        api.revoke(&authorization, &candidate.token.id)?;
                        transition.candidate = None;
                        transition.previous_secret.clone_from(&primary);
                    }
                }
                transition.authorization = authorization;
                transition.requested_expiry = now.saturating_add(lifetime.saturating_sub(60));
                transition.cancelled = false;
            }
            record.automatic = true;
            persist(store, tokens, name, &record)?;
        }
        finish_transition(store, tokens, api, name, &mut record, now)?;
        // Recovery is idempotent: do not immediately issue a second replacement.
        return status(&record);
    }
    if let Some(active) = &record.active {
        if primary.as_deref() != Some(active.secret.as_str()) {
            clear_management(store, tokens, name)?;
            return Err(RuntimeError::InvalidInput(
                "the token was changed outside automatic management; reconnect to enable it again"
                    .to_owned(),
            ));
        }
    }
    let authorization = if let Some(cookie) = session {
        ApiAuthorization::Session(cookie.to_owned())
    } else {
        ApiAuthorization::Token(primary.clone().ok_or(RuntimeError::Unauthorized)?)
    };
    identity_matches(api, &authorization, &profile.username)?;
    let lifetime = api
        .maximum_duration(&authorization)?
        .unwrap_or(DEFAULT_LIFETIME)
        .min(DEFAULT_LIFETIME);
    if lifetime < 360 {
        return Err(RuntimeError::InvalidInput(
            "the server token lifetime is too short for automatic renewal".to_owned(),
        ));
    }
    record.automatic = true;
    record.transition = Some(Transition {
        authorization,
        previous_secret: primary,
        label: unique_label(name)?,
        requested_expiry: now.saturating_add(lifetime.saturating_sub(60)),
        candidate: None,
        enrollment,
        cancelled: false,
    });
    // Persist intent before contacting the server, so a lost response can be reconciled by label.
    persist(store, tokens, name, &record)?;
    finish_transition(store, tokens, api, name, &mut record, now)?;
    status(&record)
}

fn finish_transition<T: TokenStore, A: TokenApi>(
    store: &LocalStore,
    tokens: &T,
    api: &A,
    name: &str,
    record: &mut Record,
    now: u64,
) -> Result<(), RuntimeError> {
    let Some(mut transition) = record.transition.clone() else {
        return Ok(());
    };
    if transition.cancelled {
        let primary = tokens.get_optional(name)?;
        if transition
            .candidate
            .as_ref()
            .is_none_or(|candidate| primary.as_deref() != Some(candidate.secret.as_str()))
        {
            if let Some(candidate) = &transition.candidate {
                api.revoke(&transition.authorization, &candidate.token.id)?;
            } else {
                for token in api
                    .list(&transition.authorization)?
                    .into_iter()
                    .filter(|token| token.label == transition.label)
                {
                    api.revoke(&transition.authorization, &token.id)?;
                }
            }
            record.transition = None;
            return persist(store, tokens, name, record);
        }
        // The primary was already committed before interruption: finish bookkeeping only.
    }
    if transition.candidate.is_none() {
        // A response may have been lost before journaling the secret. Only our exact pending
        // label is eligible for cleanup; imported tokens and unrelated labels are never removed.
        for orphan in api
            .list(&transition.authorization)?
            .into_iter()
            .filter(|token| token.label == transition.label)
        {
            api.revoke(&transition.authorization, &orphan.id)?;
        }
        let candidate = api.issue(
            &transition.authorization,
            &transition.label,
            transition.requested_expiry,
        )?;
        if candidate.token.expiry_epoch()? <= now {
            return Err(RuntimeError::Incompatible(
                "server issued an expired token".to_owned(),
            ));
        }
        transition.candidate = Some(candidate);
        record.transition = Some(transition.clone());
        // If this fails the old primary is still usable. The pending label remains recoverable.
        persist(store, tokens, name, record)?;
    }
    let candidate = transition
        .candidate
        .as_ref()
        .ok_or_else(|| RuntimeError::Incompatible("missing pending token".to_owned()))?;
    let authorization = ApiAuthorization::Token(candidate.secret.clone());
    if let Err(error) = identity_matches(api, &authorization, &record.username) {
        if matches!(
            error,
            RuntimeError::Unauthorized | RuntimeError::InvalidInput(_)
        ) && api
            .revoke(&transition.authorization, &candidate.token.id)
            .is_ok()
        {
            record.transition = None;
            persist(store, tokens, name, record)?;
        }
        return Err(error);
    }
    let current = tokens.get_optional(name)?;
    if current.as_deref() != Some(candidate.secret.as_str()) {
        if current != transition.previous_secret {
            return Err(RuntimeError::InvalidInput(
                "the profile token changed during renewal; no replacement was applied".to_owned(),
            ));
        }
        if let Some(enrollment) = &transition.enrollment {
            LocalConfiguration::new(store, tokens).apply_preserving_management(
                ConfigurationMutation::SaveProfile {
                    profile: enrollment.profile.clone(),
                    token: candidate.secret.clone(),
                    known_hosts: enrollment.known_hosts.clone(),
                },
            )?;
        } else {
            tokens.set(name, &candidate.secret)?;
        }
    }
    if let Some(old) = record.active.take() {
        if old.token.id != candidate.token.id {
            record.retirements.push(Retirement {
                id: old.token.id,
                not_before: now.saturating_add(RETIREMENT_GRACE),
            });
        }
    }
    record.active = Some(candidate.clone());
    record.transition = None;
    // Revocation is only reachable after the new primary and recovery state are durable.
    persist(store, tokens, name, record)?;
    cleanup_retired(store, tokens, api, name, record, now)
}
fn cleanup_retired<T: TokenStore, A: TokenApi>(
    store: &LocalStore,
    tokens: &T,
    api: &A,
    name: &str,
    record: &mut Record,
    now: u64,
) -> Result<(), RuntimeError> {
    let Some(active) = &record.active else {
        return Ok(());
    };
    let authorization = ApiAuthorization::Token(active.secret.clone());
    let mut remaining = Vec::new();
    let mut attempted = false;
    for old in &record.retirements {
        if old.not_before > now || attempted {
            remaining.push(old.clone());
            continue;
        }
        attempted = true;
        if api.revoke(&authorization, &old.id).is_err() {
            remaining.push(old.clone());
        }
    }
    record.retirements = remaining;
    persist(store, tokens, name, record)
}

/// Change the user's explicit automatic-renewal choice without creating a server credential.
/// # Errors
/// Returns an error for missing management state or inaccessible secure storage.
pub fn set_automatic<T: TokenStore>(
    store: &LocalStore,
    tokens: &T,
    name: &str,
    enabled: bool,
) -> Result<(), RuntimeError> {
    let _guard = ConfigurationLock::acquire(store)?;
    let mut record = read_record(tokens, name)?.ok_or_else(|| {
        RuntimeError::InvalidInput(
            "connect this profile before enabling automatic renewal".to_owned(),
        )
    })?;
    record.automatic = enabled;
    if !enabled {
        if let Some(transition) = &mut record.transition {
            transition.cancelled = true;
        }
    }
    persist(store, tokens, name, &record)
}

/// Renew due managed credentials from the independent synchronization agent, including after
/// the companion has quit. Unmanaged credentials are never adopted implicitly.
/// # Errors
/// Returns an error when a due credential cannot be renewed; the last usable credential remains.
pub fn renew_due<T: TokenStore>(
    store: &LocalStore,
    tokens: &T,
    now: u64,
) -> Result<(), RuntimeError> {
    let names: Vec<String> = statuses(store)?.into_keys().collect();
    let mut first_error = None;
    for name in names {
        if let Err(error) = renew_one_due(store, tokens, &name, now) {
            // Never race a simultaneous manual mutation when updating the derived status.
            if let Ok(_guard) = ConfigurationLock::acquire(store) {
                if read_record(tokens, &name)?.is_some() {
                    if let Some(mut value) = statuses(store)?.remove(&name) {
                        value.last_error = Some(
                            "Automatic token renewal failed. Check the profile and secure storage."
                                .to_owned(),
                        );
                        value.renewal_failures = value.renewal_failures.saturating_add(1);
                        let delay =
                            60_u64
                                .saturating_mul(2_u64.saturating_pow(
                                    value.renewal_failures.saturating_sub(1).min(6),
                                ))
                                .min(3600);
                        value.next_attempt_epoch_seconds = Some(now.saturating_add(delay));
                        update_cache(store, &name, Some(value))?;
                    }
                }
            }
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }
    first_error.map_or(Ok(()), Err)
}

fn renew_one_due<T: TokenStore>(
    store: &LocalStore,
    tokens: &T,
    name: &str,
    now: u64,
) -> Result<(), RuntimeError> {
    let _guard = ConfigurationLock::acquire(store)?;
    if statuses(store)?
        .get(name)
        .and_then(|state| state.next_attempt_epoch_seconds)
        .is_some_and(|retry| retry > now)
    {
        return Ok(());
    }
    let Some(mut record) = read_record(tokens, name)? else {
        return Ok(());
    };
    let profile = store.load_profiles()?.find(name).cloned();
    if let Some(profile) = &profile {
        if profile.base_url != record.base_url || profile.username != record.username {
            return Ok(());
        }
    } else if record
        .transition
        .as_ref()
        .and_then(|t| t.enrollment.as_ref())
        .is_none()
    {
        return Ok(());
    }
    let api = ApiClient::new(&record.base_url)?;
    if record.transition.is_some() {
        let cancelled = record.transition.as_ref().is_some_and(|t| t.cancelled);
        let result = finish_transition(store, tokens, &api, name, &mut record, now);
        return if cancelled { Ok(()) } else { result };
    }
    let Some(active) = &record.active else {
        return Ok(());
    };
    if tokens.get_optional(name)?.as_deref() != Some(active.secret.as_str()) {
        clear_management(store, tokens, name)?;
        return Ok(());
    }
    cleanup_retired(store, tokens, &api, name, &mut record, now)?;
    if !record.automatic {
        return Ok(());
    }
    if status(&record)?
        .renew_after_epoch_seconds
        .is_some_and(|when| when <= now)
    {
        let profile = profile
            .ok_or_else(|| RuntimeError::InvalidInput("unknown managed profile".to_owned()))?;
        rotate_locked(store, tokens, &api, &profile, None, now, None)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
