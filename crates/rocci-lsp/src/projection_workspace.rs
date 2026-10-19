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
    if let Some(root) = source_dir.and_then(worker_root) {
        return stage_worker(dir, type_name, &root);
    }
    let _ = fs::remove_file(dir.join("worker-profile.json"));
    write_if_missing(dir.join("Html.roc"), HTML)?;
    write_if_missing(dir.join("Datastar.roc"), DATASTAR)?;
    fs::write(dir.join("main.roc"), stub_main(type_name))
        .map_err(|err| format!("write projection main.roc: {err}"))?;
    Ok(())
}

pub fn source_dir(uri: &Uri, source_name: &str) -> Option<PathBuf> {
    let decoded = url::Url::parse(uri.as_str())
        .ok()
        .and_then(|url| url.to_file_path().ok());
    let path = decoded.as_deref().unwrap_or_else(|| Path::new(source_name));
    let parent = path.parent()?;
    if parent.as_os_str().is_empty() || parent == Path::new("/") || parent.parent().is_none() {
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

/// Nearest application configuration owns target selection, including nested apps.
pub fn worker_root(source_dir: &Path) -> Option<PathBuf> {
    let root = source_dir
        .ancestors()
        .find(|path| path.join("rocci.toml").is_file())?;
    let source = fs::read_to_string(root.join("rocci.toml")).ok()?;
    let config: toml::Value = toml::from_str(&source).ok()?;
    config.get("worker")?;
    Some(root.to_path_buf())
}

fn stage_worker(dir: &Path, type_name: &str, root: &Path) -> Result<(), String> {
    let source = fs::read_to_string(root.join("rocci.toml")).map_err(|err| err.to_string())?;
    if let Err(error) = rocci_core::Config::from_toml_worker(&source) {
        fs::write(
            dir.join("worker-profile.json"),
            serde_json::json!({"error":error.to_string()}).to_string(),
        )
        .map_err(|err| err.to_string())?;
        return Err(error.to_string());
    }
    fs::create_dir_all(dir.join("platform")).map_err(|err| err.to_string())?;
    for (name, source) in rocci_worker_platform::sources::ROC_SOURCES {
        fs::write(
            dir.join("platform").join(name),
            rocci_worker_platform::sources::compatibility_source(
                name.trim_end_matches(".roc"),
                source,
            ),
        )
        .map_err(|err| err.to_string())?;
    }
    fs::write(
        dir.join("platform/WorkerRoutes.roc"),
        "WorkerRoutes := [].{ live_paths : List(Str), live_paths = [] }\n",
    )
    .map_err(|err| err.to_string())?;
    let marker =
        fs::read(root.join(".rocci/worker-editor.json")).unwrap_or_else(|_| b"{}".to_vec());
    fs::write(dir.join("worker-profile.json"), marker).map_err(|err| err.to_string())?;
    fs::write(dir.join("main.roc"),format!(r#"app [Context, program] {{
    pf: platform "platform/main.roc",
    http: "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst",
}}
import {type_name}
import pf.Server
import http.Response
Context : {{}}
program = {{init!,respond!,shutdown!}}
init! = || Ok({{config:Server.default_config,context:{{}}}})
respond! = |_,_| Ok(Server.respond(Response.from_status(204)))
shutdown! = |_,_| Ok({{}})
"#)).map_err(|err|err.to_string())?;
    Ok(())
}

/// Keep imported platform names and source offsets identical to CLI staging.
pub fn worker_module(
    roc: &str,
    segments: &[rocci_template::Segment],
) -> (String, Vec<rocci_template::Segment>) {
    let mut generated = String::new();
    let mut insertions = Vec::new();
    let mut offset = 0;
    for line in roc.split_inclusive('\n') {
        if matches!(line.trim(), "import Html" | "import Datastar") {
            insertions.push(offset + 7);
            generated.push_str(&line.replacen("import ", "import pf.", 1));
        } else {
            generated.push_str(line);
        }
        offset += line.len();
    }
    let mut segments = segments.to_vec();
    for segment in &mut segments {
        segment.generated.start += 3 * insertions
            .iter()
            .filter(|position| **position <= segment.generated.start as usize)
            .count() as u32;
        segment.generated.end += 3 * insertions
            .iter()
            .filter(|position| **position < segment.generated.end as usize)
            .count() as u32;
    }
    (generated, segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stages_platform_app_and_keeps_existing_html() {
        let dir = std::env::temp_dir().join(format!(
            "rocci-lsp-workspace-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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
