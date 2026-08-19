#!/usr/bin/env python3
"""Fixture adapter for rocci-browser host tests. Serves a static hello origin."""

from __future__ import annotations

import json
import os
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Optional


ADAPTER_ID = os.environ.get("ROCCI_BROWSER_FIXTURE_ID", "fixture")
LABEL = os.environ.get("ROCCI_BROWSER_FIXTURE_LABEL", "Fixture")
DOCUMENTS = [
    {
        "id": "home",
        "title": "Home",
        "path": "index.html",
        "route": "/",
    },
    {
        "id": "about",
        "title": "About",
        "path": "about.html",
        "route": "/about",
    },
]

httpd: Optional[ThreadingHTTPServer] = None


class HelloHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802
        path = self.path.split("?", 1)[0].rstrip("/") or "/"
        body = b"about" if path == "/about" else b"hello"
        self.send_response(200)
        self.send_header("Content-Type", "text/plain; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return


def ensure_server() -> tuple[str, str]:
    global httpd
    if httpd is None:
        httpd = ThreadingHTTPServer(("127.0.0.1", 0), HelloHandler)
        thread = threading.Thread(target=httpd.serve_forever, daemon=True)
        thread.start()
    port = httpd.server_address[1]
    return f"http://127.0.0.1:{port}", f"http://127.0.0.1:{port}/inspect"


def reply(msg_id: object, result: object) -> None:
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": msg_id, "result": result}) + "\n")
    sys.stdout.flush()


def main() -> None:
    for raw in sys.stdin:
        line = raw.strip()
        if not line:
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        method = message.get("method")
        msg_id = message.get("id")
        params = message.get("params") or {}
        if method == "initialize":
            reply(
                msg_id,
                {
                    "protocolVersion": 1,
                    "adapterId": ADAPTER_ID,
                    "capabilities": ["probe", "listDocuments", "open", "shutdown"],
                },
            )
        elif method == "probe":
            path = params.get("path") or ""
            if os.path.isdir(path):
                reply(msg_id, {"claimed": True, "label": LABEL})
            else:
                reply(msg_id, {"claimed": False})
        elif method == "listDocuments":
            reply(msg_id, {"documents": DOCUMENTS})
        elif method == "open":
            origin, inspector = ensure_server()
            document = params.get("document")
            if document == "about":
                reply(
                    msg_id,
                    {
                        "url": f"{origin}/about",
                        "title": "About",
                        "inspectorUrl": inspector,
                    },
                )
            else:
                reply(
                    msg_id,
                    {"url": f"{origin}/", "title": "Hello", "inspectorUrl": inspector},
                )
        elif method == "shutdown":
            global httpd
            if httpd is not None:
                httpd.shutdown()
                httpd = None
            reply(msg_id, {})
            return
        else:
            continue


if __name__ == "__main__":
    main()
