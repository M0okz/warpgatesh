//! Explicit, reversible zsh integration backed only by the local snapshot.
use std::collections::BTreeSet;

use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::storage::{LocalStore, SyncedTarget};

const INIT: &str = include_str!("shell.zsh");

/// Execute a shell management command.
///
/// # Errors
/// Returns an error for invalid arguments, unreadable state or unsafe startup files.
pub fn run(arguments: &[String]) -> Result<(), RuntimeError> {
    match arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["init", "zsh"] => print!("{INIT}"),
        ["aliases"] => {
            if let Some(snapshot) = LocalStore::for_current_user()?.load_snapshot()? {
                let mut aliases = BTreeSet::new();
                for target in snapshot.targets {
                    aliases.insert(target.qualified_alias);
                    aliases.extend(target.short_alias);
                }
                for alias in aliases {
                    if valid_alias(&alias) {
                        println!("{alias}");
                    }
                }
            }
        }
        ["commands"] => {
            let store = LocalStore::for_current_user()?;
            let profiles = store.load_profiles()?;
            if let Some(snapshot) = store.load_snapshot()? {
                let mut commands = BTreeSet::new();
                for target in &snapshot.targets {
                    if let Some(name) = command_name(target, profiles.default_profile.as_deref()) {
                        commands.insert((name, target.qualified_alias.as_str()));
                    }
                }
                for (name, destination) in commands {
                    println!("{name}\t{destination}");
                }
            }
        }
        [operation @ ("install" | "uninstall"), "zsh"] => {
            let integration =
                warpgatesh_runtime::shell_integration::set_enabled(*operation == "install")?;
            let state = if *operation == "install" {
                "enabled"
            } else {
                "removed"
            };
            println!(
                "Zsh integration {state}: {}. Open a new terminal to apply the change.",
                integration.path
            );
        }
        _ => {
            return Err(RuntimeError::InvalidInput(
                "usage: warpgatesh shell <init|install|uninstall> zsh | shell <aliases|commands>"
                    .to_owned(),
            ));
        }
    }
    Ok(())
}

fn command_name<'a>(target: &'a SyncedTarget, default_profile: Option<&str>) -> Option<&'a str> {
    if !valid_alias(&target.qualified_alias) {
        return None;
    }
    let preferred = if default_profile == Some(target.profile.as_str()) {
        target
            .qualified_alias
            .strip_suffix(&format!(".{}", target.profile))
            .unwrap_or(&target.qualified_alias)
    } else {
        &target.qualified_alias
    };
    Some(if valid_alias(preferred) {
        preferred
    } else {
        &target.qualified_alias
    })
}

fn valid_alias(alias: &str) -> bool {
    alias
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric)
        && matches!(
            crate::parse([alias.to_owned()]),
            Ok(crate::CliCommand::Connect { .. })
        )
        && alias
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_command_per_target_uses_short_default_names_and_qualified_destinations() {
        let mut target = SyncedTarget {
            profile: "hblk".to_owned(),
            target_id: "1".to_owned(),
            name: "trust-admin-01".to_owned(),
            short_alias: None,
            qualified_alias: "trust-admin-01.hblk".to_owned(),
        };
        // An SSH alias conflict must not prevent a short shell command, which
        // connects through the qualified destination instead of that SSH alias.
        assert_eq!(command_name(&target, Some("hblk")), Some("trust-admin-01"));
        assert_eq!(
            command_name(&target, Some("other")),
            Some("trust-admin-01.hblk")
        );
        target.qualified_alias = "sync.hblk".to_owned();
        assert_eq!(command_name(&target, Some("hblk")), Some("sync.hblk"));
    }

    #[test]
    fn rejects_shell_syntax_in_aliases() {
        for alias in [
            "$(touch /tmp/oops)",
            "-option",
            "name;id",
            "",
            "a\nb",
            "sync",
            "shell",
        ] {
            assert!(!valid_alias(alias));
        }
        assert!(valid_alias("trust-admin-01.homeblack"));
    }
}
