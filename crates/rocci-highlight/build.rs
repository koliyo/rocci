fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        return;
    }
    // Compile Roc Tree-sitter parser
    let roc_dir = std::path::PathBuf::from("grammars/roc/src");
    let mut roc_build = cc::Build::new();
    roc_build.include(&roc_dir);
    roc_build.file(roc_dir.join("parser.c"));
    roc_build.file(roc_dir.join("scanner.c"));
    roc_build.warnings(false);
    roc_build.compile("tree-sitter-roc");

    // Compile CSS Tree-sitter parser
    let css_dir = std::path::PathBuf::from("grammars/css/src");
    let mut css_build = cc::Build::new();
    css_build.include(&css_dir);
    css_build.file(css_dir.join("parser.c"));
    css_build.file(css_dir.join("scanner.c"));
    css_build.warnings(false);
    css_build.compile("tree-sitter-css");

    // Compile HTML Tree-sitter parser
    let html_dir = std::path::PathBuf::from("grammars/html/src");
    let mut html_build = cc::Build::new();
    html_build.include(&html_dir);
    html_build.file(html_dir.join("parser.c"));
    html_build.file(html_dir.join("scanner.c"));
    html_build.warnings(false);
    html_build.compile("tree-sitter-html");

    println!("cargo:rerun-if-changed=grammars/roc/src/parser.c");
    println!("cargo:rerun-if-changed=grammars/roc/src/scanner.c");
    println!("cargo:rerun-if-changed=grammars/css/src/parser.c");
    println!("cargo:rerun-if-changed=grammars/css/src/scanner.c");
    println!("cargo:rerun-if-changed=grammars/html/src/parser.c");
    println!("cargo:rerun-if-changed=grammars/html/src/scanner.c");
}
