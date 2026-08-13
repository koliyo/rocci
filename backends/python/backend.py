#!/usr/bin/env python3
"""Standard-library reference implementation of the Roc backend contract."""

from __future__ import annotations

import argparse
import http.cookies
import http.server
import pathlib
import queue
import secrets
import socketserver
import sys
import threading
import urllib.parse


SECURITY_HEADERS = {
    "Cache-Control": "no-store",
    "Content-Security-Policy": (
        "default-src 'self'; script-src 'self' 'unsafe-eval'; "
        "connect-src 'self'; img-src 'self' data:; style-src 'self'; "
        "object-src 'none'; base-uri 'none'; frame-ancestors 'none'"
    ),
    "Referrer-Policy": "no-referrer",
    "X-Content-Type-Options": "nosniff",
    "X-Frame-Options": "DENY",
}


def datastar_counter(count: int) -> str:
    return f'''<section id="counter" class="counter-card">
  <p class="eyebrow">Shared Python state</p>
  <output>{count}</output>
  <p>Every window connected to the event stream sees the same value.</p>
</section>'''


def htmx_counter(count: int) -> str:
    return f'<output id="htmx-counter">{count}</output>'


def patch_elements(elements: str) -> bytes:
    lines = ["event: datastar-patch-elements"]
    lines.extend(f"data: elements {line}" for line in elements.splitlines())
    return ("\n".join(lines) + "\n\n").encode()


class State:
    def __init__(self, assets: pathlib.Path, templates: pathlib.Path) -> None:
        self.assets = assets
        self.templates = templates
        self.token = secrets.token_hex(32)
        self.host = ""
        self.count = 0
        self.lock = threading.Lock()
        self.subscribers: list[queue.Queue[int]] = []

    def update(self, value: int | None = None) -> int:
        with self.lock:
            self.count = self.count + 1 if value is None else value
            count = self.count
            subscribers = list(self.subscribers)
        for subscriber in subscribers:
            subscriber.put(count)
        return count

    def current(self) -> int:
        with self.lock:
            return self.count


class ThreadingServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = False

    def handle_error(self, request: object, client_address: object) -> None:
        error = sys.exception()
        if isinstance(error, (BrokenPipeError, ConnectionResetError)):
            return
        super().handle_error(request, client_address)


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    server_version = "RocPythonBackend/0.1"

    @property
    def state(self) -> State:
        return self.server.state  # type: ignore[attr-defined]

    def log_message(self, message: str, *args: object) -> None:
        print(f"python-backend: {message % args}", file=sys.stderr)

    def end_headers(self) -> None:
        for key, value in SECURITY_HEADERS.items():
            self.send_header(key, value)
        super().end_headers()

    def do_GET(self) -> None:
        path = urllib.parse.urlsplit(self.path).path
        if path.startswith("/_roc/bootstrap/"):
            self.bootstrap(path)
        elif not self.authorized(mutation=False):
            self.text(401, "desktop session required")
        elif path == "/":
            self.page("datastar.html", datastar_counter(self.state.current()))
        elif path == "/health":
            self.text(200, "ok")
        elif path == "/api/counter/events":
            self.events()
        elif path == "/htmx":
            self.page("htmx.html", htmx_counter(self.state.current()))
        elif path.startswith("/assets/"):
            self.asset(path.removeprefix("/assets/"))
        else:
            self.text(404, "not found")

    def do_POST(self) -> None:
        # BaseHTTPRequestHandler leaves request bodies unread. Drain Datastar's
        # JSON signals before the persistent connection is reused for another
        # request, otherwise the next request line would begin with `{}`.
        content_length = int(self.headers.get("Content-Length", "0"))
        if content_length:
            self.rfile.read(content_length)
        path = urllib.parse.urlsplit(self.path).path
        if not self.authorized(mutation=True):
            self.text(401, "desktop session required")
        elif path == "/api/counter/increment":
            self.sse_once(self.state.update())
        elif path == "/api/counter/reset":
            self.sse_once(self.state.update(0))
        elif path == "/htmx/counter/increment":
            self.html(200, htmx_counter(self.state.update()))
        else:
            self.text(404, "not found")

    def bootstrap(self, path: str) -> None:
        supplied = path.removeprefix("/_roc/bootstrap/")
        if self.headers.get("Host") != self.state.host or not secrets.compare_digest(
            supplied, self.state.token
        ):
            self.text(404, "not found")
            return
        self.send_response(303)
        self.send_header("Location", "/")
        self.send_header(
            "Set-Cookie",
            f"roc_session={self.state.token}; HttpOnly; SameSite=Strict; Path=/",
        )
        self.send_header("Content-Length", "0")
        self.end_headers()

    def authorized(self, mutation: bool) -> bool:
        if self.headers.get("Host") != self.state.host:
            return False
        cookie = http.cookies.SimpleCookie(self.headers.get("Cookie", ""))
        session = cookie.get("roc_session")
        if session is None or not secrets.compare_digest(session.value, self.state.token):
            return False
        return not mutation or self.headers.get("Origin") == f"http://{self.state.host}"

    def events(self) -> None:
        subscriber: queue.Queue[int] = queue.Queue()
        with self.state.lock:
            self.state.subscribers.append(subscriber)
            initial = self.state.count
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Connection", "keep-alive")
        self.end_headers()
        try:
            self.wfile.write(patch_elements(datastar_counter(initial)))
            self.wfile.flush()
            while True:
                try:
                    count = subscriber.get(timeout=15)
                    self.wfile.write(patch_elements(datastar_counter(count)))
                except queue.Empty:
                    self.wfile.write(b": keep-alive\n\n")
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            pass
        finally:
            with self.state.lock:
                if subscriber in self.state.subscribers:
                    self.state.subscribers.remove(subscriber)

    def sse_once(self, count: int) -> None:
        body = patch_elements(datastar_counter(count))
        self.response(200, "text/event-stream", body)

    def page(self, name: str, counter: str) -> None:
        template = (self.state.templates / name).read_text(encoding="utf-8")
        self.html(200, template.replace("{{counter}}", counter))

    def asset(self, name: str) -> None:
        allowed = {
            "app.css": "text/css; charset=utf-8",
            "datastar.js": "text/javascript; charset=utf-8",
            "htmx.min.js": "text/javascript; charset=utf-8",
        }
        content_type = allowed.get(name)
        if content_type is None:
            self.text(404, "not found")
            return
        self.response(200, content_type, (self.state.assets / name).read_bytes())

    def html(self, status: int, body: str) -> None:
        self.response(status, "text/html; charset=utf-8", body.encode())

    def text(self, status: int, body: str) -> None:
        self.response(status, "text/plain; charset=utf-8", body.encode())

    def response(self, status: int, content_type: str, body: bytes) -> None:
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--assets", type=pathlib.Path, required=True)
    args = parser.parse_args()
    templates = pathlib.Path(__file__).resolve().parent / "templates"
    state = State(args.assets.resolve(), templates)
    server = ThreadingServer(("127.0.0.1", 0), Handler)
    server.state = state  # type: ignore[attr-defined]
    state.host = f"127.0.0.1:{server.server_port}"
    print(
        f"ROC_BACKEND_READY http://{state.host}/_roc/bootstrap/{state.token}",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
