use std::fs;

use serde::{Deserialize, Serialize};
use warpgatesh_core::profiles::Profile;

use crate::RuntimeError;
use crate::keychain::TokenStore;
use crate::ssh::{install_managed_include, save_host_keys};
use crate::storage::{LocalStore, Preferences};

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConfigurationMutation {
    SaveProfile {
        profile: Profile,
        token: String,
        known_hosts: String,
    },
    RenewToken {
        name: String,
        token: String,
        username: String,
        warpgate_version: Option<String>,
    },
    RemoveProfile {
        name: String,
    },
    SavePreferences {
        preferences: Preferences,
        default_profile: Option<String>,
    },
}

impl ConfigurationMutation {
    /// Decode a mutation received through the private local protocol.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the payload is not a valid mutation.
    pub fn from_json(payload: &str) -> Result<Self, RuntimeError> {
        Ok(serde_json::from_str(payload)?)
    }
}

/// Owns every persistent local configuration mutation.
///
/// Callers provide intent through [`ConfigurationMutation`]; this module owns
/// the ordering and validation of profile, keychain, host-key and preference
/// writes.
pub struct LocalConfiguration<'a, T> {
    store: &'a LocalStore,
    tokens: &'a T,
}

impl<'a, T: TokenStore> LocalConfiguration<'a, T> {
    #[must_use]
    pub const fn new(store: &'a LocalStore, tokens: &'a T) -> Self {
        Self { store, tokens }
    }

    /// Apply one mutation to the current user's local configuration.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when validation, secret storage, or an atomic
    /// file replacement fails.
    pub fn apply(&self, mutation: ConfigurationMutation) -> Result<(), RuntimeError> {
        match mutation {
            ConfigurationMutation::SaveProfile {
                profile,
                token,
                known_hosts,
            } => self.save_profile(profile, &token, &known_hosts),
            ConfigurationMutation::RenewToken {
                name,
                token,
                username,
                warpgate_version,
            } => self.renew_token(&name, &token, username, warpgate_version),
            ConfigurationMutation::RemoveProfile { name } => self.remove_profile(&name),
            ConfigurationMutation::SavePreferences {
                preferences,
                default_profile,
            } => self.save_preferences(&preferences, default_profile),
        }
    }

    fn save_profile(
        &self,
        profile: Profile,
        token: &str,
        known_hosts: &str,
    ) -> Result<(), RuntimeError> {
        if token.trim().is_empty() || known_hosts.trim().is_empty() {
            return Err(RuntimeError::InvalidInput(
                "a profile requires an API token and approved SSH host keys".to_owned(),
            ));
        }
        let name = profile.name.clone();
        let mut catalog = self.store.load_profiles()?;
        catalog.upsert(profile)?;
        self.tokens.set(&name, token.trim())?;
        save_host_keys(self.store.paths(), &name, known_hosts)?;
        self.store.save_profiles(&catalog)?;
        install_managed_include(self.store.paths())?;
        Ok(())
    }

    fn renew_token(
        &self,
        name: &str,
        token: &str,
        username: String,
        warpgate_version: Option<String>,
    ) -> Result<(), RuntimeError> {
        if token.trim().is_empty() {
            return Err(RuntimeError::InvalidInput(
                "an API token is required".to_owned(),
            ));
        }
        let mut catalog = self.store.load_profiles()?;
        let existing = catalog
            .find(name)
            .cloned()
            .ok_or_else(|| RuntimeError::InvalidInput(format!("unknown profile '{name}'")))?;
        catalog.upsert(Profile {
            username,
            warpgate_version,
            ..existing
        })?;
        self.tokens.set(name, token.trim())?;
        self.store.save_profiles(&catalog)
    }

    fn remove_profile(&self, name: &str) -> Result<(), RuntimeError> {
        let mut catalog = self.store.load_profiles()?;
        if !catalog.remove(name) {
            return Err(RuntimeError::InvalidInput(format!(
                "unknown profile '{name}'"
            )));
        }
        self.tokens.delete(name)?;
        let host_keys = self.store.paths().known_hosts_directory.join(name);
        match fs::remove_file(host_keys) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        self.store.save_profiles(&catalog)
    }

    fn save_preferences(
        &self,
        preferences: &Preferences,
        default_profile: Option<String>,
    ) -> Result<(), RuntimeError> {
        preferences.validate()?;
        let mut catalog = self.store.load_profiles()?;
        match default_profile.as_deref() {
            Some(name) if catalog.find(name).is_none() => {
                return Err(RuntimeError::InvalidInput(format!(
                    "unknown profile '{name}'"
                )));
            }
            None if !catalog.profiles.is_empty() => {
                return Err(RuntimeError::InvalidInput(
                    "a default profile is required when profiles are configured".to_owned(),
                ));
            }
            _ => {}
        }
        catalog.default_profile = default_profile;
        self.store.save_preferences(preferences)?;
        self.store.save_profiles(&catalog)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use tempfile::TempDir;
    use warpgatesh_core::paths::WarpgatePaths;

    use super::*;

    #[derive(Default)]
    struct MemoryTokens(Mutex<HashMap<String, String>>);

    impl TokenStore for MemoryTokens {
        fn set(&self, profile: &str, token: &str) -> Result<(), RuntimeError> {
            self.0
                .lock()
                .expect("tokens")
                .insert(profile.to_owned(), token.to_owned());
            Ok(())
        }

        fn get(&self, profile: &str) -> Result<String, RuntimeError> {
            self.0
                .lock()
                .expect("tokens")
                .get(profile)
                .cloned()
                .ok_or_else(|| RuntimeError::Keychain("missing test token".to_owned()))
        }

        fn delete(&self, profile: &str) -> Result<(), RuntimeError> {
            self.0.lock().expect("tokens").remove(profile);
            Ok(())
        }
    }

    fn profile(name: &str) -> Profile {
        Profile {
            name: name.to_owned(),
            base_url: "https://warpgate.example/".to_owned(),
            username: "gregory".to_owned(),
            warpgate_version: Some("0.27.1".to_owned()),
            ssh_host: "ssh.warpgate.example".to_owned(),
            ssh_port: 2222,
        }
    }

    #[test]
    fn applies_configuration_through_one_interface() {
        let home = TempDir::new().expect("temporary home");
        let store = LocalStore::new(WarpgatePaths::for_home(home.path()));
        let tokens = MemoryTokens::default();
        let configuration = LocalConfiguration::new(&store, &tokens);

        configuration
            .apply(ConfigurationMutation::SaveProfile {
                profile: profile("lab"),
                token: "secret".to_owned(),
                known_hosts: "ssh.example ssh-ed25519 AAAA\n".to_owned(),
            })
            .expect("save profile");
        configuration
            .apply(ConfigurationMutation::SavePreferences {
                preferences: Preferences {
                    sync_interval_seconds: 900,
                    ..Preferences::default()
                },
                default_profile: Some("lab".to_owned()),
            })
            .expect("save preferences");

        assert_eq!(store.load_profiles().expect("profiles").profiles.len(), 1);
        assert_eq!(tokens.get("lab").expect("token"), "secret");
        assert_eq!(
            store
                .load_preferences()
                .expect("preferences")
                .sync_interval_seconds,
            900
        );
        configuration
            .apply(ConfigurationMutation::RemoveProfile {
                name: "lab".to_owned(),
            })
            .expect("remove profile");
        assert!(store.load_profiles().expect("profiles").profiles.is_empty());
    }
}
