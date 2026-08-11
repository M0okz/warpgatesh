use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Url(url::ParseError),
    Http(reqwest::Error),
    Unauthorized,
    Incompatible(String),
    InvalidInput(String),
    Keychain(String),
    Command(String),
    Profile(warpgatesh_core::profiles::ProfileError),
    SshConfig(warpgatesh_core::ssh_config::SshConfigError),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "invalid JSON: {error}"),
            Self::Url(error) => write!(formatter, "invalid URL: {error}"),
            Self::Http(error) => write!(formatter, "Warpgate request failed: {error}"),
            Self::Unauthorized => formatter.write_str("Warpgate rejected the API token"),
            Self::Incompatible(message) => {
                write!(formatter, "incompatible Warpgate API: {message}")
            }
            Self::InvalidInput(message) | Self::Command(message) => formatter.write_str(message),
            Self::Keychain(message) => write!(formatter, "keychain error: {message}"),
            Self::Profile(error) => error.fmt(formatter),
            Self::SshConfig(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Url(error) => Some(error),
            Self::Http(error) => Some(error),
            Self::Profile(error) => Some(error),
            Self::SshConfig(error) => Some(error),
            Self::Unauthorized
            | Self::Incompatible(_)
            | Self::InvalidInput(_)
            | Self::Keychain(_)
            | Self::Command(_) => None,
        }
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for RuntimeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<url::ParseError> for RuntimeError {
    fn from(error: url::ParseError) -> Self {
        Self::Url(error)
    }
}

impl From<reqwest::Error> for RuntimeError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<warpgatesh_core::profiles::ProfileError> for RuntimeError {
    fn from(error: warpgatesh_core::profiles::ProfileError) -> Self {
        Self::Profile(error)
    }
}

impl From<warpgatesh_core::ssh_config::SshConfigError> for RuntimeError {
    fn from(error: warpgatesh_core::ssh_config::SshConfigError) -> Self {
        Self::SshConfig(error)
    }
}
