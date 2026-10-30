use super::*;
use crate::{api::InstanceMetadata, token_api::TokenMetadata};
use chrono::{DateTime, Utc};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use tempfile::TempDir;
use warpgatesh_core::{
    paths::WarpgatePaths,
    profiles::{ProfileCatalog, SshAuthentication},
};

#[derive(Default)]
struct MemoryTokens {
    values: RefCell<HashMap<String, String>>,
    fail_primary: Cell<bool>,
    managed_writes: Cell<usize>,
    fail_managed_write: Cell<Option<usize>>,
}
impl TokenStore for MemoryTokens {
    fn get_optional(&self, name: &str) -> Result<Option<String>, RuntimeError> {
        Ok(self.values.borrow().get(name).cloned())
    }
    fn get(&self, name: &str) -> Result<String, RuntimeError> {
        self.get_optional(name)?
            .ok_or_else(|| RuntimeError::Keychain("missing test item".to_owned()))
    }
    fn set(&self, name: &str, value: &str) -> Result<(), RuntimeError> {
        if name == "lab" && self.fail_primary.get() {
            return Err(RuntimeError::Keychain(
                "injected primary failure".to_owned(),
            ));
        }
        if name.starts_with("managed-token::") {
            let count = self.managed_writes.get() + 1;
            self.managed_writes.set(count);
            if self.fail_managed_write.get() == Some(count) {
                return Err(RuntimeError::Keychain(
                    "injected journal failure".to_owned(),
                ));
            }
        }
        self.values
            .borrow_mut()
            .insert(name.to_owned(), value.to_owned());
        Ok(())
    }
    fn delete(&self, name: &str) -> Result<(), RuntimeError> {
        self.values.borrow_mut().remove(name);
        Ok(())
    }
}
struct FakeApi<'a> {
    tokens: &'a MemoryTokens,
    issued: Cell<u64>,
    entries: RefCell<Vec<TokenMetadata>>,
    revoked: RefCell<Vec<(String, String)>>,
    now: u64,
    wrong_identity: Cell<bool>,
}
impl<'a> FakeApi<'a> {
    fn new(tokens: &'a MemoryTokens) -> Self {
        Self {
            tokens,
            issued: Cell::new(0),
            entries: RefCell::new(Vec::new()),
            revoked: RefCell::new(Vec::new()),
            now: 2_000_000_000,
            wrong_identity: Cell::new(false),
        }
    }
}
impl TokenApi for FakeApi<'_> {
    fn identity(&self, auth: &ApiAuthorization) -> Result<InstanceMetadata, RuntimeError> {
        let wrong = self.wrong_identity.get()
            && matches!(auth,ApiAuthorization::Token(value) if value.starts_with("new-secret"));
        Ok(InstanceMetadata {
            username: if wrong { "other" } else { "alice" }.to_owned(),
            version: Some("0.30.2".to_owned()),
            ssh_host: "ssh.example.test".to_owned(),
            ssh_port: 2222,
        })
    }
    fn maximum_duration(&self, _: &ApiAuthorization) -> Result<Option<u64>, RuntimeError> {
        Ok(Some(86_400))
    }
    fn issue(
        &self,
        _: &ApiAuthorization,
        label: &str,
        expiry: u64,
    ) -> Result<IssuedToken, RuntimeError> {
        let id = self.issued.get() + 1;
        self.issued.set(id);
        let token = TokenMetadata {
            id: format!("00000000-0000-0000-0000-{id:012}"),
            label: label.to_owned(),
            created: DateTime::<Utc>::from_timestamp(i64::try_from(self.now).unwrap(), 0)
                .unwrap()
                .to_rfc3339(),
            expiry: DateTime::<Utc>::from_timestamp(i64::try_from(expiry).unwrap(), 0)
                .unwrap()
                .to_rfc3339(),
        };
        self.entries.borrow_mut().push(token.clone());
        Ok(IssuedToken {
            token,
            secret: format!("new-secret-{id}"),
        })
    }
    fn list(&self, _: &ApiAuthorization) -> Result<Vec<TokenMetadata>, RuntimeError> {
        Ok(self.entries.borrow().clone())
    }
    fn revoke(&self, _: &ApiAuthorization, id: &str) -> Result<(), RuntimeError> {
        self.revoked
            .borrow_mut()
            .push((id.to_owned(), self.tokens.get("lab").unwrap_or_default()));
        self.entries.borrow_mut().retain(|t| t.id != id);
        Ok(())
    }
}
fn fixture() -> (TempDir, LocalStore, MemoryTokens, Profile) {
    let dir = TempDir::new().unwrap();
    let store = LocalStore::new(WarpgatePaths::for_home(dir.path()));
    let profile = Profile {
        name: "lab".to_owned(),
        base_url: "https://warpgate.example.test/".to_owned(),
        username: "alice".to_owned(),
        warpgate_version: Some("0.30.2".to_owned()),
        ssh_host: "ssh.example.test".to_owned(),
        ssh_port: 2222,
        ssh_authentication: SshAuthentication::Auto,
    };
    let mut catalog = ProfileCatalog::default();
    catalog.upsert(profile.clone()).unwrap();
    store.save_profiles(&catalog).unwrap();
    let tokens = MemoryTokens::default();
    tokens.set("lab", "imported-secret").unwrap();
    (dir, store, tokens, profile)
}
#[test]
fn imported_tokens_are_not_implicitly_managed() {
    let (_dir, store, tokens, _) = fixture();
    renew_due(&store, &tokens, 2_000_000_000).unwrap();
    assert_eq!(tokens.get("lab").unwrap(), "imported-secret");
    assert!(statuses(&store).unwrap().is_empty());
}
#[test]
fn explicit_adoption_obeys_policy_and_never_revokes_an_imported_token() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    let state = rotate(&store, &tokens, &api, "lab", None, api.now).unwrap();
    assert!(state.managed && state.automatic_renewal);
    assert_eq!(state.expires_at_epoch_seconds, Some(api.now + 86_340));
    assert_eq!(tokens.get("lab").unwrap(), "new-secret-1");
    assert!(api.revoked.borrow().is_empty());
    let cache = fs::read_to_string(cache_path(&store)).unwrap();
    assert!(!cache.contains("imported-secret"));
    assert!(!cache.contains("new-secret"));
}
#[test]
fn owned_old_token_is_retired_only_after_durable_replacement_and_grace() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    rotate(&store, &tokens, &api, "lab", None, api.now).unwrap();
    rotate(&store, &tokens, &api, "lab", None, api.now + 1).unwrap();
    assert!(api.revoked.borrow().is_empty());
    assert_eq!(tokens.get("lab").unwrap(), "new-secret-2");
    let mut record = read_record(&tokens, "lab").unwrap().unwrap();
    cleanup_retired(
        &store,
        &tokens,
        &api,
        "lab",
        &mut record,
        api.now + RETIREMENT_GRACE + 2,
    )
    .unwrap();
    assert_eq!(
        api.revoked.borrow().as_slice(),
        &[(
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "new-secret-2".to_owned()
        )]
    );
}
#[test]
fn primary_write_failure_keeps_old_credential_and_resumes_same_candidate() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    tokens.fail_primary.set(true);
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    assert_eq!(tokens.get("lab").unwrap(), "imported-secret");
    assert!(api.revoked.borrow().is_empty());
    tokens.fail_primary.set(false);
    rotate(&store, &tokens, &api, "lab", None, api.now + 1).unwrap();
    assert_eq!(api.issued.get(), 1);
    assert_eq!(tokens.get("lab").unwrap(), "new-secret-1");
}
#[test]
fn lost_secret_response_is_reconciled_only_by_our_pending_label() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    tokens.fail_managed_write.set(Some(2));
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    assert_eq!(tokens.get("lab").unwrap(), "imported-secret");
    tokens.fail_managed_write.set(None);
    rotate(&store, &tokens, &api, "lab", None, api.now + 1).unwrap();
    assert_eq!(api.issued.get(), 2);
    assert_eq!(tokens.get("lab").unwrap(), "new-secret-2");
    assert_eq!(
        api.revoked.borrow()[0],
        (
            "00000000-0000-0000-0000-000000000001".to_owned(),
            "imported-secret".to_owned()
        )
    );
}
#[test]
fn recovery_after_primary_commit_does_not_issue_another_token() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    tokens.fail_managed_write.set(Some(3));
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    assert_eq!(tokens.get("lab").unwrap(), "new-secret-1");
    tokens.fail_managed_write.set(None);
    rotate(&store, &tokens, &api, "lab", None, api.now + 1).unwrap();
    assert_eq!(api.issued.get(), 1);
    assert!(statuses(&store).unwrap()["lab"].managed);
}
#[test]
fn unexpected_primary_replacement_is_never_overwritten() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    tokens.fail_primary.set(true);
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    tokens.fail_primary.set(false);
    tokens.set("lab", "manually-replaced").unwrap();
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now + 1).is_err());
    assert_eq!(tokens.get("lab").unwrap(), "manually-replaced");
}
#[test]
fn disabling_pending_rotation_cleans_candidate_without_replacing_old_token() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    tokens.fail_primary.set(true);
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    tokens.fail_primary.set(false);
    set_automatic(&store, &tokens, "lab", false).unwrap();
    let state = rotate(&store, &tokens, &api, "lab", None, api.now + 1).unwrap();
    assert!(!state.automatic_renewal);
    assert_eq!(tokens.get("lab").unwrap(), "imported-secret");
    assert_eq!(api.issued.get(), 1);
    assert_eq!(api.revoked.borrow().len(), 1);
}
#[test]
fn new_identity_is_rejected_before_primary_is_replaced() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    api.wrong_identity.set(true);
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    assert_eq!(tokens.get("lab").unwrap(), "imported-secret");
    assert_eq!(api.revoked.borrow().len(), 1);
}
#[test]
fn configuration_lock_rejects_concurrent_writers_without_waiting() {
    let (_dir, store, tokens, _) = fixture();
    let api = FakeApi::new(&tokens);
    let _guard = ConfigurationLock::acquire(&store).unwrap();
    assert!(rotate(&store, &tokens, &api, "lab", None, api.now).is_err());
    assert_eq!(api.issued.get(), 0);
}

#[test]
fn assisted_enrollment_persists_profile_only_after_candidate_is_validated() {
    let (_dir, store, tokens, profile) = fixture();
    store.save_profiles(&ProfileCatalog::default()).unwrap();
    tokens.delete("lab").unwrap();
    let api = FakeApi::new(&tokens);
    let state = enroll(
        &store,
        &tokens,
        &api,
        &profile,
        "ssh.example.test ssh-ed25519 AAAATEST\n".to_owned(),
        "warpgate-http-session=test-session",
        api.now,
    )
    .unwrap();
    assert!(state.automatic_renewal);
    assert_eq!(
        store.load_profiles().unwrap().find("lab").unwrap().username,
        "alice"
    );
    assert_eq!(tokens.get("lab").unwrap(), "new-secret-1");
    assert!(
        read_record(&tokens, "lab")
            .unwrap()
            .unwrap()
            .transition
            .is_none()
    );
    let cache = fs::read_to_string(cache_path(&store)).unwrap();
    assert!(!cache.contains("test-session"));
    assert!(!cache.contains("new-secret"));
}

#[test]
fn automatic_renewal_failure_sets_bounded_backoff_and_preserves_usable_token() {
    let (_dir, store, tokens, mut profile) = fixture();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    profile.base_url = format!("http://{}/", listener.local_addr().unwrap());
    drop(listener);
    let mut catalog = ProfileCatalog::default();
    catalog.upsert(profile).unwrap();
    store.save_profiles(&catalog).unwrap();
    let api = FakeApi::new(&tokens);
    let initial = rotate(&store, &tokens, &api, "lab", None, api.now).unwrap();
    let before = tokens.get("lab").unwrap();
    let due = initial.renew_after_epoch_seconds.unwrap();
    assert!(renew_due(&store, &tokens, due).is_err());
    let failed = statuses(&store).unwrap().remove("lab").unwrap();
    assert_eq!(failed.renewal_failures, 1);
    assert_eq!(failed.next_attempt_epoch_seconds, Some(due + 60));
    assert!(failed.last_error.is_some());
    renew_due(&store, &tokens, due + 1).unwrap();
    assert_eq!(statuses(&store).unwrap()["lab"].renewal_failures, 1);
    assert!(renew_due(&store, &tokens, due + 60).is_err());
    assert_eq!(
        statuses(&store).unwrap()["lab"].next_attempt_epoch_seconds,
        Some(due + 180)
    );
    assert_eq!(tokens.get("lab").unwrap(), before);
}
