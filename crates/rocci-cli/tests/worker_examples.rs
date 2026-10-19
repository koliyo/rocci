use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn sources(directory: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("roc" | "rocci")
            )
        })
        .map(|path| {
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect()
}

#[test]
fn worker_stages_canonical_examples_without_modifying_authored_sources() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocci/standalone");
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "rocci-worker-examples-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )));
    fs::create_dir_all(&scratch.0).unwrap();
    let archive = scratch.0.join("host.a");
    // Staging copies the archive; compilation validates its ABI separately.
    fs::write(&archive, b"staging test archive").unwrap();

    for (application, module, live_paths) in [
        ("counter", "Counter", "live_paths = []"),
        ("live-counter", "LiveCounter", "live_paths = [\"/sse\"]"),
    ] {
        let directory = examples.join(application);
        let authored = sources(&directory);
        let output = scratch.0.join(application);
        rocci_cli::worker::stage(
            &directory.join(format!("{module}.rocci")),
            &output,
            &archive,
        )
        .unwrap();

        assert_eq!(sources(&directory), authored);
        let main = fs::read_to_string(output.join("main.roc")).unwrap();
        assert!(main.contains("program = { init!, respond!, shutdown! }"));
        assert!(main.contains("platform/main.roc"));
        assert!(
            fs::read_to_string(output.join(format!("{module}.roc")))
                .unwrap()
                .contains("import pf.Sqlite")
        );
        assert!(
            fs::read_to_string(output.join("platform/WorkerRoutes.roc"))
                .unwrap()
                .contains(live_paths)
        );
        assert!(output.join("assets/rocci.css").is_file());
    }
}
