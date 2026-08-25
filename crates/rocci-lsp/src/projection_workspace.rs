use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use lsp_types::Uri;

const PLATFORM: &str = "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst";
const HTML: &str = include_str!("../../rocci-cli/runtime/Html.roc");
const DATASTAR: &str = include_str!("../../rocci-cli/runtime/Datastar.roc");

pub fn workspace_dir(base: &Path, uri_key: &str) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    uri_key.hash(&mut hasher);
    base.join(format!("{:x}", hasher.finish()))
}

pub fn projection_path(base: &Path, uri_key: &str, type_name: &str) -> PathBuf {
    workspace_dir(base, uri_key).join(format!("{type_name}.roc"))
}

pub fn stage_package(dir: &Path, type_name: &str, source_dir: Option<&Path>) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|err| format!("create projection workspace: {err}"))?;
    if let Some(source_dir) = source_dir {
        copy_sibling_roc(source_dir, dir, type_name)?;
    }
    write_if_missing(dir.join("Html.roc"), HTML)?;
    write_if_missing(dir.join("Datastar.roc"), DATASTAR)?;
    fs::write(dir.join("main.roc"), stub_main(type_name))
        .map_err(|err| format!("write projection main.roc: {err}"))?;
    Ok(())
}

pub fn source_dir(uri: &Uri, source_name: &str) -> Option<PathBuf> {
    let raw = uri.path().as_str();
    let path = if raw.is_empty() {
        Path::new(source_name)
    } else {
        Path::new(raw)
    };
    let parent = path.parent()?;
    if parent.as_os_str().is_empty() || parent == Path::new("/") {
        return None;
    }
    parent.is_dir().then(|| parent.to_path_buf())
}

fn stub_main(type_name: &str) -> String {
    format!(
        r#"app [main!] {{
    pf: platform "{PLATFORM}",
}}

import pf.Stdout
import {type_name}

main! = |_| {{
    Ok({{}})
}}
"#
    )
}

fn write_if_missing(path: PathBuf, contents: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    fs::write(&path, contents).map_err(|err| format!("write {}: {err}", path.display()))
}

fn copy_sibling_roc(src_dir: &Path, dest: &Path, type_name: &str) -> Result<(), String> {
    let skip = format!("{type_name}.roc");
    let entries =
        fs::read_dir(src_dir).map_err(|err| format!("read {}: {err}", src_dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read {}: {err}", src_dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("roc") {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "main.roc" || name == skip {
            continue;
        }
        fs::copy(&path, dest.join(name.as_ref()))
            .map_err(|err| format!("copy {}: {err}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_platform_app_and_keeps_existing_html() {
        let dir = std::env::temp_dir().join(format!("rocci-lsp-workspace-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Html.roc"), "custom html\n").unwrap();
        stage_package(&dir, "Counter", None).unwrap();
        let main = fs::read_to_string(dir.join("main.roc")).unwrap();
        assert!(main.contains("import Counter"), "{main}");
        assert!(main.contains("pf: platform"), "{main}");
        assert_eq!(
            fs::read_to_string(dir.join("Html.roc")).unwrap(),
            "custom html\n"
        );
        assert!(
            fs::read_to_string(dir.join("Datastar.roc"))
                .unwrap()
                .contains("Datastar :=")
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
