use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use flate2::{Compression, write::GzEncoder};
use sha2::{Digest, Sha256};

use crate::build::{BuildOptions, BuildReport, absolute, build_configured_with_options};
use crate::plan::{ArtifactInspect, PublishPage, PublishReport};
use crate::service::IslandRoute;

const PUBLISH_JSON: &str = "publish.json";
const DEFAULT_ARCHIVE: &str = "site.tgz";

#[derive(Debug, Clone)]
pub struct PackageOptions {
    pub host: Option<rocci_roc_host::HostChoice>,
    pub archive: Option<PathBuf>,
    pub write_archive: bool,
}

impl Default for PackageOptions {
    fn default() -> Self {
        Self {
            host: None,
            archive: None,
            write_archive: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PackageManifest {
    pub pages: Vec<PublishPage>,
    pub datastar: bool,
    pub service_origin: String,
    pub service_routes: Vec<IslandRoute>,
    pub artifacts: Vec<ArtifactInspect>,
    pub output_hash: String,
    pub files: Vec<String>,
    pub rocdown_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roc_version: Option<String>,
}

impl PackageManifest {
    fn from_report(report: &BuildReport, files: Vec<String>, output_hash: String) -> Self {
        let publish = report.publish_report();
        Self {
            pages: publish.pages,
            datastar: publish.datastar,
            service_origin: publish.service_origin,
            service_routes: publish.service_routes,
            artifacts: publish.artifacts,
            output_hash,
            files,
            rocdown_version: env!("CARGO_PKG_VERSION").to_string(),
            roc_version: roc_version(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageReport {
    pub build: BuildReport,
    pub output: PathBuf,
    pub publish_json: PathBuf,
    pub archive: Option<PathBuf>,
    pub manifest: PackageManifest,
}

impl PackageReport {
    pub fn render(&self) -> String {
        let mut out = self.build.render_publish();
        out.push_str(&format!("wrote {}\n", self.publish_json.display()));
        if let Some(archive) = &self.archive {
            out.push_str(&format!("wrote {}\n", archive.display()));
        }
        out
    }
}

pub fn package_configured(
    root: &Path,
    output_override: Option<&Path>,
    options: PackageOptions,
) -> Result<PackageReport> {
    let report = build_configured_with_options(
        root,
        output_override,
        BuildOptions {
            host: options.host,
            cdn_only: true,
        },
    )?;
    let output = match output_override {
        Some(output) => absolute(output)?,
        None => {
            let loaded = crate::site::load_site(root)?;
            loaded.root.join(&loaded.config.build.output)
        }
    };
    finish_package(&output, report, options)
}

fn finish_package(
    output: &Path,
    report: BuildReport,
    options: PackageOptions,
) -> Result<PackageReport> {
    let (files, output_hash) = hash_tree(output)?;
    let manifest = PackageManifest::from_report(&report, files, output_hash);
    let publish_json = output.join(PUBLISH_JSON);
    let json =
        serde_json::to_string_pretty(&manifest).context("failed to serialize publish.json")?;
    fs::write(&publish_json, json.as_bytes())
        .with_context(|| format!("failed to write {}", publish_json.display()))?;

    let archive = if options.write_archive {
        let path = archive_path(output, options.archive.as_deref());
        write_archive_atomically(output, &path)?;
        Some(path)
    } else {
        None
    };

    Ok(PackageReport {
        build: report,
        output: output.to_path_buf(),
        publish_json,
        archive,
        manifest,
    })
}

fn archive_path(output: &Path, archive: Option<&Path>) -> PathBuf {
    let archive = archive.unwrap_or_else(|| Path::new(DEFAULT_ARCHIVE));
    if archive.is_absolute() {
        archive.to_path_buf()
    } else {
        output
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(archive)
    }
}

fn write_archive_atomically(dist: &Path, archive: &Path) -> Result<()> {
    let file_name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("site.tgz");
    let tmp = archive
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{file_name}.tmp"));
    if let Err(err) = write_archive(dist, &tmp) {
        let _ = fs::remove_file(&tmp);
        return Err(err);
    }
    if let Some(parent) = archive.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::rename(&tmp, archive).with_context(|| {
        format!(
            "failed to replace {} with staged site archive",
            archive.display()
        )
    })?;
    Ok(())
}

fn write_archive(dist: &Path, archive: &Path) -> Result<()> {
    if let Some(parent) = archive.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let file =
        File::create(archive).with_context(|| format!("failed to create {}", archive.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = tar::Builder::new(encoder);
    append_tree(&mut tar, dist, Path::new(""))?;
    let encoder = tar
        .into_inner()
        .with_context(|| format!("failed to finish tar {}", archive.display()))?;
    encoder
        .finish()
        .with_context(|| format!("failed to finish gzip {}", archive.display()))?;
    Ok(())
}

fn append_tree<W: Write>(tar: &mut tar::Builder<W>, dir: &Path, prefix: &Path) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let rel = prefix.join(&name);
        if path.is_dir() {
            append_tree(tar, &path, &rel)?;
        } else {
            tar.append_path_with_name(&path, &rel)
                .with_context(|| format!("failed to add {} to archive", rel.display()))?;
        }
    }
    Ok(())
}

fn hash_tree(dir: &Path) -> Result<(Vec<String>, String)> {
    let mut files = Vec::new();
    collect_files(dir, Path::new(""), &mut files)?;
    files.sort();
    let mut hasher = Sha256::new();
    for relative in &files {
        hasher.update(relative.as_bytes());
        hasher.update([0u8]);
        hasher.update(&fs::read(dir.join(relative))?);
    }
    Ok((
        files,
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    ))
}

fn collect_files(dir: &Path, prefix: &Path, files: &mut Vec<String>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let rel = prefix.join(entry.file_name());
        if path.is_dir() {
            collect_files(&path, &rel, files)?;
        } else {
            let name = rel.to_string_lossy().replace('\\', "/");
            if name == PUBLISH_JSON || name.ends_with(".tgz") {
                continue;
            }
            files.push(name);
        }
    }
    Ok(())
}

fn roc_version() -> Option<String> {
    for arg in ["version", "--version"] {
        let Ok(output) = Command::new("roc").arg(arg).output() else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().next().unwrap_or(text.as_ref()).trim();
        if !line.is_empty() {
            return Some(line.to_string());
        }
    }
    None
}

pub fn ensure_built_tree(dist: &Path) -> Result<()> {
    if !dist.is_dir() {
        bail!("{} is not a directory", dist.display());
    }
    if !dist.join("index.html").is_file() {
        bail!(
            "`{}` is not a built Rocdown tree (missing index.html); run `rocdown package` first",
            dist.display()
        );
    }
    Ok(())
}

impl BuildReport {
    pub fn publish_report(&self) -> PublishReport {
        PublishReport {
            pages: self.pages.clone(),
            datastar: self.datastar,
            service_origin: self.service_origin.clone(),
            service_routes: self.service_routes.clone(),
            artifacts: self.artifacts.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::tests::{ROC_LOCK, skip_without_roc};
    use crate::build::unique_temp;

    fn write_page(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join(name), body).unwrap();
    }

    #[test]
    fn package_static_fixture_writes_publish_json_without_live_pages() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = unique_temp("pkg-static-src").unwrap();
        write_page(
            &root,
            "index.rocdown",
            "@page { route: \"/\", meta: { title: \"Home\" } }\n\n# Home\n",
        );
        let parent = unique_temp("pkg-static-parent").unwrap();
        let output = parent.join("dist");
        let archive_path = parent.join("site.tgz");
        let report = package_configured(
            &root,
            Some(&output),
            PackageOptions {
                host: None,
                archive: Some(archive_path.clone()),
                write_archive: true,
            },
        )
        .unwrap();
        assert!(!report.manifest.datastar);
        assert!(report.manifest.service_origin.is_empty());
        assert!(report.manifest.service_routes.is_empty());
        assert!(
            report
                .manifest
                .pages
                .iter()
                .all(|page| page.kind == crate::article::PageKind::Static)
        );
        assert!(report.manifest.pages.iter().all(|page| !page.datastar));
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&report.publish_json).unwrap()).unwrap();
        assert_eq!(json["datastar"], false);
        assert!(json["service_origin"].as_str().unwrap().is_empty());
        assert!(
            json["files"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| { item.as_str() == Some("index.html") })
        );
        assert!(!json["output_hash"].as_str().unwrap().is_empty());
        assert_eq!(json["rocdown_version"], env!("CARGO_PKG_VERSION"));

        let archive = report.archive.expect("archive");
        assert!(archive.is_file());
        let names = archive_names(&archive);
        assert!(
            names
                .iter()
                .any(|name| name == "index.html" || name == "./index.html")
        );
        assert!(names.iter().any(|name| name.ends_with("publish.json")));

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn package_live_fixture_fails_with_rd2302_and_keeps_archive() {
        let root = unique_temp("pkg-live-src").unwrap();
        write_page(
            &root,
            "index.rocdown",
            "@page { route: \"/\", meta: { title: \"Live\" } }\n\n@on:post(\"/actions/x\") = |_| {\n    Html.text(\"x\")\n}\n\n# Live\n",
        );
        let parent = unique_temp("pkg-live-parent").unwrap();
        let output = parent.join("dist");
        fs::create_dir_all(&output).unwrap();
        let archive = parent.join("site.tgz");
        fs::write(&archive, b"previous-archive").unwrap();
        let err = package_configured(
            &root,
            Some(&output),
            PackageOptions {
                host: None,
                archive: Some(archive.clone()),
                write_archive: true,
            },
        )
        .unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("RD2302"), "{message}");
        assert_eq!(fs::read(&archive).unwrap(), b"previous-archive");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&parent);
    }

    fn archive_names(path: &Path) -> Vec<String> {
        let file = File::open(path).unwrap();
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);
        archive
            .entries()
            .unwrap()
            .map(|entry| {
                entry
                    .unwrap()
                    .path()
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }
}
