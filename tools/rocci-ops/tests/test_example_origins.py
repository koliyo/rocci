from pathlib import Path

from rocci_ops.paths import repo_root


def test_example_caddy_routes_by_host_without_stealing_site_actions() -> None:
    root = repo_root()
    hybrid = (root / "docker/cdn/Caddyfile").read_text(encoding="utf-8")
    examples = (root / "docker/examples/Caddyfile").read_text(encoding="utf-8")
    compose = (root / "docker/compose.examples.yml").read_text(encoding="utf-8")
    assert "handle /actions/*" in hybrid
    assert "handle /sse" in hybrid
    assert "handle /actions/*" not in examples
    assert "live-counter.examples.rocci.dev" in examples
    assert "datastar.examples.rocci.dev" in examples
    assert "live-counter.examples.staging.rocci.dev" in examples
    assert "datastar:" in compose
    assert "live-counter:" in compose
    assert "snake" not in compose
    assert Path(root / "examples/rocci/apps.toml").is_file()
