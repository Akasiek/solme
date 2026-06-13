use std::{
    fs,
    path::{Path, PathBuf},
};

use super::models::StoredServerProfile;

pub struct ServerProfileStore {
    path: PathBuf,
}

impl ServerProfileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Option<StoredServerProfile>, String> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path)
            .map_err(|error| format!("Failed to read saved server profile: {error}"))?;
        serde_json::from_str(&contents)
            .map(Some)
            .map_err(|error| format!("Saved server profile is invalid: {error}"))
    }

    pub fn save(&self, profile: &StoredServerProfile) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "Server profile path has no parent directory".to_string())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create application config directory: {error}"))?;

        let contents = serde_json::to_vec_pretty(profile)
            .map_err(|error| format!("Failed to serialize server profile: {error}"))?;
        let temporary_path = temporary_path(&self.path);
        fs::write(&temporary_path, contents)
            .map_err(|error| format!("Failed to write server profile: {error}"))?;
        fs::rename(&temporary_path, &self.path)
            .map_err(|error| format!("Failed to replace server profile: {error}"))
    }

    pub fn delete(&self) -> Result<(), String> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("Failed to delete saved server profile: {error}")),
        }
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use uuid::Uuid;

    use super::ServerProfileStore;
    use crate::server::models::{ServerType, StoredServerProfile};

    #[test]
    fn saves_profile() {
        let directory = std::env::temp_dir().join(format!("solme-{}", Uuid::new_v4()));
        let path = directory.join("server-profile.json");
        let store = ServerProfileStore::new(path.clone());
        let profile = StoredServerProfile {
            id: Uuid::new_v4().to_string(),
            server_type: ServerType::Navidrome,
            url: "https://music.example.com".to_string(),
            username: "listener".to_string(),
        };

        store.save(&profile).unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        assert!(!contents.contains("password"));

        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.id, profile.id);
        assert_eq!(loaded.url, profile.url);
        assert_eq!(loaded.username, profile.username);

        store.delete().unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
