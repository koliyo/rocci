use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;

use crate::run;

const SERVER_NAME: &str = "server";

pub fn bundle(config_path: &Path) -> Result<()> {
    let config = Config::from_file(config_path)?;
    let root = workspace_root(config_path)?;
    let app_dir = resolve_app_dir(config_path, &config)?;

    run::compile_rocci_modules(&app_dir)?;

    match env::consts::OS {
        "macos" => bundle_macos(&root, &app_dir, &config, config_path),
        other => bail!("development bundling is not implemented for {other} yet"),
    }
}

fn bundle_macos(root: &Path, app_dir: &Path, config: &Config, config_path: &Path) -> Result<()> {
    let host_name = host_binary_name(&config.app.name)?;
    let bundle_dir = root
        .join("target/release/bundle/macos")
        .join(format!("{}.app", config.app.name));
    let contents = bundle_dir.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    let bundled_app = resources.join("app");

    if bundle_dir.exists() {
        fs::remove_dir_all(&bundle_dir)
            .with_context(|| format!("failed to replace {}", bundle_dir.display()))?;
    }
    fs::create_dir_all(&macos)?;
    fs::create_dir_all(&bundled_app)?;

    let server = bundled_app.join(SERVER_NAME);
    build_roc_server(app_dir, &server)?;
    let host = build_host(root)?;
    fs::copy(&host, macos.join(&host_name))
        .with_context(|| format!("failed to copy {}", host.display()))?;

    copy_app_assets(app_dir, &bundled_app)?;
    for resource in &config.bundle.resources {
        let from = root.join(&resource.from);
        let to = resources.join(&resource.to);
        copy_tree(&from, &to)
            .with_context(|| format!("failed to copy {} -> {}", from.display(), to.display()))?;
    }

    let dest_config = resources.join("rocci.toml");
    fs::copy(
        root.join(config_path_relative(root, config_path)?),
        &dest_config,
    )
    .or_else(|_| fs::copy(config_path, &dest_config))
    .with_context(|| "failed to copy rocci.toml into the app bundle")?;

    fs::write(
        contents.join("Info.plist"),
        info_plist(root, config, &host_name)?,
    )?;
    fs::write(contents.join("PkgInfo"), b"APPL????")?;

    let status = Command::new("codesign")
        .args(["--force", "--deep", "--sign", "-"])
        .arg(&bundle_dir)
        .status()
        .context("failed to run codesign")?;
    if !status.success() {
        bail!("ad-hoc codesign failed");
    }

    println!("{}", bundle_dir.display());
    Ok(())
}

fn build_roc_server(app_dir: &Path, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let status = Command::new("roc")
        .current_dir(app_dir)
        .arg("build")
        .arg("main.roc")
        .arg(format!("--output={}", output.display()))
        .status()
        .context("failed to run `roc build`; is roc on PATH?")?;
    if !status.success() {
        bail!("roc build failed");
    }
    if !output.is_file() {
        bail!("roc build did not write {}", output.display());
    }
    Ok(())
}

fn build_host(root: &Path) -> Result<PathBuf> {
    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(root)
        .args(["build", "--release", "-p", "rocci-cli"])
        .status()
        .context("failed to run cargo build")?;
    if !status.success() {
        bail!("cargo build failed");
    }
    let host = root.join("target/release/rocci");
    if !host.is_file() {
        bail!("cargo build did not write {}", host.display());
    }
    Ok(host)
}

fn copy_app_assets(app_dir: &Path, bundled_app: &Path) -> Result<()> {
    let assets = app_dir.join("assets");
    if assets.is_dir() {
        copy_tree(&assets, &bundled_app.join("assets"))?;
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    if from.is_file() {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(from, to)?;
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        copy_tree(&entry.path(), &dest)?;
    }
    Ok(())
}

fn workspace_root(config_path: &Path) -> Result<PathBuf> {
    let mut dir = env::current_dir()?;
    if config_path.is_absolute() {
        if let Some(parent) = config_path.parent() {
            dir = parent.to_path_buf();
        }
    } else if let Some(parent) = config_path.parent()
        && !parent.as_os_str().is_empty()
    {
        dir = dir.join(parent);
    }
    loop {
        let cargo = dir.join("Cargo.toml");
        if cargo.is_file() {
            let text = fs::read_to_string(&cargo)?;
            if text.contains("[workspace]") {
                return Ok(dir);
            }
        }
        dir = dir
            .parent()
            .map(Path::to_path_buf)
            .context("rocci.toml is not inside a Cargo workspace")?;
    }
}

fn config_path_relative(root: &Path, config_path: &Path) -> Result<PathBuf> {
    if config_path.is_absolute() {
        return Ok(config_path
            .strip_prefix(root)
            .unwrap_or(config_path)
            .to_path_buf());
    }
    Ok(config_path.to_path_buf())
}

fn resolve_app_dir(config_path: &Path, config: &Config) -> Result<PathBuf> {
    let config_dir = config_file_dir(config_path)?;
    let app = match &config.bundle.app {
        Some(app) if app.as_os_str() != "." => {
            if app.is_absolute() {
                app.clone()
            } else {
                config_dir.join(app)
            }
        }
        _ => config_dir,
    };
    if !app.join("main.roc").is_file() {
        bail!("bundle.app {} has no main.roc", app.display());
    }
    Ok(app)
}

fn config_file_dir(config_path: &Path) -> Result<PathBuf> {
    let parent = config_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    if config_path.is_absolute() {
        Ok(parent)
    } else {
        Ok(env::current_dir()?.join(parent))
    }
}

fn host_binary_name(name: &str) -> Result<String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        bail!("app.name {name:?} is not a valid bundle executable name");
    }
    Ok(name.to_string())
}

fn info_plist(root: &Path, config: &Config, executable: &str) -> Result<String> {
    if let Some(template) = &config.bundle.macos_plist {
        let path = root.join(template);
        let plist = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        return Ok(rewrite_plist(&plist, config, executable));
    }
    Ok(generate_plist(config, executable))
}

fn rewrite_plist(template: &str, config: &Config, executable: &str) -> String {
    let identifier = config
        .bundle
        .identifier
        .as_deref()
        .unwrap_or(&config.app.identifier);
    let version = config.app.version.as_deref().unwrap_or("0.1.0");
    template
        .replace("{{appName}}", &config.app.name)
        .replace("{{executable}}", executable)
        .replace("{{identifier}}", identifier)
        .replace("{{version}}", version)
}

fn generate_plist(config: &Config, executable: &str) -> String {
    let identifier = config
        .bundle
        .identifier
        .as_deref()
        .unwrap_or(&config.app.identifier);
    let version = config.app.version.as_deref().unwrap_or("0.1.0");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>{name}</string>
  <key>CFBundleExecutable</key>
  <string>{executable}</string>
  <key>CFBundleIdentifier</key>
  <string>{identifier}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>{name}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{version}</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>12.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key>
  <true/>
</dict>
</plist>
"#,
        name = config.app.name
    )
}

pub fn bundled_resources() -> Option<PathBuf> {
    bundled_resources_from(&env::current_exe().ok()?)
}

fn bundled_resources_from(exe: &Path) -> Option<PathBuf> {
    let resources = exe.parent()?.parent()?.join("Resources");
    if resources.join("rocci.toml").is_file() && resources.join("app").join(SERVER_NAME).is_file() {
        Some(resources)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("rocci-bundle-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn host_binary_name_uses_app_name() {
        assert_eq!(host_binary_name("Counter").unwrap(), "Counter");
        assert!(host_binary_name("Counter App").is_err());
        assert!(host_binary_name("").is_err());
    }

    #[test]
    fn generated_plist_sets_executable_and_identity() {
        let config = Config::from_toml(
            r#"
            [app]
            name = "Counter"
            identifier = "dev.rocci.counter"
            version = "0.1.0"
            "#,
        )
        .unwrap();
        let plist = generate_plist(&config, "Counter");
        assert!(plist.contains("<string>Counter</string>"));
        assert!(plist.contains("<string>dev.rocci.counter</string>"));
        assert!(plist.contains("<key>CFBundleExecutable</key>"));
    }

    #[test]
    fn rewrite_plist_fills_placeholders() {
        let config = Config::from_toml(
            r#"
            [app]
            name = "Counter"
            identifier = "dev.rocci.counter"
            version = "2.0.0"
            "#,
        )
        .unwrap();
        let rewritten = rewrite_plist(
            r#"
            <key>CFBundleExecutable</key>
            <string>{{appName}}</string>
            <key>CFBundleDisplayName</key>
            <string>{{appName}}</string>
            <key>CFBundleIdentifier</key>
            <string>{{identifier}}</string>
            <key>CFBundleShortVersionString</key>
            <string>{{version}}</string>
            "#,
            &config,
            "Counter",
        );
        assert!(rewritten.contains("<string>Counter</string>"));
        assert!(rewritten.contains("<string>dev.rocci.counter</string>"));
        assert!(rewritten.contains("<string>2.0.0</string>"));
        assert!(!rewritten.contains("{{appName}}"));
        assert!(!rewritten.contains("{{identifier}}"));
        assert!(!rewritten.contains("{{version}}"));
    }

    #[test]
    fn bundled_resources_detects_macos_layout() {
        let dir = temp_dir("layout");
        let macos = dir.join("Contents/MacOS");
        let resources = dir.join("Contents/Resources");
        fs::create_dir_all(&macos).unwrap();
        fs::create_dir_all(resources.join("app")).unwrap();
        fs::write(resources.join("rocci.toml"), "").unwrap();
        fs::write(resources.join("app/server"), "").unwrap();
        let exe = macos.join("Counter");
        fs::write(&exe, "").unwrap();
        assert_eq!(bundled_resources_from(&exe), Some(resources));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundled_resources_ignores_plain_binaries() {
        let dir = temp_dir("plain");
        let exe = dir.join("rocci");
        fs::write(&exe, "").unwrap();
        assert_eq!(bundled_resources_from(&exe), None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_app_dir_joins_relative_bundle_app() {
        let dir = temp_dir("app");
        let app = dir.join("examples/counter");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("main.roc"), "app").unwrap();
        fs::write(dir.join("rocci.toml"), "").unwrap();
        let config = Config::from_toml(
            r#"
            [app]
            identifier = "dev.rocci.counter"
            [bundle]
            app = "examples/counter"
            "#,
        )
        .unwrap();
        let resolved = resolve_app_dir(&dir.join("rocci.toml"), &config).unwrap();
        assert_eq!(resolved, app);
        let _ = fs::remove_dir_all(&dir);
    }
}
