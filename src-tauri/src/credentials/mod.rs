mod system;

pub use system::SystemCredentialStore;

pub trait CredentialStore: Send + Sync {
    fn save(&self, id: &str, password: &str) -> Result<(), String>;
    fn load(&self, id: &str) -> Result<String, String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}
