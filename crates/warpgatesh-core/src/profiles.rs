use std::fmt;

use serde::{Deserialize, Serialize};

use crate::aliases::is_valid_profile_name;

pub const PROFILE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile {
    pub name: String,
    pub base_url: String,
    pub username: String,
    pub warpgate_version: Option<String>,
    pub ssh_host: String,
    pub ssh_port: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProfileCatalog {
    pub schema_version: u32,
    pub default_profile: Option<String>,
    pub profiles: Vec<Profile>,
}

impl Default for ProfileCatalog {
    fn default() -> Self {
        Self {
            schema_version: PROFILE_SCHEMA_VERSION,
            default_profile: None,
            profiles: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileError {
    InvalidName(String),
    UnsupportedSchema(u32),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(formatter, "invalid profile name '{name}'"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported profile schema version {version}")
            }
        }
    }
}

impl std::error::Error for ProfileError {}

impl ProfileCatalog {
    /// Validate the persisted catalog before it is used.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] for an unsupported schema or invalid profile.
    pub fn validate(&self) -> Result<(), ProfileError> {
        if self.schema_version != PROFILE_SCHEMA_VERSION {
            return Err(ProfileError::UnsupportedSchema(self.schema_version));
        }

        if let Some(invalid) = self
            .profiles
            .iter()
            .find(|profile| !is_valid_profile_name(&profile.name))
        {
            return Err(ProfileError::InvalidName(invalid.name.clone()));
        }

        Ok(())
    }

    /// Insert or replace a profile by name.
    ///
    /// The first profile becomes the default automatically.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError::InvalidName`] when the profile name is invalid.
    pub fn upsert(&mut self, profile: Profile) -> Result<(), ProfileError> {
        if !is_valid_profile_name(&profile.name) {
            return Err(ProfileError::InvalidName(profile.name));
        }

        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|existing| existing.name == profile.name)
        {
            *existing = profile;
        } else {
            let first = self.profiles.is_empty();
            let name = profile.name.clone();
            self.profiles.push(profile);
            self.profiles
                .sort_by(|left, right| left.name.cmp(&right.name));
            if first {
                self.default_profile = Some(name);
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.name == name)
    }

    #[must_use]
    pub fn is_default(&self, name: &str) -> bool {
        self.default_profile.as_deref() == Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(name: &str) -> Profile {
        Profile {
            name: name.to_owned(),
            base_url: "https://warpgate.example".to_owned(),
            username: "gregory".to_owned(),
            warpgate_version: Some("0.27.0".to_owned()),
            ssh_host: "ssh.warpgate.example".to_owned(),
            ssh_port: 2222,
        }
    }

    #[test]
    fn first_profile_becomes_default() {
        let mut catalog = ProfileCatalog::default();
        catalog.upsert(profile("homeblack")).expect("valid profile");
        assert_eq!(catalog.default_profile.as_deref(), Some("homeblack"));
    }

    #[test]
    fn replacing_a_profile_preserves_the_default() {
        let mut catalog = ProfileCatalog::default();
        catalog.upsert(profile("homeblack")).expect("valid profile");
        let mut replacement = profile("homeblack");
        replacement.ssh_port = 22;
        catalog.upsert(replacement).expect("valid replacement");
        assert_eq!(catalog.profiles.len(), 1);
        assert_eq!(catalog.profiles[0].ssh_port, 22);
        assert!(catalog.is_default("homeblack"));
    }
}
