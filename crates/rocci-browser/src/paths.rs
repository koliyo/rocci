use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

use crate::{Error, Result};

const GUI_PATH_DIRS: &[&str] = &["/usr/bin", "/bin", "/usr/sbin", "/sbin"];
const LAST_ROOT_FILE: &str = "last-root";

#[derive(Clone, Debug)]
pub struct Paths {
    pub browser_dir: PathBuf,
    pub cwd: PathBuf,
    pub plugins_env: Option<String>,
    pub ignore_cwd_repo_local: bool,
}

pub struct LaunchEnv<'a> {
    pub browser_dir: PathBuf,
    pub cwd: PathBuf,
    pub exe: &'a Path,
    pub plugins_env: Option<String>,
    pub explicit_root: Option<PathBuf>,
}

impl Paths {
    pub fn new(browser_dir: PathBuf, cwd: PathBuf) -> Self {
        Self {
            browser_dir,
            cwd,
            plugins_env: None,
            ignore_cwd_repo_local: false,
        }
    }

    pub fn from_env() -> Result<Self> {
        apply_gui_path_repair();
        let exe = env::current_exe().unwrap_or_default();
        Ok(Self::for_launch(LaunchEnv {
            browser_dir: browser_dir()?,
            cwd: env::current_dir()?,
            exe: &exe,
            plugins_env: env::var("ROCCI_BROWSER_PLUGINS")
                .ok()
                .filter(|s| !s.is_empty()),
            explicit_root: None,
        }))
    }

    pub fn for_launch(env: LaunchEnv<'_>) -> Self {
        if let Some(root) = env.explicit_root {
            return Self {
                browser_dir: env.browser_dir,
                cwd: root,
                plugins_env: env.plugins_env,
                ignore_cwd_repo_local: false,
            };
        }

        let bundled = is_bundled_exe(env.exe);
        let mut cwd = env.cwd;
        let mut ignore_cwd_repo_local = bundled || is_filesystem_root(&cwd);
        if bundled && let Some(last) = read_last_root_file(&env.browser_dir) {
            if last.is_dir() {
                cwd = fs::canonicalize(&last).unwrap_or(last);
                ignore_cwd_repo_local = false;
            }
        }

        Self {
            browser_dir: env.browser_dir,
            cwd,
            plugins_env: env.plugins_env,
            ignore_cwd_repo_local,
        }
    }

    pub fn set_explicit_root(&mut self, root: PathBuf) {
        self.cwd = root;
        self.ignore_cwd_repo_local = false;
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.browser_dir.join("plugins")
    }

    pub fn projects_path(&self) -> PathBuf {
        self.browser_dir.join("projects.json")
    }

    pub fn last_root_path(&self) -> PathBuf {
        self.browser_dir.join(LAST_ROOT_FILE)
    }

    pub fn repo_local_path(&self) -> PathBuf {
        self.cwd.join(".rocci").join("browser.toml")
    }

    pub fn repo_root(&self) -> PathBuf {
        self.cwd.clone()
    }

    pub fn persist_last_root(&self) -> Result<()> {
        if self.ignore_cwd_repo_local || is_filesystem_root(&self.cwd) {
            return Ok(());
        }
        ensure_dir(&self.browser_dir)?;
        fs::write(self.last_root_path(), self.cwd.to_string_lossy().as_bytes())?;
        Ok(())
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
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn is_bundled_exe(exe: &Path) -> bool {
    let Some(macos) = exe.parent() else {
        return false;
    };
    if macos.file_name().and_then(|name| name.to_str()) != Some("MacOS") {
        return false;
    }
    let Some(contents) = macos.parent() else {
        return false;
    };
    if contents.file_name().and_then(|name| name.to_str()) != Some("Contents") {
        return false;
    }
    let Some(app) = contents.parent() else {
        return false;
    };
    app.extension().and_then(|ext| ext.to_str()) == Some("app")
}

pub fn is_macos_gui_default_path(path: &str) -> bool {
    let dirs: Vec<&str> = path.split(':').filter(|dir| !dir.is_empty()).collect();
    !dirs.is_empty() && dirs.iter().all(|dir| GUI_PATH_DIRS.contains(dir))
}

pub fn repaired_gui_path(
    path: &str,
    home: Option<&Path>,
    mut exists: impl FnMut(&Path) -> bool,
) -> String {
    if !is_macos_gui_default_path(path) {
        return path.to_string();
    }
    let mut extras = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ];
    if let Some(home) = home {
        extras.push(home.join(".local/bin"));
        extras.push(home.join(".cargo/bin"));
    }
    let mut prefix = Vec::new();
    for dir in extras {
        if exists(&dir) {
            prefix.push(dir.display().to_string());
        }
    }
    if prefix.is_empty() {
        path.to_string()
    } else {
        format!("{}:{path}", prefix.join(":"))
    }
}

fn apply_gui_path_repair() {
    let Ok(current) = env::var("PATH") else {
        return;
    };
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"));
    let home = home.map(PathBuf::from);
    let repaired = repaired_gui_path(current.as_str(), home.as_deref(), |path| path.is_dir());
    if repaired != current {
        // SAFETY: called once at process start before adapter spawn; PATH is process-wide config.
        unsafe { env::set_var("PATH", repaired) };
    }
}

fn is_filesystem_root(path: &Path) -> bool {
    let mut components = path.components();
    matches!(components.next(), Some(Component::RootDir)) && components.next().is_none()
}

fn read_last_root_file(browser_dir: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(browser_dir.join(LAST_ROOT_FILE)).ok()?;
    let path = PathBuf::from(raw.trim());
    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "rocci-browser-paths-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn bundled_exe(root: &Path) -> PathBuf {
        let exe = root
            .join("Rocci Browser.app")
            .join("Contents")
            .join("MacOS")
            .join("rocci-browser");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, []).unwrap();
        exe
    }

    fn write_repo_local(root: &Path, id: &str) {
        fs::create_dir_all(root.join(".rocci")).unwrap();
        fs::write(
            root.join(".rocci").join("browser.toml"),
            format!("[[project]]\nid = \"{id}\"\npath = \".\"\n"),
        )
        .unwrap();
    }

    #[test]
    fn bundled_layout_is_detected() {
        let root = temp_dir();
        assert!(is_bundled_exe(&bundled_exe(&root)));
        assert!(!is_bundled_exe(&root.join("target/debug/rocci-browser")));
    }

    #[test]
    fn filesystem_root_skips_repo_local() {
        let root = temp_dir();
        let paths = Paths::for_launch(LaunchEnv {
            browser_dir: root.join("browser"),
            cwd: PathBuf::from("/"),
            exe: &root.join("rocci-browser"),
            plugins_env: None,
            explicit_root: None,
        });
        assert!(paths.ignore_cwd_repo_local);
        assert_eq!(paths.cwd, PathBuf::from("/"));
        assert!(crate::discovery::load_repo_local(&paths).unwrap().is_none());
    }

    #[test]
    fn bundled_launch_skips_cwd_repo_local() {
        let root = temp_dir();
        write_repo_local(&root, "from-cwd");
        let paths = Paths::for_launch(LaunchEnv {
            browser_dir: root.join("browser"),
            cwd: root.clone(),
            exe: &bundled_exe(&root),
            plugins_env: None,
            explicit_root: None,
        });
        assert!(paths.ignore_cwd_repo_local);
        assert!(crate::discovery::load_repo_local(&paths).unwrap().is_none());
    }

    #[test]
    fn explicit_root_wins_over_bundle_and_last_root() {
        let root = temp_dir();
        let last = root.join("last");
        let chosen = root.join("chosen");
        fs::create_dir_all(&last).unwrap();
        write_repo_local(&chosen, "chosen");
        fs::create_dir_all(root.join("browser")).unwrap();
        fs::write(
            root.join("browser").join(LAST_ROOT_FILE),
            last.display().to_string(),
        )
        .unwrap();
        let paths = Paths::for_launch(LaunchEnv {
            browser_dir: root.join("browser"),
            cwd: PathBuf::from("/"),
            exe: &bundled_exe(&root),
            plugins_env: None,
            explicit_root: Some(chosen.clone()),
        });
        assert!(!paths.ignore_cwd_repo_local);
        assert_eq!(paths.cwd, chosen);
        let local = crate::discovery::load_repo_local(&paths).unwrap().unwrap();
        assert_eq!(local.project[0].id, "chosen");
    }

    #[test]
    fn bundled_paths_use_user_registry_and_last_root() {
        let root = temp_dir();
        let project = root.join("project");
        let browser_dir = root.join("browser");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&browser_dir).unwrap();
        write_repo_local(&project, "from-last");
        fs::write(
            browser_dir.join("projects.json"),
            r#"{"projects":[{"id":"user","path":"/tmp/user"}]}"#,
        )
        .unwrap();
        fs::write(
            browser_dir.join(LAST_ROOT_FILE),
            project.display().to_string(),
        )
        .unwrap();

        let paths = Paths::for_launch(LaunchEnv {
            browser_dir,
            cwd: PathBuf::from("/"),
            exe: &bundled_exe(&root),
            plugins_env: None,
            explicit_root: None,
        });
        assert!(!paths.ignore_cwd_repo_local);
        assert_eq!(paths.cwd, fs::canonicalize(&project).unwrap());
        assert_ne!(paths.cwd, PathBuf::from("/"));
        let registry = Registry::load(&paths).unwrap();
        assert_eq!(registry.projects.len(), 2);
        assert_eq!(registry.projects[0].id, "user");
        assert_eq!(registry.projects[1].id, "from-last");
    }

    #[test]
    fn persist_last_root_skips_filesystem_root() {
        let root = temp_dir();
        let paths = Paths {
            browser_dir: root.join("browser"),
            cwd: PathBuf::from("/"),
            plugins_env: None,
            ignore_cwd_repo_local: false,
        };
        paths.persist_last_root().unwrap();
        assert!(!paths.last_root_path().exists());
    }

    #[test]
    fn persist_last_root_writes_real_cwd() {
        let root = temp_dir();
        let paths = Paths::new(root.join("browser"), root.clone());
        paths.persist_last_root().unwrap();
        assert_eq!(
            fs::read_to_string(paths.last_root_path()).unwrap(),
            root.to_string_lossy()
        );
    }

    #[test]
    fn gui_path_prepends_existing_user_bins() {
        let home = Path::new("/Users/me");
        let repaired = repaired_gui_path("/usr/bin:/bin:/usr/sbin:/sbin", Some(home), |path| {
            path == Path::new("/opt/homebrew/bin") || path == home.join(".cargo/bin").as_path()
        });
        assert_eq!(
            repaired,
            "/opt/homebrew/bin:/Users/me/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin"
        );
    }

    #[test]
    fn gui_path_leaves_terminal_path_unchanged() {
        let repaired = repaired_gui_path(
            "/opt/homebrew/bin:/usr/bin:/bin",
            Some(Path::new("/Users/me")),
            |_| true,
        );
        assert_eq!(repaired, "/opt/homebrew/bin:/usr/bin:/bin");
    }
}
