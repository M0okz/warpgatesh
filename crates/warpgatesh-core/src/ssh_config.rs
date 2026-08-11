use std::collections::HashSet;
use std::fmt::{self, Write as _};
use std::hash::BuildHasher;

use crate::aliases::{AliasError, Target, allocate_aliases};
use crate::profiles::Profile;

pub const SSH_INCLUDE_LINE: &str = "Include ~/.ssh/warpgatesh/config";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SshConfigError {
    Alias(AliasError),
    UnsafeValue(&'static str),
}

impl fmt::Display for SshConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Alias(error) => error.fmt(formatter),
            Self::UnsafeValue(field) => write!(formatter, "unsafe control character in {field}"),
        }
    }
}

impl std::error::Error for SshConfigError {}

impl From<AliasError> for SshConfigError {
    fn from(error: AliasError) -> Self {
        Self::Alias(error)
    }
}

#[must_use]
pub fn ensure_managed_include(existing: &str) -> (String, bool) {
    if existing.lines().any(|line| line.trim() == SSH_INCLUDE_LINE) {
        return (existing.to_owned(), false);
    }

    if existing.is_empty() {
        return (format!("{SSH_INCLUDE_LINE}\n"), true);
    }

    (format!("{SSH_INCLUDE_LINE}\n\n{existing}"), true)
}

#[must_use]
pub fn remove_managed_include(existing: &str) -> (String, bool) {
    let installed_prefix = format!("{SSH_INCLUDE_LINE}\n\n");
    if let Some(original) = existing.strip_prefix(&installed_prefix) {
        return (original.to_owned(), true);
    }
    if existing == format!("{SSH_INCLUDE_LINE}\n") || existing == SSH_INCLUDE_LINE {
        return (String::new(), true);
    }

    let mut updated = String::with_capacity(existing.len());
    let mut removed = false;
    for line in existing.split_inclusive('\n') {
        if line.trim() == SSH_INCLUDE_LINE {
            removed = true;
        } else {
            updated.push_str(line);
        }
    }

    if removed {
        (updated, true)
    } else {
        (existing.to_owned(), false)
    }
}

/// Render one profile into the managed OpenSSH configuration.
///
/// # Errors
///
/// Returns [`SshConfigError`] when aliases cannot be allocated or a value
/// contains a control character that could alter the generated configuration.
pub fn render_profile<S: BuildHasher>(
    profile: &Profile,
    is_default_profile: bool,
    targets: &[Target],
    blocked_short_aliases: &HashSet<String, S>,
) -> Result<String, SshConfigError> {
    let host_name = quote_value(&profile.ssh_host, "SSH host")?;
    let aliases = allocate_aliases(&profile.name, is_default_profile, targets)?;
    let known_hosts = format!("~/.ssh/warpgatesh/known_hosts/{}", profile.name);
    let mut output = format!("# Profile {}\n", profile.name);

    for (target, aliases) in targets.iter().zip(aliases) {
        let mut host_aliases = Vec::with_capacity(2);
        if let Some(short) = aliases.short {
            if !blocked_short_aliases.contains(&short.to_ascii_lowercase()) {
                host_aliases.push(short);
            }
        }
        host_aliases.push(aliases.qualified);

        let selector_separator = if target.name.contains('#') { '#' } else { ':' };
        let selector = format!("{}{selector_separator}{}", profile.username, target.name);
        let user = quote_value(&selector, "Warpgate SSH selector")?;

        write!(
            output,
            "\nHost {}\n  HostName {host_name}\n  Port {}\n  User {user}\n  UserKnownHostsFile {known_hosts}\n  StrictHostKeyChecking yes\n  KbdInteractiveAuthentication yes\n  PasswordAuthentication yes\n  PubkeyAuthentication yes\n",
            host_aliases.join(" "),
            profile.ssh_port,
        )
        .expect("writing to a String cannot fail");
    }

    Ok(output)
}

fn quote_value(value: &str, field: &'static str) -> Result<String, SshConfigError> {
    if value.chars().any(char::is_control) {
        return Err(SshConfigError::UnsafeValue(field));
    }

    Ok(format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::Profile;

    fn profile() -> Profile {
        Profile {
            name: "homeblack".to_owned(),
            base_url: "https://warpgate.example".to_owned(),
            username: "gregory".to_owned(),
            warpgate_version: Some("0.27.0".to_owned()),
            ssh_host: "ssh.warpgate.example".to_owned(),
            ssh_port: 2222,
        }
    }

    #[test]
    fn puts_the_include_before_existing_configuration() {
        let (updated, changed) = ensure_managed_include("Host example\n  User gregory\n");
        assert!(changed);
        assert_eq!(
            updated,
            "Include ~/.ssh/warpgatesh/config\n\nHost example\n  User gregory\n"
        );
    }

    #[test]
    fn is_idempotent() {
        let existing = "Include ~/.ssh/warpgatesh/config\n\nHost example\n";
        let (updated, changed) = ensure_managed_include(existing);
        assert!(!changed);
        assert_eq!(updated, existing);
    }

    #[test]
    fn removes_only_the_managed_include() {
        let existing = "Include ~/.ssh/warpgatesh/config\n\nHost example\n  User gregory\n";
        let (updated, changed) = remove_managed_include(existing);
        assert!(changed);
        assert_eq!(updated, "Host example\n  User gregory\n");
    }

    #[test]
    fn leaves_an_unmanaged_configuration_unchanged() {
        let existing = "Host example\n  User gregory\n";
        let (updated, changed) = remove_managed_include(existing);
        assert!(!changed);
        assert_eq!(updated, existing);
    }

    #[test]
    fn restores_leading_whitespace_from_the_original_configuration() {
        let original = "\nHost example\n";
        let (installed, _) = ensure_managed_include(original);
        let (restored, changed) = remove_managed_include(&installed);
        assert!(changed);
        assert_eq!(restored, original);
    }

    #[test]
    fn renders_short_and_qualified_aliases() {
        let rendered = render_profile(
            &profile(),
            true,
            &[Target {
                id: "target-1".to_owned(),
                name: "dmz-nextcloud-01".to_owned(),
            }],
            &HashSet::new(),
        )
        .expect("configuration should render");

        assert!(rendered.contains("Host dmz-nextcloud-01 dmz-nextcloud-01.homeblack"));
        assert!(rendered.contains("User \"gregory:dmz-nextcloud-01\""));
        assert!(rendered.contains("StrictHostKeyChecking yes"));
    }

    #[test]
    fn omits_a_blocked_short_alias_but_keeps_the_qualified_alias() {
        let rendered = render_profile(
            &profile(),
            true,
            &[Target {
                id: "target-1".to_owned(),
                name: "db".to_owned(),
            }],
            &HashSet::from(["db".to_owned()]),
        )
        .expect("configuration should render");

        assert!(rendered.contains("Host db.homeblack\n"));
        assert!(!rendered.contains("Host db db.homeblack"));
    }
}
