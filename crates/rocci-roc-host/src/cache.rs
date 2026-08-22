use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

use crate::fingerprint::InputFingerprint;
use crate::manifest::{Manifest, write_atomic_manifest};

#[derive(Debug, Clone)]
pub struct CachedRoc {
    pub root: PathBuf,
    pub modules_dir: PathBuf,
    pub maps_dir: PathBuf,
    pub manifest: Manifest,
}

pub struct TwoTierCache {
    pub root: PathBuf,
}

impl TwoTierCache {
    pub fn default_dir() -> PathBuf {
        if let Ok(cache_env) = env::var("ROCCI_CACHE")
            && !cache_env.trim().is_empty()
        {
            return PathBuf::from(cache_env.trim());
        }
        if let Ok(home) = env::var("HOME")
            && !home.trim().is_empty()
        {
            return PathBuf::from(home).join(".rocci/cache");
        }
        env::temp_dir().join("rocci-cache")
    }

    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn roc_dir(&self, gen_hash: &str) -> PathBuf {
        self.root.join("roc").join(gen_hash)
    }

    pub fn renderer_dir(&self, compile_hash: &str) -> PathBuf {
        self.root.join("renderers").join(compile_hash)
    }
}

impl Default for TwoTierCache {
    fn default() -> Self {
        Self::new(Self::default_dir())
    }
}

impl TwoTierCache {
    pub fn lookup_roc(&self, gen_hash: &str) -> Option<CachedRoc> {
        let dir = self.roc_dir(gen_hash);
        let manifest_path = dir.join("manifest.json");
        if !manifest_path.is_file() {
            return None;
        }
        let manifest_str = fs::read_to_string(&manifest_path).ok()?;
        let mut manifest: Manifest = serde_json::from_str(&manifest_str).ok()?;
        manifest.touch();
        let _ = write_atomic_manifest(&dir, &manifest);
        Some(CachedRoc {
            modules_dir: dir.join("modules"),
            maps_dir: dir.join("maps"),
            root: dir,
            manifest,
        })
    }

    pub fn store_roc(
        &self,
        gen_hash: &str,
        modules: &[(&str, &str)],
        maps: &[(&str, &str)],
        fingerprints: &[InputFingerprint],
    ) -> Result<PathBuf> {
        let dir = self.roc_dir(gen_hash);
        let tmp_dir = self
            .root
            .join("roc")
            .join(format!("{gen_hash}.tmp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(tmp_dir.join("modules"))?;
        fs::create_dir_all(tmp_dir.join("maps"))?;

        for (name, content) in modules {
            fs::write(tmp_dir.join("modules").join(name), content)?;
        }
        for (name, content) in maps {
            fs::write(tmp_dir.join("maps").join(name), content)?;
        }
        let fp_json = serde_json::to_string_pretty(fingerprints)?;
        fs::write(tmp_dir.join("fingerprints.json"), fp_json)?;

        let manifest = Manifest::new(None, None);
        write_atomic_manifest(&tmp_dir, &manifest)?;

        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        if let Some(parent) = dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&tmp_dir, &dir)?;
        Ok(dir)
    }

    pub fn lookup_renderer(&self, compile_hash: &str, target: &str) -> Option<PathBuf> {
        let dir = self.renderer_dir(compile_hash);
        let manifest_path = dir.join("manifest.json");
        if !manifest_path.is_file() {
            return None;
        }
        let manifest_str = fs::read_to_string(&manifest_path).ok()?;
        let mut manifest: Manifest = serde_json::from_str(&manifest_str).ok()?;
        let expected_name = if target == "wasm32" {
            "components.wasm"
        } else {
            "apply"
        };
        let artifact_path = dir.join(expected_name);
        if !artifact_path.is_file() {
            let _ = fs::remove_dir_all(&dir);
            return None;
        }

        let Ok(bytes) = fs::read(&artifact_path) else {
            let _ = fs::remove_dir_all(&dir);
            return None;
        };
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_sha256 = hex_sha256(hasher);

        let sha_file = dir.join("artifact.sha256");
        let stored_sha = fs::read_to_string(&sha_file)
            .ok()
            .map(|s| s.trim().to_string());
        if stored_sha.as_deref() != Some(&actual_sha256)
            || manifest.artifact_sha256.as_deref() != Some(&actual_sha256)
        {
            let _ = fs::remove_dir_all(&dir);
            return None;
        }

        manifest.touch();
        let _ = write_atomic_manifest(&dir, &manifest);
        Some(artifact_path)
    }

    pub fn store_renderer(
        &self,
        compile_hash: &str,
        target: &str,
        artifact_bytes: &[u8],
        fingerprints: &[InputFingerprint],
    ) -> Result<PathBuf> {
        let dir = self.renderer_dir(compile_hash);
        let tmp_dir = self
            .root
            .join("renderers")
            .join(format!("{compile_hash}.tmp-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir)?;

        let artifact_name = if target == "wasm32" {
            "components.wasm"
        } else {
            "apply"
        };
        let artifact_path = tmp_dir.join(artifact_name);
        fs::write(&artifact_path, artifact_bytes)?;

        #[cfg(unix)]
        if target != "wasm32" {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&artifact_path, fs::Permissions::from_mode(0o755));
        }

        let mut hasher = Sha256::new();
        hasher.update(artifact_bytes);
        let sha256 = hex_sha256(hasher);

        fs::write(tmp_dir.join("artifact.sha256"), format!("{sha256}\n"))?;
        let fp_json = serde_json::to_string_pretty(fingerprints)?;
        fs::write(tmp_dir.join("fingerprints.json"), fp_json)?;

        let manifest = Manifest::new(Some(sha256), Some(target.to_string()));
        write_atomic_manifest(&tmp_dir, &manifest)?;

        if dir.exists() {
            let _ = fs::remove_dir_all(&dir);
        }
        if let Some(parent) = dir.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&tmp_dir, &dir)?;
        Ok(dir.join(artifact_name))
    }
}

pub fn compute_gen_hash(
    template_version: &str,
    lower_options: &str,
    modules: &[(&str, &[u8])],
    headers: &[(&str, &[u8])],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(template_version.as_bytes());
    hasher.update(b"\n");
    hasher.update(lower_options.as_bytes());
    hasher.update(b"\n");
    let mut sorted_modules = modules.to_vec();
    sorted_modules.sort_by_key(|(name, _)| *name);
    for (name, bytes) in sorted_modules {
        hasher.update(name.as_bytes());
        hasher.update(b":");
        hasher.update(bytes);
        hasher.update(b"\n");
    }
    let mut sorted_headers = headers.to_vec();
    sorted_headers.sort_by_key(|(name, _)| *name);
    for (name, bytes) in sorted_headers {
        hasher.update(name.as_bytes());
        hasher.update(b":");
        hasher.update(bytes);
        hasher.update(b"\n");
    }
    hex_sha256(hasher)
}

pub fn compute_compile_hash(
    gen_hash: &str,
    roc_version: &str,
    target: &str,
    opt_level: &str,
    platform_id: &str,
    host_version: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(gen_hash.as_bytes());
    hasher.update(b"\n");
    hasher.update(roc_version.as_bytes());
    hasher.update(b"\n");
    hasher.update(target.as_bytes());
    hasher.update(b"\n");
    hasher.update(opt_level.as_bytes());
    hasher.update(b"\n");
    hasher.update(platform_id.as_bytes());
    hasher.update(b"\n");
    hasher.update(host_version.as_bytes());
    hex_sha256(hasher)
}

fn hex_sha256(hasher: Sha256) -> String {
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
