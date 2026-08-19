use std::{
    env,
    path::{Path, PathBuf},
};

use crate::{Error, Result};

#[derive(Clone, Debug)]
pub struct Paths {
    pub browser_dir: PathBuf,
    pub cwd: PathBuf,
    pub plugins_env: Option<String>,
}

impl Paths {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            browser_dir: browser_dir()?,
            cwd: env::current_dir()?,
            plugins_env: env::var("ROCCI_BROWSER_PLUGINS")
                .ok()
                .filter(|s| !s.is_empty()),
        })
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.browser_dir.join("plugins")
    }

    pub fn projects_path(&self) -> PathBuf {
        self.browser_dir.join("projects.json")
    }

    pub fn repo_local_path(&self) -> PathBuf {
        self.cwd.join(".rocci").join("browser.toml")
    }
}

pub fn browser_dir() -> Result<PathBuf> {
    if let Ok(dir) = env::var("ROCCI_BROWSER_DIR")
        && !dir.is_empty()
    {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(home) = env::var("ROCCI_HOME")
        && !home.is_empty()
    {
        return Ok(PathBuf::from(home).join(".rocci").join("browser"));
    }
    let home = env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .ok_or_else(|| Error::message("HOME is not set"))?;
    Ok(PathBuf::from(home).join(".rocci").join("browser"))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}
