use std::path::PathBuf;

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

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
    // WASI cannot stat host paths; Zed spawns this command on the host.
    Some(
        root.join("target")
            .join("debug")
            .join(exe)
            .to_string_lossy()
            .into_owned(),
    )
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
            .or_else(|| cargo_target_binary(worktree))
            .ok_or_else(|| {
                "rocci-language-server not found. Build it with `cargo build -p rocci-lsp` or set lsp.rocci-language-server.binary.path.".to_string()
            })?;

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
