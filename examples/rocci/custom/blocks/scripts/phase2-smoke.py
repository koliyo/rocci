#!/usr/bin/env python3
"""Two-session join, duplicate lock, and SQLite integrity for Phase 2."""

from __future__ import annotations

import http.cookiejar
import json
import os
import sqlite3
import time
import urllib.error
import urllib.request

BASE = os.environ.get("BLOCKS_BASE", "http://127.0.0.1:8000")
DB_PATH = os.environ.get("DB_PATH", "/tmp/blocks-phase2.db")


def opener():
    jar = http.cookiejar.CookieJar()
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar)), jar


def call(op, method, path, data=None, headers=None):
    body = None if data is None else data.encode()
    req = urllib.request.Request(
        BASE + path,
        data=body,
        method=method,
        headers=headers or {},
    )
    try:
        with op.open(req) as resp:
            return resp.status, resp.read().decode()
    except urllib.error.HTTPError as err:
        return err.code, err.read().decode()


def main() -> int:
    a, _ = opener()
    b, _ = opener()
    status, html = call(a, "GET", "/play/blocks/")
    assert status == 200 and "Join" in html, html[:200]
    status, _ = call(a, "POST", "/play/blocks/join")
    assert status in (200, 303)
    status, _ = call(b, "POST", "/play/blocks/join")
    assert status in (200, 303)
    call(a, "POST", "/play/blocks/command/ready")
    call(b, "POST", "/play/blocks/command/ready")
    time.sleep(0.4)
    status, play = call(a, "GET", "/play/blocks/")
    assert status == 200 and "blocks-canvas" in play, play[:300]

    lock = {
        "piece": "I",
        "rotation": 0,
        "x": 3,
        "y": 18,
        "board_revision": 0,
        "sequence": 1,
    }
    headers = {"content-type": "application/json"}
    status, body = call(a, "POST", "/play/blocks/command/lock", json.dumps(lock), headers)
    first = json.loads(body)
    status2, body2 = call(a, "POST", "/play/blocks/command/lock", json.dumps(lock), headers)
    second = json.loads(body2)
    assert first.get("ok") in (0, 1)
    assert second.get("board") == first.get("board")
    assert second.get("revision") == first.get("revision")

    lock["sequence"] = 0
    status3, body3 = call(a, "POST", "/play/blocks/command/lock", json.dumps(lock), headers)
    assert status3 in (409, 200)
    third = json.loads(body3)
    assert third.get("board") == first.get("board")

    db = sqlite3.connect(DB_PATH)
    ok, *_ = db.execute("PRAGMA integrity_check").fetchone()
    assert ok == "ok", ok
    db.close()
    print("phase2-smoke ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
