use std::fs;
use std::path::PathBuf;

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

const UNSUPPORTED_PLATFORM: &str = "Unsupported platform. Rocci GitHub releases currently publish aarch64-apple-darwin and x86_64-unknown-linux-gnu.";

struct RocciExtension;

fn language_server_binary_name() -> &'static str {
    if zed::current_platform().0 == zed::Os::Windows {
        "rocci-language-server.exe"
    } else {
        "rocci-language-server"
    }
}

fn cargo_target_binary(worktree: &zed::Worktree) -> Option<String> {
    let cargo_toml = worktree.read_text_file("Cargo.toml").ok()?;
    if !cargo_toml.contains("rocci-lsp") {
        return None;
    }

    let exe = language_server_binary_name();
    let root = PathBuf::from(worktree.root_path());
    Some(
        root.join("target")
            .join("debug")
            .join(exe)
            .to_string_lossy()
            .into_owned(),
    )
}

fn github_asset_name(version: &str, triple: &str) -> String {
    format!("rocci-{version}-{triple}.tar.gz")
}

fn rust_triple(os: zed::Os, arch: zed::Architecture) -> Result<&'static str> {
    match (os, arch) {
        (zed::Os::Mac, zed::Architecture::Aarch64) => Ok("aarch64-apple-darwin"),
        (zed::Os::Linux, zed::Architecture::X8664) => Ok("x86_64-unknown-linux-gnu"),
        _ => Err(UNSUPPORTED_PLATFORM.into()),
    }
}

fn find_extracted_server(dir: &str) -> Option<String> {
    let name = language_server_binary_name();
    let direct = format!("{dir}/{name}");
    if fs::metadata(&direct).map(|meta| meta.is_file()).unwrap_or(false) {
        return Some(direct);
    }
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

fn download_language_server(language_server_id: &LanguageServerId) -> Result<String> {
    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::CheckingForUpdate,
    );
    let release = zed::latest_github_release(
        "koliyo/rocci",
        zed::GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )?;
    let (os, arch) = zed::current_platform();
    let triple = rust_triple(os, arch)?;
    let asset_name = github_asset_name(&release.version, triple);
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("no GitHub asset named {asset_name}"))?;

    let version_dir = format!("releases/{}", release.version);
    if let Some(existing) = find_extracted_server(&version_dir) {
        return Ok(existing);
    }

    zed::set_language_server_installation_status(
        language_server_id,
        &zed::LanguageServerInstallationStatus::Downloading,
    );
    zed::download_file(
        &asset.download_url,
        &version_dir,
        zed::DownloadedFileType::GzipTar,
    )
    .map_err(|error| format!("failed to download {asset_name}: {error}"))?;
    let binary = find_extracted_server(&version_dir)
        .ok_or_else(|| format!("downloaded archive did not contain {name}", name = language_server_binary_name()))?;
    zed::make_file_executable(&binary)?;
    Ok(binary)
}

impl zed::Extension for RocciExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .unwrap_or_default();
        let binary = settings.binary.as_ref();

        let command = binary
            .and_then(|settings| settings.path.as_deref())
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| worktree.which(language_server_binary_name()))
            .or_else(|| cargo_target_binary(worktree));
        let command = match command {
            Some(command) => command,
            None => download_language_server(language_server_id)?,
        };

        let args = binary
            .and_then(|settings| settings.arguments.clone())
            .unwrap_or_default();
        let env = binary
            .and_then(|settings| settings.env.clone())
            .map(|env| env.into_iter().collect())
            .unwrap_or_default();

        Ok(zed::Command {
            command,
            args,
            env,
        })
    }
}

zed::register_extension!(RocciExtension);

#[cfg(test)]
mod tests {
    use super::github_asset_name;

    #[test]
    fn asset_name_matches_release_archives() {
        assert_eq!(
            github_asset_name("0.1.0", "aarch64-apple-darwin"),
            "rocci-0.1.0-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            github_asset_name("dev", "x86_64-unknown-linux-gnu"),
            "rocci-dev-x86_64-unknown-linux-gnu.tar.gz"
        );
    }
}
