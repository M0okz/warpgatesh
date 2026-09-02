use crate::RuntimeError;

const SERVICE: &str = "dev.warpgatesh.api-token";

pub trait TokenStore {
    /// Store or replace the API token for a profile.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the native secret store rejects the write.
    fn set(&self, profile: &str, token: &str) -> Result<(), RuntimeError>;

    /// Retrieve the API token for a profile.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the token is absent or unreadable.
    fn get(&self, profile: &str) -> Result<String, RuntimeError>;

    /// Delete a profile API token.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the native secret store rejects deletion.
    fn delete(&self, profile: &str) -> Result<(), RuntimeError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemKeychain;

#[cfg(target_os = "macos")]
impl TokenStore for SystemKeychain {
    fn set(&self, profile: &str, token: &str) -> Result<(), RuntimeError> {
        security_framework::passwords::set_generic_password(SERVICE, profile, token.as_bytes())
            .map_err(|error| RuntimeError::Keychain(error.to_string()))
    }

    fn get(&self, profile: &str) -> Result<String, RuntimeError> {
        let bytes = security_framework::passwords::get_generic_password(SERVICE, profile)
            .map_err(|error| RuntimeError::Keychain(error.to_string()))?;
        String::from_utf8(bytes)
            .map_err(|_| RuntimeError::Keychain("stored token is not valid UTF-8".to_owned()))
    }

    fn delete(&self, profile: &str) -> Result<(), RuntimeError> {
        match security_framework::passwords::delete_generic_password(SERVICE, profile) {
            Ok(()) => Ok(()),
            // A partially removed profile must not make a later cleanup fail.
            Err(error) if error.code() == -25_300 => Ok(()),
            Err(error) => Err(RuntimeError::Keychain(error.to_string())),
        }
    }
}

#[cfg(target_os = "linux")]
impl TokenStore for SystemKeychain {
    fn set(&self, profile: &str, token: &str) -> Result<(), RuntimeError> {
        linux_entry(profile)?
            .set_password(token)
            .map_err(|error| linux_error(&error))
    }

    fn get(&self, profile: &str) -> Result<String, RuntimeError> {
        linux_entry(profile)?
            .get_password()
            .map_err(|error| linux_error(&error))
    }

    fn delete(&self, profile: &str) -> Result<(), RuntimeError> {
        match linux_entry(profile)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(linux_error(&error)),
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_entry(profile: &str) -> Result<keyring::Entry, RuntimeError> {
    keyring::Entry::new(SERVICE, profile).map_err(|error| linux_error(&error))
}

#[cfg(target_os = "linux")]
fn linux_error(error: &keyring::Error) -> RuntimeError {
    RuntimeError::Keychain(format!("Linux Secret Service: {error}"))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl TokenStore for SystemKeychain {
    fn set(&self, _profile: &str, _token: &str) -> Result<(), RuntimeError> {
        Err(unsupported())
    }

    fn get(&self, _profile: &str) -> Result<String, RuntimeError> {
        Err(unsupported())
    }

    fn delete(&self, _profile: &str) -> Result<(), RuntimeError> {
        Err(unsupported())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn unsupported() -> RuntimeError {
    RuntimeError::Keychain("native secret storage is not supported on this platform".to_owned())
}
