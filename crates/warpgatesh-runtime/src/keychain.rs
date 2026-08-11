use crate::RuntimeError;

#[cfg(target_os = "macos")]
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
        security_framework::passwords::delete_generic_password(SERVICE, profile)
            .map_err(|error| RuntimeError::Keychain(error.to_string()))
    }
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(not(target_os = "macos"))]
fn unsupported() -> RuntimeError {
    RuntimeError::Keychain("native Linux secret storage is not implemented yet".to_owned())
}
