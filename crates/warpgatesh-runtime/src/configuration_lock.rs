use crate::{RuntimeError, storage::LocalStore};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};

pub(crate) struct ConfigurationLock(File);
impl ConfigurationLock {
    pub(crate) fn acquire(store: &LocalStore) -> Result<Self, RuntimeError> {
        fs::create_dir_all(&store.paths().application_support)?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            options.mode(0o600);
            fs::set_permissions(
                &store.paths().application_support,
                fs::Permissions::from_mode(0o700),
            )?;
        }
        let file = options.open(store.paths().application_support.join("configuration.lock"))?;
        file.try_lock_exclusive().map_err(|_| {
            RuntimeError::Command(
                "A credential operation is already in progress. Try again shortly.".to_owned(),
            )
        })?;
        Ok(Self(file))
    }
}
impl Drop for ConfigurationLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}
