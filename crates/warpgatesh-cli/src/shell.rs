//! Explicit, reversible zsh integration backed only by the local snapshot.
use std::collections::BTreeSet;

use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::storage::LocalStore;

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
                "usage: warpgatesh shell <init|install|uninstall> zsh | shell aliases".to_owned(),
            ));
        }
    }
    Ok(())
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
