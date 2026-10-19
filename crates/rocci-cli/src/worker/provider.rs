use anyhow::{Context, Result, bail, ensure};
use rocci_core::{Config, WorkerExecution};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Project {
    pub root: PathBuf,
    pub config: Config,
    pub provider: Value,
    pub provider_path: PathBuf,
    pub entry: PathBuf,
    pub class: Option<String>,
}

pub fn root(input: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(input)?;
    let start = if path.is_dir() {
        path.as_path()
    } else {
        path.parent().unwrap()
    };
    start
        .ancestors()
        .find(|path| path.join("rocci.toml").is_file())
        .map(Path::to_path_buf)
        .context("No rocci.toml found; run rocci worker init in the application directory")
}
pub fn load(input: &Path) -> Result<Project> {
    let root = root(input)?;
    let config = Config::from_toml_worker(&fs::read_to_string(root.join("rocci.toml"))?)?;
    let provider_path = ["wrangler.jsonc", "wrangler.json", "wrangler.toml"]
        .into_iter()
        .map(|name| root.join(name))
        .find(|path| path.is_file())
        .context("Missing Wrangler configuration; run rocci worker init")?;
    let source = fs::read_to_string(&provider_path)?;
    let provider = if provider_path.extension().is_some_and(|ext| ext == "toml") {
        serde_json::to_value(toml::from_str::<toml::Value>(&source)?)?
    } else {
        parse_jsonc(&source)?
    };
    let profile = config.worker.as_ref().unwrap();
    ensure!(
        provider["env"]
            .as_object()
            .is_none_or(|environments| environments.is_empty()),
        "Worker preview validates the default Wrangler environment only; use a separate application configuration for each deployment profile"
    );
    ensure!(
        provider["compatibility_date"].as_str() == Some("2026-10-19"),
        "Worker runtime is qualified for compatibility_date 2026-10-19; qualify upgrades before changing it"
    );
    let expected_main = profile
        .custom_entry
        .as_deref()
        .unwrap_or(".rocci/worker/index.mjs");
    ensure!(
        provider["main"].as_str() == Some(expected_main),
        "Wrangler main must match worker.custom_entry or .rocci/worker/index.mjs"
    );
    if profile.custom_entry.is_some() {
        ensure!(
            root.join(expected_main).is_file(),
            "worker.custom_entry does not exist"
        );
    }
    ensure!(
        provider["assets"]["directory"].as_str() == Some(".rocci/worker/public")
            && provider["assets"]["binding"].as_str() == Some("ASSETS"),
        "Wrangler assets must use directory .rocci/worker/public and binding ASSETS"
    );
    let class = match profile.execution {
        WorkerExecution::Stateless => {
            ensure!(
                provider["durable_objects"]["bindings"]
                    .as_array()
                    .is_none_or(|values| values.is_empty()),
                "Stateless profile cannot declare Durable Object bindings"
            );
            None
        }
        WorkerExecution::DurableObject => {
            let bindings = provider["durable_objects"]["bindings"]
                .as_array()
                .context("Missing Wrangler durable_objects.bindings")?;
            let binding = bindings
                .iter()
                .find(|binding| binding["name"].as_str() == profile.binding.as_deref())
                .context("worker.binding does not match a Wrangler object binding")?;
            ensure!(
                binding["script_name"].is_null(),
                "The application object must belong to this Worker, not another script"
            );
            let class = binding["class_name"]
                .as_str()
                .context("Missing object class_name")?;
            ensure!(
                identifier(class),
                "Object class_name must be a JavaScript identifier"
            );
            ensure!(
                provider["migrations"]
                    .as_array()
                    .is_some_and(|migrations| migrations.iter().any(|migration| {
                        migration["new_sqlite_classes"]
                            .as_array()
                            .is_some_and(|classes| {
                                classes.iter().any(|item| item.as_str() == Some(class))
                            })
                    })),
                "Wrangler must retain the initial SQLite migration for class {class}"
            );
            Some(class.to_owned())
        }
    };
    let entry = match config.app.entry.as_deref() {
        Some(entry) => root.join(entry),
        None if root.join("main.roc").is_file() => root.join("main.roc"),
        None => crate::run::resolve_standalone_entry(&root)?,
    };
    ensure!(
        entry.is_file(),
        "Application entry does not exist: {}",
        entry.display()
    );
    Ok(Project {
        root,
        config,
        provider,
        provider_path,
        entry,
        class,
    })
}
pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && !value.as_bytes()[0].is_ascii_digit()
}

pub fn parse_jsonc(source: &str) -> Result<Value> {
    let mut bytes = source.as_bytes().to_vec();
    let mut i = 0;
    while i < bytes.len() {
        let before = i;
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => i = (i + 2).min(bytes.len()),
                        b'"' => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    bytes[i] = b' ';
                    i += 1;
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                bytes[i] = b' ';
                bytes[i + 1] = b' ';
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    if bytes[i] != b'\n' {
                        bytes[i] = b' ';
                    }
                    i += 1;
                }
                if i + 1 >= bytes.len() {
                    bail!("Unclosed JSONC comment");
                }
                bytes[i] = b' ';
                bytes[i + 1] = b' ';
                i += 2;
            }
            _ => i += 1,
        }
        ensure!(i > before, "JSONC scanner failed to advance");
    }
    i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else {
            if bytes[i] == b',' {
                let mut next = i + 1;
                while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                    next += 1;
                }
                if matches!(bytes.get(next), Some(b']' | b'}')) {
                    bytes[i] = b' ';
                }
            }
            i += 1;
        }
    }
    serde_json::from_slice(&bytes).context("Invalid Wrangler JSON/JSONC")
}
