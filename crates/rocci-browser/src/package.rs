use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub const APP_DISPLAY_NAME: &str = "Rocci Browser";
pub const EXECUTABLE_NAME: &str = "rocci-browser";
pub const BUNDLE_IDENTIFIER: &str = "dev.rocci.browser";
pub const ICON_FILE: &str = "AppIcon";
pub const DOCUMENT_ICON_FILE: &str = "RocciDocument";

pub struct AssembleOptions {
    pub executable: PathBuf,
    pub app_dir: PathBuf,
    pub icon_png: Option<PathBuf>,
}

pub fn run(executable: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (executable, output);
        bail!("macOS app packaging is only available on macOS");
    }
    #[cfg(target_os = "macos")]
    {
        let workspace = detect_workspace_root()?;
        let executable = match executable {
            Some(path) => path,
            None => default_executable(&workspace),
        };
        if !executable.is_file() {
            bail!(
                "executable not found at {} (build with cargo build --release -p rocci-browser or pass --exe)",
                executable.display()
            );
        }
        let app_dir = match output {
            Some(path) => path,
            None => default_app_dir(&workspace),
        };
        assemble(AssembleOptions {
            executable,
            app_dir: app_dir.clone(),
            icon_png: Some(desktop_icon_png()),
        })?;
        codesign(&app_dir)?;
        println!("{}", app_dir.display());
        Ok(())
    }
}

pub fn assemble(options: AssembleOptions) -> Result<()> {
    let AssembleOptions {
        executable,
        app_dir,
        icon_png,
    } = options;
    if !executable.is_file() {
        bail!("executable not found: {}", executable.display());
    }
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)
            .with_context(|| format!("failed to replace {}", app_dir.display()))?;
    }
    let contents = app_dir.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");
    fs::create_dir_all(&macos)?;
    fs::create_dir_all(&resources)?;

    let dest_exe = macos.join(EXECUTABLE_NAME);
    fs::copy(&executable, &dest_exe)
        .with_context(|| format!("failed to copy {}", executable.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_exe, perms)?;
    }

    fs::write(contents.join("Info.plist"), info_plist())?;
    fs::write(contents.join("PkgInfo"), b"APPL????")?;

    if let Some(png) = icon_png {
        install_icns(&png, &resources.join(format!("{ICON_FILE}.icns")))?;
    }
    let document_png = document_icon_png();
    if document_png.is_file() {
        install_icns(
            &document_png,
            &resources.join(format!("{DOCUMENT_ICON_FILE}.icns")),
        )?;
    }
    Ok(())
}

pub fn default_executable(workspace: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "rocci-browser.exe"
    } else {
        EXECUTABLE_NAME
    };
    cargo_target_dir(workspace).join("release").join(name)
}

pub fn default_app_dir(workspace: &Path) -> PathBuf {
    cargo_target_dir(workspace)
        .join("release/bundle/macos")
        .join(format!("{APP_DISPLAY_NAME}.app"))
}

fn cargo_target_dir(workspace: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("target"))
}

pub fn info_plist() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>{APP_DISPLAY_NAME}</string>
  <key>CFBundleExecutable</key>
  <string>{EXECUTABLE_NAME}</string>
  <key>CFBundleIconFile</key>
  <string>{ICON_FILE}</string>
  <key>CFBundleIdentifier</key>
  <string>{BUNDLE_IDENTIFIER}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>{APP_DISPLAY_NAME}</string>
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
  <key>UTExportedTypeDeclarations</key>
  <array>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>dev.rocci.source</string>
      <key>UTTypeDescription</key>
      <string>Rocci Template</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.source-code</string>
        <string>public.plain-text</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>rocci</string>
        </array>
      </dict>
    </dict>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>dev.rocci.document</string>
      <key>UTTypeDescription</key>
      <string>Rocdown Document</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.source-code</string>
        <string>public.plain-text</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>rocdown</string>
        </array>
      </dict>
    </dict>
  </array>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Rocci Template</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>dev.rocci.source</string>
      </array>
      <key>CFBundleTypeIconFile</key>
      <string>{DOCUMENT_ICON_FILE}</string>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Rocdown Document</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>dev.rocci.document</string>
      </array>
      <key>CFBundleTypeIconFile</key>
      <string>{DOCUMENT_ICON_FILE}</string>
    </dict>
  </array>
</dict>
</plist>
"#
    )
}

fn document_icon_png() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../brand/rocci-file.png")
}

#[cfg(target_os = "macos")]
fn desktop_icon_png() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../rocci-desktop/assets/rocci-icon.png")
}

#[cfg(target_os = "macos")]
fn detect_workspace_root() -> Result<PathBuf> {
    let mut dir = env::current_dir()?;
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let raw = fs::read_to_string(&manifest)?;
            if raw.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            bail!("could not find the workspace root (no Cargo.toml with [workspace])");
        }
    }
}

fn install_icns(png: &Path, dest: &Path) -> Result<()> {
    if !png.is_file() {
        bail!("icon PNG not found: {}", png.display());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = dest;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let stem = dest
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("AppIcon");
        let staging = dest
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{stem}.iconset"));
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        fs::create_dir_all(&staging)?;
        let sizes: &[(u32, &str)] = &[
            (16, "icon_16x16.png"),
            (32, "icon_16x16@2x.png"),
            (32, "icon_32x32.png"),
            (64, "icon_32x32@2x.png"),
            (128, "icon_128x128.png"),
            (256, "icon_128x128@2x.png"),
            (256, "icon_256x256.png"),
            (512, "icon_256x256@2x.png"),
            (512, "icon_512x512.png"),
            (1024, "icon_512x512@2x.png"),
        ];
        for (size, name) in sizes {
            let status = Command::new("sips")
                .args(["-z", &size.to_string(), &size.to_string()])
                .arg(png)
                .args(["--out", &staging.join(name).display().to_string()])
                .stdout(Stdio::null())
                .status()
                .context("failed to run sips")?;
            if !status.success() {
                bail!("sips failed while building {name}");
            }
        }
        let status = Command::new("iconutil")
            .args(["-c", "icns"])
            .arg(&staging)
            .arg("-o")
            .arg(dest)
            .status()
            .context("failed to run iconutil")?;
        let _ = fs::remove_dir_all(&staging);
        if !status.success() {
            bail!("iconutil failed to write {}", dest.display());
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn codesign(app_dir: &Path) -> Result<()> {
    let status = Command::new("codesign")
        .args(["--force", "--deep", "--sign", "-"])
        .arg(app_dir)
        .status()
        .context("failed to run codesign")?;
    if !status.success() {
        bail!("ad-hoc codesign failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let dir = env::temp_dir().join(format!(
            "rocci-browser-package-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn default_output_is_rocci_browser_app() {
        let workspace = Path::new("/tmp/rocci");
        let app = default_app_dir(workspace);
        assert!(app.ends_with("Rocci Browser.app"));
        assert!(app.components().any(|part| part.as_os_str() == "bundle"));
        assert!(default_executable(workspace).ends_with("rocci-browser"));
    }

    #[test]
    fn assemble_writes_plist_and_layout_without_codesign() {
        let dir = temp_dir();
        let exe = dir.join("rocci-browser");
        fs::write(&exe, b"#!/bin/sh\necho rocci-browser\n").unwrap();
        let app = dir.join("Rocci Browser.app");
        assemble(AssembleOptions {
            executable: exe,
            app_dir: app.clone(),
            icon_png: None,
        })
        .unwrap();

        assert!(app.join("Contents/MacOS/rocci-browser").is_file());
        assert_eq!(fs::read(app.join("Contents/PkgInfo")).unwrap(), b"APPL????");
        let plist = fs::read_to_string(app.join("Contents/Info.plist")).unwrap();
        assert!(plist.contains("<string>Rocci Browser</string>"));
        assert!(plist.contains("<string>rocci-browser</string>"));
        assert!(plist.contains("<string>dev.rocci.browser</string>"));
        assert!(plist.contains("<key>CFBundleIconFile</key>"));
        assert!(plist.contains("<string>AppIcon</string>"));
        assert!(plist.contains("<string>dev.rocci.source</string>"));
        assert!(plist.contains("<string>dev.rocci.document</string>"));
        assert!(plist.contains("<string>Alternate</string>"));
        assert!(plist.contains("<string>rocci</string>"));
        assert!(plist.contains("<string>rocdown</string>"));
        assert!(plist.contains(&format!("<string>{DOCUMENT_ICON_FILE}</string>")));
        assert!(plist.contains("<key>LSMinimumSystemVersion</key>"));
        assert!(plist.contains("<string>12.0</string>"));
        assert!(plist.contains("<key>NSHighResolutionCapable</key>"));
        assert!(!plist.contains("LSUIElement"));
        assert!(!plist.contains("LSEnvironment"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn package_command_errors_off_macos() {
        let err = run(None, None).unwrap_err();
        assert!(err.to_string().contains("only available on macOS"), "{err}");
    }
}
