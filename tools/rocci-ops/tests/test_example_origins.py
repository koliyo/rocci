import tomllib
from pathlib import Path

from rocci_ops.paths import repo_root


def _actions_handle_block(caddy: str) -> str:
    start = caddy.find("handle /actions/*")
    assert start >= 0
    rest = caddy[start:]
    end = rest.find("\n\thandle /sse")
    if end < 0:
        end = rest.find("\nhandle /sse")
    assert end > 0
    return rest[:end]


def test_example_caddy_routes_by_host_without_stealing_site_actions() -> None:
    root = repo_root()
    hybrid = (root / "docker/cdn/Caddyfile").read_text(encoding="utf-8")
    examples = (root / "docker/examples/Caddyfile").read_text(encoding="utf-8")
    compose = (root / "docker/compose.examples.yml").read_text(encoding="utf-8")
    origin = (root / "docker/compose.origin.yml").read_text(encoding="utf-8")
    hybrid_compose = (root / "docker/compose.hybrid.yml").read_text(encoding="utf-8")
    assert "handle /actions/*" in hybrid
    assert "handle /sse" in hybrid
    assert "handle /actions/*" not in examples
    for host in (
        "live-counter-example.rocci.dev",
        "live-counter-example-staging.rocci.dev",
        "live-counter.examples.rocci.dev",
        "live-counter.examples.staging.rocci.dev",
        "live-counter.examples.localhost",
        "datastar-example.rocci.dev",
        "datastar-example-staging.rocci.dev",
        "datastar.examples.rocci.dev",
        "datastar.examples.staging.rocci.dev",
        "datastar.examples.localhost",
    ):
        assert host in hybrid
        assert host in examples
    live_at = hybrid.find("@live-counter host")
    datastar_at = hybrid.find("@datastar host")
    play_live_at = hybrid.find("handle_path /play/live-counter")
    play_datastar_at = hybrid.find("handle_path /play/datastar")
    actions_at = hybrid.find("handle /actions/*")
    sse_at = hybrid.find("handle /sse")
    assert 0 <= live_at < actions_at
    assert 0 <= datastar_at < actions_at
    assert 0 <= play_live_at < actions_at
    assert 0 <= play_datastar_at < actions_at
    assert 0 <= actions_at < sse_at
    assert "reverse_proxy live-counter:8000" in hybrid
    assert "reverse_proxy datastar:8000" in hybrid
    live_handle = hybrid[play_live_at : hybrid.find("}", play_live_at) + 1]
    assert "reverse_proxy live-counter:8000" in live_handle
    assert "redir /play/live-counter /play/live-counter/" in hybrid
    assert "redir /play/datastar /play/datastar/" in hybrid
    actions = _actions_handle_block(hybrid)
    assert "reverse_proxy islands:8001" in actions
    assert "live-counter:8000" not in actions
    assert "datastar:8000" not in actions
    assert "datastar:" in compose
    assert "live-counter:" in compose
    assert "edge:" in compose
    assert "snake" not in compose
    assert "datastar:" in origin
    assert "live-counter:" in origin
    assert "edge:" not in origin
    assert "snake" not in origin
    assert "edge:" not in hybrid_compose
    assert "snake" not in hybrid_compose
    readme = (root / "docker/README.md").read_text(encoding="utf-8")
    assert "Do **not** run that edge on the VPS" in readme
    workflow = (root / ".github/workflows/site.yml").read_text(encoding="utf-8")
    assert '"examples/**"' in workflow or "- \"examples/**\"" in workflow
    assert "- \"playground/**\"" in workflow
    assert "dist/examples-live/**" in workflow
    assert "dist/examples-live/live-counter/server" not in workflow
    assert "dist/examples-live/datastar/server" not in workflow
    ingress = (root / "docker/prod/cloudflared-ingress.yml.example").read_text(encoding="utf-8")
    assert '"*.examples.rocci.dev"' in ingress or "*.examples.rocci.dev" in ingress
    assert "*.examples.staging.rocci.dev" in ingress
    assert "live-counter-example-staging.rocci.dev" in ingress
    assert "datastar-example-staging.rocci.dev" in ingress
    assert Path(root / "examples/rocci/apps.toml").is_file()


def test_retired_news_urls_have_exact_origin_dispositions() -> None:
    hybrid = (repo_root() / "docker/cdn/Caddyfile").read_text(encoding="utf-8")
    assert "redir /news" not in hybrid
    assert "@retired_news path /news /news/ /news/feed.xml" in hybrid
    assert "handle @retired_news" in hybrid
    assert "respond 410" in hybrid
    assert "respond @retired_news 410" not in hybrid
    assert "Content-Type application/wasm" in hybrid
    assert "path /news/*" not in hybrid


def test_examples_nav_matches_site_true_catalog_ids() -> None:
    root = repo_root()
    catalog = tomllib.loads((root / "examples/rocci/apps.toml").read_text(encoding="utf-8"))
    site_ids = {app["id"] for app in catalog["app"] if app.get("site", True)}
    excluded = {app["id"] for app in catalog["app"] if not app.get("site", True)}
    site_cfg = tomllib.loads((root / "site/rocdown.toml").read_text(encoding="utf-8"))
    examples = next(nav for nav in site_cfg["nav"] if nav.get("label") == "Examples")
    items = examples["items"]
    assert items[0] == "examples/index"
    nav_ids = []
    for item in items[1:]:
        assert item.startswith("examples/")
        assert item.endswith("/index")
        nav_ids.append(item.removeprefix("examples/").removesuffix("/index"))
    assert set(nav_ids) == site_ids
    assert excluded.isdisjoint(nav_ids)


def test_package_and_build_site_do_not_pass_all() -> None:
    local = (repo_root() / "tools/rocci-ops/src/rocci_ops/local.py").read_text(encoding="utf-8")
    assert '"--all"' not in local
    assert "'--all'" not in local
