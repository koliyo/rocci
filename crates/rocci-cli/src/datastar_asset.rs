use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use rocci_core::{Config, DatastarAsset, parse_datastar_version};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DEFAULT_VERSION: &str = "1.0.2";

const USER_AGENT: &str = "rocci";
const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
const JSDELIVR_URL: &str =
    "https://cdn.jsdelivr.net/gh/starfederation/datastar@{tag}/bundles/datastar.js";
const GITHUB_RAW_URL: &str =
    "https://raw.githubusercontent.com/starfederation/datastar/{tag}/bundles/datastar.js";
const GITHUB_LATEST_URL: &str =
    "https://api.github.com/repos/starfederation/datastar/releases/latest";

pub enum HintMode {
    Print,
    Quiet,
}

pub fn cache_dir() -> Result<PathBuf> {
    if let Ok(path) = env::var("ROCCI_CACHE") {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            bail!("ROCCI_CACHE must not be empty");
        }
        return Ok(path);
    }
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .context("cannot determine home directory for ~/.rocci/cache")?;
    Ok(PathBuf::from(home).join(".rocci").join("cache"))
}

pub fn tag_name(version: &str) -> String {
    format!(
        "v{}",
        parse_datastar_version(version).unwrap_or_else(|_| version.trim().to_string())
    )
}

pub fn ensure_cached(version: &str) -> Result<PathBuf> {
    let version = parse_datastar_version(version)?;
    let tag = tag_name(&version);
    let dir = cache_dir()?.join("datastar").join(&tag);
    let js = dir.join("datastar.js");
    let sha_path = dir.join("sha256");
    if js.is_file()
        && let Ok(bytes) = fs::read(&js)
        && looks_like_datastar_js(&bytes)
    {
        let actual = hex_sha256(&bytes);
        match fs::read_to_string(&sha_path) {
            Ok(expected) if expected.trim() == actual => return Ok(js),
            Ok(_) => {}
            Err(_) => {
                fs::write(&sha_path, &actual)
                    .with_context(|| format!("failed to write {}", sha_path.display()))?;
                return Ok(js);
            }
        }
    }

    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let bytes = download_datastar(&tag)?;
    let hash = hex_sha256(&bytes);
    let tmp = dir.join("datastar.js.tmp");
    fs::write(&tmp, &bytes).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::write(&sha_path, &hash)
        .with_context(|| format!("failed to write {}", sha_path.display()))?;
    fs::rename(&tmp, &js).with_context(|| format!("failed to write {}", js.display()))?;
    Ok(js)
}

pub fn stage_into(assets_dir: &Path, version: &str) -> Result<PathBuf> {
    let cached = ensure_cached(version)?;
    fs::create_dir_all(assets_dir)
        .with_context(|| format!("failed to create {}", assets_dir.display()))?;
    let dest = assets_dir.join("datastar.js");
    copy_if_changed(&cached, &dest)?;
    Ok(dest)
}

pub fn ensure_app(app_dir: &Path, hints: HintMode) -> Result<()> {
    let config = load_app_config(app_dir);
    let dest = datastar_dest(app_dir, config.as_ref());
    let assets_dir = dest.parent().unwrap_or(app_dir);

    let version = match config
        .as_ref()
        .and_then(|config| config.assets.datastar.as_ref())
    {
        Some(DatastarAsset::Disabled) => return Ok(()),
        Some(DatastarAsset::Version(version)) => {
            stage_into(assets_dir, version)?;
            version.clone()
        }
        None if dest.is_file() => fs::read(&dest)
            .ok()
            .and_then(|bytes| parse_version_comment(&bytes))
            .unwrap_or_else(|| DEFAULT_VERSION.to_string()),
        None => {
            stage_into(assets_dir, DEFAULT_VERSION)?;
            DEFAULT_VERSION.to_string()
        }
    };

    if matches!(hints, HintMode::Print) {
        print_hint(&version);
    }
    Ok(())
}

pub fn stage_version_for_dir(dir: &Path) -> Option<String> {
    match load_app_config(dir).and_then(|config| config.assets.datastar) {
        Some(DatastarAsset::Disabled) => None,
        Some(DatastarAsset::Version(version)) => Some(version),
        None => Some(DEFAULT_VERSION.to_string()),
    }
}

pub fn pin_app(app: &Path, version: &str) -> Result<()> {
    let app_dir = resolve_datastar_app(app)?;
    let version = parse_datastar_version(version)?;
    ensure_cached(&version)?;
    write_pin(&app_dir, &version)?;
    stage_into(&app_assets_dir(&app_dir), &version)?;
    println!(
        "{}",
        crate::style::pinned(&format!("Datastar {version} for {}", app_dir.display()))
    );
    Ok(())
}

pub fn update_app(app: &Path) -> Result<()> {
    let tag = fetch_latest_tag().context("failed to look up the latest Datastar release")?;
    pin_app(app, &tag)
}

pub fn print_hint(current: &str) {
    if let Some(hint) = maybe_update_hint(current) {
        let mut lines = hint.lines();
        if let Some(first) = lines.next() {
            if let Some(rest) = first.strip_prefix("note: ") {
                eprintln!("{}", crate::style::note(rest));
            } else {
                eprintln!("{first}");
            }
        }
        for line in lines {
            eprintln!("{line}");
        }
    }
}

pub fn format_update_hint(current: &str, latest: &str) -> Option<String> {
    let current = parse_datastar_version(current)
        .ok()
        .unwrap_or_else(|| current.trim().trim_start_matches('v').to_string());
    let latest = parse_datastar_version(latest)
        .ok()
        .unwrap_or_else(|| latest.trim().trim_start_matches('v').to_string());
    if latest.is_empty() || !version_newer(&latest, &current) {
        return None;
    }
    Some(format!(
        "note: Datastar {latest} is available (this app is on {current})\n      upgrade with `rocci datastar update`"
    ))
}

pub(crate) fn check_is_due(last_check_unix: u64, now: u64) -> bool {
    now.saturating_sub(last_check_unix) >= CHECK_INTERVAL_SECS
}

pub(crate) fn set_datastar_pin_in_toml(source: &str, version: &str) -> String {
    let pin_line = format!("datastar = \"{version}\"");
    let mut out = String::new();
    let mut in_assets = false;
    let mut replaced = false;
    let mut saw_assets = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            if in_assets && !replaced {
                out.push_str(&pin_line);
                out.push('\n');
                replaced = true;
            }
            in_assets = trimmed == "[assets]";
            if in_assets {
                saw_assets = true;
            }
        }
        if in_assets && trimmed.starts_with("datastar") {
            out.push_str(&pin_line);
            out.push('\n');
            replaced = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if in_assets && !replaced {
        out.push_str(&pin_line);
        out.push('\n');
        replaced = true;
    }
    if !saw_assets {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("[assets]\n");
        out.push_str(&pin_line);
        out.push('\n');
    } else if !replaced {
        out.push_str(&pin_line);
        out.push('\n');
    }
    out
}

fn maybe_update_hint(current: &str) -> Option<String> {
    let now = now_unix();
    let mut state = load_state().unwrap_or_default();
    if !check_is_due(state.last_check_unix, now) {
        return format_update_hint(current, &state.latest_tag);
    }
    match fetch_latest_tag() {
        Ok(tag) => {
            state.last_check_unix = now;
            state.latest_tag = tag.clone();
            let _ = save_state(&state);
            format_update_hint(current, &tag)
        }
        Err(_) => format_update_hint(current, &state.latest_tag),
    }
}

fn download_datastar(tag: &str) -> Result<Vec<u8>> {
    let primary = JSDELIVR_URL.replace("{tag}", tag);
    let fallback = GITHUB_RAW_URL.replace("{tag}", tag);
    match http_get_bytes(&primary) {
        Ok(bytes) if looks_like_datastar_js(&bytes) => Ok(bytes),
        Ok(_) | Err(_) => {
            let bytes = http_get_bytes(&fallback)
                .with_context(|| format!("failed to download Datastar {tag}"))?;
            if !looks_like_datastar_js(&bytes) {
                bail!("downloaded Datastar {tag} did not look like a JS bundle");
            }
            Ok(bytes)
        }
    }
}

fn fetch_latest_tag() -> Result<String> {
    let body = http_get_text(GITHUB_LATEST_URL)
        .context("failed to query GitHub for the latest Datastar release")?;
    parse_latest_tag(&body)
}

fn parse_latest_tag(body: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }
    let release: Release = serde_json::from_str(body).context("invalid GitHub release JSON")?;
    Ok(tag_name(&parse_datastar_version(&release.tag_name)?))
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("GET {url} failed"))?;
    let mut buf = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut buf)
        .with_context(|| format!("failed to read {url}"))?;
    Ok(buf)
}

fn http_get_text(url: &str) -> Result<String> {
    ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("GET {url} failed"))?
        .into_string()
        .with_context(|| format!("failed to read {url}"))
}

fn load_app_config(app_dir: &Path) -> Option<Config> {
    let path = app_dir.join("rocci.toml");
    if path.is_file() {
        Config::from_file(&path).ok()
    } else {
        None
    }
}

fn datastar_dest(app_dir: &Path, config: Option<&Config>) -> PathBuf {
    app_assets_dir_from_config(app_dir, config).join("datastar.js")
}

fn app_assets_dir(app_dir: &Path) -> PathBuf {
    app_assets_dir_from_config(app_dir, load_app_config(app_dir).as_ref())
}

fn app_assets_dir_from_config(app_dir: &Path, config: Option<&Config>) -> PathBuf {
    let directory = config
        .and_then(|config| config.assets.directory.clone())
        .unwrap_or_else(|| PathBuf::from("assets"));
    if directory.is_absolute() {
        directory
    } else {
        app_dir.join(directory)
    }
}

fn resolve_datastar_app(app: &Path) -> Result<PathBuf> {
    let path = if app.is_absolute() {
        app.to_path_buf()
    } else {
        env::current_dir()?.join(app)
    };
    if path.is_file() {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if (name == "rocci.toml" || name == "main.roc")
            && let Some(parent) = path.parent()
        {
            return Ok(parent.to_path_buf());
        }
        bail!(
            "expected an app directory, rocci.toml, or main.roc: {}",
            path.display()
        );
    }
    if path.join("main.roc").is_file() || path.join("rocci.toml").is_file() {
        return Ok(path);
    }
    bail!(
        "no main.roc or rocci.toml in {}; pass --app <dir>",
        path.display()
    );
}

fn write_pin(app_dir: &Path, version: &str) -> Result<()> {
    let path = app_dir.join("rocci.toml");
    if path.is_file() {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let updated = set_datastar_pin_in_toml(&source, version);
        fs::write(&path, updated).with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        fs::write(&path, format!("[assets]\ndatastar = \"{version}\"\n"))
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn copy_if_changed(from: &Path, to: &Path) -> Result<()> {
    if to.is_file() {
        let src = fs::read(from)?;
        let dst = fs::read(to)?;
        if src == dst {
            return Ok(());
        }
    }
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(from, to)
        .with_context(|| format!("failed to copy {} -> {}", from.display(), to.display()))?;
    Ok(())
}

fn looks_like_datastar_js(bytes: &[u8]) -> bool {
    bytes.len() > 1000 && !bytes.starts_with(b"<")
}

fn parse_version_comment(bytes: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let first = text.lines().next()?.trim();
    let rest = first.strip_prefix("// Datastar ")?.trim();
    parse_datastar_version(rest).ok()
}

fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn version_newer(latest: &str, current: &str) -> bool {
    parse_ver(latest) > parse_ver(current)
}

fn parse_ver(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.');
    let major = parse_ver_part(parts.next().unwrap_or("0"));
    let minor = parse_ver_part(parts.next().unwrap_or("0"));
    let patch = parse_ver_part(parts.next().unwrap_or("0"));
    (major, minor, patch)
}

fn parse_ver_part(part: &str) -> u64 {
    let digits: String = part.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    digits.parse().unwrap_or(0)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct CacheState {
    #[serde(default)]
    last_check_unix: u64,
    #[serde(default)]
    latest_tag: String,
}

fn state_path() -> Result<PathBuf> {
    Ok(cache_dir()?.join("datastar").join("state.toml"))
}

fn load_state() -> Result<CacheState> {
    let path = state_path()?;
    if !path.is_file() {
        return Ok(CacheState::default());
    }
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&source).with_context(|| format!("invalid {}", path.display()))
}

fn save_state(state: &CacheState) -> Result<()> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, toml::to_string_pretty(state)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_cache<T>(f: impl FnOnce(&Path) -> T) -> T {
        let _guard = TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = env::temp_dir().join(format!(
            "rocci-datastar-test-{}-{}",
            std::process::id(),
            now_unix()
        ));
        fs::create_dir_all(&dir).unwrap();
        let previous = env::var("ROCCI_CACHE").ok();
        // SAFETY: tests that mutate ROCCI_CACHE hold TEST_LOCK.
        unsafe { env::set_var("ROCCI_CACHE", &dir) };
        let result = f(&dir);
        match previous {
            Some(value) => unsafe { env::set_var("ROCCI_CACHE", value) },
            None => unsafe { env::remove_var("ROCCI_CACHE") },
        }
        let _ = fs::remove_dir_all(&dir);
        result
    }

    fn seed_cached(version: &str, body: &[u8]) -> PathBuf {
        let tag = tag_name(version);
        let dir = cache_dir().unwrap().join("datastar").join(tag);
        fs::create_dir_all(&dir).unwrap();
        let js = dir.join("datastar.js");
        fs::write(&js, body).unwrap();
        fs::write(dir.join("sha256"), hex_sha256(body)).unwrap();
        js
    }

    fn js_fixture(marker: &str) -> Vec<u8> {
        let mut bytes = format!("// Datastar {marker}\n").into_bytes();
        bytes.extend(std::iter::repeat_n(b'x', 1200));
        bytes
    }

    #[test]
    fn format_hint_when_newer() {
        let hint = format_update_hint("1.0.2", "v1.0.3").unwrap();
        assert!(hint.contains("Datastar 1.0.3"));
        assert!(hint.contains("1.0.2"));
        assert!(hint.contains("rocci datastar update"));
    }

    #[test]
    fn format_hint_skips_same_version() {
        assert!(format_update_hint("1.0.2", "v1.0.2").is_none());
    }

    #[test]
    fn daily_check_interval() {
        assert!(check_is_due(0, CHECK_INTERVAL_SECS));
        assert!(!check_is_due(100, 100 + CHECK_INTERVAL_SECS - 1));
        assert!(check_is_due(100, 100 + CHECK_INTERVAL_SECS));
    }

    #[test]
    fn parses_github_latest_tag() {
        let tag = parse_latest_tag(r#"{"tag_name":"v1.0.2"}"#).unwrap();
        assert_eq!(tag, "v1.0.2");
    }

    #[test]
    fn pin_toml_inserts_assets_section() {
        let updated = set_datastar_pin_in_toml("[app]\nname = \"Demo\"\n", "1.0.2");
        assert!(updated.contains("[assets]"));
        assert!(updated.contains("datastar = \"1.0.2\""));
        assert!(updated.contains("name = \"Demo\""));
    }

    #[test]
    fn pin_toml_replaces_existing_datastar() {
        let source = "[assets]\ndirectory = \"assets\"\ndatastar = false\n";
        let updated = set_datastar_pin_in_toml(source, "1.0.3");
        assert!(updated.contains("datastar = \"1.0.3\""));
        assert!(!updated.contains("datastar = false"));
        assert!(updated.contains("directory = \"assets\""));
    }

    #[test]
    fn stage_copies_from_cache_and_is_noop_when_unchanged() {
        with_cache(|_cache| {
            let body = js_fixture("v1.0.2");
            seed_cached("1.0.2", &body);
            let assets = cache_dir().unwrap().join("app-assets");
            let dest = stage_into(&assets, "1.0.2").unwrap();
            assert_eq!(fs::read(&dest).unwrap(), body);
            let first_modified = fs::metadata(&dest).unwrap().modified().unwrap();
            stage_into(&assets, "1.0.2").unwrap();
            let second_modified = fs::metadata(&dest).unwrap().modified().unwrap();
            assert_eq!(first_modified, second_modified);
        });
    }

    #[test]
    fn ensure_app_leaves_unmanaged_file() {
        with_cache(|_cache| {
            let app = cache_dir().unwrap().join("app");
            let assets = app.join("assets");
            fs::create_dir_all(&assets).unwrap();
            let dest = assets.join("datastar.js");
            let original = js_fixture("custom");
            fs::write(&dest, &original).unwrap();
            ensure_app(&app, HintMode::Quiet).unwrap();
            assert_eq!(fs::read(&dest).unwrap(), original);
        });
    }

    #[test]
    fn ensure_app_honors_disabled_pin() {
        with_cache(|_cache| {
            let app = cache_dir().unwrap().join("app");
            fs::create_dir_all(&app).unwrap();
            fs::write(app.join("rocci.toml"), "[assets]\ndatastar = false\n").unwrap();
            ensure_app(&app, HintMode::Quiet).unwrap();
            assert!(!app.join("assets/datastar.js").exists());
        });
    }

    #[test]
    fn ensure_app_copies_pinned_version() {
        with_cache(|_cache| {
            let body = js_fixture("v1.0.2");
            seed_cached("1.0.2", &body);
            let app = cache_dir().unwrap().join("app");
            fs::create_dir_all(&app).unwrap();
            fs::write(app.join("rocci.toml"), "[assets]\ndatastar = \"1.0.2\"\n").unwrap();
            ensure_app(&app, HintMode::Quiet).unwrap();
            assert_eq!(fs::read(app.join("assets/datastar.js")).unwrap(), body);
        });
    }

    #[test]
    fn ensure_app_uses_default_when_missing() {
        with_cache(|_cache| {
            let body = js_fixture("v1.0.2");
            seed_cached("1.0.2", &body);
            let app = cache_dir().unwrap().join("app");
            fs::create_dir_all(&app).unwrap();
            ensure_app(&app, HintMode::Quiet).unwrap();
            assert_eq!(fs::read(app.join("assets/datastar.js")).unwrap(), body);
        });
    }

    #[test]
    fn parse_version_from_js_comment() {
        let bytes = js_fixture("v1.0.2");
        assert_eq!(parse_version_comment(&bytes).as_deref(), Some("1.0.2"));
    }
}
