use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use rocci_roc_host::{InputFingerprint, NativeHost, compute_compile_hash, compute_gen_hash};
use rocci_template::{
    LowerOptions, MappedModule, SourceFile, compile, component_matches, format_diagnostic,
    remap_roc_output, type_name_from_path, wrap_type_module,
};

const BASIC_CLI_PLATFORM: &str = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst";
const TEMPLATE_REL: &str = "templates/PreviewNav.rocci";
const COMMITTED_REL: &str = "generated/preview_nav.html";
const STAMP_REL: &str = "generated/preview_nav.sha256";

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let template = manifest_dir.join(TEMPLATE_REL);
    let committed = manifest_dir.join(COMMITTED_REL);
    let stamp = manifest_dir.join(STAMP_REL);
    let out_html = out_dir.join("preview_nav.html");

    println!("cargo:rerun-if-changed={TEMPLATE_REL}");
    println!("cargo:rerun-if-env-changed=ROCCI_REQUIRE_ROC");

    let src = fs::read(&template).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", template.display());
    });
    let digest = InputFingerprint::from_bytes("PreviewNav.rocci", &src).sha256;
    let stamp_ok = fs::read_to_string(&stamp)
        .map(|value| value.trim() == digest)
        .unwrap_or(false);

    if stamp_ok && committed.is_file() {
        copy_html(&committed, &out_html);
        return;
    }

    if roc_available() {
        let html = render_fragment(&template, &src);
        write_html(&out_html, &html);
        write_html(&committed, &html);
        write_if_changed(&stamp, format!("{digest}\n").as_bytes());
        return;
    }

    if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
        panic!(
            "{TEMPLATE_REL} is dirty and roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH"
        );
    }

    if !committed.is_file() {
        panic!("{COMMITTED_REL} is missing and roc is not on PATH; cannot embed preview chrome");
    }
    println!(
        "cargo:warning={TEMPLATE_REL} changed; roc not on PATH, using committed {COMMITTED_REL}"
    );
    copy_html(&committed, &out_html);
}

fn roc_available() -> bool {
    Command::new("roc")
        .arg("help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn copy_html(from: &Path, to: &Path) {
    fs::copy(from, to).unwrap_or_else(|err| {
        panic!(
            "failed to copy {} -> {}: {err}",
            from.display(),
            to.display()
        );
    });
}

fn write_html(path: &Path, html: &str) {
    let mut body = html.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    write_if_changed(path, body.as_bytes());
}

fn write_if_changed(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| {
            panic!("failed to create {}: {err}", parent.display());
        });
    }
    if fs::read(path).ok().as_deref() == Some(bytes) {
        return;
    }
    fs::write(path, bytes).unwrap_or_else(|err| {
        panic!("failed to write {}: {err}", path.display());
    });
}

fn render_fragment(template: &Path, src_bytes: &[u8]) -> String {
    let src = std::str::from_utf8(src_bytes).unwrap_or_else(|err| {
        panic!("{} is not valid UTF-8: {err}", template.display());
    });
    let filename = "PreviewNav.rocci";
    let source = SourceFile::new(filename, src);
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        println!(
            "cargo:warning={}",
            format_diagnostic(source, diagnostic).replace('\n', " | ")
        );
    }
    if compiled.has_errors() {
        panic!("failed to compile {filename}");
    }
    let type_name = type_name_from_path(Path::new(filename));
    let fixture = compiled
        .fixtures
        .first()
        .unwrap_or_else(|| panic!("{filename} needs a @fixture for static chrome"));
    let component = compiled
        .components
        .iter()
        .find(|component| component_matches(&component.name, &fixture.target))
        .unwrap_or_else(|| {
            panic!(
                "fixture `{}` targets `{}`, which was not found",
                fixture.name, fixture.target
            )
        });
    let call = format!("{type_name}.{}({})", component.name, fixture.value);
    stage_and_render(
        filename,
        src,
        &compiled.roc,
        &compiled.segments,
        &type_name,
        &call,
    )
}

fn stage_and_render(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    type_name: &str,
    call: &str,
) -> String {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let workspace = out_dir.join("preview-nav-roc");
    if workspace.exists() {
        let _ = fs::remove_dir_all(&workspace);
    }
    fs::create_dir_all(&workspace).unwrap_or_else(|err| {
        panic!("failed to create {}: {err}", workspace.display());
    });
    fs::write(workspace.join("Html.roc"), rocci_ui::HTML_ROC).unwrap();
    let wrapped = wrap_type_module(roc, type_name);
    fs::write(workspace.join(format!("{type_name}.roc")), &wrapped).unwrap();
    let main = snapshot_main(type_name, call);
    fs::write(workspace.join("main.roc"), &main).unwrap();

    let html_roc = rocci_ui::HTML_ROC;
    let gen_hash = compute_gen_hash(
        env!("CARGO_PKG_VERSION"),
        "desktop-chrome",
        &[
            (&format!("{type_name}.roc"), wrapped.as_bytes()),
            ("main.roc", main.as_bytes()),
        ],
        &[("Html.roc", html_roc.as_bytes())],
    );
    let compile_hash = compute_compile_hash(
        &gen_hash,
        "roc",
        &format!("native:{}", env::consts::ARCH),
        "dev",
        BASIC_CLI_PLATFORM,
        env!("CARGO_PKG_VERSION"),
    );
    let fingerprints = [
        InputFingerprint::from_bytes(&format!("{type_name}.roc"), wrapped.as_bytes()),
        InputFingerprint::from_bytes("main.roc", main.as_bytes()),
        InputFingerprint::from_bytes("Html.roc", html_roc.as_bytes()),
    ];
    let host = NativeHost::default();
    let (apply_bin, _) = host
        .compile_or_cached(&workspace, &compile_hash, &fingerprints)
        .unwrap_or_else(|err| {
            panic!(
                "{}",
                annotate_roc_error(filename, source, roc, segments, type_name, &err.to_string())
            )
        });

    let output = Command::new(&apply_bin)
        .current_dir(&workspace)
        .output()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", apply_bin.display()));
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        panic!(
            "{}",
            annotate_roc_error(
                filename,
                source,
                roc,
                segments,
                type_name,
                &format!("{stdout}{stderr}"),
            )
        );
    }
    let html = stdout.trim_end_matches(['\r', '\n']).to_string();
    if html.is_empty() {
        panic!("roc snapshot produced no HTML");
    }
    html
}

fn snapshot_main(type_name: &str, call: &str) -> String {
    format!(
        r#"app [main!] {{ pf: platform "{BASIC_CLI_PLATFORM}" }}

import pf.Stdout
import {type_name}
import Html

main! = |_args| {{
    _ = Stdout.line!(Html.render({call}))
    Ok({{}})
}}
"#
    )
}

fn annotate_roc_error(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    type_name: &str,
    output: &str,
) -> String {
    let mapped = remap_roc_output(
        output,
        &[MappedModule {
            type_name: type_name.to_string(),
            generated: roc.to_string(),
            source_name: filename.to_string(),
            source_src: source.to_string(),
            segments: segments.to_vec(),
        }],
    );
    if mapped.is_empty() {
        return format!("roc failed to render HTML:\n{}", output.trim());
    }
    let frames: Vec<String> = mapped.iter().map(|frame| frame.message.clone()).collect();
    format!(
        "roc failed to render HTML:\n{}\n{}",
        frames.join("\n"),
        output.trim()
    )
}
