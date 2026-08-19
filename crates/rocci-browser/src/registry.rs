use std::fs;

use serde::{Deserialize, Serialize};

use crate::{
    Result,
    paths::{Paths, ensure_dir},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub projects: Vec<Project>,
}

impl Registry {
    pub fn load(paths: &Paths) -> Result<Self> {
        ensure_dir(&paths.browser_dir)?;
        let file = paths.projects_path();
        if !file.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(file)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        ensure_dir(&paths.browser_dir)?;
        let file = paths.projects_path();
        let raw = serde_json::to_string_pretty(self)?;
        fs::write(file, raw)?;
        Ok(())
    }

    pub fn add(&mut self, id: String, path: String) {
        match self.projects.iter_mut().find(|project| project.id == id) {
            Some(existing) => existing.path = path,
            None => self.projects.push(Project { id, path }),
        }
    }

    pub fn remove(&mut self, query: &str) -> bool {
        let before = self.projects.len();
        self.projects
            .retain(|project| project.id != query && project.path != query);
        self.projects.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_paths() -> Paths {
        let dir = std::env::temp_dir().join(format!(
            "rocci-browser-reg-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        Paths {
            browser_dir: dir,
            cwd: std::env::temp_dir(),
            plugins_env: None,
        }
    }

    #[test]
    fn round_trips_projects_json() {
        let paths = temp_paths();
        let mut registry = Registry::default();
        registry.add("fixture".into(), "/tmp/fixture".into());
        registry.save(&paths).unwrap();
        let loaded = Registry::load(&paths).unwrap();
        assert_eq!(loaded.projects[0].id, "fixture");
        assert!(registry.remove("fixture"));
        registry.save(&paths).unwrap();
        assert!(Registry::load(&paths).unwrap().projects.is_empty());
    }
}
