from pathlib import Path
from types import SimpleNamespace
import io
import tarfile

from rocci_ops.origin import publish, wait_health


class FakeResponse:
    def __init__(self, status: int) -> None:
        self.status = status

    def __enter__(self):
        return self

    def __exit__(self, *_args) -> None:
        return None


def test_wait_health_succeeds(monkeypatch) -> None:
    monkeypatch.setenv("ROCCI_HTTP_PORT", "8080")
    sleeps: list[float] = []
    assert wait_health(fetch=lambda _url, timeout=5: FakeResponse(200), sleeper=sleeps.append)
    assert sleeps == []


def test_wait_health_retries_then_ok() -> None:
    n = {"i": 0}

    def fetch(_url, timeout=5):
        n["i"] += 1
        if n["i"] < 3:
            raise OSError("down")
        return FakeResponse(200)

    sleeps: list[float] = []
    assert wait_health(fetch=fetch, sleeper=sleeps.append, delay=0)
    assert len(sleeps) == 2


def test_publish_rolls_back_on_health_failure(monkeypatch, tmp_path: Path) -> None:
    origin = tmp_path / "origin"
    incoming = origin / "incoming" / "abc"
    previous = origin / "releases" / "old"
    incoming.mkdir(parents=True)
    previous.mkdir(parents=True)
    (previous / "dist").mkdir()
    (incoming / "site.tgz").write_bytes(b"")
    (incoming / "islands").write_bytes(b"bin")

    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        info = tarfile.TarInfo("index.html")
        data = b"<html></html>"
        info.size = len(data)
        tar.addfile(info, io.BytesIO(data))
    (incoming / "site.tgz").write_bytes(buf.getvalue())

    docker = tmp_path / "repo" / "docker" / "islands"
    docker.mkdir(parents=True)
    (docker / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
    (tmp_path / "repo" / "docker").mkdir(exist_ok=True)
    (tmp_path / "repo" / "docker" / "compose.hybrid.yml").write_text("services: {}\n", encoding="utf-8")
    (tmp_path / "repo" / "tools" / "rocci-ops").mkdir(parents=True)
    (tmp_path / "repo" / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")

    current = origin / "current"
    current.symlink_to(previous)

    monkeypatch.setenv("ROCCI_ORIGIN_ROOT", str(origin))
    monkeypatch.setenv("ROCCI_REPO_ROOT", str(tmp_path / "repo"))

    compose_roots: list[str] = []
    compose_argv: list[list[str]] = []

    def runner(argv, **kwargs):
        compose_roots.append(kwargs.get("env", {}).get("ROCCI_DIST", ""))
        compose_argv.append(list(argv))
        return SimpleNamespace(returncode=0)

    def fetch(_url, timeout=5):
        raise OSError("unhealthy")

    try:
        publish("abc", runner=runner, fetch=fetch, sleeper=lambda _s: None)
    except SystemExit:
        pass
    else:
        raise AssertionError("expected SystemExit")
    assert len(compose_roots) >= 2
    assert compose_roots[0].endswith("/releases/abc/dist")
    assert compose_roots[-1].endswith("/releases/old/dist")
    assert "--profile" not in compose_argv[0]
