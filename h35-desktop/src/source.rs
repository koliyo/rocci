use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

pub fn resolve_source_file(root: &Path, spec: &str) -> Option<PathBuf> {
    let root = root.canonicalize().ok()?;
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    for rel in candidates(spec) {
        if let Some(path) = confine(&root, &rel)
            && path.is_file()
        {
            return Some(path);
        }
    }
    None
}

fn candidates(spec: &str) -> Vec<String> {
    let trimmed = spec.trim_matches('/');
    if looks_like_file(spec) || looks_like_file(trimmed) {
        return vec![trimmed.to_string()];
    }
    if trimmed.is_empty() {
        return vec!["index.md".into(), "index.html".into()];
    }
    vec![
        format!("{trimmed}.md"),
        format!("{trimmed}.html"),
        format!("{trimmed}/index.md"),
        format!("{trimmed}/index.html"),
    ]
}

fn looks_like_file(spec: &str) -> bool {
    Path::new(spec).extension().is_some()
}

fn confine(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = Path::new(rel);
    if rel.is_absolute() {
        return None;
    }
    if rel
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return None;
    }
    let canonical = root.join(rel).canonicalize().ok()?;
    canonical.starts_with(root).then_some(canonical)
}

pub fn reveal_in_file_manager(path: &Path) {
    if let Err(error) = reveal(path) {
        tracing::error!(%error, path = %path.display(), "failed to reveal source file");
    }
}

pub fn copy_file_text(path: &Path) {
    match fs::read_to_string(path) {
        Ok(text) => {
            if let Err(error) = copy_text(&text) {
                tracing::error!(%error, path = %path.display(), "failed to copy source file");
            }
        }
        Err(error) => {
            tracing::error!(%error, path = %path.display(), "failed to read source file");
        }
    }
}

fn reveal(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg("-R").arg(path).spawn()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let uri = format!("file://{}", path.display());
        let shown = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("array:string:{uri}"),
                "string:",
            ])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if shown {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            Command::new("xdg-open").arg(parent).spawn()?;
        }
        Ok(())
    }
}

fn copy_text(text: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        pipe_stdin("pbcopy", &[], text)
    }
    #[cfg(target_os = "windows")]
    {
        pipe_stdin("cmd", &["/c", "clip"], text)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if pipe_stdin("wl-copy", &[], text).is_ok() {
            return Ok(());
        }
        pipe_stdin("xclip", &["-selection", "clipboard"], text)
    }
}

fn pipe_stdin(program: &str, args: &[&str], text: &str) -> std::io::Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    let status = child.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{program} exited with {status}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "h35-preview-source-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(root.join("architecture")).unwrap();
        fs::write(root.join("index.md"), "# Home\n").unwrap();
        fs::write(root.join("architecture/overview.md"), "# Overview\n").unwrap();
        fs::write(root.join("guide.md"), "# Guide\n").unwrap();
        root
    }

    #[test]
    fn resolves_catalog_paths_and_routes() {
        let root = temp_root();
        assert_eq!(
            resolve_source_file(&root, "architecture/overview.md")
                .unwrap()
                .file_name()
                .unwrap(),
            "overview.md"
        );
        assert!(
            resolve_source_file(&root, "/architecture/overview/")
                .unwrap()
                .ends_with("overview.md")
        );
        assert!(
            resolve_source_file(&root, "/")
                .unwrap()
                .ends_with("index.md")
        );
        assert!(
            resolve_source_file(&root, "guide.md")
                .unwrap()
                .ends_with("guide.md")
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_escape_and_missing_files() {
        let root = temp_root();
        assert!(resolve_source_file(&root, "../secret.md").is_none());
        assert!(resolve_source_file(&root, "/review/").is_none());
        assert!(resolve_source_file(&root, "").is_none());
        let _ = fs::remove_dir_all(&root);
    }
}
