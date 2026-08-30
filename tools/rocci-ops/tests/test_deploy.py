import os
from pathlib import Path

from rocci_ops.deploy import origin_publish_cmd, push
from rocci_ops.sshutil import ssh_opts, validate_sha


def test_validate_sha_accepts_hex() -> None:
    assert validate_sha("abcDEF012") == "abcDEF012"


def test_validate_sha_rejects_non_hex() -> None:
    try:
        validate_sha("not a sha")
    except SystemExit as exc:
        assert "hex" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_ssh_opts_include_proxy_when_access_set(monkeypatch) -> None:
    monkeypatch.setenv("CF_ACCESS_CLIENT_ID", "token")
    monkeypatch.delenv("ROCCI_SSH_VERBOSE", raising=False)
    opts = ssh_opts()
    joined = " ".join(opts)
    assert "ProxyCommand=" in joined
    assert "access-ssh-proxy.sh" in joined


def test_origin_publish_cmd() -> None:
    cmd = origin_publish_cmd("deadbeef", "/srv/rocci")
    assert "uv run --no-dev rocci-ops origin publish 'deadbeef'" in cmd
    assert "ROCCI_ORIGIN_ROOT='/srv/rocci'" in cmd
    assert "ROCCI_HTTP_PORT='8080'" in cmd
    assert "COMPOSE_PROJECT_NAME='rocci-prod'" in cmd
    assert "ROCCI_PUBLISH_LIVE='1'" in cmd


def test_origin_publish_cmd_staging_lane(monkeypatch) -> None:
    monkeypatch.setenv("ROCCI_LANE", "staging")
    cmd = origin_publish_cmd("deadbeef")
    assert "cd '/srv/rocci/staging'" in cmd
    assert "ROCCI_LANE='staging'" in cmd
    assert "ROCCI_HTTP_PORT='8081'" in cmd
    assert "COMPOSE_PROJECT_NAME='rocci-staging'" in cmd
    assert "ROCCI_PUBLISH_LIVE='1'" in cmd
    assert "ROCCI_IMAGE_TAG='staging'" in cmd


def test_origin_publish_cmd_production_lane(monkeypatch) -> None:
    monkeypatch.setenv("ROCCI_LANE", "production")
    cmd = origin_publish_cmd("deadbeef")
    assert "cd '/srv/rocci/prod'" in cmd
    assert "ROCCI_PUBLISH_LIVE='0'" in cmd
    assert "ROCCI_IMAGE_TAG='prod'" in cmd


def test_push_invokes_bootstrap_scp_and_publish(monkeypatch, tmp_path: Path) -> None:
    artifact = tmp_path / "artifacts"
    artifact.mkdir()
    (artifact / "site.tgz").write_bytes(b"tgz")
    (artifact / "islands").write_bytes(b"bin")
    monkeypatch.setenv("DEPLOY_HOST", "ssh.rocci.dev")
    monkeypatch.setenv("DEPLOY_USER", "deploy")
    calls: list[list[str]] = []

    def runner(argv, **_kwargs):
        calls.append(argv)

        class Result:
            returncode = 0

        return Result()

    push(artifact, "abc123", runner=runner)
    flat = [" ".join(call) for call in calls]
    assert any("mkdir -p" in item and "/srv/rocci/prod/tools/rocci-ops" in item for item in flat)
    assert any(str(artifact / "site.tgz") in item for item in flat)
    assert any("compose.origin.yml" in item for item in flat)
    assert any("origin publish 'abc123'" in item for item in flat)
    assert any("examples.caddy" in item for item in flat)
    assert any("examples.stub.caddy" in item for item in flat)
    assert not any("/blocks/" in item or "docker/blocks" in item for item in flat)


def test_push_staging_lane_uses_staging_root(monkeypatch, tmp_path: Path) -> None:
    artifact = tmp_path / "artifacts"
    artifact.mkdir()
    (artifact / "site.tgz").write_bytes(b"tgz")
    (artifact / "islands").write_bytes(b"bin")
    monkeypatch.setenv("DEPLOY_HOST", "ssh.rocci.dev")
    monkeypatch.setenv("DEPLOY_USER", "deploy")
    monkeypatch.setenv("ROCCI_LANE", "staging")
    calls: list[list[str]] = []

    def runner(argv, **_kwargs):
        calls.append(argv)

        class Result:
            returncode = 0

        return Result()

    push(artifact, "abc123", runner=runner)
    flat = [" ".join(call) for call in calls]
    assert any("mkdir -p" in item and "/srv/rocci/staging/tools/rocci-ops" in item for item in flat)
    assert any("/srv/rocci/staging/incoming/abc123" in item for item in flat)
    assert any("ROCCI_LANE='staging'" in item and "origin publish 'abc123'" in item for item in flat)


def test_push_copies_examples_live_tree(monkeypatch, tmp_path: Path) -> None:
    artifact = tmp_path / "artifacts"
    live = artifact / "examples-live" / "live-counter"
    live.mkdir(parents=True)
    (artifact / "site.tgz").write_bytes(b"tgz")
    (artifact / "islands").write_bytes(b"bin")
    (live / "server").write_bytes(b"srv")
    monkeypatch.setenv("DEPLOY_HOST", "ssh.rocci.dev")
    monkeypatch.setenv("DEPLOY_USER", "deploy")
    calls: list[list[str]] = []

    def runner(argv, **_kwargs):
        calls.append(argv)

        class Result:
            returncode = 0

        return Result()

    push(artifact, "abc123", runner=runner)
    flat = [" ".join(call) for call in calls]
    assert any(str(artifact / "examples-live") in item and "-r" in item for item in flat)
