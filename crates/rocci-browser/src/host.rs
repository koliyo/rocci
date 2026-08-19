use crate::{
    Error, Result,
    client::{AdapterClient, documents_reason},
    discovery::{PluginSpec, discover_plugins, resolve_bin},
    paths::Paths,
    picker::rank_targets,
    protocol::{Document, OpenParams, OpenResult},
    registry::Registry,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub id: String,
    pub path: String,
    pub adapter_id: String,
    pub label: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OpenRequest<'a> {
    pub query: &'a str,
    pub document: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opened {
    pub url: String,
    pub title: String,
    pub inspector_url: Option<String>,
    pub target: Target,
    pub document: Option<String>,
}

pub struct Host {
    pub paths: Paths,
    pub warnings: Vec<String>,
    adapters: Vec<AdapterClient>,
}

impl Host {
    pub fn connect(paths: Paths) -> Result<Self> {
        let (specs, mut warnings) = discover_plugins(&paths)?;
        let mut adapters = Vec::new();
        for spec in specs {
            match spawn_plugin(spec, &mut warnings) {
                Some(client) => adapters.push(client),
                None => {}
            }
        }
        Ok(Self {
            paths,
            warnings,
            adapters,
        })
    }

    pub fn registry(&self) -> Result<Registry> {
        Registry::load(&self.paths)
    }

    pub fn add_project(&self, id: String, path: String) -> Result<Registry> {
        let mut registry = Registry::load(&self.paths)?;
        registry.add(id, path);
        registry.save(&self.paths)?;
        Ok(registry)
    }

    pub fn remove_project(&self, query: &str) -> Result<bool> {
        let mut registry = Registry::load(&self.paths)?;
        let removed = registry.remove(query);
        registry.save(&self.paths)?;
        Ok(removed)
    }

    pub fn probe_targets(&mut self) -> Result<Vec<Target>> {
        let registry = Registry::load(&self.paths)?;
        let mut targets = Vec::new();
        for project in registry.projects {
            let mut claimed = false;
            for adapter in &mut self.adapters {
                match adapter.probe(&project.path) {
                    Ok(result) if result.claimed => {
                        claimed = true;
                        targets.push(Target {
                            id: project.id.clone(),
                            path: project.path.clone(),
                            adapter_id: adapter.adapter_id().to_string(),
                            label: result.label.unwrap_or_else(|| adapter.adapter_id().into()),
                            detail: result.detail,
                        });
                    }
                    Ok(_) => {}
                    Err(error) => self
                        .warnings
                        .push(format!("{}: {error}", adapter.adapter_id())),
                }
            }
            if !claimed {
                self.warnings
                    .push(format!("no adapter claimed {}", project.path));
            }
        }
        Ok(targets)
    }

    pub fn list_documents(&mut self, adapter_id: &str, root: &str) -> Result<Vec<Document>> {
        let adapter = adapter_mut(&mut self.adapters, adapter_id)?;
        Ok(adapter.list_documents(root)?.documents)
    }

    pub fn open_target(
        &mut self,
        adapter_id: &str,
        root: &str,
        document: Option<&str>,
    ) -> Result<Opened> {
        let adapter = adapter_mut(&mut self.adapters, adapter_id)?;
        let result: OpenResult = adapter.open(OpenParams {
            root: root.to_string(),
            document: document.map(str::to_string),
            port: None,
        })?;
        let target = Target {
            id: Default::default(),
            path: root.to_string(),
            adapter_id: adapter.adapter_id().to_string(),
            label: result.title.clone(),
            detail: None,
        };
        Ok(Opened {
            url: result.url,
            title: result.title,
            inspector_url: result.inspector_url,
            target,
            document: document.map(str::to_string),
        })
    }

    pub fn open(&mut self, request: OpenRequest<'_>) -> Result<Opened> {
        let targets = self.probe_targets()?;
        let ranked = rank_targets(&targets, request.query);
        let Some((_, target)) = ranked.first() else {
            return Err(Error::message(format!(
                "no target matched '{}'",
                request.query
            )));
        };
        let target = (*target).clone();
        self.open_target(&target.adapter_id, &target.path, request.document)
            .map(|mut opened| {
                opened.target = target;
                opened
            })
    }

    pub fn documents_or_reason(
        &mut self,
        adapter_id: &str,
        root: &str,
    ) -> Result<(Vec<Document>, Option<String>)> {
        let documents = self.list_documents(adapter_id, root)?;
        let reason = documents_reason(&documents);
        Ok((documents, reason))
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        for adapter in &mut self.adapters {
            let _ = adapter.shutdown();
        }
    }
}

fn spawn_plugin(spec: PluginSpec, warnings: &mut Vec<String>) -> Option<AdapterClient> {
    let Some(bin) = resolve_bin(&spec.bin) else {
        warnings.push(format!("missing plugin binary {} ({})", spec.id, spec.bin));
        return None;
    };
    match AdapterClient::spawn(spec.clone(), &bin) {
        Ok(client) => {
            if client.initialize.protocol_version != crate::PROTOCOL_VERSION {
                warnings.push(format!(
                    "plugin {} protocol {}",
                    spec.id, client.initialize.protocol_version
                ));
            }
            Some(client)
        }
        Err(error) => {
            warnings.push(format!("plugin {}: {error}", spec.id));
            None
        }
    }
}

fn adapter_mut<'a>(
    adapters: &'a mut [AdapterClient],
    adapter_id: &str,
) -> Result<&'a mut AdapterClient> {
    adapters
        .iter_mut()
        .find(|adapter| adapter.adapter_id() == adapter_id || adapter.spec.id == adapter_id)
        .ok_or_else(|| Error::message(format!("adapter {adapter_id} is not connected")))
}
