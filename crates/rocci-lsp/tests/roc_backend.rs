use std::env;
use std::path::PathBuf;

use lsp_types::{HoverContents, Position};
use rocci_lsp::{ChildRocBackend, RocBackend};

fn require_roc() -> bool {
    env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1")
}

#[test]
#[ignore]
fn live_child_hovers_generated_type_module() {
    let backend = match ChildRocBackend::spawn_from_env() {
        Ok(backend) => backend,
        Err(err) => {
            if require_roc() {
                panic!("roc experimental-lsp required: {err}");
            }
            return;
        }
    };
    let mut backend = backend;
    let dir = env::temp_dir().join(format!("rocci-lsp-roc-{}", std::process::id()));
    let path = dir.join("Hello.roc");
    let text = "Hello := [].{\n    greet = |name| name\n}\n";
    backend
        .sync_projection(&path, text)
        .expect("sync projection");
    let hover = backend
        .hover(&path, Position::new(1, 4))
        .expect("hover on greet");
    let HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markup hover");
    };
    assert!(
        markup.value.contains("->") || markup.value.contains("greet"),
        "unexpected hover: {}",
        markup.value
    );
    let _keep: PathBuf = dir;
}
