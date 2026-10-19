//! Private example build input generator; not a supported CLI target.
use std::{env, fs, path::PathBuf};

use rocci_template::{LowerOptions, SourceFile, compile, wrap_type_module};

fn main() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = PathBuf::from(env::args().nth(1).expect("output directory"));
    fs::create_dir_all(output.join("assets"))?;
    let mut css = String::new();
    for name in ["LiveCounterUi", "CounterDocument"] {
        let path = root.join(format!(
            "examples/rocci/standalone/live-counter/{name}.rocci"
        ));
        let source = fs::read_to_string(&path)?;
        let result = compile(
            SourceFile::new(path.to_str().unwrap(), &source),
            &LowerOptions {
                embed_css: false,
                stylesheet_href: Some("/assets/rocci.css".into()),
                ..Default::default()
            },
        );
        anyhow::ensure!(!result.has_errors(), "{name}: {:?}", result.diagnostics);
        for style in &result.styles {
            css.push_str(&style.css);
            css.push('\n');
        }
        fs::write(
            output.join(format!("{name}.roc")),
            wrap_type_module(&result.roc, name),
        )?;
    }
    fs::write(output.join("assets/rocci.css"), css)?;
    fs::copy(
        root.join("crates/rocci-rocdown/runtime/Html.roc"),
        output.join("Html.roc"),
    )?;
    fs::copy(
        root.join("crates/rocci-rocdown/runtime/Datastar.roc"),
        output.join("Datastar.roc"),
    )?;
    for name in ["Origin", "CounterCore"] {
        fs::copy(
            root.join(format!("examples/rocci/standalone/live-counter/{name}.roc")),
            output.join(format!("{name}.roc")),
        )?;
    }
    Ok(())
}
