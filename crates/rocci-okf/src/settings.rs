//! One-shot settings surface for `okf.toml` (`/settings/`).

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, bail};

use crate::config::{
    DirectoryRoot, GitRoot, Incoming, OkfUserConfig, PollSetting, RootConfig, config_path, load,
    save, to_toml, validate_config,
};
use crate::edges::edge_allowed;
use crate::git_root::{git_root_dir, now_unix, read_meta, sync_git_root};
use crate::resolve::{SyncMode, okf_cache_dir, resolve_all_with};

pub fn http(method: &str, path: &str, raw: &[u8]) -> Option<String> {
    if path == "/settings" || path == "/settings/" {
        if method == "GET" {
            return Some(render_article(None));
        }
        return None;
    }
    if path != "/__rocci_okf/settings" {
        return None;
    }
    if method != "POST" {
        return None;
    }
    let body = form_body(raw);
    let fields = parse_form(&body);
    let article = match apply_action(&fields) {
        Ok(message) => render_article(Some(&message)),
        Err(error) => render_article(Some(&error.to_string())),
    };
    Some(article)
}

pub fn render_article(message: Option<&str>) -> String {
    let config = load().unwrap_or_default();
    render_article_from(&config, message)
}

fn render_article_from(config: &OkfUserConfig, message: Option<&str>) -> String {
    let resolved = resolve_all_with(config, &okf_cache_dir(), now_unix(), SyncMode::Never);
    let mut out = String::from("<div class=\"okf-settings\" id=\"okf-settings\">\n");
    out.push_str("<h1 class=\"rd-header-1\">Knowledge roots</h1>\n");
    out.push_str(
        "<p class=\"rd-paragraph\">A knowledge root is a local OKF bundle <code>rocci-okf</code> can check, search, and preview. Agents list resolved folders with <code>rocci-okf roots</code>.</p>\n",
    );
    if let Some(path) = config_path() {
        out.push_str(&format!(
            "<p class=\"rd-paragraph okf-settings-help\">Saved in <code>{}</code>.</p>\n",
            escape(&path.display().to_string())
        ));
    }
    if let Some(message) = message.filter(|m| !m.is_empty()) {
        out.push_str(&format!(
            "<p class=\"okf-settings-msg\">{}</p>\n",
            escape(message)
        ));
    }
    out.push_str(&format!(
        "<p class=\"rd-paragraph okf-settings-help\">Default git poll interval: <code>{}</code> (examples: <code>5m</code>, <code>off</code>).</p>\n",
        escape(&config.poll.as_form_value())
    ));
    out.push_str(&render_root_list(config, &resolved));
    out.push_str(&render_matrix(config));
    out.push_str(&render_add_forms());
    out.push_str(SETTINGS_PAGE_SCRIPT);
    out.push_str("</div>\n");
    out
}

fn render_root_list(config: &OkfUserConfig, resolved: &[crate::git_root::ResolvedRoot]) -> String {
    let mut out = String::from("<h2 class=\"rd-header-2\">Configured roots</h2>\n");
    if config.roots.is_empty() {
        out.push_str(
            "<div class=\"okf-settings-empty\"><p class=\"rd-paragraph\">No roots yet. Add a folder on this computer (usually named <code>knowledge</code>, with <code>index.md</code>) or a git repository below.</p><p class=\"okf-settings-help\">Example: <code>~/Projects/rocci/knowledge</code></p></div>\n",
        );
        return out;
    }
    out.push_str("<div class=\"okf-settings-cards\">\n");
    for root in &config.roots {
        let status = resolved.iter().find(|item| item.id == root.id());
        out.push_str(&render_root_card(root, status));
    }
    out.push_str("</div>\n");
    out
}

fn render_root_card(root: &RootConfig, status: Option<&crate::git_root::ResolvedRoot>) -> String {
    let mut out = String::from("<article class=\"okf-settings-card\">\n");
    out.push_str(&format!(
        "<header class=\"okf-settings-card-head\"><h3 class=\"rd-header-3\"><code>{}</code></h3><span class=\"okf-action-pill\">{}</span></header>\n",
        escape(root.id()),
        escape(match root {
            RootConfig::Directory(_) => "folder",
            RootConfig::Git(_) => "git",
        })
    ));
    match root {
        RootConfig::Directory(dir) => {
            out.push_str(&format!(
                "<p class=\"rd-paragraph\">Configured path: <code>{}</code></p>\n",
                escape(&dir.path)
            ));
            if let Some(warning) = index_md_warning(dir) {
                out.push_str(&format!(
                    "<p class=\"okf-settings-warn\">{}</p>\n",
                    escape(&warning)
                ));
            }
        }
        RootConfig::Git(git) => {
            out.push_str(&format!(
                "<p class=\"rd-paragraph\">Remote: <code>{}</code></p>\n",
                escape(&git.url)
            ));
        }
    }
    if let Some(status) = status {
        if let Some(path) = &status.path {
            out.push_str(&format!(
                "<p class=\"rd-paragraph\">Resolved locally: <code>{}</code></p>\n",
                escape(&path.display().to_string())
            ));
        }
        if status.enabled() {
            out.push_str("<p class=\"okf-settings-help\">Status: available</p>\n");
        } else {
            out.push_str("<p class=\"okf-settings-warn\">Status: missing or not a directory</p>\n");
        }
        if let Some(revision) = &status.revision {
            out.push_str(&format!(
                "<p class=\"okf-settings-help\">Revision: <code>{}</code></p>\n",
                escape(revision)
            ));
        }
        if let Some(error) = &status.error {
            out.push_str(&format!(
                "<p class=\"okf-settings-warn\">{}</p>\n",
                escape(error)
            ));
        }
    }
    if let RootConfig::Git(git) = root {
        let meta = read_meta(&git_root_dir(&okf_cache_dir(), &git.id).join("meta.toml"));
        out.push_str(&format!(
            "<p class=\"okf-settings-help\">Last fetch: {}</p>\n",
            escape(&format_last_fetch(
                meta.as_ref().and_then(|meta| meta.last_fetch_unix)
            ))
        ));
        if let Some(error) = meta.as_ref().and_then(|meta| meta.last_error.as_deref()) {
            out.push_str(&format!(
                "<p class=\"okf-settings-warn\">Last sync error: {}</p>\n",
                escape(error)
            ));
        }
    }
    out.push_str("<p class=\"okf-settings-help\">Incoming citations: who may use <code>okf:");
    out.push_str(&escape(root.id()));
    out.push_str("/…</code> links into this root.</p>\n");
    out.push_str(&incoming_form(root));
    out.push_str("<div class=\"okf-settings-actions\">");
    if let RootConfig::Git(_) = root {
        out.push_str(&format!(
            "<form method=\"post\" action=\"/__rocci_okf/settings\"><input type=\"hidden\" name=\"action\" value=\"sync\" /><input type=\"hidden\" name=\"id\" value=\"{}\" /><button type=\"submit\" class=\"okf-filter-btn\">Sync now</button></form>",
            escape(root.id())
        ));
    }
    out.push_str(&format!(
        "<form method=\"post\" action=\"/__rocci_okf/settings\"><input type=\"hidden\" name=\"action\" value=\"remove\" /><input type=\"hidden\" name=\"id\" value=\"{}\" /><button type=\"submit\" class=\"okf-filter-btn\">Remove</button></form>",
        escape(root.id())
    ));
    out.push_str("</div>\n");
    if let RootConfig::Git(git) = root {
        out.push_str("<details class=\"okf-settings-advanced\"><summary>Git options</summary>");
        out.push_str(&edit_git_form(git));
        out.push_str("</details>\n");
    }
    out.push_str("</article>\n");
    out
}

fn incoming_form(root: &RootConfig) -> String {
    let id = escape(root.id());
    let incoming = root.incoming();
    format!(
        "<form method=\"post\" action=\"/__rocci_okf/settings\"><input type=\"hidden\" name=\"action\" value=\"incoming\" /><input type=\"hidden\" name=\"id\" value=\"{id}\" /><label>Who may cite this root <select name=\"incoming\" onchange=\"this.form.submit()\"><option value=\"allow\" {}>Anyone may cite this</option><option value=\"deny\" {}>Only listed roots may cite this</option></select></label></form>",
        if incoming == Incoming::Allow {
            "selected"
        } else {
            ""
        },
        if incoming == Incoming::Deny {
            "selected"
        } else {
            ""
        },
    )
}

fn edit_git_form(git: &GitRoot) -> String {
    let token_hint = if git.token.as_ref().is_some_and(|t| !t.is_empty()) {
        "A token is stored. Leave blank to keep it. Prefer an environment variable name instead."
    } else {
        "Prefer an environment variable. A pasted token is write-only and never shown again."
    };
    format!(
        "<form method=\"post\" action=\"/__rocci_okf/settings\" class=\"okf-settings-stack\"><input type=\"hidden\" name=\"action\" value=\"edit_git\" /><input type=\"hidden\" name=\"id\" value=\"{id}\" />\
<label>Branch <input name=\"branch\" value=\"{branch}\" /></label>\
<p class=\"okf-settings-help\">Git branch to check out.</p>\
<label>Bundle subfolder <input name=\"bundle\" value=\"{bundle}\" placeholder=\"knowledge\" /></label>\
<p class=\"okf-settings-help\">Path inside the repo that contains <code>index.md</code>. Leave empty for the repo root.</p>\
<label>Token environment variable <input name=\"token_env\" value=\"{token_env}\" placeholder=\"GITHUB_TOKEN\" /></label>\
<p class=\"okf-settings-help\">Name of an env var with a PAT. Preferred over storing a token in okf.toml.</p>\
<label>Token <input name=\"token\" type=\"password\" autocomplete=\"off\" /></label>\
<p class=\"okf-settings-help\">{hint}</p>\
<label>Poll <input name=\"poll\" value=\"{poll}\" placeholder=\"5m\" /></label>\
<p class=\"okf-settings-help\">How often preview may fetch. Use <code>5m</code> or <code>off</code>.</p>\
<button type=\"submit\" class=\"okf-filter-btn\">Save git options</button></form>",
        id = escape(&git.id),
        branch = escape(&git.branch),
        bundle = escape(&git.bundle),
        token_env = escape(git.token_env.as_deref().unwrap_or("")),
        poll = escape(&git.poll.map(PollSetting::as_form_value).unwrap_or_default()),
        hint = escape(token_hint),
    )
}

fn render_matrix(config: &OkfUserConfig) -> String {
    if config.roots.len() < 2 {
        return String::new();
    }
    let ids: Vec<&str> = config.roots.iter().map(RootConfig::id).collect();
    let mut out = String::from("<h2 class=\"rd-header-2\">Citation matrix</h2>\n");
    out.push_str(
        "<p class=\"rd-paragraph\">Each checked cell means the <strong>row</strong> root may cite the <strong>column</strong> root (for example, row <code>rocci</code> column <code>notes</code> allows rocci to use <code>okf:notes/…</code>). Incoming defaults above still apply when a cell is empty.</p>\n",
    );
    out.push_str("<form method=\"post\" action=\"/__rocci_okf/settings\">");
    out.push_str("<input type=\"hidden\" name=\"action\" value=\"matrix\" />");
    out.push_str("<div class=\"okf-table-container\"><table class=\"okf-review-table\"><thead><tr><th>from \\ to</th>");
    for id in &ids {
        out.push_str(&format!("<th><code>{}</code></th>", escape(id)));
    }
    out.push_str("</tr></thead><tbody>\n");
    for from in &ids {
        out.push_str("<tr>");
        out.push_str(&format!("<th><code>{}</code></th>", escape(from)));
        for to in &ids {
            if from == to {
                out.push_str("<td>—</td>");
                continue;
            }
            let checked = if edge_allowed(from, to, config) {
                " checked"
            } else {
                ""
            };
            out.push_str(&format!(
                "<td><input type=\"checkbox\" name=\"edge.{from}.{to}\" value=\"1\"{checked} /></td>"
            ));
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody></table></div>");
    out.push_str("<button type=\"submit\" class=\"okf-cta-btn\">Save matrix</button></form>\n");
    out
}

fn render_add_forms() -> String {
    r#"<h2 class="rd-header-2">Add a folder on this computer</h2>
<p class="okf-settings-help">Pick the directory that contains <code>index.md</code> (often named <code>knowledge</code>). Choose folder is available in the desktop preview window.</p>
<form method="post" action="/__rocci_okf/settings" class="okf-settings-stack">
<input type="hidden" name="action" value="add_directory" />
<label>Short id <input id="okf-dir-id" name="id" placeholder="rocci" /></label>
<p class="okf-settings-help">Stable name used in <code>okf:id/…</code> links. Suggested from the folder name after you pick one.</p>
<label>Folder path <input id="okf-dir-path" name="path" placeholder="~/Projects/rocci/knowledge" /></label>
<p class="okf-settings-help">Absolute path, or <code>~</code> for your home directory. You can paste a path if Choose folder is hidden.</p>
<button type="button" class="okf-filter-btn" id="okf-choose-folder" hidden>Choose folder…</button>
<button type="submit" class="okf-cta-btn">Add folder</button>
</form>
<h2 class="rd-header-2">Add a git repository</h2>
<p class="okf-settings-help">rocci-okf clones into the local cache. Prefer <code>token_env</code> over pasting a token.</p>
<form method="post" action="/__rocci_okf/settings" class="okf-settings-stack">
<input type="hidden" name="action" value="add_git" />
<label>Short id <input name="id" placeholder="notes" /></label>
<label>Repository URL <input name="url" placeholder="https://github.com/org/notes.git" /></label>
<p class="okf-settings-help"><code>https://</code>, <code>ssh://</code>, <code>git@</code>, or <code>file://</code>.</p>
<label>Branch <input name="branch" value="main" /></label>
<label>Bundle subfolder <input name="bundle" placeholder="knowledge" /></label>
<p class="okf-settings-help">Leave empty if <code>index.md</code> is at the repo root.</p>
<label>Token environment variable <input name="token_env" placeholder="GITHUB_TOKEN" /></label>
<label>Token <input name="token" type="password" autocomplete="off" /></label>
<p class="okf-settings-help">Write-only. Never shown after save.</p>
<label>Poll <input name="poll" placeholder="5m" /></label>
<button type="submit" class="okf-cta-btn">Add git repository</button>
</form>
"#
    .into()
}

fn apply_action(fields: &BTreeMap<String, String>) -> Result<String> {
    let action = fields.get("action").map(String::as_str).unwrap_or("");
    let mut config = load()?;
    let message = match action {
        "add_directory" => {
            add_directory(&mut config, fields)?;
            let id = fields["id"].as_str();
            let mut message = format!("added directory root `{id}`");
            if let Some(RootConfig::Directory(dir)) = config.roots.last()
                && let Some(warning) = index_md_warning(dir)
            {
                message.push_str(". ");
                message.push_str(&warning);
            }
            message
        }
        "add_git" => {
            add_git(&mut config, fields)?;
            format!("added git root `{}`", fields["id"])
        }
        "remove" => {
            let id = required(fields, "id")?;
            let before = config.roots.len();
            config.roots.retain(|root| root.id() != id);
            if config.roots.len() == before {
                bail!("unknown root id `{id}`");
            }
            format!("removed `{id}`")
        }
        "incoming" => {
            let id = required(fields, "id")?;
            let incoming = Incoming::parse(required(fields, "incoming")?)?;
            set_incoming(&mut config, id, incoming)?;
            format!("updated incoming for `{id}`")
        }
        "edit_git" => {
            edit_git(&mut config, fields)?;
            format!("updated git root `{}`", fields["id"])
        }
        "matrix" => {
            apply_matrix(&mut config, fields)?;
            "updated citation matrix".into()
        }
        "sync" => {
            let id = required(fields, "id")?;
            sync_one(&config, id)?
        }
        other => bail!("unknown settings action `{other}`"),
    };
    if action != "sync" {
        persist(&config)?;
    }
    Ok(message)
}

fn persist(config: &OkfUserConfig) -> Result<()> {
    validate_config(config)?;
    let encoded = to_toml(config)?;
    validate_config(&crate::config::parse(&encoded)?)?;
    let path = config_path().context("cannot resolve OKF config path")?;
    save(config, &path)
}

fn add_directory(config: &mut OkfUserConfig, fields: &BTreeMap<String, String>) -> Result<()> {
    let id = required(fields, "id")?.to_string();
    let path = required(fields, "path")?.to_string();
    config.roots.push(RootConfig::Directory(DirectoryRoot {
        id,
        path,
        incoming: Incoming::Allow,
        allow_from: Vec::new(),
        deny_from: Vec::new(),
        poll: None,
        extra: toml::Table::new(),
    }));
    Ok(())
}

fn add_git(config: &mut OkfUserConfig, fields: &BTreeMap<String, String>) -> Result<()> {
    let token = fields
        .get("token")
        .cloned()
        .filter(|value| !value.is_empty());
    config.roots.push(RootConfig::Git(GitRoot {
        id: required(fields, "id")?.to_string(),
        url: required(fields, "url")?.to_string(),
        branch: fields
            .get("branch")
            .cloned()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "main".into()),
        bundle: fields.get("bundle").cloned().unwrap_or_default(),
        token,
        token_env: fields
            .get("token_env")
            .cloned()
            .filter(|value| !value.is_empty()),
        incoming: Incoming::Deny,
        allow_from: Vec::new(),
        deny_from: Vec::new(),
        poll: parse_optional_poll(fields.get("poll").map(String::as_str).unwrap_or(""))?,
        extra: toml::Table::new(),
    }));
    Ok(())
}

fn edit_git(config: &mut OkfUserConfig, fields: &BTreeMap<String, String>) -> Result<()> {
    let id = required(fields, "id")?;
    let RootConfig::Git(git) = config
        .roots
        .iter_mut()
        .find(|root| root.id() == id)
        .ok_or_else(|| anyhow::anyhow!("unknown root id `{id}`"))?
    else {
        bail!("root `{id}` is not a git root");
    };
    if let Some(branch) = fields.get("branch").filter(|value| !value.is_empty()) {
        git.branch = branch.clone();
    }
    if let Some(bundle) = fields.get("bundle") {
        git.bundle = bundle.clone();
    }
    git.token_env = fields
        .get("token_env")
        .cloned()
        .filter(|value| !value.is_empty());
    if let Some(token) = fields.get("token").filter(|value| !value.is_empty()) {
        git.token = Some(token.clone());
    }
    git.poll = parse_optional_poll(fields.get("poll").map(String::as_str).unwrap_or(""))?;
    Ok(())
}

fn set_incoming(config: &mut OkfUserConfig, id: &str, incoming: Incoming) -> Result<()> {
    match config.roots.iter_mut().find(|root| root.id() == id) {
        Some(RootConfig::Directory(dir)) => dir.incoming = incoming,
        Some(RootConfig::Git(git)) => git.incoming = incoming,
        None => bail!("unknown root id `{id}`"),
    }
    Ok(())
}

fn apply_matrix(config: &mut OkfUserConfig, fields: &BTreeMap<String, String>) -> Result<()> {
    let ids: Vec<String> = config
        .roots
        .iter()
        .map(|root| root.id().to_string())
        .collect();
    let mut allowed = BTreeSet::new();
    for from in &ids {
        for to in &ids {
            if from == to {
                continue;
            }
            if fields.contains_key(&format!("edge.{from}.{to}")) {
                allowed.insert((from.clone(), to.clone()));
            }
        }
    }
    for to in &ids {
        let mut allow = Vec::new();
        let mut deny = Vec::new();
        for from in &ids {
            if from == to {
                continue;
            }
            if allowed.contains(&(from.clone(), to.clone())) {
                allow.push(from.clone());
            } else {
                deny.push(from.clone());
            }
        }
        let (incoming, allow_from, deny_from) = if deny.is_empty() {
            (Incoming::Allow, Vec::new(), Vec::new())
        } else if allow.is_empty() {
            (Incoming::Deny, Vec::new(), Vec::new())
        } else {
            (Incoming::Deny, allow, Vec::new())
        };
        match config.roots.iter_mut().find(|root| root.id() == to) {
            Some(RootConfig::Directory(dir)) => {
                dir.incoming = incoming;
                dir.allow_from = allow_from;
                dir.deny_from = deny_from;
            }
            Some(RootConfig::Git(git)) => {
                git.incoming = incoming;
                git.allow_from = allow_from;
                git.deny_from = deny_from;
            }
            None => {}
        }
    }
    Ok(())
}

fn sync_one(config: &OkfUserConfig, id: &str) -> Result<String> {
    let Some(RootConfig::Git(git)) = config.roots.iter().find(|root| root.id() == id) else {
        bail!("git root `{id}` not found");
    };
    let token = git.resolved_token();
    let resolved = sync_git_root(git, &okf_cache_dir(), token.as_deref());
    if let Some(error) = resolved.error {
        bail!("sync `{id}` failed: {error}");
    }
    Ok(format!("synced `{id}`"))
}

fn parse_optional_poll(raw: &str) -> Result<Option<PollSetting>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    if raw.eq_ignore_ascii_case("off") || raw.eq_ignore_ascii_case("false") {
        return Ok(Some(PollSetting::Off));
    }
    let parsed = crate::config::parse(&format!("poll = \"{raw}\""))?;
    Ok(Some(parsed.poll))
}

fn required<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    let value = fields.get(key).map(String::as_str).unwrap_or("").trim();
    if value.is_empty() {
        bail!("{key} is required");
    }
    Ok(value)
}

fn form_body(raw: &[u8]) -> String {
    raw.windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| String::from_utf8_lossy(&raw[index + 4..]).into_owned())
        .unwrap_or_default()
}

fn parse_form(body: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for pair in body.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        fields.insert(url_decode(key), url_decode(value));
    }
    fields
}

fn url_decode(input: &str) -> String {
    let mut bytes = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '+' => {
                bytes.push(b' ');
                i += 1;
            }
            '%' if i + 2 < chars.len() => {
                let hex: String = chars[i + 1..i + 3].iter().collect();
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    bytes.push(byte);
                    i += 3;
                } else {
                    bytes.extend(chars[i].to_string().as_bytes());
                    i += 1;
                }
            }
            ch => {
                bytes.extend(ch.to_string().as_bytes());
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn index_md_warning(dir: &DirectoryRoot) -> Option<String> {
    let path = dir.expanded_path();
    if path.is_dir() && !path.join("index.md").is_file() {
        Some(
            "This folder has no index.md. rocci-okf still expects an OKF bundle; you can add it anyway."
                .into(),
        )
    } else {
        None
    }
}

fn format_last_fetch(unix: Option<u64>) -> String {
    let Some(fetched) = unix else {
        return "never".into();
    };
    let ago = now_unix().saturating_sub(fetched);
    if ago < 60 {
        format!("{ago}s ago")
    } else if ago < 3600 {
        format!("{}m ago", ago / 60)
    } else if ago < 86400 {
        format!("{}h ago", ago / 3600)
    } else {
        format!("{}d ago", ago / 86400)
    }
}

const SETTINGS_PAGE_SCRIPT: &str = r#"<script>
(function(){
  if (location.pathname === "/__rocci_okf/settings") {
    history.replaceState(null, "", "/settings/");
  }
  var browse = document.getElementById("okf-choose-folder");
  var path = document.getElementById("okf-dir-path");
  var id = document.getElementById("okf-dir-id");
  if (browse && window.ipc && window.ipc.postMessage) {
    browse.hidden = false;
    window.addEventListener("rocci-pick-folder", function(ev){
      var picked = ev.detail && ev.detail.path;
      if (!picked || !path) return;
      path.value = picked;
      if (id && !id.value.trim()) {
        var parts = String(picked).replace(/\\/g, "/").replace(/\/$/, "").split("/");
        id.value = parts[parts.length - 1] || "";
      }
    });
    browse.addEventListener("click", function(e){
      e.preventDefault();
      window.ipc.postMessage("pick-folder");
    });
  }
})();
</script>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_redacts_tokens_and_has_forms() {
        let config = crate::config::parse(
            r#"
[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/n.git"
token = "super-secret-token"
"#,
        )
        .unwrap();
        let html = render_article_from(&config, None);
        assert!(!html.contains("super-secret-token"), "{html}");
        assert!(html.contains("type=\"password\""));
        assert!(html.contains("Add folder"));
        assert!(html.contains("Add git repository"));
        assert!(html.contains("Choose folder"));
        assert!(html.contains("Knowledge roots"));
        assert!(html.contains("pick-folder"));
        assert!(html.contains("Anyone may cite this"));
    }

    #[test]
    fn matrix_checkboxes_compile_without_both_listed() {
        let mut config = crate::config::parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "/tmp/a"

[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/n.git"
incoming = "deny"
"#,
        )
        .unwrap();
        let mut fields = BTreeMap::new();
        fields.insert("edge.rocci.notes".into(), "1".into());
        apply_matrix(&mut config, &fields).unwrap();
        validate_config(&config).unwrap();
        assert!(edge_allowed("rocci", "notes", &config));
        assert!(!edge_allowed("notes", "rocci", &config));
    }

    #[test]
    fn http_get_settings_returns_article() {
        let article = http("GET", "/settings/", b"").unwrap();
        assert!(article.contains("Knowledge roots"));
        assert!(article.contains("/__rocci_okf/settings"));
    }

    #[test]
    fn post_add_directory_writes_toml_and_hides_token() {
        let _lock = crate::config::CONFIG_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let dir = std::env::temp_dir().join(format!(
            "rocci-okf-settings-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("okf.toml");
        std::fs::write(&path, "poll = \"5m\"\n").unwrap();
        let original = std::env::var("ROCCI_OKF_CONFIG").ok();
        unsafe { std::env::set_var("ROCCI_OKF_CONFIG", &path) };

        let added = http(
            "POST",
            "/__rocci_okf/settings",
            b"\r\n\r\naction=add_directory&id=notes&path=%2Ftmp%2Fknowledge",
        )
        .unwrap();
        assert!(added.contains("added directory root"), "{added}");
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("id = \"notes\""), "{saved}");
        assert!(saved.contains("kind = \"directory\""), "{saved}");

        let git = http(
            "POST",
            "/__rocci_okf/settings",
            b"\r\n\r\naction=add_git&id=private&url=https%3A%2F%2Fexample.com%2Fn.git&token=super-secret-token",
        )
        .unwrap();
        assert!(!git.contains("super-secret-token"), "{git}");
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("super-secret-token"), "{saved}");
        assert!(saved.contains("id = \"private\""), "{saved}");

        match original {
            Some(value) => unsafe { std::env::set_var("ROCCI_OKF_CONFIG", value) },
            None => unsafe { std::env::remove_var("ROCCI_OKF_CONFIG") },
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn directory_card_shows_resolved_path_and_index_warning() {
        let bundle = std::env::temp_dir().join(format!(
            "rocci-okf-settings-bundle-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&bundle).unwrap();
        let with_index = bundle.join("with-index");
        std::fs::create_dir_all(&with_index).unwrap();
        std::fs::write(with_index.join("index.md"), "# Knowledge\n").unwrap();
        let without_index = bundle.join("empty");
        std::fs::create_dir_all(&without_index).unwrap();

        let config = crate::config::parse(&format!(
            r#"
[[roots]]
id = "docs"
kind = "directory"
path = "{}"

[[roots]]
id = "empty"
kind = "directory"
path = "{}"
"#,
            with_index.display(),
            without_index.display()
        ))
        .unwrap();
        let html = render_article_from(&config, None);
        let resolved = with_index.canonicalize().unwrap();
        assert!(html.contains(&resolved.display().to_string()), "{html}");
        assert!(html.contains("Status: available"), "{html}");
        assert!(html.contains("no index.md"), "{html}");
        let _ = std::fs::remove_dir_all(&bundle);
    }
}
