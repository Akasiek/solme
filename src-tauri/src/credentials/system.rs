use keyring::{Entry, Error};

use super::CredentialStore;

const KEYRING_SERVICE: &str = "com.solme.app";

pub struct SystemCredentialStore;

impl SystemCredentialStore {
    fn entry(id: &str) -> Result<Entry, String> {
        Entry::new(KEYRING_SERVICE, id)
            .map_err(|error| format!("Failed to access the system keyring: {error}"))
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
