//! One preopen directory for `Server.file_root`. Native OS paths are not granted.

use std::path::{Component, Path, PathBuf};

use anyhow::{Result, bail};

pub fn resolve_preopen(root: &Path, rel: &str) -> Result<PathBuf> {
    let rel = rel.trim_start_matches('/');
    let mut out = PathBuf::from(root);
    for component in Path::new(rel).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("path escapes preopen: {rel}");
            }
        }
    }
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    if let Ok(canonical) = out.canonicalize()
        && !canonical.starts_with(&canonical_root)
    {
        bail!("path escapes preopen: {rel}");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "embedder")]
    use crate::abi::{IncomingRequest, OutcomeToHost, ServerRequest};
    #[cfg(feature = "embedder")]
    use crate::guest::RocGuest;
    #[cfg(feature = "embedder")]
    use crate::handle::Adapter;

    #[cfg(feature = "embedder")]
    struct FileGuest;

    #[cfg(feature = "embedder")]
    impl RocGuest for FileGuest {
        fn init(&mut self) {}
        fn respond(&mut self, request: &ServerRequest) -> OutcomeToHost {
            OutcomeToHost::File {
                rel_path: request.target_path.trim_start_matches('/').to_string(),
            }
        }
        fn shutdown(&mut self) {}
    }

    #[cfg(feature = "embedder")]
    #[tokio::test(flavor = "current_thread")]
    async fn get_static_file_from_preopen() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/static");
        let mut adapter = Adapter::new(FileGuest).with_file_root(dir);
        let response = adapter
            .handle(IncomingRequest {
                method: "GET".into(),
                path: "/hello.txt".into(),
                headers: vec![],
                body: Vec::new(),
            })
            .await
            .unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"preopen-bytes");
    }

    #[test]
    fn rejects_parent_escape() {
        let err = resolve_preopen(Path::new("/tmp"), "../etc/passwd").unwrap_err();
        assert!(err.to_string().contains("escapes"), "{err}");
    }
}
