use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

static ROC_LOCK: Mutex<()> = Mutex::new(());

fn skip_without_roc() -> bool {
    let help_ok = Command::new("roc")
        .arg("help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !help_ok {
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
            panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
        }
        eprintln!("skipping: roc not on PATH");
        return true;
    }
    let test_dir = env::temp_dir().join(format!("roc-probe-{}", std::process::id()));
    let _ = fs::create_dir_all(&test_dir);
    let probe_file = test_dir.join("main.roc");
    let _ = fs::write(
        &probe_file,
        "app [main!] { pf: platform \"https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst\" }\nmain! = |_| Ok({})\n",
    );
    let build_ok = Command::new("roc")
        .arg("build")
        .arg(&probe_file)
        .arg("--opt=dev")
        .current_dir(&test_dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let _ = fs::remove_dir_all(&test_dir);
    if !build_ok {
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
            panic!("roc compilation failed during environment probe");
        }
        eprintln!("skipping: roc compilation not functional in this environment");
        return true;
    }
    false
}

fn rocdown_bin() -> PathBuf {
    let mut path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rocdown");
    if !path.is_file() {
        path = PathBuf::from(env!("CARGO_BIN_EXE_rocdown"));
    }
    path
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn temp_dir(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("rocdown-parity-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn collect_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    fn walk(dir: &Path, prefix: &Path, files: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let rel = prefix.join(entry.file_name());
            if path.is_dir() {
                walk(&path, &rel, files);
            } else {
                files.push((rel.to_string_lossy().into_owned(), fs::read(path).unwrap()));
            }
        }
    }
    walk(dir, Path::new(""), &mut files);
    files
}

#[test]
fn parity_build_output_matches_rocs_on_example_site() {
    if skip_without_roc() {
        return;
    }
    let _lock = ROC_LOCK.lock().unwrap();
    let root = repo_root();
    let site_root = root.join("examples/rocs");

    let rocs_out = temp_dir("rocs-out");
    let rocdown_out = temp_dir("rocdown-out");

    // Build with rocs library / builder
    rocs::build(&site_root, &rocs_out).unwrap();

    // Build with rocdown CLI
    let bin = rocdown_bin();
    let output = Command::new(&bin)
        .arg("build")
        .arg(&site_root)
        .arg("--output")
        .arg(&rocdown_out)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "rocdown build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let rocs_files = collect_files(&rocs_out);
    let rocdown_files = collect_files(&rocdown_out);

    assert_eq!(
        rocs_files.len(),
        rocdown_files.len(),
        "file count mismatch: rocs has {} files, rocdown has {} files",
        rocs_files.len(),
        rocdown_files.len()
    );

    for (rocs_name, rocs_bytes) in &rocs_files {
        let rocdown_match = rocdown_files
            .iter()
            .find(|(name, _)| name == rocs_name)
            .unwrap_or_else(|| panic!("missing file {rocs_name} in rocdown output"));
        assert_eq!(
            rocs_bytes, &rocdown_match.1,
            "byte mismatch for file {rocs_name}"
        );
    }

    let _ = fs::remove_dir_all(rocs_out);
    let _ = fs::remove_dir_all(rocdown_out);
}
