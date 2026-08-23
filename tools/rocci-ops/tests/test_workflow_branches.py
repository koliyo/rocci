from pathlib import Path

WORKFLOWS = Path(__file__).resolve().parents[3] / ".github" / "workflows"


def _on_push_branches(text: str) -> str:
    after_push = text.split("on:\n", 1)[1]
    for line in after_push.splitlines():
        stripped = line.strip()
        if stripped.startswith("branches:"):
            return stripped
    raise AssertionError("no push branches list found")


def test_ci_and_knowledge_push_include_production() -> None:
    for name in ("ci.yml", "knowledge.yml"):
        text = (WORKFLOWS / name).read_text(encoding="utf-8")
        assert "pull_request" not in text
        assert "self-hosted" not in text
        assert _on_push_branches(text) == "branches: [main, staging, production]"


def test_site_push_is_staging_and_production_only() -> None:
    text = (WORKFLOWS / "site.yml").read_text(encoding="utf-8")
    assert "pull_request" not in text
    assert _on_push_branches(text) == "branches: [staging, production]"
    assert 'github.ref == \'refs/heads/staging\' || github.ref == \'refs/heads/production\'' in text
