//! Deployment choices are explicit and independent of desktop validation.
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerConfig {
    pub provider: WorkerProvider,
    pub execution: WorkerExecution,
    pub binding: Option<String>,
    pub object_key: Option<String>,
    pub sqlite: Option<WorkerSqlite>,
    /// Optional authored Wrangler entry which imports the generated runtime.
    pub custom_entry: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerProvider {
    Cloudflare,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerExecution {
    Stateless,
    DurableObject,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerSqlite {
    pub path: String,
    pub storage: WorkerSqliteStorage,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkerSqliteStorage {
    Object,
}

impl WorkerConfig {
    pub fn validate(&self) -> Result<()> {
        if let Some(entry) = &self.custom_entry {
            let path = std::path::Path::new(entry);
            if path.as_os_str().is_empty()
                || path
                    .components()
                    .any(|part| !matches!(part, std::path::Component::Normal(_)))
                || entry.starts_with(".rocci/")
            {
                return Err(Error::config(
                    "worker.custom_entry must be an authored relative file under the application root",
                ));
            }
        }
        match self.execution {
            WorkerExecution::Stateless => {
                if self.binding.is_some() || self.object_key.is_some() || self.sqlite.is_some() {
                    return Err(Error::config(
                        "stateless Workers cannot declare an object binding, object_key or SQLite mapping",
                    ));
                }
            }
            WorkerExecution::DurableObject => {
                let binding = self.binding.as_deref().unwrap_or("");
                if binding.is_empty()
                    || !binding
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    || binding.as_bytes()[0].is_ascii_digit()
                    || binding == "ASSETS"
                {
                    return Err(Error::config(
                        "worker.binding must be a JavaScript identifier distinct from ASSETS",
                    ));
                }
                let key = self.object_key.as_deref().unwrap_or("");
                if key.is_empty() || key.len() > 128 || key.chars().any(char::is_control) {
                    return Err(Error::config(
                        "worker.object_key must be an explicit stable singleton name of 1 to 128 bytes",
                    ));
                }
            }
        }
        if let Some(sqlite) = &self.sqlite {
            let mut seen = std::collections::HashSet::new();
            for path in std::iter::once(&sqlite.path).chain(&sqlite.aliases) {
                if path.is_empty()
                    || path.len() > 1024
                    || path.chars().any(char::is_control)
                    || path == ":memory:"
                    || !seen.insert(path)
                {
                    return Err(Error::config(
                        "worker.sqlite paths must be unique, nonempty logical aliases; :memory: is unsupported",
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn sqlite_aliases(&self) -> Vec<String> {
        self.sqlite
            .as_ref()
            .map(|sqlite| {
                std::iter::once(sqlite.path.clone())
                    .chain(sqlite.aliases.iter().cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;

    #[test]
    fn worker_profile_has_no_window_or_listen_requirement() {
        let source = "windows=[]\n[http]\nhost='0.0.0.0'\n[worker]\nprovider='cloudflare'\nexecution='stateless'\n";
        assert!(Config::from_toml_worker(source).is_ok());
        assert!(Config::from_toml(source).is_err());
    }

    #[test]
    fn rejects_inconsistent_identity_and_storage_choices() {
        for source in [
            "[worker]\nprovider='cloudflare'\nexecution='durable-object'\n",
            "[worker]\nprovider='cloudflare'\nexecution='stateless'\nobject_key='tenant'\n",
            "[worker]\nprovider='cloudflare'\nexecution='durable-object'\nbinding='APP'\nobject_key='app'\n[worker.sqlite]\npath=':memory:'\nstorage='object'\n",
            "[worker]\nprovider='cloudflare'\nexecution='stateless'\ntenant_header='X-Tenant'\n",
        ] {
            assert!(Config::from_toml_worker(source).is_err(), "{source}");
        }
    }
}
