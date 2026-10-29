//! Explicit, reversible zsh integration backed only by the local snapshot.
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use warpgatesh_runtime::RuntimeError;
use warpgatesh_runtime::storage::LocalStore;

const START: &str = "# >>> WarpgateSH zsh integration >>>";
const END: &str = "# <<< WarpgateSH zsh integration <<<";
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
            let path = startup_path()?;
            if fs::symlink_metadata(&path).is_ok_and(|meta| !meta.file_type().is_file()) {
                return Err(RuntimeError::InvalidInput(format!(
                    "{} is not a regular file; configure the integration manually",
                    path.display()
                )));
            }
            let original = match fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(error) => return Err(error.into()),
            };
            let updated = edit_startup(&original, *operation == "install")?;
            if updated != original {
                let permissions = fs::metadata(&path).ok().map(|meta| meta.permissions());
                let parent = path.parent().ok_or_else(|| {
                    RuntimeError::InvalidInput("startup file has no parent directory".to_owned())
                })?;
                let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
                temporary.write_all(updated.as_bytes())?;
                if let Some(permissions) = permissions {
                    temporary.as_file().set_permissions(permissions)?;
                }
                temporary.as_file().sync_all()?;
                temporary
                    .persist(&path)
                    .map_err(|error| RuntimeError::Io(error.error))?;
            }
            let state = if *operation == "install" {
                "enabled"
            } else {
                "removed"
            };
            println!(
                "Zsh integration {state}: {}. Open a new terminal to apply the change.",
                path.display()
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

fn startup_path() -> Result<PathBuf, RuntimeError> {
    std::env::var_os("ZDOTDIR")
        .or_else(|| std::env::var_os("HOME"))
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join(".zshrc"))
        .ok_or_else(|| RuntimeError::InvalidInput("HOME or ZDOTDIR is required".to_owned()))
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

fn edit_startup(original: &str, install: bool) -> Result<String, RuntimeError> {
    let starts: Vec<_> = original.match_indices(START).collect();
    let ends: Vec<_> = original.match_indices(END).collect();
    let mut contents = original.to_owned();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => {}
        ([(start, _)], [(end, _)])
            if start < end
                && (*start == 0 || original[..*start].ends_with('\n'))
                && original[start + START.len()..].starts_with('\n')
                && original[..*end].ends_with('\n')
                && (end + END.len() == original.len()
                    || original[end + END.len()..].starts_with('\n')) =>
        {
            let after = end + END.len();
            let after = after + usize::from(original[after..].starts_with('\n'));
            contents.replace_range(*start..after, "");
        }
        _ => {
            return Err(RuntimeError::InvalidInput(
                "ambiguous WarpgateSH markers in .zshrc; no changes made".to_owned(),
            ));
        }
    }
    if install {
        if !contents.is_empty() && !contents.ends_with('\n') {
            contents.push('\n');
        }
        contents.push_str(START);
        contents.push_str("\nif command -v warpgatesh >/dev/null 2>&1; then\n  eval \"$(command warpgatesh shell init zsh)\"\nfi\n");
        contents.push_str(END);
        contents.push('\n');
    }
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installation_is_idempotent_and_removal_preserves_user_lines() {
        let original = "export EDITOR=vim\n# personal configuration\n";
        let installed = edit_startup(original, true).unwrap();
        assert_eq!(edit_startup(&installed, true).unwrap(), installed);
        assert_eq!(edit_startup(&installed, false).unwrap(), original);
        assert!(edit_startup(START, false).is_err());
        assert!(edit_startup(&format!("echo '{START}'\n{END}\n"), false).is_err());
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
