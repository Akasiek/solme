use keyring_core::{CredentialStore as KeyringStore, Entry, Error};

use super::CredentialStore;

const KEYRING_SERVICE: &str = "com.akasiek.solme";

pub struct SystemCredentialStore;

impl SystemCredentialStore {
    pub fn new() -> Result<Self, String> {
        let store = system_store()
            .map_err(|error| format!("Failed to initialize the system keyring: {error}"))?;
        keyring_core::set_default_store(store);
        Ok(Self)
    }

    fn entry(id: &str) -> Result<Entry, String> {
        Entry::new(KEYRING_SERVICE, id)
            .map_err(|error| format!("Failed to access the system keyring: {error}"))
    }
}

impl Drop for SystemCredentialStore {
    fn drop(&mut self) {
        keyring_core::unset_default_store();
    }
}

impl CredentialStore for SystemCredentialStore {
    fn save(&self, id: &str, password: &str) -> Result<(), String> {
        Self::entry(id)?
            .set_password(password)
            .map_err(|error| format!("Failed to save password in the system keyring: {error}"))
    }

    fn load(&self, id: &str) -> Result<String, String> {
        Self::entry(id)?
            .get_password()
            .map_err(|error| format!("Failed to load password from the system keyring: {error}"))
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        match Self::entry(id)?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(format!(
                "Failed to delete password from the system keyring: {error}"
            )),
        }
    }
}

#[cfg(target_os = "linux")]
fn system_store() -> keyring_core::Result<std::sync::Arc<KeyringStore>> {
    dbus_secret_service_keyring_store::Store::new().map(|store| store as _)
}

#[cfg(target_os = "macos")]
fn system_store() -> keyring_core::Result<std::sync::Arc<KeyringStore>> {
    apple_native_keyring_store::keychain::Store::new().map(|store| store as _)
}

#[cfg(target_os = "windows")]
fn system_store() -> keyring_core::Result<std::sync::Arc<KeyringStore>> {
    windows_native_keyring_store::Store::new().map(|store| store as _)
}
