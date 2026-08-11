use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WarpgatePaths {
    pub application_support: PathBuf,
    pub profiles: PathBuf,
    pub agent_socket: PathBuf,
    pub ssh_directory: PathBuf,
    pub ssh_config: PathBuf,
    pub known_hosts_directory: PathBuf,
}

impl WarpgatePaths {
    #[must_use]
    pub fn for_home(home: &Path) -> Self {
        let application_support = if cfg!(target_os = "macos") {
            home.join("Library/Application Support/WarpgateSH")
        } else {
            home.join(".local/share/warpgatesh")
        };
        let ssh_directory = home.join(".ssh/warpgatesh");

        Self {
            profiles: application_support.join("profiles.json"),
            agent_socket: application_support.join("agent.sock"),
            ssh_config: ssh_directory.join("config"),
            known_hosts_directory: ssh_directory.join("known_hosts"),
            application_support,
            ssh_directory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolates_ssh_files_below_the_managed_directory() {
        let paths = WarpgatePaths::for_home(Path::new("/Users/tester"));
        assert_eq!(
            paths.ssh_config,
            Path::new("/Users/tester/.ssh/warpgatesh/config")
        );
        assert_eq!(
            paths.known_hosts_directory,
            Path::new("/Users/tester/.ssh/warpgatesh/known_hosts")
        );
    }
}
