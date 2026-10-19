use std::{env, fs, path::Path};

fn main() {
    let directory = Path::new("../rocci-platform/platform");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut files = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "roc"))
        .collect::<Vec<_>>();
    files.sort();
    let mut generated = String::from("pub const ROC_SOURCES: &[(&str, &str)] = &[\n");
    for path in files {
        generated.push_str(&format!(
            "({:?}, include_str!({:?})),\n",
            path.file_name().unwrap().to_str().unwrap(),
            fs::canonicalize(&path).unwrap()
        ));
    }
    generated.push_str("];\n");
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("roc_sources.rs"),
        generated,
    )
    .unwrap();
}
