from pathlib import Path

from rocci_ops.deploy import origin_publish_cmd, push, stage_incoming, stage_origin_kit
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


def _record_runner(calls: list[list[str]]):
    def runner(argv, **_kwargs):
        calls.append(argv)

        class Result:
            returncode = 0

        return Result()

    return runner


def test_stage_origin_kit_copies_compose_and_caddy(tmp_path: Path) -> None:
    stage_origin_kit(tmp_path)
    assert (tmp_path / "docker" / "compose.origin.yml").is_file()
    assert (tmp_path / "docker" / "cdn" / "examples.caddy").is_file()
    assert (tmp_path / "docker" / "cdn" / "examples.stub.caddy").is_file()
    assert (tmp_path / "tools" / "rocci-ops" / "src" / "rocci_ops" / "lanes.py").is_file()
    assert not (tmp_path / "docker" / "blocks").exists()


def test_push_uses_one_ssh_tar_connection(monkeypatch, tmp_path: Path) -> None:
    artifact = tmp_path / "artifacts"
    artifact.mkdir()
    (artifact / "site.tgz").write_bytes(b"tgz")
    (artifact / "islands").write_bytes(b"bin")
    monkeypatch.setenv("DEPLOY_HOST", "ssh.rocci.dev")
    monkeypatch.setenv("DEPLOY_USER", "deploy")
    calls: list[list[str]] = []
    push(artifact, "abc123", runner=_record_runner(calls))
    assert len(calls) == 1
    assert calls[0][0] == "ssh"
    remote = calls[0][-1]
    assert "tar -xzf - -C '/srv/rocci/prod'" in remote
    assert "mkdir -p '/srv/rocci/prod'" in remote
    assert "origin publish 'abc123'" in remote
    assert not any(call[0] == "scp" for call in calls)


def test_push_staging_lane_uses_staging_root(monkeypatch, tmp_path: Path) -> None:
    artifact = tmp_path / "artifacts"
    artifact.mkdir()
    (artifact / "site.tgz").write_bytes(b"tgz")
    (artifact / "islands").write_bytes(b"bin")
    monkeypatch.setenv("DEPLOY_HOST", "ssh.rocci.dev")
    monkeypatch.setenv("DEPLOY_USER", "deploy")
    monkeypatch.setenv("ROCCI_LANE", "staging")
    calls: list[list[str]] = []
    push(artifact, "abc123", runner=_record_runner(calls))
    assert len(calls) == 1
    remote = calls[0][-1]
    assert "tar -xzf - -C '/srv/rocci/staging'" in remote
    assert "ROCCI_LANE='staging'" in remote
    assert "origin publish 'abc123'" in remote


def test_push_copies_examples_live_tree(monkeypatch, tmp_path: Path) -> None:
    artifact = tmp_path / "artifacts"
    live = artifact / "examples-live" / "live-counter"
    live.mkdir(parents=True)
    (artifact / "site.tgz").write_bytes(b"tgz")
    (artifact / "islands").write_bytes(b"bin")
    (live / "server").write_bytes(b"srv")
    dest = tmp_path / "tree"
    dest.mkdir()
    stage_incoming(dest, artifact, "abc123")
    assert (dest / "incoming" / "abc123" / "examples-live" / "live-counter" / "server").is_file()
    monkeypatch.setenv("DEPLOY_HOST", "ssh.rocci.dev")
    monkeypatch.setenv("DEPLOY_USER", "deploy")
    calls: list[list[str]] = []
    push(artifact, "abc123", runner=_record_runner(calls))
    remote = calls[0][-1]
    assert "origin publish 'abc123'" in remote
