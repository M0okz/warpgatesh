use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use glob::glob;
use warpgatesh_core::paths::WarpgatePaths;
use warpgatesh_core::ssh_config::{ensure_managed_include, remove_managed_include};

use crate::RuntimeError;
use crate::storage::atomic_write;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannedHostKeys {
    pub known_hosts: String,
    pub fingerprints: String,
}

/// Open the personal API-token page in the system browser.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the browser command cannot be started.
pub fn open_token_page(url: &str) -> Result<(), RuntimeError> {
    #[cfg(target_os = "macos")]
    let status = Command::new("/usr/bin/open").arg(url).status()?;

    #[cfg(target_os = "linux")]
    let status = Command::new("xdg-open").arg(url).status()?;

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Err(RuntimeError::Command(
        "opening a browser is unsupported on this platform".to_owned(),
    ));

    if status.success() {
        Ok(())
    } else {
        Err(RuntimeError::Command(format!(
            "could not open the browser (exit status {status})"
        )))
    }
}

/// Retrieve the SSH host keys and their SHA256 fingerprints using system
/// OpenSSH tools.
///
/// # Errors
///
/// Returns [`RuntimeError`] when no key can be retrieved or fingerprinted.
pub fn scan_host_keys(host: &str, port: u16) -> Result<ScannedHostKeys, RuntimeError> {
    let scan = Command::new("/usr/bin/ssh-keyscan")
        .args(["-T", "5", "-p", &port.to_string(), host])
        .output()?;
    if !scan.status.success() || scan.stdout.is_empty() {
        let details = String::from_utf8_lossy(&scan.stderr);
        let details = details.trim();
        return Err(RuntimeError::Command(format!(
            "could not retrieve the SSH host key for {host}:{port}{}",
            if details.is_empty() {
                " (connection timed out or returned no host key)".to_owned()
            } else {
                format!(": {details}")
            }
        )));
    }

    let known_hosts = String::from_utf8(scan.stdout)
        .map_err(|_| RuntimeError::Command("ssh-keyscan returned non-UTF-8 output".to_owned()))?;
    let mut keygen = Command::new("/usr/bin/ssh-keygen")
        .args(["-lf", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    keygen
        .stdin
        .take()
        .ok_or_else(|| RuntimeError::Command("could not open ssh-keygen input".to_owned()))?
        .write_all(known_hosts.as_bytes())?;
    let fingerprints = keygen.wait_with_output()?;
    if !fingerprints.status.success() {
        return Err(RuntimeError::Command(
            "could not fingerprint the SSH host key".to_owned(),
        ));
    }

    Ok(ScannedHostKeys {
        known_hosts,
        fingerprints: String::from_utf8(fingerprints.stdout).map_err(|_| {
            RuntimeError::Command("ssh-keygen returned non-UTF-8 output".to_owned())
        })?,
    })
}

/// Save the pinned SSH keys for one profile.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the managed file cannot be replaced.
pub fn save_host_keys(
    paths: &WarpgatePaths,
    profile: &str,
    known_hosts: &str,
) -> Result<(), RuntimeError> {
    atomic_write(
        &paths.known_hosts_directory.join(profile),
        known_hosts.as_bytes(),
    )
}

/// Verify that an SSH endpoint still presents exactly the keys pinned by the user.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the endpoint is unreachable, the pin cannot be
/// read, or the presented key material differs from the approved material.
pub fn verify_host_keys(
    paths: &WarpgatePaths,
    profile: &str,
    host: &str,
    port: u16,
) -> Result<(), RuntimeError> {
    let pinned = fs::read_to_string(paths.known_hosts_directory.join(profile))?;
    let presented = scan_host_keys(host, port)?;
    if key_material(&pinned) != key_material(&presented.known_hosts) {
        return Err(RuntimeError::Command(format!(
            "SSH host keys changed for profile '{profile}' at {host}:{port}; review and add the profile again before synchronizing"
        )));
    }
    Ok(())
}

fn key_material(known_hosts: &str) -> BTreeSet<(&str, &str)> {
    known_hosts
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let _hosts = fields.next()?;
            Some((fields.next()?, fields.next()?))
        })
        .collect()
}

/// Install the single managed `Include` directive when absent.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the user's SSH configuration cannot be read or
/// replaced.
pub fn install_managed_include(paths: &WarpgatePaths) -> Result<bool, RuntimeError> {
    let existing = match fs::read_to_string(&paths.user_ssh_config) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let (updated, changed) = ensure_managed_include(&existing);
    if changed {
        atomic_write(&paths.user_ssh_config, updated.as_bytes())?;
    }
    Ok(changed)
}

/// Remove only the `Include` directive owned by `WarpgateSH`.
///
/// # Errors
///
/// Returns [`RuntimeError`] when the user's SSH configuration cannot be read or
/// replaced.
pub fn uninstall_managed_include(paths: &WarpgatePaths) -> Result<bool, RuntimeError> {
    let existing = match fs::read_to_string(&paths.user_ssh_config) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let (updated, changed) = remove_managed_include(&existing);
    if changed {
        atomic_write(&paths.user_ssh_config, updated.as_bytes())?;
    }
    Ok(changed)
}

/// Collect literal aliases declared in the user's SSH files, excluding the
/// WarpgateSH-managed file itself.
///
/// # Errors
///
/// Returns [`RuntimeError`] when an existing configuration file cannot be read.
pub fn manual_host_aliases(paths: &WarpgatePaths) -> Result<HashSet<String>, RuntimeError> {
    let mut aliases = HashSet::new();
    let mut visited = HashSet::new();
    collect_aliases(
        &paths.user_ssh_config,
        &paths.ssh_config,
        &paths.user_ssh_config,
        &mut visited,
        &mut aliases,
    )?;
    Ok(aliases)
}

fn collect_aliases(
    file: &Path,
    managed_file: &Path,
    root_config: &Path,
    visited: &mut HashSet<PathBuf>,
    aliases: &mut HashSet<String>,
) -> Result<(), RuntimeError> {
    if same_path(file, managed_file) || !visited.insert(file.to_path_buf()) {
        return Ok(());
    }
    let content = match fs::read_to_string(file) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut words = trimmed.split_whitespace();
        let Some(keyword) = words.next() else {
            continue;
        };

        if keyword.eq_ignore_ascii_case("host") {
            for alias in words.filter(|word| {
                !word.starts_with('!') && !word.chars().any(|character| "*?!".contains(character))
            }) {
                aliases.insert(alias.to_ascii_lowercase());
            }
        } else if keyword.eq_ignore_ascii_case("include") {
            for pattern in words {
                let expanded = expand_include(pattern.trim_matches(['\'', '"']), root_config);
                let pattern = expanded.to_string_lossy();
                for included in glob(&pattern)
                    .map_err(|error| RuntimeError::InvalidInput(error.to_string()))?
                    .flatten()
                {
                    collect_aliases(&included, managed_file, root_config, visited, aliases)?;
                }
            }
        }
    }
    Ok(())
}

fn expand_include(pattern: &str, root_config: &Path) -> PathBuf {
    if let Some(relative) = pattern.strip_prefix("~/") {
        if let Some(home) = root_config.parent().and_then(Path::parent) {
            return home.join(relative);
        }
    }
    let path = Path::new(pattern);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_config
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path)
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    left == right
        || fs::canonicalize(left)
            .ok()
            .zip(fs::canonicalize(right).ok())
            .is_some_and(|(left, right)| left == right)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn discovers_literal_aliases_in_included_files() {
        let home = TempDir::new().expect("temporary home");
        let paths = WarpgatePaths::for_home(home.path());
        let included = home.path().join(".ssh/config.d");
        fs::create_dir_all(&included).expect("include directory");
        fs::write(
            &paths.user_ssh_config,
            "Include ~/.ssh/config.d/*.conf\nInclude ~/.ssh/warpgatesh/config\nHost manual\n",
        )
        .expect("root config");
        fs::write(
            included.join("lab.conf"),
            "Host db web-* !web-secret\n  User gregory\n",
        )
        .expect("included config");
        fs::create_dir_all(&paths.ssh_directory).expect("managed directory");
        fs::write(&paths.ssh_config, "Host generated\n").expect("managed config");

        let aliases = manual_host_aliases(&paths).expect("manual aliases");
        assert_eq!(
            aliases,
            HashSet::from(["manual".to_owned(), "db".to_owned()])
        );
    }

    #[test]
    fn installs_the_include_only_once() {
        let home = TempDir::new().expect("temporary home");
        let paths = WarpgatePaths::for_home(home.path());
        assert!(install_managed_include(&paths).expect("first install"));
        assert!(!install_managed_include(&paths).expect("second install"));
    }

    #[test]
    fn uninstalls_only_the_managed_include() {
        let directory = TempDir::new().expect("temporary directory");
        let paths = WarpgatePaths::for_home(directory.path());
        fs::create_dir_all(paths.user_ssh_config.parent().expect("SSH directory"))
            .expect("create SSH directory");
        fs::write(
            &paths.user_ssh_config,
            "Include ~/.ssh/warpgatesh/config\n\nHost example\n  User gregory\n",
        )
        .expect("write SSH config");

        assert!(uninstall_managed_include(&paths).expect("uninstall include"));
        assert_eq!(
            fs::read_to_string(&paths.user_ssh_config).expect("read SSH config"),
            "Host example\n  User gregory\n"
        );
        assert!(!uninstall_managed_include(&paths).expect("second uninstall"));
    }

    #[test]
    fn compares_host_keys_without_depending_on_host_labels_or_order() {
        let pinned = "host-a ssh-ed25519 AAAA\nhost-a ssh-rsa BBBB\n";
        let presented = "[host-b]:2222 ssh-rsa BBBB\n[host-b]:2222 ssh-ed25519 AAAA\n";
        assert_eq!(key_material(pinned), key_material(presented));
        assert_ne!(
            key_material(pinned),
            key_material("host-b ssh-ed25519 CCCC\n")
        );
    }
}
