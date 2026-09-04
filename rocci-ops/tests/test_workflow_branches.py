from pathlib import Path

WORKFLOWS = Path(__file__).resolve().parents[2] / ".github" / "workflows"


def _on_push_branches(text: str) -> str:
    after_push = text.split("on:\n", 1)[1]
    for line in after_push.splitlines():
        stripped = line.strip()
        if stripped.startswith("branches:"):
            return stripped
    raise AssertionError("no push branches list found")


def test_knowledge_workflow_installs_released_okmate() -> None:
    text = (WORKFLOWS / "knowledge.yml").read_text(encoding="utf-8")
    pin = (WORKFLOWS.parent / "okmate-version").read_text(encoding="utf-8").strip()
    assert pin.startswith("v")
    assert "gh release download" in text
    assert "koliyo/okmate" in text
    assert "okmate-x86_64-unknown-linux-gnu" in text
    assert ".github/okmate-version" in text
    assert "repository: koliyo/okmate" not in text
    assert "rust-toolchain" not in text
    assert "OKMATE_DIR" not in text
    assert "rocci-ops ci knowledge" in text


def test_ci_and_knowledge_push_include_production() -> None:
    for name in ("ci.yml", "knowledge.yml"):
        text = (WORKFLOWS / name).read_text(encoding="utf-8")
        assert "pull_request" not in text
        assert "self-hosted" not in text
        assert _on_push_branches(text) == "branches: [main, staging, production]"
        assert 'tags: ["v*", "dev"]' in text


def test_ci_roc_job_installs_linux_musl_target() -> None:
    text = (WORKFLOWS / "ci.yml").read_text(encoding="utf-8")
    roc_job = text.split("  roc:\n", 1)[1].split("\n  test:\n", 1)[0]
    assert "x86_64-unknown-linux-musl" in roc_job
    assert "rocci-ops ci roc" in roc_job
    assert "rocci-platform.tar.zst" in roc_job
    assert "upload-artifact" in roc_job
    assert "if-no-files-found: error" in roc_job


def test_release_push_includes_version_and_dev_tags() -> None:
    text = (WORKFLOWS / "release.yml").read_text(encoding="utf-8")
    assert 'tags: ["v*", "dev"]' in text
    assert "rocci-ops archive version" in text
    assert "rocci-ops archive package" in text
    assert "rocci-ops archive wait-ci" in text
    assert "rocci-ops archive params" in text
    assert "rocci-ops archive publish" in text
    assert "rocci-platform.tar.zst" in text
    assert "bundle.sh --skip-build" in text
    assert "archive package-platform" in text
    assert "archive merge-libhosts" in text
    assert "libhost-${{ runner.os }}" in text
    assert "ROCCI_RELEASE_TAG" in text


def test_workflows_do_not_call_hosted_release_helpers_as_release() -> None:
    for path in sorted(WORKFLOWS.glob("*.yml")):
        text = path.read_text(encoding="utf-8")
        assert "rocci-ops release version" not in text
        assert "rocci-ops release package" not in text
        assert "rocci-ops release wait-ci" not in text
        assert "rocci-ops release params" not in text
        assert "rocci-ops release publish" not in text


def test_cut_release_is_workflow_dispatch_only() -> None:
    text = (WORKFLOWS / "cut-release.yml").read_text(encoding="utf-8")
    assert "workflow_dispatch:" in text
    assert 'run-name: "Cut release: ${{ inputs.spec }}"' in text
    assert "environment:" not in text
    assert "rocci-ops release" in text
    assert "HOMEBREW" not in text


def test_site_push_is_staging_and_production_only() -> None:
    text = (WORKFLOWS / "site.yml").read_text(encoding="utf-8")
    assert "pull_request" not in text
    assert _on_push_branches(text) == "branches: [staging, production]"
    assert 'github.ref == \'refs/heads/staging\' || github.ref == \'refs/heads/production\'' in text
    assert "ROCCI_LANE: ${{ github.ref_name }}" in text


def test_no_workflow_uses_self_hosted_runners() -> None:
    for path in sorted(WORKFLOWS.glob("*.yml")):
        text = path.read_text(encoding="utf-8")
        assert "self-hosted" not in text
        assert "rocci-linux" not in text
        assert "/home/nils/.cache" not in text
