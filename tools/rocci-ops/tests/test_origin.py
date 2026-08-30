from email.message import Message
from pathlib import Path
from types import SimpleNamespace
import io
import tarfile
import urllib.error

from rocci_ops.origin import health_probe, live_app_env_key, health_checks, publish, wait_health
from rocci_ops.lanes import resolved_lane, should_publish_live


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


def test_live_app_env_and_health_hosts() -> None:
    assert live_app_env_key("live-counter") == "ROCCI_LIVE_COUNTER_CONTEXT"
    assert live_app_env_key("datastar") == "ROCCI_DATASTAR_CONTEXT"
    checks = health_checks(["live-counter", "datastar"])
    urls = [url for url, _headers in checks]
    assert urls[0] == "http://127.0.0.1:8080/health"
    assert "http://127.0.0.1:8080/play/live-counter/health" in urls
    assert "http://127.0.0.1:8080/play/datastar/health" in urls
    assert checks[0][1] == {}
    play = [headers for url, headers in checks if "/play/" in url]
    assert play == [{}, {}]
    hosts = [headers["Host"] for _url, headers in checks if headers]
    assert hosts == [
        "live-counter-example-staging.rocci.dev",
        "live-counter.examples.localhost",
        "live-counter-example.rocci.dev",
        "datastar-example-staging.rocci.dev",
        "datastar.examples.localhost",
        "datastar-example.rocci.dev",
    ]


def test_staging_lane_health_omits_production_example_hosts(monkeypatch) -> None:
    monkeypatch.setenv("ROCCI_LANE", "staging")
    checks = health_checks(["live-counter"])
    hosts = [headers["Host"] for _url, headers in checks if headers]
    assert hosts == [
        "live-counter-example-staging.rocci.dev",
        "live-counter.examples.localhost",
    ]
    assert "live-counter-example.rocci.dev" not in hosts
    assert any("/play/live-counter/health" in url for url, _headers in checks)
    assert resolved_lane().http_port == "8081"
    assert resolved_lane().origin_root == "/srv/rocci-staging"


def test_unknown_lane_exits(monkeypatch) -> None:
    monkeypatch.setenv("ROCCI_LANE", "lab")
    try:
        resolved_lane()
    except SystemExit as exc:
        assert "unknown ROCCI_LANE" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_production_lane_skips_live_health_and_compose(monkeypatch) -> None:
    monkeypatch.setenv("ROCCI_LANE", "production")
    assert should_publish_live(["live-counter"]) is False
    assert health_checks([]) == [("http://127.0.0.1:8080/health", {})]
    hosts = [headers.get("Host") for _url, headers in health_checks(["live-counter"]) if headers]
    assert hosts == []
    assert resolved_lane().http_port == "8080"
    assert resolved_lane().origin_root == "/srv/rocci"
    assert resolved_lane().image_tag == "prod"


def test_health_probe_reports_http_error() -> None:
    def fetch(_url, timeout=5):
        raise urllib.error.HTTPError(_url, 502, "bad gateway", Message(), None)

    ok, detail = health_probe("http://127.0.0.1:8080/play/live-counter/health", fetch=fetch)
    assert ok is False
    assert detail == "502"


def test_health_ok_bypasses_env_proxy(monkeypatch) -> None:
    import http.server
    import socketserver
    import threading

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b"ok")

        def log_message(self, *_args):
            return None

    httpd = socketserver.TCPServer(("127.0.0.1", 0), Handler)
    port = httpd.server_address[1]
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    monkeypatch.setenv("HTTP_PROXY", "http://127.0.0.1:1")
    monkeypatch.setenv("http_proxy", "http://127.0.0.1:1")
    try:
        ok, detail = health_probe(f"http://127.0.0.1:{port}/health")
        assert ok is True
        assert detail == "200"
    finally:
        httpd.shutdown()


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
    assert not any("compose.origin.yml" in " ".join(argv) for argv in compose_argv)


def test_publish_live_apps_use_origin_compose_and_rollback(monkeypatch, tmp_path: Path) -> None:
    origin = tmp_path / "origin"
    incoming = origin / "incoming" / "abc"
    previous = origin / "releases" / "old"
    incoming.mkdir(parents=True)
    previous.mkdir(parents=True)
    (previous / "dist").mkdir()
    live = incoming / "examples-live" / "live-counter"
    live.mkdir(parents=True)
    (live / "server").write_bytes(b"srv")
    (incoming / "islands").write_bytes(b"bin")

    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        info = tarfile.TarInfo("index.html")
        data = b"<html></html>"
        info.size = len(data)
        tar.addfile(info, io.BytesIO(data))
    (incoming / "site.tgz").write_bytes(buf.getvalue())

    docker = tmp_path / "repo" / "docker"
    (docker / "islands").mkdir(parents=True)
    (docker / "app").mkdir(parents=True)
    (docker / "islands" / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
    (docker / "app" / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
    (docker / "app" / "entrypoint.sh").write_text("#!/bin/sh\n", encoding="utf-8")
    (docker / "compose.hybrid.yml").write_text("services: {}\n", encoding="utf-8")
    (docker / "compose.origin.yml").write_text("services: {}\n", encoding="utf-8")
    (tmp_path / "repo" / "tools" / "rocci-ops").mkdir(parents=True)
    (tmp_path / "repo" / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")

    current = origin / "current"
    current.symlink_to(previous)

    monkeypatch.setenv("ROCCI_ORIGIN_ROOT", str(origin))
    monkeypatch.setenv("ROCCI_REPO_ROOT", str(tmp_path / "repo"))

    compose_argv: list[list[str]] = []
    compose_env: list[dict[str, str]] = []

    def runner(argv, **kwargs):
        compose_argv.append(list(argv))
        compose_env.append(kwargs.get("env", {}))
        return SimpleNamespace(returncode=0)

    def fetch(_url, timeout=5):
        raise OSError("unhealthy")

    try:
        publish("abc", runner=runner, fetch=fetch, sleeper=lambda _s: None)
    except SystemExit:
        pass
    else:
        raise AssertionError("expected SystemExit")
    assert any("compose.origin.yml" in argv for argv in compose_argv[0])
    assert compose_env[0]["ROCCI_LIVE_COUNTER_CONTEXT"].endswith("/releases/abc/examples-live/live-counter")
    assert compose_env[0]["ROCCI_IMAGE_TAG"] == "local"
    assert "--remove-orphans" in compose_argv[0]
    assert compose_argv[-1].count("-f") == 1


def test_publish_live_disabled_skips_origin_compose(monkeypatch, tmp_path: Path) -> None:
    origin = tmp_path / "origin"
    incoming = origin / "incoming" / "abc"
    incoming.mkdir(parents=True)
    live = incoming / "examples-live" / "live-counter"
    live.mkdir(parents=True)
    (live / "server").write_bytes(b"srv")
    (incoming / "islands").write_bytes(b"bin")

    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w:gz") as tar:
        info = tarfile.TarInfo("index.html")
        data = b"<html></html>"
        info.size = len(data)
        tar.addfile(info, io.BytesIO(data))
    (incoming / "site.tgz").write_bytes(buf.getvalue())

    docker = tmp_path / "repo" / "docker"
    (docker / "islands").mkdir(parents=True)
    (docker / "app").mkdir(parents=True)
    (docker / "islands" / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
    (docker / "app" / "Dockerfile").write_text("FROM scratch\n", encoding="utf-8")
    (docker / "app" / "entrypoint.sh").write_text("#!/bin/sh\n", encoding="utf-8")
    (docker / "compose.hybrid.yml").write_text("services: {}\n", encoding="utf-8")
    (docker / "compose.origin.yml").write_text("services: {}\n", encoding="utf-8")
    (tmp_path / "repo" / "tools" / "rocci-ops").mkdir(parents=True)
    (tmp_path / "repo" / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")

    monkeypatch.setenv("ROCCI_ORIGIN_ROOT", str(origin))
    monkeypatch.setenv("ROCCI_REPO_ROOT", str(tmp_path / "repo"))
    monkeypatch.setenv("ROCCI_LANE", "production")

    compose_argv: list[list[str]] = []

    def runner(argv, **_kwargs):
        compose_argv.append(list(argv))
        return SimpleNamespace(returncode=0)

    def fetch(_url, timeout=5):
        return FakeResponse(200)

    assert publish("abc", runner=runner, fetch=fetch, sleeper=lambda _s: None) == 0
    assert compose_argv
    assert not any("compose.origin.yml" in argv for argv in compose_argv)
    assert "--remove-orphans" in compose_argv[0]
