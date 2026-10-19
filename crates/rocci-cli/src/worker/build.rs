use super::{provider, toolchain};
use anyhow::{Context, Result, bail, ensure};
use rocci_core::{DatastarAsset, WorkerExecution};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Component, Path},
    process::Command,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const RUNTIME: &[(&str, &str)] = &[
    (
        "admission.mjs",
        include_str!("../../../rocci-worker-platform/runtime/admission.mjs"),
    ),
    (
        "guest.mjs",
        include_str!("../../../rocci-worker-platform/runtime/guest.mjs"),
    ),
    (
        "sql.mjs",
        include_str!("../../../rocci-worker-platform/runtime/sql.mjs"),
    ),
    (
        "transactions.mjs",
        include_str!("../../../rocci-worker-platform/runtime/transactions.mjs"),
    ),
    (
        "cloudflare.d.mts",
        include_str!("../../../rocci-worker-platform/runtime/cloudflare.d.mts"),
    ),
    (
        "cloudflare.mjs",
        include_str!("../../../rocci-worker-platform/runtime/cloudflare.mjs"),
    ),
];

fn runtime_hash() -> String {
    let mut bytes = Vec::new();
    for (name, source) in RUNTIME {
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(source.as_bytes());
    }
    for (name, source) in rocci_worker_platform::sources::ROC_SOURCES {
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(
            rocci_worker_platform::sources::compatibility_source(
                name.trim_end_matches(".roc"),
                source,
            )
            .as_bytes(),
        );
    }
    toolchain::hash(&bytes)
}
fn ignored(name: &str) -> bool {
    name.starts_with('.')
        || name == "rocci-worker.lock"
        || matches!(name, "target" | "dist" | "node_modules" | "generated")
}
fn files(
    root: &Path,
    directory: &Path,
    output: &mut BTreeMap<String, String>,
    source: bool,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if source && ignored(&name) {
            continue;
        }
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Worker inputs cannot be symlinks: {}",
            path.display()
        );
        if kind.is_dir() {
            files(root, &path, output, source)?;
        } else if kind.is_file() {
            output.insert(
                path.strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/"),
                toolchain::file_hash(&path)?,
            );
        }
    }
    Ok(())
}
fn cached(output: &Path, key: &str) -> bool {
    let check = || -> Result<bool> {
        let manifest: Value =
            serde_json::from_str(&fs::read_to_string(output.join("manifest.json"))?)?;
        if manifest["input_key"].as_str() != Some(key) {
            return Ok(false);
        }
        let mut actual = BTreeMap::new();
        files(output, output, &mut actual, false)?;
        actual.remove("manifest.json");
        Ok(serde_json::to_value(actual)? == manifest["files"])
    };
    check().unwrap_or(false)
}
fn copy_public(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Public Worker assets cannot be symlinks: {}",
            path.display()
        );
        if kind.is_dir() {
            copy_public(&path, &to.join(&name))?;
        } else if kind.is_file() {
            ensure!(
                entry.metadata()?.len() <= 16 * 1024 * 1024,
                "Public asset exceeds 16 MiB: {}",
                path.display()
            );
            fs::copy(&path, to.join(name))?;
        }
    }
    Ok(())
}
fn public_assets(project: &provider::Project, stage: &Path, offline: bool) -> Result<Vec<String>> {
    let public = stage.join("public");
    fs::create_dir_all(public.join("assets"))?;
    let source = project
        .config
        .assets
        .directory
        .as_ref()
        .map(|path| project.root.join(path))
        .unwrap_or_else(|| project.root.join("assets"));
    if source.is_dir() {
        copy_public(&source, &public.join("assets"))?;
    }
    let generated = stage.join("assets/rocci.css");
    if generated.is_file() {
        fs::copy(generated, public.join("assets/rocci.css"))?;
    }
    match &project.config.assets.datastar {
        Some(DatastarAsset::Disabled) => {}
        pin => {
            let version = match pin {
                Some(DatastarAsset::Version(version)) => version.as_str(),
                _ => crate::datastar_asset::DEFAULT_VERSION,
            };
            if offline {
                let cache = crate::datastar_asset::cache_dir()?
                    .join("datastar")
                    .join(crate::datastar_asset::tag_name(version));
                let bytes = fs::read(cache.join("datastar.js")).context(
                    "Offline build requires a cached Datastar asset or assets.datastar = false",
                )?;
                ensure!(
                    fs::read_to_string(cache.join("sha256"))?.trim() == toolchain::hash(&bytes),
                    "Offline Datastar cache checksum mismatch"
                );
            }
            crate::datastar_asset::stage_into(&public.join("assets"), version)?;
        }
    }
    if project.entry.extension().is_some_and(|ext| ext == "rocci") {
        let plan = crate::run::standalone_http_module_app_plan(&project.entry)?;
        for url in plan.modules.iter().flat_map(|module| &module.local_assets) {
            let relative = url
                .trim_start_matches('/')
                .trim_start_matches("./")
                .split(['?', '#'])
                .next()
                .unwrap_or("");
            let path = Path::new(relative);
            if path
                .components()
                .any(|part| !matches!(part, Component::Normal(_)))
            {
                continue;
            }
            let from = project.root.join(path);
            if from.is_file() {
                ensure!(
                    !fs::symlink_metadata(&from)?.file_type().is_symlink(),
                    "Referenced asset is a symlink"
                );
                let to = public.join(path);
                fs::create_dir_all(to.parent().unwrap())?;
                fs::copy(from, to)?;
            }
        }
    }
    let mut assets = BTreeMap::new();
    files(&public, &public, &mut assets, false)?;
    Ok(assets.keys().map(|name| format!("/{name}")).collect())
}
fn inspect(bytes: &[u8], execution: &WorkerExecution) -> Result<Vec<String>> {
    wasmparser::Validator::new()
        .validate_all(bytes)
        .context("Generated artifact is not valid core Wasm")?;
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(bytes) {
        match payload? {
            wasmparser::Payload::ImportSection(section) => {
                for import in section.into_imports() {
                    let import = import?;
                    ensure!(
                        import.module == "rocci"
                            && matches!(
                                import.name,
                                "resource_drop"
                                    | "env"
                                    | "log"
                                    | "sql"
                                    | "transaction"
                                    | "body"
                                    | "http"
                                    | "clock"
                                    | "sleep"
                            ),
                        "Unsupported reachable Worker effect: {}.{}; select the native platform or remove this effect from the reachable handler",
                        import.module,
                        import.name
                    );
                    ensure!(
                        *execution == WorkerExecution::DurableObject
                            || !matches!(import.name, "sql" | "transaction"),
                        "Reachable {} effect requires worker.execution = durable-object",
                        import.name
                    );
                    imports.push(import.name.to_string());
                }
            }
            wasmparser::Payload::ExportSection(section) => {
                for export in section {
                    exports.push(export?.name.to_string());
                }
            }
            wasmparser::Payload::MemorySection(section) => {
                for memory in section {
                    let memory = memory?;
                    ensure!(
                        !memory.shared && !memory.memory64,
                        "Worker ABI requires unshared Wasm32 memory"
                    );
                }
            }
            _ => {}
        }
    }
    for required in [
        "memory",
        "worker_abi_version",
        "worker_input_ptr",
        "worker_input_capacity",
        "worker_output_ptr",
        "worker_output_len",
        "worker_invoke",
    ] {
        ensure!(
            exports.iter().any(|name| name == required),
            "Worker host archive is missing ABI export {required}"
        );
    }
    Ok(imports)
}

pub fn build(
    input: &Path,
    roc: Option<&Path>,
    archive: Option<&Path>,
    locked: bool,
    offline: bool,
    opt: Option<crate::native_target::RocOpt>,
) -> Result<()> {
    let started = Instant::now();
    let project = provider::load(input)?;
    let profile = project.config.worker.as_ref().unwrap();
    let opt = opt.map(|opt| opt.as_roc_opt()).unwrap_or("speed");
    ensure!(
        opt != "interpreter",
        "Workers require a compiled dev, speed or size backend"
    );
    let compiler = toolchain::compiler(roc, offline)?;
    let archive = toolchain::host_archive(archive, offline)?;
    let lock_path = project.root.join("rocci-worker.lock");
    let compiler_digest = toolchain::file_hash(&compiler)?;
    let compiler_host = format!("{}-{}", env::consts::OS, env::consts::ARCH);
    let mut compiler_hashes = fs::read(&lock_path)
        .ok()
        .and_then(|source| serde_json::from_slice::<toolchain::Lock>(&source).ok())
        .filter(|lock| lock.compiler == rocci_worker_platform::COMPILER_VERSION)
        .map(|lock| lock.compiler_sha256)
        .unwrap_or_default();
    compiler_hashes.insert(compiler_host, compiler_digest.clone());
    let lock = toolchain::Lock {
        format: 1,
        abi: rocci_worker_platform::ABI_VERSION,
        compiler: rocci_worker_platform::COMPILER_VERSION.into(),
        compiler_sha256: compiler_hashes,
        host_sha256: toolchain::file_hash(&archive)?,
        runtime_sha256: runtime_hash(),
    };
    if locked {
        let recorded:toolchain::Lock=serde_json::from_str(&fs::read_to_string(&lock_path).context("Missing rocci-worker.lock; build once without --locked and commit the generated lock")?)?;
        ensure!(
            recorded == lock,
            "Worker toolchain differs from rocci-worker.lock; review the compiler/host/runtime change (including a first build on this development host) and rebuild without --locked"
        );
    }
    let mut inputs = BTreeMap::new();
    files(&project.root, &project.root, &mut inputs, true)?;
    let key = toolchain::hash(&serde_json::to_vec(
        &json!({"inputs":inputs,"toolchain":lock,"opt":opt,"cli":toolchain::file_hash(&env::current_exe()?)?}),
    )?);
    let output = project.root.join(".rocci/worker");
    fs::create_dir_all(project.root.join(".rocci"))?;
    let editor = serde_json::to_vec(
        &json!({"compiler":compiler,"version":rocci_worker_platform::COMPILER_VERSION,"sha256":compiler_digest}),
    )?;
    let editor_path = project.root.join(".rocci/worker-editor.json");
    if fs::read(&editor_path).ok().as_deref() != Some(&editor) {
        fs::write(&editor_path, editor)?;
    }
    if cached(&output, &key) {
        let lock_source = format!("{}\n", serde_json::to_string_pretty(&lock)?);
        if fs::read(&lock_path).ok().as_deref() != Some(lock_source.as_bytes()) {
            fs::write(&lock_path, lock_source)?;
        }
        println!(
            "Worker build cached ({} ms): {}",
            started.elapsed().as_millis(),
            output.display()
        );
        return Ok(());
    }
    let failure_path = project.root.join(".rocci/worker-failure.json");
    if env::var("WRANGLER_COMMAND").as_deref() == Ok("dev")
        && let Ok(source) = fs::read_to_string(&failure_path)
        && let Ok(previous) = serde_json::from_str::<Value>(&source)
        && previous["input_key"].as_str() == Some(&key)
    {
        bail!(
            "Previous Worker build failed for these inputs:\n{}\nEdit a source input or retry rocci worker build directly.",
            previous["error"]
                .as_str()
                .unwrap_or("Unknown build failure")
        );
    }
    fs::create_dir_all(output.parent().unwrap())?;
    let guard = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(project.root.join(".rocci/worker-build.lock"))?;
    guard
        .try_lock()
        .context("Another Worker build is active for this application")?;
    if output.exists() {
        let previous: Value = serde_json::from_str(
            &fs::read_to_string(output.join("manifest.json"))
                .context("Refusing to replace an output tree without a Rocci manifest")?,
        )?;
        ensure!(
            previous["format"] == 1,
            "Unrecognized Worker output manifest"
        );
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let next = project.root.join(format!(".rocci/worker.next-{nonce}"));
    fs::create_dir(&next)?;
    let result = (|| -> Result<()> {
        let maps = super::stage_with_metadata(&project.entry, &next, &archive)?;
        fs::create_dir_all(next.join("runtime"))?;
        for (name, source) in RUNTIME {
            fs::write(next.join("runtime").join(name), source)?;
        }
        let assets = public_assets(&project, &next, offline)?;
        let settings = json!({"sqliteAliases":profile.sqlite_aliases(),"assetPaths":assets,"csp":project.config.csp(),"sources":source_locations(&project.entry,&project.root)?});
        let entry = if let Some(class) = &project.class {
            format!(
                "import module from './app.wasm';\nimport {{WorkerObject,objectWorker}} from './runtime/cloudflare.mjs';\nconst profile={settings};\nexport class {class} extends WorkerObject{{constructor(ctx,env){{super(module,ctx,env,profile);}}}}\nexport default objectWorker({}, {}, profile);\n",
                serde_json::to_string(profile.binding.as_deref().unwrap())?,
                serde_json::to_string(profile.object_key.as_deref().unwrap())?
            )
        } else {
            format!(
                "import module from './app.wasm';\nimport {{statelessWorker}} from './runtime/cloudflare.mjs';\nexport default statelessWorker(module,{settings});\n"
            )
        };
        fs::write(next.join("index.mjs"), entry)?;
        let source_maps=maps.iter().map(|map|json!({"module":map.type_name,"source":Path::new(&map.source_name).strip_prefix(&project.root).unwrap_or(Path::new(&map.source_name)).to_string_lossy(),"segments":map.segments.iter().map(|segment|json!({"generated":[segment.generated.start,segment.generated.end],"source":[segment.source.start,segment.source.end]})).collect::<Vec<_>>()})).collect::<Vec<_>>();
        fs::write(
            next.join("source-maps.json"),
            serde_json::to_vec_pretty(&source_maps)?,
        )?;
        let cache = toolchain::cache()?.join("compiler-cache");
        fs::create_dir_all(&cache)?;
        if offline {
            offline_packages(&next, &cache.join("roc/packages"))?;
        }
        let process = Command::new(&compiler)
            .current_dir(&next)
            .env("XDG_CACHE_HOME", cache)
            .args([
                "build",
                "--no-cache",
                "--target=wasm32",
                "--specialize=yes",
                "--wasm-stack-size=1048576",
                &format!("--opt={opt}"),
                "--output=app.wasm",
                "main.roc",
            ])
            .output()?;
        let diagnostic = format!(
            "{}{}",
            String::from_utf8_lossy(&process.stdout),
            String::from_utf8_lossy(&process.stderr)
        );
        let artifact = next.join("app.wasm");
        if !process.status.success()
            && !(process.status.code() == Some(2)
                && artifact.is_file()
                && diagnostic
                    .lines()
                    .any(|line| line.trim_start().starts_with("0 errors and ")))
        {
            for frame in rocci_template::remap_roc_output(&diagnostic, &maps) {
                eprintln!(
                    "{}:{}:{}: {}\n{}\n{}",
                    frame.file,
                    frame.line,
                    frame.column,
                    frame.message,
                    frame.source_line,
                    frame.caret_line()
                );
            }
            bail!("Worker Roc compilation failed:\n{diagnostic}");
        }
        let wasm = fs::read(&artifact).context("Compiler produced no Worker Wasm")?;
        let imports = inspect(&wasm, &profile.execution)?;
        let mut outputs = BTreeMap::new();
        files(&next, &next, &mut outputs, false)?;
        let manifest = json!({"format":1,"input_key":key,"toolchain":lock,"opt":opt,"inputs":inputs,"files":outputs,"imports":imports,"provider":{"configuration":project.provider_path.file_name().unwrap().to_string_lossy(),"compatibility_date":project.provider["compatibility_date"],"class":project.class},"limits":{"queued_invocations":32,"resources":1024,"streams":128,"sql_rows":10000,"sql_bytes":1048576,"body_bytes":1048576,"fetch_bytes":262144},"source_edits":[],"generated_transport_adjustments":["Retry Always for declared live GET actions unless explicitly overridden"],"elapsed_ms":started.elapsed().as_millis()});
        fs::write(
            next.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        let lock_next = next.join("toolchain.lock.next");
        fs::write(
            &lock_next,
            format!("{}\n", serde_json::to_string_pretty(&lock)?),
        )?;
        let previous = project.root.join(format!(".rocci/worker.previous-{nonce}"));
        if output.exists() {
            fs::rename(&output, &previous)?;
        }
        if let Err(error) = fs::rename(&next, &output) {
            if previous.exists() {
                fs::rename(&previous, &output)?;
            }
            return Err(error.into());
        }
        if let Err(error) = fs::rename(output.join("toolchain.lock.next"), &lock_path) {
            fs::rename(&output, &next)?;
            if previous.exists() {
                fs::rename(&previous, &output)?;
            }
            return Err(error.into());
        }
        if previous.exists()
            && let Err(error) = fs::remove_dir_all(previous)
        {
            eprintln!("[rocci worker] previous output cleanup: {error}");
        }
        if failure_path.exists() {
            let _ = fs::remove_file(&failure_path);
        }
        println!(
            "Worker built in {} ms: {} ({} Wasm bytes)",
            started.elapsed().as_millis(),
            output.display(),
            wasm.len()
        );
        Ok(())
    })();
    if next.exists() {
        fs::remove_dir_all(&next)?;
    }
    if let Err(error) = &result {
        let failure =
            serde_json::to_vec_pretty(&json!({"input_key":key,"error":format!("{error:#}")}))?;
        if fs::read(&failure_path).ok().as_deref() != Some(&failure) {
            fs::write(&failure_path, failure)?;
        }
    }
    result
}

// Roc currently has no offline flag. Admit only a closed, preseeded package
// graph, before starting the compiler; never let a cache miss trigger a fetch.
fn offline_packages(stage: &Path, cache: &Path) -> Result<()> {
    fn visit(
        directory: &Path,
        cache: &Path,
        seen: &mut std::collections::BTreeSet<String>,
    ) -> Result<()> {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(&path, cache, seen)?;
                continue;
            }
            if !path.extension().is_some_and(|extension| extension == "roc") {
                continue;
            }
            let source = fs::read_to_string(path)?;
            for line in source
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
            {
                for token in line.split('"').filter(|token| {
                    token.starts_with("https://")
                        && (token.ends_with(".tar.zst") || token.ends_with(".tar.gz"))
                }) {
                    let filename = token.rsplit('/').next().unwrap();
                    let hash = filename
                        .trim_end_matches(".tar.zst")
                        .trim_end_matches(".tar.gz");
                    ensure!(
                        hash.bytes().all(|byte| byte.is_ascii_alphanumeric()
                            || byte == b'_'
                            || byte == b'-'),
                        "Unsupported offline package address {token}"
                    );
                    if !seen.insert(hash.to_owned()) {
                        continue;
                    }
                    let directory = cache.join(hash);
                    ensure!(
                        directory.is_dir(),
                        "Offline Worker package is not cached: {token}; build once online or preseed {}",
                        directory.display()
                    );
                    let metadata: Value = serde_json::from_str(
                        &fs::read_to_string(cache.join(format!("{hash}.deps.json")))
                            .context("Offline package requires compiler dependency metadata")?,
                    )?;
                    ensure!(
                        metadata["format"] == 3
                            && metadata["kind"] == "package"
                            && metadata["deps"]
                                .as_array()
                                .is_some_and(|deps| deps.is_empty()),
                        "Offline Worker currently requires packages with an empty format-3 dependency manifest; resolve this graph online first"
                    );
                    visit(&directory, cache, seen)?;
                }
            }
        }
        Ok(())
    }
    visit(stage, cache, &mut std::collections::BTreeSet::new())
}

/// Attribute provider errors to authored declarations, independently of Wasm PCs.
fn source_locations(entry: &Path, root: &Path) -> Result<Value> {
    let fallback = format!(
        "{}:1",
        entry
            .strip_prefix(root)
            .unwrap_or(entry)
            .to_string_lossy()
            .replace('\\', "/")
    );
    if entry
        .extension()
        .is_some_and(|extension| extension == "roc")
    {
        return Ok(json!({"fallback":fallback}));
    }
    let plan = crate::run::standalone_http_module_app_plan(entry)?;
    let mut routes = Vec::new();
    let mut init = None;
    for module in &plan.modules {
        let source =
            rocci_template::SourceFile::new(&module.mapped.source_name, &module.mapped.source_src);
        let file = Path::new(&module.mapped.source_name)
            .strip_prefix(root)
            .unwrap_or(Path::new(&module.mapped.source_name))
            .to_string_lossy()
            .replace('\\', "/");
        let location = |offset: u32| {
            // Declaration spans include leading documentation comments.
            let offset = offset as usize
                + module.mapped.source_src[offset as usize..]
                    .split_inclusive('\n')
                    .take_while(|line| {
                        line.trim().is_empty() || line.trim_start().starts_with("##")
                    })
                    .map(str::len)
                    .sum::<usize>();
            format!(
                "{file}:{}",
                source
                    .position(offset as u32, rocci_template::PositionEncoding::Utf8)
                    .0
                    + 1
            )
        };
        if let Some(info) = &module.init {
            init = Some(location(info.span.start));
        }
        for route in &module.routes {
            routes.push(json!({"method":route.method.to_ascii_uppercase(),"path":route.path,"location":location(route.span.start)}));
        }
        for route in &module.lives {
            routes.push(json!({"method":route.method.to_ascii_uppercase(),"path":route.path,"location":location(route.span.start)}));
        }
    }
    Ok(json!({"fallback":fallback,"init":init,"routes":routes}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_sources_point_to_authored_declarations() {
        let entry =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/worker/c0/Counter.rocci");
        let sources = source_locations(&entry, entry.parent().unwrap()).unwrap();
        assert_eq!(sources["init"], "Counter.rocci:9");
        assert!(
            sources["routes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|route| route["path"] == "/" && route["location"] == "Counter.rocci:34"),
            "{sources}"
        );
    }
}
