use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rocci_cli::driver::{self, DriverOptions, GenericAppPlan, GenericModule};
use rocci_cli::serve::PortArg;
use rocci_template::{RouteInfo, SourceFile, type_name_from_path};
use serde::Serialize;

use crate::article::PageKind;
use crate::catalog::ResolvedSite;
use crate::site::{LoadedSite, load_site, resolve_loaded};
use crate::standalone::StandaloneModule;
use crate::{CompileOptions, compile};

const ACTION_METHODS: &[&str] = &["get", "post", "put", "patch", "delete"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IslandRoute {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct IslandServicePlan {
    pub primary_name: String,
    pub modules: Vec<StandaloneModule>,
    pub redirect_trailing_slash: bool,
}

impl IslandServicePlan {
    pub fn into_app_plan(self) -> GenericAppPlan {
        GenericAppPlan {
            primary_name: self.primary_name,
            modules: self
                .modules
                .into_iter()
                .map(|module| GenericModule {
                    type_name: module.type_name,
                    roc: module.roc,
                    state_type: module.state_type,
                    init: module.init,
                    routes: module.routes,
                    mapped: module.mapped,
                    local_assets: module.local_assets,
                })
                .collect(),
            redirect_trailing_slash: self.redirect_trailing_slash,
        }
    }
}

pub fn live_csp(service_origin: &str) -> String {
    let origin = service_origin.trim().trim_end_matches('/');
    let connect = if origin.is_empty() {
        "'self'".to_string()
    } else {
        origin.to_string()
    };
    format!(
        "default-src 'none'; script-src 'self' 'unsafe-eval'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src {connect}; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'"
    )
}

pub fn prefix_action_urls(html: &str, service_origin: &str) -> String {
    let origin = service_origin.trim().trim_end_matches('/');
    if origin.is_empty() {
        return html.to_string();
    }
    let mut out = html.to_string();
    for method in ACTION_METHODS {
        for quote in ["'", "\"", "&#39;", "&quot;"] {
            let needle = format!("@{method}({quote}/");
            let replacement = format!("@{method}({quote}{origin}/");
            out = out.replace(&needle, &replacement);
        }
    }
    out
}

pub fn plan_island_service(root: &Path) -> Result<IslandServicePlan> {
    match generated_island_plan(root)? {
        Some(plan) => Ok(plan),
        None => {
            let loaded = load_site(root)?;
            if !loaded.config.http.service.is_empty() {
                bail!(
                    "site uses [http].service `{}`; run that .rocci app instead of a generated island service",
                    loaded.config.http.service
                );
            }
            bail!("no live pages to serve; add `@on` handlers or configure [http].service")
        }
    }
}

pub fn generated_island_plan(root: &Path) -> Result<Option<IslandServicePlan>> {
    let loaded = load_site(root)?;
    if !loaded.config.http.service.is_empty() {
        return Ok(None);
    }
    let result = resolve_loaded(&loaded);
    if result.has_errors() {
        bail!("{}", result.error_summary());
    }
    if !result
        .site
        .pages
        .iter()
        .any(|page| !page.draft && page.kind == PageKind::Live)
    {
        return Ok(None);
    }
    Ok(Some(plan_island_service_from(&loaded, &result.site)?))
}

pub fn island_routes(root: &Path, site: &ResolvedSite) -> Result<Vec<IslandRoute>> {
    let modules = compile_live_modules(root, site)?;
    let mut routes: Vec<IslandRoute> = modules
        .into_iter()
        .flat_map(|module| module.routes)
        .map(|route| IslandRoute {
            method: route.method,
            path: route.path,
        })
        .collect();
    routes.sort_by(|left, right| {
        left.method
            .cmp(&right.method)
            .then_with(|| left.path.cmp(&right.path))
    });
    routes.dedup();
    Ok(routes)
}

pub fn plan_island_service_from(
    loaded: &LoadedSite,
    site: &ResolvedSite,
) -> Result<IslandServicePlan> {
    if !loaded.config.http.service.is_empty() {
        bail!(
            "site uses [http].service `{}`; run that .rocci app instead of a generated island service",
            loaded.config.http.service
        );
    }

    let mut modules = compile_live_modules(&loaded.root, site)?;
    if modules.is_empty() {
        bail!("no live pages to serve; add `@on` handlers or configure [http].service");
    }

    let mut seen = HashSet::new();
    for module in &modules {
        for route in &module.routes {
            if !seen.insert((route.method.clone(), route.path.clone())) {
                bail!(
                    "duplicate island route {} {} from {}",
                    route.method,
                    route.path,
                    module.type_name
                );
            }
        }
    }

    let context_modules: Vec<_> = modules
        .iter()
        .filter(|module| module.state_type.is_some() || module.init.is_some())
        .map(|module| module.type_name.as_str())
        .collect();
    if context_modules.len() > 1 {
        bail!(
            "live pages declare multiple `@context` / `@init` modules ({}); v1 allows one",
            context_modules.join(", ")
        );
    }
    if let Some(primary) = context_modules.first()
        && let Some(index) = modules
            .iter()
            .position(|module| module.type_name == *primary)
    {
        modules.swap(0, index);
    }

    let has_mutation = modules.iter().any(|module| {
        module
            .routes
            .iter()
            .any(|route| route.method != "GET" && route.method != "HEAD")
    });
    if !has_mutation {
        bail!(
            "live pages have no mutation `@on` handlers; add `@on:post` (or friends) or configure [http].service"
        );
    }

    Ok(IslandServicePlan {
        primary_name: modules[0].type_name.clone(),
        modules,
        redirect_trailing_slash: true,
    })
}

pub fn serve_islands(root: &Path, no_window: bool, port: PortArg) -> Result<()> {
    let loaded = load_site(root)?;
    if !loaded.config.http.service.is_empty() {
        let service = loaded.root.join(&loaded.config.http.service);
        if !service.is_file() {
            bail!(
                "configured [http].service `{}` does not exist",
                loaded.config.http.service
            );
        }
        return rocci_cli::run::run(&service, &[], no_window, port, true);
    }

    let plan = plan_island_service(&loaded.root)?;
    let title = loaded.config.site.title.clone();
    let src_dir = loaded.root.clone();
    let app = plan.into_app_plan();
    let options = DriverOptions {
        args: Vec::new(),
        no_window,
        live_reload: true,
        port,
        db_path: None,
        title,
        preview_path: Some("/health".to_string()),
        profile: rocci_cli::profile::SpanRecorder::new().finish(),
        inspect_pages: Vec::new(),
        state_key: Some("rocdown-islands".to_string()),
    };
    driver::execute_app_plan(&app, &src_dir, &options)
}

fn compile_live_modules(root: &Path, site: &ResolvedSite) -> Result<Vec<StandaloneModule>> {
    let mut live: Vec<_> = site
        .pages
        .iter()
        .filter(|page| !page.draft && page.kind == PageKind::Live)
        .collect();
    live.sort_by(|a, b| a.source_path.cmp(&b.source_path));
    if live.is_empty() {
        return Ok(Vec::new());
    }

    let page_paths = site_page_paths(site);
    let mut modules = Vec::new();
    for page in &live {
        let path = root.join(&page.source_path);
        let src = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let source_name = page.source_path.as_str();
        let compiled = compile(
            SourceFile::new(source_name, &src),
            &CompileOptions {
                check_assets: false,
                ..CompileOptions::default()
            },
        );
        if compiled.has_errors() {
            let messages: Vec<_> = compiled
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.is_error())
                .map(|diagnostic| diagnostic.message.as_str())
                .collect();
            bail!(
                "failed to compile island service from {}: {}",
                page.source_path,
                messages.join("; ")
            );
        }
        let routes: Vec<RouteInfo> = compiled
            .routes
            .into_iter()
            .filter(|route| keep_island_route(route, &page_paths))
            .collect();
        let type_name = type_name_from_path(&path);
        modules.push(StandaloneModule {
            type_name: type_name.clone(),
            roc: compiled.roc.clone(),
            state_type: compiled.state_type,
            init: compiled.init,
            routes,
            mapped: rocci_template::MappedModule {
                type_name,
                generated: compiled.roc,
                source_name: source_name.to_string(),
                source_src: src,
                segments: compiled.segments,
            },
            local_assets: Vec::new(),
        });
    }
    Ok(modules)
}

fn site_page_paths(site: &ResolvedSite) -> HashSet<String> {
    let mut paths = HashSet::new();
    for page in &site.pages {
        paths.insert(crate::catalog::with_trailing_slash(&page.route));
        for alias in &page.aliases {
            paths.insert(crate::catalog::with_trailing_slash(alias));
        }
    }
    paths
}

fn keep_island_route(route: &RouteInfo, page_paths: &HashSet<String>) -> bool {
    if route.method != "GET" {
        return true;
    }
    let normalized = crate::catalog::with_trailing_slash(&route.path);
    normalized != "/" && !page_paths.contains(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    fn temp(name: &str) -> std::path::PathBuf {
        let path = env::temp_dir().join(format!("rocdown-service-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn live_csp_uses_self_without_origin() {
        let csp = live_csp("");
        assert!(csp.contains("script-src 'self' 'unsafe-eval'"), "{csp}");
        assert!(csp.contains("connect-src 'self'"), "{csp}");
        assert!(!csp.contains("script-src 'none'"), "{csp}");
    }

    #[test]
    fn live_csp_names_service_origin() {
        let csp = live_csp("https://islands.example.com/");
        assert!(
            csp.contains("connect-src https://islands.example.com"),
            "{csp}"
        );
        assert!(!csp.contains("connect-src 'none'"), "{csp}");
    }

    #[test]
    fn prefix_action_urls_rewrites_relative_posts() {
        let html = r#"<button data-on:click="@post('/actions/reveal/show')">Go</button>"#;
        let out = prefix_action_urls(html, "https://islands.example.com/");
        assert!(
            out.contains("@post('https://islands.example.com/actions/reveal/show')"),
            "{out}"
        );
        assert_eq!(prefix_action_urls(html, ""), html);
    }

    #[test]
    fn prefix_action_urls_rewrites_escaped_quotes() {
        let html = r#"data-on:click="@post(&#39;/actions/x&#39;)""#;
        let out = prefix_action_urls(html, "http://127.0.0.1:9000");
        assert!(
            out.contains("@post(&#39;http://127.0.0.1:9000/actions/x&#39;)"),
            "{out}"
        );
    }

    #[test]
    fn keep_island_route_drops_cdn_gets_and_keeps_actions() {
        let mut pages = HashSet::new();
        pages.insert("/".into());
        pages.insert("/guides/docs-components/".into());
        pages.insert("/about/".into());
        let get = |path: &str| RouteInfo {
            method: "GET".into(),
            path: path.into(),
            fn_name: "on_get".into(),
            span: rocci_template::Span::new(0, 0),
        };
        let post = |path: &str| RouteInfo {
            method: "POST".into(),
            path: path.into(),
            fn_name: "on_post".into(),
            span: rocci_template::Span::new(0, 0),
        };
        assert!(!keep_island_route(&get("/"), &pages));
        assert!(!keep_island_route(&get("/guides/docs-components"), &pages));
        assert!(!keep_island_route(&get("/guides/docs-components/"), &pages));
        assert!(!keep_island_route(&get("/about"), &pages));
        assert!(keep_island_route(&get("/health"), &pages));
        assert!(keep_island_route(
            &post("/actions/counter/increment"),
            &pages
        ));
        assert!(keep_island_route(&post("/"), &pages));
    }

    #[test]
    fn plan_keeps_mutation_routes_and_drops_page_gets() {
        let root = temp("routes");
        fs::write(
            root.join("index.rocdown"),
            r#"
@page { route: "/", meta: { title: "Live" } }

@component
RevealTip = |{ open }| {
    <div id="reveal-tip">
        <button type="button" data-on:click=@post("/actions/reveal/show")>Show tip</button>
    </div>
}

@on:post("/actions/reveal/show") = |_| {
    revealTip({ open: True })
}

@on:post("/actions/reveal/hide") = |_| {
    revealTip({ open: False })
}

# Live

@render {
    revealTip({ open: False })
}
"#,
        )
        .unwrap();
        let plan = plan_island_service(&root).unwrap();
        let routes =
            island_routes(&root, &resolve_loaded(&load_site(&root).unwrap()).site).unwrap();
        assert!(
            routes
                .iter()
                .any(|route| route.method == "POST" && route.path == "/actions/reveal/show"),
            "{routes:?}"
        );
        assert_eq!(plan.modules.len(), 1);
        let paths: Vec<_> = plan.modules[0]
            .routes
            .iter()
            .map(|route| (route.method.as_str(), route.path.as_str()))
            .collect();
        assert!(
            paths.contains(&("POST", "/actions/reveal/show")),
            "{paths:?}"
        );
        assert!(
            paths.contains(&("POST", "/actions/reveal/hide")),
            "{paths:?}"
        );
        assert!(
            !paths
                .iter()
                .any(|(method, path)| *method == "GET" && *path == "/")
        );
        let main = plan.into_app_plan().main_roc();
        assert!(
            main.contains("(\"POST\", \"/actions/reveal/show\")"),
            "{main}"
        );
        assert!(!main.contains("(\"GET\", \"/\") =>"), "{main}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plan_drops_preview_as_site_gets() {
        let root = temp("preview-gets");
        fs::create_dir_all(root.join("guides")).unwrap();
        fs::write(
            root.join("index.rocdown"),
            r#"
@page { route: "/", meta: { title: "Live" } }

@on:post("/actions/counter/increment") = |_| {
    Html.text("1")
}

# Home
"#,
        )
        .unwrap();
        fs::write(
            root.join("guides/docs-components.rocdown"),
            "# Components\n",
        )
        .unwrap();
        fs::write(
            root.join("live.rocdown"),
            r#"
@page { route: "/live", meta: { title: "Also live" } }

@on:post("/actions/live/ping") = |_| {
    Html.text("ok")
}

# Live
"#,
        )
        .unwrap();
        let plan = plan_island_service(&root).unwrap();
        let paths: Vec<_> = plan
            .modules
            .iter()
            .flat_map(|module| module.routes.iter())
            .map(|route| (route.method.as_str(), route.path.as_str()))
            .collect();
        assert!(
            paths.contains(&("POST", "/actions/counter/increment")),
            "{paths:?}"
        );
        assert!(paths.contains(&("POST", "/actions/live/ping")), "{paths:?}");
        assert!(
            !paths.iter().any(|(method, path)| *method == "GET"
                && (*path == "/"
                    || *path == "/live"
                    || *path == "/live/"
                    || *path == "/guides/docs-components/")),
            "{paths:?}"
        );
        let main = plan.into_app_plan().main_roc();
        assert!(!main.contains("(\"GET\", \"/\") =>"), "{main}");
        assert!(!main.contains("(\"GET\", \"/live\") =>"), "{main}");
        assert!(!main.contains("(\"GET\", \"/live/\") =>"), "{main}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn generated_plan_skips_static_and_sibling_service_sites() {
        let root = temp("static-plan");
        fs::write(
            root.join("index.rocdown"),
            r#"
@page { route: "/", meta: { title: "Static" } }

# Hello
"#,
        )
        .unwrap();
        assert!(generated_island_plan(&root).unwrap().is_none());
        let _ = fs::remove_dir_all(&root);

        let sibling = temp("sibling-plan");
        fs::write(
            sibling.join("rocdown.toml"),
            "[site]\ntitle = \"Sibling\"\n[http]\nservice = \"Islands.rocci\"\n",
        )
        .unwrap();
        fs::write(
            sibling.join("index.rocdown"),
            r#"
@page { route: "/", meta: { title: "Live" } }

# Hello
"#,
        )
        .unwrap();
        fs::write(
            sibling.join("Islands.rocci"),
            "@on:post(\"/x\") = |_| <p>x</p>\n",
        )
        .unwrap();
        assert!(generated_island_plan(&sibling).unwrap().is_none());
        let err = plan_island_service(&sibling).unwrap_err().to_string();
        assert!(err.contains("[http].service"), "{err}");
        let _ = fs::remove_dir_all(sibling);
    }
}
