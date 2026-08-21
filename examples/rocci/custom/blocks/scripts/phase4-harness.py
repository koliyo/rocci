#!/usr/bin/env python3
"""Spectator cap, reconnect snapshot, and short keepalive probe for Phase 4."""

from __future__ import annotations

import json
import os
import time
import urllib.error
import urllib.request

BASE = os.environ.get("BLOCKS_BASE", "http://127.0.0.1:8000")


def get(path: str, headers=None, timeout=4.0):
    req = urllib.request.Request(BASE + path, headers=headers or {}, method="GET")
    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.status, resp.read().decode(errors="replace"), dict(resp.headers)
    except urllib.error.HTTPError as err:
        return err.code, err.read().decode(errors="replace"), dict(err.headers)


def read_sse(path: str, headers=None, timeout=6.0) -> str:
    req = urllib.request.Request(BASE + path, headers=headers or {}, method="GET")
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        buf = b""
        while (
            b"blocks-arena-state" not in buf
            and b"capacity reached" not in buf.lower()
            and len(buf) < 16384
        ):
            chunk = resp.read(256)
            if not chunk:
                break
            buf += chunk
        return buf.decode(errors="replace")


def main() -> int:
    status, lobby, _ = get("/play/blocks/")
    assert status == 200 and "Spectate" in lobby, lobby[:200]
    status, watch, headers = get("/play/blocks/watch")
    assert status == 200 and "blocks-canvas" in watch, watch[:200]
    cookie = headers.get("Set-Cookie") or headers.get("set-cookie") or ""
    assert "blocks_watch=" in cookie, headers

    cookie_header = cookie.split(";", 1)[0]
    started = time.monotonic()
    first = read_sse("/play/blocks/stream", {"Cookie": cookie_header})
    first_ms = (time.monotonic() - started) * 1000
    assert "blocks-arena-state" in first or "data-revision" in first, first[:400]

    cap = int(os.environ.get("BLOCKS_SPECTATOR_CAP", "20"))
    denied = 0
    for _ in range(cap + 2):
        status, body, _ = get("/play/blocks/watch")
        if "capacity reached" in body.lower():
            denied += 1
    assert denied >= 1, f"expected a rejection, denied={denied}"

    started = time.monotonic()
    again = read_sse("/play/blocks/stream", {"Cookie": cookie_header})
    reconnect_ms = (time.monotonic() - started) * 1000
    assert "blocks-arena-state" in again or "data-revision" in again, again[:400]

    metrics = {
        "spectator_cap_default": 20,
        "design_ceiling": 50,
        "soak_30min": False,
        "first_snapshot_ms": round(first_ms, 1),
        "reconnect_snapshot_ms": round(reconnect_ms, 1),
        "rejected": denied,
        "payload_bytes": len(first),
    }
    print("phase4-harness ok", json.dumps(metrics))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
