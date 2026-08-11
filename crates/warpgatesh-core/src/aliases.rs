use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

/// A Warpgate SSH target as exposed by the user-facing API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Target {
    pub id: String,
    pub name: String,
}

/// The OpenSSH aliases allocated to one target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TargetAliases {
    pub target_id: String,
    pub short: Option<String>,
    pub qualified: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AliasError {
    InvalidProfileName(String),
    DuplicateTargetId(String),
}

impl fmt::Display for AliasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfileName(name) => write!(
                formatter,
                "invalid profile name '{name}': use lowercase letters, digits, and hyphens"
            ),
            Self::DuplicateTargetId(id) => {
                write!(formatter, "duplicate Warpgate target identifier '{id}'")
            }
        }
    }
}

impl std::error::Error for AliasError {}

#[must_use]
pub fn is_valid_profile_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 63
        && bytes[0].is_ascii_alphanumeric()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

#[must_use]
pub fn target_alias_base(name: &str) -> String {
    if is_openssh_safe_alias(name) {
        return name.to_owned();
    }

    let mut normalized = String::with_capacity(name.len());
    let mut previous_was_hyphen = false;

    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
            previous_was_hyphen = false;
        } else if !previous_was_hyphen && !normalized.is_empty() {
            normalized.push('-');
            previous_was_hyphen = true;
        }
    }

    while normalized.ends_with('-') {
        normalized.pop();
    }

    if normalized.is_empty() {
        "target".to_owned()
    } else {
        normalized
    }
}

/// Allocate deterministic aliases. Qualified aliases are always present; short
/// aliases are emitted only for the default profile.
///
/// # Errors
///
/// Returns [`AliasError::InvalidProfileName`] when `profile` is invalid, or
/// [`AliasError::DuplicateTargetId`] when Warpgate returns a duplicate target.
pub fn allocate_aliases(
    profile: &str,
    is_default_profile: bool,
    targets: &[Target],
) -> Result<Vec<TargetAliases>, AliasError> {
    if !is_valid_profile_name(profile) {
        return Err(AliasError::InvalidProfileName(profile.to_owned()));
    }

    let mut ids = HashSet::new();
    let mut base_counts = HashMap::new();
    let bases: Vec<String> = targets
        .iter()
        .map(|target| {
            if !ids.insert(target.id.as_str()) {
                return Err(AliasError::DuplicateTargetId(target.id.clone()));
            }

            let base = target_alias_base(&target.name);
            *base_counts
                .entry(base.to_ascii_lowercase())
                .or_insert(0_usize) += 1;
            Ok(base)
        })
        .collect::<Result<_, _>>()?;

    Ok(targets
        .iter()
        .zip(bases)
        .map(|(target, base)| {
            let collision_key = base.to_ascii_lowercase();
            let resolved = if base_counts[&collision_key] > 1 {
                format!("{base}-{}", stable_suffix(&target.id))
            } else {
                base
            };

            TargetAliases {
                target_id: target.id.clone(),
                short: is_default_profile.then(|| resolved.clone()),
                qualified: format!("{resolved}.{profile}"),
            }
        })
        .collect())
}

fn is_openssh_safe_alias(name: &str) -> bool {
    let bytes = name.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_alphanumeric()
        && bytes[bytes.len() - 1].is_ascii_alphanumeric()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'-'))
}

fn stable_suffix(value: &str) -> String {
    // FNV-1a is intentionally simple and stable across Rust versions. This is
    // an identifier suffix, not a security primitive.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{:08x}", hash & u64::from(u32::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, name: &str) -> Target {
        Target {
            id: id.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn validates_profile_names() {
        assert!(is_valid_profile_name("homeblack"));
        assert!(is_valid_profile_name("lab-2"));
        assert!(!is_valid_profile_name("HomeBlack"));
        assert!(!is_valid_profile_name("-lab"));
        assert!(!is_valid_profile_name("lab_2"));
    }

    #[test]
    fn preserves_safe_target_names() {
        assert_eq!(target_alias_base("dmz-nextcloud_01"), "dmz-nextcloud_01");
    }

    #[test]
    fn normalizes_unsafe_target_names() {
        assert_eq!(target_alias_base("DMZ / Nextcloud 01"), "dmz-nextcloud-01");
        assert_eq!(target_alias_base("💻"), "target");
    }

    #[test]
    fn emits_short_and_qualified_aliases_for_default_profile() {
        let aliases =
            allocate_aliases("homeblack", true, &[target("target-1", "dmz-nextcloud-01")])
                .expect("aliases should be valid");

        assert_eq!(aliases[0].short.as_deref(), Some("dmz-nextcloud-01"));
        assert_eq!(aliases[0].qualified, "dmz-nextcloud-01.homeblack");
    }

    #[test]
    fn omits_short_alias_for_non_default_profile() {
        let aliases = allocate_aliases("customer", false, &[target("target-1", "db")])
            .expect("aliases should be valid");

        assert_eq!(aliases[0].short, None);
        assert_eq!(aliases[0].qualified, "db.customer");
    }

    #[test]
    fn resolves_normalization_collisions_stably() {
        let aliases = allocate_aliases(
            "lab",
            true,
            &[target("alpha-id", "Prod DB"), target("beta-id", "prod-db")],
        )
        .expect("aliases should be valid");

        assert_ne!(aliases[0].short, aliases[1].short);
        assert!(aliases[0].short.as_deref().unwrap().starts_with("prod-db-"));
        assert!(aliases[1].short.as_deref().unwrap().starts_with("prod-db-"));
    }
}
