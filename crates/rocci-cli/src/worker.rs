//! Worker staging reuses the ordinary application plan and response policy.
use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{dispatch, driver, run};

mod build;
mod provider;
mod scaffold;
mod toolchain;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Add ordinary Wrangler configuration without changing application source.
    Init {
        #[arg(default_value = ".")]
        input: std::path::PathBuf,
        #[arg(long, default_value = "cloudflare", value_parser = ["cloudflare"])]
        provider: String,
        #[arg(long)]
        stateless: bool,
    },
    /// Compile and atomically package the application for Wrangler.
    Build {
        #[arg(default_value = ".")]
        input: std::path::PathBuf,
        #[arg(long, env = "ROCCI_WORKER_ROC")]
        roc: Option<std::path::PathBuf>,
        #[arg(long, env = "ROCCI_WORKER_HOST")]
        host_archive: Option<std::path::PathBuf>,
        #[arg(long)]
        locked: bool,
        #[arg(long)]
        offline: bool,
        #[arg(long, value_enum)]
        opt: Option<crate::native_target::RocOpt>,
    },
}

pub fn execute(command: Command) -> Result<()> {
    match command {
        Command::Init {
            input,
            provider: _,
            stateless,
        } => scaffold::init(&input, stateless),
        Command::Build {
            input,
            roc,
            host_archive,
            locked,
            offline,
            opt,
        } => build::build(
            &input,
            roc.as_deref(),
            host_archive.as_deref(),
            locked,
            offline,
            opt,
        ),
    }
}

/// Stage generated Roc and the compatible platform without altering app sources.
/// Compilation and provider admission are separate from this deterministic step.
pub fn stage(input: &Path, output: &Path, host_archive: &Path) -> Result<()> {
    stage_with_metadata(input, output, host_archive).map(|_| ())
}

pub(crate) fn stage_with_metadata(
    input: &Path,
    output: &Path,
    host_archive: &Path,
) -> Result<Vec<rocci_template::MappedModule>> {
    let input = fs::canonicalize(input)?;
    let entry = if input.is_dir() {
        if input.join("main.roc").is_file() {
            input.join("main.roc")
        } else {
            run::resolve_standalone_entry(&input)?
        }
    } else {
        input
    };
    let plan = if entry.extension().is_some_and(|ext| ext == "roc") {
        None
    } else {
        let plan = run::standalone_http_module_app_plan(&entry)?;
        plan.validate_dispatch()?;
        Some(plan)
    };
    fs::create_dir_all(output.join("platform/targets/wasm32"))?;
    fs::create_dir_all(output.join("assets"))?;
    for (name, source) in rocci_worker_platform::sources::ROC_SOURCES {
        fs::write(
            output.join("platform").join(name),
            rocci_worker_platform::sources::compatibility_source(
                name.trim_end_matches(".roc"),
                source,
            ),
        )?;
    }
    let platform_main = rocci_worker_platform::sources::ROC_SOURCES
        .iter()
        .find(|(name, _)| *name == "main.roc")
        .unwrap()
        .1;
    let wasm_target = r#"        wasm32: { inputs: ["libhost.a", app], exports: ["worker_abi_version", "worker_input_ptr", "worker_input_capacity", "worker_output_ptr", "worker_output_len", "worker_live_bytes", "worker_live_allocations", "worker_invoke"] },
"#;
    fs::write(
        output.join("platform/main.roc"),
        platform_main.replacen("targets: {", &format!("targets: {{\n{wasm_target}"), 1),
    )?;
    fs::copy(
        host_archive,
        output.join("platform/targets/wasm32/libhost.a"),
    )
    .context("stage the Worker host archive built for wasm32-unknown-unknown")?;
    let live_paths = plan
        .as_ref()
        .map(|plan| {
            plan.modules
                .iter()
                .flat_map(|module| module.lives.iter().map(|live| format!("{:?}", live.path)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    fs::write(
        output.join("platform/WorkerRoutes.roc"),
        format!(
            "WorkerRoutes := [].{{\n    live_paths : List(Str)\n    live_paths = [{}]\n}}\n",
            live_paths.join(",")
        ),
    )?;
    let mut maps = Vec::new();
    if let Some(plan) = plan {
        for module in &plan.modules {
            let rewritten = dispatch::rewrite_runtime_imports_for_pin(&module.roc, None);
            let mut segments = module.mapped.segments.clone();
            let mut insertions = Vec::new();
            let mut offset = 0;
            for line in module.roc.split_inclusive('\n') {
                if matches!(line.trim(), "import Html" | "import Datastar") {
                    insertions.push(offset + 7);
                }
                offset += line.len();
            }
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
            let projected =
                rocci_template::project_type_module(&rewritten, &segments, &module.type_name);
            fs::write(
                output.join(format!("{}.roc", module.type_name)),
                &projected.roc,
            )?;
            maps.push(rocci_template::MappedModule {
                type_name: module.type_name.clone(),
                generated: projected.roc,
                segments: projected.segments,
                source_name: module.mapped.source_name.clone(),
                source_src: module.mapped.source_src.clone(),
            });
        }
        driver::copy_sibling_roc(entry.parent().unwrap(), output, &plan.primary_name)?;
        fs::write(
            output.join("main.roc"),
            dispatch::rewrite_runtime_imports_for_pin(
                &plan.main_roc_with_platform("./platform/main.roc"),
                None,
            ),
        )?;
        fs::write(output.join("assets/rocci.css"), plan.extracted_css())?;
    } else {
        driver::copy_sibling_roc(
            entry.parent().unwrap(),
            output,
            entry.file_stem().unwrap().to_str().unwrap(),
        )?;
        fs::copy(&entry, output.join("main.roc"))?;
    }
    Ok(maps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn provider_jsonc_preserves_urls_and_rejects_unclosed_comments() {
        assert_eq!(
            provider::parse_jsonc(
                "{/* a */\"url\":\"https://example.test/a//b\",// b\n\"items\":[1,2,],}"
            )
            .unwrap(),
            json!({"url":"https://example.test/a//b","items":[1,2]})
        );
        for malformed in ["/*", "{\"a\": /*", "{\"a\":\"unterminated"] {
            assert!(provider::parse_jsonc(malformed).is_err());
        }
    }

    #[test]
    fn scaffold_is_repeatable_and_preserves_source_and_existing_scripts() {
        let temporary = driver::TempDir::create("worker-scaffold").unwrap();
        let root = temporary.path.join("path with spaces");
        fs::create_dir(&root).unwrap();
        let source = "@get:view(\"/\") = |_| page({})\n@component\nPage = |_| <html><body>Hello</body></html>\n";
        fs::write(root.join("Hello.rocci"), source).unwrap();
        fs::write(
            root.join("rocci.toml"),
            "# retained comment\n[app]\nname = 'Existing app'\n",
        )
        .unwrap();
        fs::write(
            root.join("package.json"),
            r#"{"scripts":{"dev":"existing-dev"}}"#,
        )
        .unwrap();
        scaffold::init(&root, true).unwrap();
        let first = fs::read(root.join("wrangler.jsonc")).unwrap();
        scaffold::init(&root, true).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("Hello.rocci")).unwrap(),
            source
        );
        assert_eq!(fs::read(root.join("wrangler.jsonc")).unwrap(), first);
        assert!(
            fs::read_to_string(root.join("rocci.toml"))
                .unwrap()
                .contains("# retained comment")
        );
        let package: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("package.json")).unwrap()).unwrap();
        assert_eq!(package["scripts"]["dev"], "existing-dev");
        assert_eq!(package["scripts"]["worker:dev"], "wrangler dev");
    }
}
