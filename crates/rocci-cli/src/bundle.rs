use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use rocci_core::Config;

use crate::datastar_asset;
use crate::run;
use crate::runtime_assets;

const SERVER_NAME: &str = "server";

pub fn bundle(
    config_path: &Path,
    target: Option<crate::native_target::NativeTarget>,
) -> Result<()> {
    let config = Config::from_file(config_path)?;
    let root = workspace_root(config_path)?;
    let app_dir = resolve_app_dir(config_path, &config)?;

    datastar_asset::ensure_app(&app_dir, datastar_asset::HintMode::Quiet)?;
    runtime_assets::stage_into(&app_dir)?;
    run::compile_rocci_modules(&app_dir)?;

    match env::consts::OS {
        "macos" => bundle_macos(&root, &app_dir, &config, config_path, target),
        other => bail!("development bundling is not implemented for {other} yet"),
    }
}

fn bundle_macos(
    root: &Path,
    app_dir: &Path,
    config: &Config,
    config_path: &Path,
    target: Option<crate::native_target::NativeTarget>,
) -> Result<()> {
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
    copy_app_icon(root, &resources)?;

    let server = bundled_app.join(SERVER_NAME);
    if target.is_some() {
        bail!(
            "macOS .app bundles require a host-native server; pass --target only for Linux process binaries"
        );
    }
    crate::native_target::build_roc_server(app_dir, &server, target)?;
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

    println!(
        "{}",
        crate::style::success_text(&bundle_dir.display().to_string())
    );
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

fn copy_app_icon(root: &Path, resources: &Path) -> Result<()> {
    let src = root.join("brand/rocci-app.icns");
    if !src.is_file() {
        bail!("missing macOS app icon {}", src.display());
    }
    fs::copy(&src, resources.join("AppIcon.icns"))
        .with_context(|| format!("failed to copy {}", src.display()))?;
    Ok(())
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

#[derive(Debug, Clone)]
pub struct ServerPackage {
    pub output: PathBuf,
    pub server: PathBuf,
}

pub fn package_server(
    input: &Path,
    output: Option<&Path>,
    target: Option<crate::native_target::NativeTarget>,
) -> Result<ServerPackage> {
    package_server_with_options(input, output, target, false)
}

pub fn package_server_with_options(
    input: &Path,
    output: Option<&Path>,
    target: Option<crate::native_target::NativeTarget>,
    verbose: bool,
) -> Result<ServerPackage> {
    package_server_with_opt(input, output, target, verbose, None, None)
}

pub fn package_server_with_opt(
    input: &Path,
    output: Option<&Path>,
    target: Option<crate::native_target::NativeTarget>,
    verbose: bool,
    opt: Option<crate::native_target::RocOpt>,
    platform: Option<String>,
) -> Result<ServerPackage> {
    let cwd = env::current_dir()?;
    let input = if input.is_absolute() {
        input.to_path_buf()
    } else {
        cwd.join(input)
    };
    let output = match output {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => cwd.join("target/release/rocci-server"),
    };
    if input.starts_with(&output) {
        bail!(
            "`--output {}` would delete the input tree {}",
            output.display(),
            input.display()
        );
    }
    if output.exists() {
        fs::remove_dir_all(&output)
            .with_context(|| format!("failed to replace {}", output.display()))?;
    }
    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create {}", output.display()))?;
    let server = output.join(SERVER_NAME);
    let assets = output.join("assets");
    fs::create_dir_all(&assets)?;

    match resolve_server_input(&input)? {
        ServerInput::AppDir(app_dir) => {
            datastar_asset::ensure_app(&app_dir, datastar_asset::HintMode::Quiet)?;
            runtime_assets::stage_into(&app_dir)?;
            run::compile_rocci_modules(&app_dir)?;
            crate::native_target::build_roc_server_with_opt(
                &app_dir, &server, target, verbose, opt,
            )?;
            copy_app_assets(&app_dir, &output)?;
        }
        ServerInput::Standalone(file) => {
            let src_dir = file
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .unwrap_or_else(|| cwd.clone());
            let mut plan = run::standalone_app_plan(&file)?;
            plan.platform = platform;
            crate::driver::compile_app_plan_with_opt(
                &plan, &src_dir, &server, target, verbose, opt,
            )?;
            if src_dir.join("assets").is_dir() {
                copy_tree(&src_dir.join("assets"), &assets)?;
            }
            let version = datastar_asset::stage_version_for_dir(&src_dir)
                .unwrap_or_else(|| datastar_asset::DEFAULT_VERSION.to_string());
            datastar_asset::stage_into(&assets, &version)?;
        }
    }
    fs::create_dir_all(&assets)?;
    if !server.is_file() {
        bail!("server package did not write {}", server.display());
    }
    Ok(ServerPackage { output, server })
}

#[derive(Debug)]
enum ServerInput {
    AppDir(PathBuf),
    Standalone(PathBuf),
}

fn resolve_server_input(input: &Path) -> Result<ServerInput> {
    if input.is_file() {
        if input.extension().and_then(|ext| ext.to_str()) == Some("rocci") {
            return Ok(ServerInput::Standalone(input.to_path_buf()));
        }
        if input.file_name().and_then(|name| name.to_str()) == Some("rocci.toml") {
            let config = Config::from_file(input)?;
            return Ok(ServerInput::AppDir(resolve_app_dir(input, &config)?));
        }
        bail!(
            "`rocci build --release` expected a .rocci file, app directory, or rocci.toml; got {}",
            input.display()
        );
    }
    if !input.is_dir() {
        bail!("no such path: {}", input.display());
    }
    if input.join("main.roc").is_file() {
        return Ok(ServerInput::AppDir(input.to_path_buf()));
    }
    Ok(ServerInput::Standalone(run::resolve_standalone_entry(
        input,
    )?))
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
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
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

    fn skip_without_roc() -> bool {
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() != Some("1") {
            eprintln!("skipping: ROCCI_REQUIRE_ROC is not 1");
            return true;
        }
        let help_ok = Command::new("roc")
            .arg("help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !help_ok {
            panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
        }
        false
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
        assert!(plist.contains("<key>CFBundleIconFile</key>"));
        assert!(plist.contains("<string>AppIcon</string>"));
    }

    #[test]
    fn copy_app_icon_writes_resources_icns() {
        let dir = temp_dir("icon");
        let brand = dir.join("brand");
        let resources = dir.join("Contents/Resources");
        fs::create_dir_all(&brand).unwrap();
        fs::create_dir_all(&resources).unwrap();
        fs::write(brand.join("rocci-app.icns"), b"icns").unwrap();
        copy_app_icon(&dir, &resources).unwrap();
        assert_eq!(fs::read(resources.join("AppIcon.icns")).unwrap(), b"icns");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn copy_app_icon_requires_brand_icns() {
        let dir = temp_dir("icon-missing");
        let resources = dir.join("Contents/Resources");
        fs::create_dir_all(&resources).unwrap();
        let err = copy_app_icon(&dir, &resources).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("missing macOS app icon"), "{message}");
        let _ = fs::remove_dir_all(&dir);
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
        let app = dir.join("examples/rocci/standalone/counter");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("main.roc"), "app").unwrap();
        fs::write(dir.join("rocci.toml"), "").unwrap();
        let config = Config::from_toml(
            r#"
            [app]
            identifier = "dev.rocci.counter"
            [bundle]
            app = "examples/rocci/standalone/counter"
            "#,
        )
        .unwrap();
        let resolved = resolve_app_dir(&dir.join("rocci.toml"), &config).unwrap();
        assert_eq!(resolved, app);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_server_input_prefers_main_roc_then_standalone() {
        let dir = temp_dir("server-input");
        fs::write(dir.join("App.rocci"), "").unwrap();
        match resolve_server_input(&dir).unwrap() {
            ServerInput::Standalone(path) => assert_eq!(path, dir.join("App.rocci")),
            ServerInput::AppDir(_) => panic!("expected standalone"),
        }
        fs::write(dir.join("main.roc"), "app").unwrap();
        match resolve_server_input(&dir).unwrap() {
            ServerInput::AppDir(path) => assert_eq!(path, dir),
            ServerInput::Standalone(_) => panic!("expected app dir"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn resolve_server_input_live_counter_uses_unique_init() {
        let dir = repo_root().join("examples/rocci/standalone/live-counter");
        if !dir.is_dir() {
            return;
        }
        match resolve_server_input(&dir).expect("live-counter") {
            ServerInput::Standalone(path) => {
                assert_eq!(path, dir.join("LiveCounter.rocci"));
            }
            ServerInput::AppDir(_) => panic!("expected standalone LiveCounter.rocci"),
        }
    }

    #[test]
    fn resolve_server_input_nested_unique_init() {
        let dir = repo_root().join("examples/rocci/standalone/blocks");
        if !dir.is_dir() {
            return;
        }
        match resolve_server_input(&dir).expect("blocks") {
            ServerInput::Standalone(path) => {
                assert_eq!(path, dir.join("backend/Blocks.rocci"));
            }
            ServerInput::AppDir(_) => panic!("expected standalone Blocks.rocci"),
        }
    }

    #[test]
    fn resolve_server_input_ambiguous_directory_lists_candidates() {
        let dir = temp_dir("ambiguous-release");
        fs::write(
            dir.join("Alpha.rocci"),
            r#"
import Html

@get:view("/") = |_| {
    page({})
}

@component Page = |{}|
    <html><body><p>a</p></body></html>
"#,
        )
        .unwrap();
        fs::write(
            dir.join("Beta.rocci"),
            r#"
import Html

@get:view("/") = |_| {
    page({})
}

@component Page = |{}|
    <html><body><p>b</p></body></html>
"#,
        )
        .unwrap();
        let err = resolve_server_input(&dir).unwrap_err().to_string();
        assert!(err.contains("ambiguous standalone app"), "{err}");
        assert!(!err.contains("pass one .rocci file"), "{err}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_server_refuses_output_inside_input_tree() {
        let dir = temp_dir("server-unsafe");
        fs::write(dir.join("App.rocci"), "").unwrap();
        let err = package_server(&dir, Some(&dir), None).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("would delete"), "{message}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn package_datastar_answers_get_without_roc_on_path() {
        if skip_without_roc() {
            return;
        }
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest.parent().unwrap().parent().unwrap();
        let app = root.join("examples/rocci/custom/datastar");
        let output = env::temp_dir().join(format!(
            "rocci-server-pkg-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let report = package_server(&app, Some(&output), None).unwrap();
        assert!(report.server.is_file());
        assert!(output.join("assets").is_dir());

        let spy = output.join("roc-spy");
        fs::create_dir_all(&spy).unwrap();
        let spy_log = spy.join("invoked.log");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let roc = spy.join("roc");
            fs::write(
                &roc,
                format!(
                    "#!/bin/sh\necho ROC_INVOKED >> \"{}\"\nexit 42\n",
                    spy_log.display()
                ),
            )
            .unwrap();
            let mut perms = fs::metadata(&roc).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&roc, perms).unwrap();
        }
        let port = crate::serve::free_port().unwrap();
        let mut path = spy.display().to_string();
        if let Ok(existing) = env::var("PATH") {
            path.push(':');
            path.push_str(&existing);
        }
        let mut child = Command::new(&report.server)
            .current_dir(&output)
            .env("PATH", &path)
            .env("ROC_BASIC_WEBSERVER_HOST", "127.0.0.1")
            .env("ROC_BASIC_WEBSERVER_PORT", port.to_string())
            .spawn()
            .unwrap();
        let start = std::time::Instant::now();
        let body = loop {
            if let Ok(Some(status)) = child.try_wait() {
                panic!("server exited before GET / ({status})");
            }
            if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
                use std::io::{Read, Write};
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
                let req = format!(
                    "GET / HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
                );
                if stream.write_all(req.as_bytes()).is_ok() {
                    let mut buf = Vec::new();
                    let _ = stream.read_to_end(&mut buf);
                    let text = String::from_utf8_lossy(&buf).into_owned();
                    if text.contains("200") || text.contains("<html") || text.contains("<!doctype")
                    {
                        break text;
                    }
                }
            }
            if start.elapsed() > std::time::Duration::from_secs(15) {
                let _ = child.kill();
                panic!("timed out waiting for packaged server on {port}");
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        };
        let signals = {
            use std::io::{Read, Write};
            let mut stream = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            let req = format!(
                "GET /actions/signals/compose HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(req.as_bytes()).unwrap();
            let mut buf = Vec::new();
            let _ = stream.read_to_end(&mut buf);
            String::from_utf8_lossy(&buf).into_owned()
        };
        let _ = child.kill();
        let _ = child.wait();
        assert!(
            body.contains("200") || body.contains("<html") || body.contains("<!doctype"),
            "{body}"
        );
        assert!(
            !spy_log.exists(),
            "packaged server must not invoke roc: {}",
            fs::read_to_string(&spy_log).unwrap_or_default()
        );
        assert!(
            signals.contains("event: datastar-patch-elements"),
            "{signals}"
        );
        assert!(
            signals.contains("data: elements <output id=\"signal-ceiling\">ready</output>"),
            "{signals}"
        );
        assert!(
            signals.contains("event: datastar-patch-signals"),
            "{signals}"
        );
        assert!(signals.contains("data: onlyIfMissing true"), "{signals}");
        assert!(
            signals.contains("data: signals {\"notice\":\"ready\"}"),
            "{signals}"
        );
        let _ = fs::remove_dir_all(&output);
    }
}
