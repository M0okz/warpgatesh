//! Reversible, opt-in zsh startup configuration shared by the CLI and companion.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::RuntimeError;
use serde::Serialize;

const START: &str = "# >>> WarpgateSH zsh integration >>>";
const END: &str = "# <<< WarpgateSH zsh integration <<<";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellIntegration {
    pub status: String,
    pub path: String,
    pub message: Option<String>,
}

/// Inspect the managed block without modifying the user's startup file.
#[must_use]
pub fn status() -> ShellIntegration {
    match startup_path() {
        Ok(path) => inspect(&path),
        Err(error) => ShellIntegration {
            status: "unavailable".to_owned(),
            path: String::new(),
            message: Some(error.to_string()),
        },
    }
}

fn inspect(path: &Path) -> ShellIntegration {
    let result = read_startup(path).and_then(|original| {
        edit_startup(&original, false)?;
        Ok(original.contains(START))
    });
    let (status, message) = match result {
        Ok(true) => ("enabled", None),
        Ok(false) => ("disabled", None),
        Err(error) => ("manual", Some(error.to_string())),
    };
    ShellIntegration {
        status: status.to_owned(),
        path: path.display().to_string(),
        message,
    }
}

/// Enable or remove only the managed zsh startup block.
///
/// # Errors
/// Returns an error for unsafe, ambiguous or unwritable startup files.
pub fn set_enabled(enabled: bool) -> Result<ShellIntegration, RuntimeError> {
    let path = startup_path()?;
    update(&path, enabled)?;
    Ok(inspect(&path))
}

fn update(path: &Path, enabled: bool) -> Result<(), RuntimeError> {
    let original = read_startup(path)?;
    let updated = edit_startup(&original, enabled)?;
    if updated != original {
        let permissions = fs::metadata(path).ok().map(|meta| meta.permissions());
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
            .persist(path)
            .map_err(|error| RuntimeError::Io(error.error))?;
    }
    Ok(())
}

fn read_startup(path: &Path) -> Result<String, RuntimeError> {
    if fs::symlink_metadata(path).is_ok_and(|meta| !meta.file_type().is_file()) {
        return Err(RuntimeError::InvalidInput(format!(
            "{} is not a regular file; configure the integration manually",
            path.display()
        )));
    }
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error.into()),
    }
}

fn startup_path() -> Result<PathBuf, RuntimeError> {
    std::env::var_os("ZDOTDIR")
        .or_else(|| std::env::var_os("HOME"))
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join(".zshrc"))
        .ok_or_else(|| RuntimeError::InvalidInput("HOME or ZDOTDIR is required".to_owned()))
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
    fn inspection_is_read_only_and_changes_preserve_personal_configuration() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(".zshrc");
        assert_eq!(inspect(&path).status, "disabled");
        assert!(!path.exists());
        let personal = "# Personal configuration\nexport EDITOR=vim\n";
        fs::write(&path, personal).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        update(&path, true).unwrap();
        assert_eq!(inspect(&path).status, "enabled");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );
        update(&path, true).unwrap();
        update(&path, false).unwrap();
        assert_eq!(inspect(&path).status, "disabled");
        assert_eq!(fs::read_to_string(&path).unwrap(), personal);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn ambiguous_or_linked_startup_files_are_never_changed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join(".zshrc");
        fs::write(&path, START).unwrap();
        assert_eq!(inspect(&path).status, "manual");
        assert!(update(&path, true).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), START);
        fs::remove_file(&path).unwrap();
        let personal = directory.path().join("personal");
        fs::write(&personal, "# Personal configuration\n").unwrap();
        std::os::unix::fs::symlink(&personal, &path).unwrap();
        assert_eq!(inspect(&path).status, "manual");
        assert!(update(&path, true).is_err());
        assert_eq!(
            fs::read_to_string(&personal).unwrap(),
            "# Personal configuration\n"
        );
        assert!(
            fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn installation_is_idempotent_and_removal_preserves_user_lines() {
        let original = "export EDITOR=vim\n# personal configuration\n";
        let installed = edit_startup(original, true).unwrap();
        assert_eq!(edit_startup(&installed, true).unwrap(), installed);
        assert_eq!(edit_startup(&installed, false).unwrap(), original);
        assert!(edit_startup(START, false).is_err());
        assert!(edit_startup(&format!("echo '{START}'\n{END}\n"), false).is_err());
    }
}
