use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::{fs, path::Path};

pub fn init(input: &Path, stateless: bool) -> Result<()> {
    if !input.exists() {
        fs::create_dir_all(input)?;
    }
    let input = fs::canonicalize(input)?;
    let root = if input.is_dir() {
        input.clone()
    } else {
        input.parent().unwrap().to_path_buf()
    };
    let config_path = root.join("rocci.toml");
    let source = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };
    let mut document = source
        .parse::<toml_edit::DocumentMut>()
        .context("Read existing rocci.toml")?;
    let entry = if input.is_file() {
        input.clone()
    } else if let Some(entry) = document
        .get("app")
        .and_then(|app| app.get("entry"))
        .and_then(|entry| entry.as_str())
    {
        root.join(entry)
    } else if root.join("main.roc").is_file() {
        root.join("main.roc")
    } else {
        match crate::run::resolve_standalone_entry(&root) {
            Ok(entry) => entry,
            Err(error) => {
                let authored = fs::read_dir(&root)?
                    .filter_map(|entry| entry.ok())
                    .any(|entry| {
                        matches!(
                            entry.path().extension().and_then(|ext| ext.to_str()),
                            Some("rocci" | "roc")
                        )
                    });
                ensure!(
                    !authored,
                    "{error}; specify the entry .rocci file explicitly"
                );
                let entry = root.join(if stateless {
                    "Hello.rocci"
                } else {
                    "Counter.rocci"
                });
                let source = if stateless {
                    "@get:view(\"/\") = |_| {\n    page({})\n}\n\n@component\nPage = |_|\n    <html lang=\"en\"><head><title>Hello Worker</title></head><body><h1>Hello from Rocci</h1></body></html>\n"
                } else {
                    include_str!("../../../../examples/rocci/standalone/counter/Counter.rocci")
                };
                fs::write(&entry, source)?;
                entry
            }
        }
    };
    ensure!(
        entry.starts_with(&root),
        "Entry must belong to the application directory"
    );
    if document
        .get("app")
        .and_then(|app| app.get("entry"))
        .is_none()
    {
        document["app"]["entry"] = toml_edit::value(
            entry
                .strip_prefix(&root)?
                .to_string_lossy()
                .replace('\\', "/"),
        );
    }
    if document.get("worker").is_none() {
        document["worker"]["provider"] = toml_edit::value("cloudflare");
        document["worker"]["execution"] = toml_edit::value(if stateless {
            "stateless"
        } else {
            "durable-object"
        });
        if !stateless {
            document["worker"]["binding"] = toml_edit::value("APP");
            document["worker"]["object_key"] = toml_edit::value("app");
            document["worker"]["sqlite"]["path"] = toml_edit::value("./counter.db");
            document["worker"]["sqlite"]["storage"] = toml_edit::value("object");
        }
    }
    let config = rocci_core::Config::from_toml_worker(&document.to_string())?;
    let profile = config.worker.as_ref().unwrap();
    fs::write(&config_path, document.to_string())?;
    let asset_directory = config
        .assets
        .directory
        .as_ref()
        .map(|path| root.join(path))
        .unwrap_or_else(|| root.join("assets"));
    fs::create_dir_all(asset_directory)?;
    let mut watch = Vec::new();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.')
            || matches!(
                name.as_str(),
                "node_modules" | "target" | "dist" | "generated"
            )
        {
            continue;
        }
        let kind = entry.file_type()?;
        if kind.is_dir()
            || kind.is_file()
                && matches!(
                    entry
                        .path()
                        .extension()
                        .and_then(|extension| extension.to_str()),
                    Some("roc" | "rocci" | "toml" | "json" | "jsonc" | "css" | "js" | "mjs" | "ts")
                )
        {
            watch.push(name);
        }
    }
    watch.sort();
    let mut provider = json!({
        "name":root.file_name().unwrap_or_default().to_string_lossy().to_ascii_lowercase().chars().map(|ch|if ch.is_ascii_alphanumeric(){ch}else{'-'}).collect::<String>(),
        "main":".rocci/worker/index.mjs","compatibility_date":"2026-10-19",
        "build":{"command":"rocci worker build","watch_dir":watch},
        "assets":{"directory":".rocci/worker/public","binding":"ASSETS","run_worker_first":true},
    });
    if profile.execution == rocci_core::WorkerExecution::DurableObject {
        provider["durable_objects"] =
            json!({"bindings":[{"name":profile.binding,"class_name":"RocciApp"}]});
        provider["migrations"] = json!([{"tag":"v1","new_sqlite_classes":["RocciApp"]}]);
    }
    if !["wrangler.jsonc", "wrangler.json", "wrangler.toml"]
        .iter()
        .any(|name| root.join(name).exists())
    {
        fs::write(
            root.join("wrangler.jsonc"),
            format!("{}\n", serde_json::to_string_pretty(&provider)?),
        )?;
    }
    let package_path = root.join("package.json");
    let fresh = !package_path.exists();
    let mut package = if fresh {
        json!({"name":"rocci-worker-app","private":true,"type":"module","engines":{"node":"26.12.0"}})
    } else {
        serde_json::from_str(&fs::read_to_string(&package_path)?)?
    };
    if package["scripts"].is_null() {
        package["scripts"] = json!({});
    }
    if package["devDependencies"].is_null() {
        package["devDependencies"] = json!({});
    }
    for (name, command) in [
        ("build", "rocci worker build"),
        ("dev", "wrangler dev"),
        ("deploy", "wrangler deploy"),
    ] {
        let key = if package["scripts"][name].is_null()
            || package["scripts"][name].as_str() == Some(command)
        {
            name.to_string()
        } else {
            format!("worker:{name}")
        };
        package["scripts"][key] = json!(command);
    }
    package["devDependencies"]["wrangler"] = json!("4.151.1");
    fs::write(
        package_path,
        format!("{}\n", serde_json::to_string_pretty(&package)?),
    )?;
    if fresh && !root.join("package-lock.json").exists() {
        let mut lock: serde_json::Value = serde_json::from_str(include_str!(
            "../../../rocci-worker-platform/tooling/package-lock.json"
        ))?;
        lock["name"] = json!("rocci-worker-app");
        lock["packages"][""]["name"] = json!("rocci-worker-app");
        fs::write(
            root.join("package-lock.json"),
            format!("{}\n", serde_json::to_string_pretty(&lock)?),
        )?;
    }
    let ignore = root.join(".gitignore");
    let mut source = fs::read_to_string(&ignore).unwrap_or_default();
    for pattern in [
        ".rocci/",
        ".wrangler/",
        "node_modules/",
        ".dev.vars",
        ".env",
    ] {
        if !source.lines().any(|line| line == pattern) {
            if !source.is_empty() && !source.ends_with('\n') {
                source.push('\n');
            }
            source.push_str(pattern);
            source.push('\n');
        }
    }
    fs::write(ignore, source)?;
    super::provider::load(&root)?;
    println!(
        "Worker scaffold ready in {}. Run npm install, npm run dev, then npm run deploy.",
        root.display()
    );
    Ok(())
}
