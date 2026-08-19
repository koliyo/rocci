//! Generate owned AST structs from ungrammar tree specs.
//!
//! This crate does not generate scanners or parsers. Language crates must not
//! depend on it at runtime.

mod appendix;
mod dialect;
mod emit;
mod emit_kind;
mod emit_pprint;
mod inspect;
mod paths;
mod sidecar;

use std::path::{Path, PathBuf};

pub use dialect::Error;
pub use inspect::format_inspect_mapping;
pub use paths::{
    ROCCI_GENERATED, ROCCI_NODE_KIND, ROCCI_PPRINT, ROCCI_TOML, ROCCI_TREE_APPENDIX, ROCCI_UNGRAM,
    ROCDOWN_GENERATED, ROCDOWN_MARKDOWN_TOML, ROCDOWN_MARKDOWN_UNGRAM, ROCDOWN_MD_GENERATED,
    ROCDOWN_NODE_KIND, ROCDOWN_PPRINT, ROCDOWN_TOML, ROCDOWN_TREE_APPENDIX, ROCDOWN_UNGRAM,
};

use dialect::lower;
use emit::emit;
use emit_kind::emit_node_kind;
use emit_pprint::emit_pprint;
use sidecar::Sidecar;

pub fn generate_source(ungram: &str, sidecar_toml: &str) -> Result<String, Error> {
    let sidecar = Sidecar::parse(sidecar_toml)?;
    let ir = lower(ungram, &sidecar)?;
    Ok(emit(&ir))
}

pub fn find_workspace_root(start: &Path) -> Result<PathBuf, Error> {
    let mut dir = if start.is_absolute() {
        start.to_path_buf()
    } else {
        std::env::current_dir()?.join(start)
    };
    loop {
        if dir.join(ROCCI_UNGRAM).is_file() {
            return Ok(dir);
        }
        if !dir.pop() {
            return Err(Error::Dialect(format!(
                "could not find workspace root containing {ROCCI_UNGRAM} from {}",
                start.display()
            )));
        }
    }
}

pub struct LanguageSpec {
    pub ungram: &'static str,
    pub sidecar: &'static str,
    pub output: &'static str,
    pub pprint: &'static str,
    pub node_kind: &'static str,
}

pub const LANGUAGES: [LanguageSpec; 3] = [
    LanguageSpec {
        ungram: ROCCI_UNGRAM,
        sidecar: ROCCI_TOML,
        output: ROCCI_GENERATED,
        pprint: ROCCI_PPRINT,
        node_kind: ROCCI_NODE_KIND,
    },
    LanguageSpec {
        ungram: ROCDOWN_UNGRAM,
        sidecar: ROCDOWN_TOML,
        output: ROCDOWN_GENERATED,
        pprint: ROCDOWN_PPRINT,
        node_kind: ROCDOWN_NODE_KIND,
    },
    LanguageSpec {
        ungram: ROCDOWN_MARKDOWN_UNGRAM,
        sidecar: ROCDOWN_MARKDOWN_TOML,
        output: ROCDOWN_MD_GENERATED,
        pprint: "",
        node_kind: "",
    },
];

pub fn generate_languages(root: &Path) -> Result<Vec<(PathBuf, String)>, Error> {
    let mut out = Vec::new();
    for lang in LANGUAGES {
        let (ast, pprint, kind) = generate_pair(root, lang.ungram, lang.sidecar)?;
        out.push((root.join(lang.output), ast));
        if !lang.pprint.is_empty() {
            out.push((root.join(lang.pprint), pprint));
        }
        if !lang.node_kind.is_empty() {
            out.push((root.join(lang.node_kind), kind));
        }
    }
    out.extend(appendix::generate_appendices(root)?);
    Ok(out)
}

pub fn write_languages(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut written = Vec::new();
    for (path, source) in generate_languages(root)? {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, source)?;
        written.push(path);
    }
    Ok(written)
}

pub fn check_languages(root: &Path) -> Result<(), Error> {
    let mut stale = Vec::new();
    for (path, source) in generate_languages(root)? {
        match std::fs::read_to_string(&path) {
            Ok(existing) if existing == source => {}
            Ok(_) => stale.push(format!("{} is stale", path.display())),
            Err(_) => stale.push(format!("{} is missing", path.display())),
        }
    }
    if stale.is_empty() {
        Ok(())
    } else {
        stale.push("run `cargo run -q -p rocci-ungram -- generate`".to_string());
        Err(Error::Dialect(stale.join("\n")))
    }
}

fn generate_pair(
    root: &Path,
    ungram_path: &str,
    sidecar_path: &str,
) -> Result<(String, String, String), Error> {
    let ungram = std::fs::read_to_string(root.join(ungram_path))?;
    let sidecar_toml = std::fs::read_to_string(root.join(sidecar_path))?;
    let sidecar = Sidecar::parse(&sidecar_toml)?;
    let ir = lower(&ungram, &sidecar)?;
    Ok((
        emit(&ir),
        emit_pprint(&ir, &sidecar),
        emit_node_kind(&ir, &sidecar),
    ))
}
