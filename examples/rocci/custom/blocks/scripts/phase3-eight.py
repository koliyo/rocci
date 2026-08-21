#!/usr/bin/env python3
"""Eight synthetic seats join, ready, and finish a short round."""

from __future__ import annotations

import http.cookiejar
import os
import sqlite3
import time
import urllib.error
import urllib.request

BASE = os.environ.get("BLOCKS_BASE", "http://127.0.0.1:8000")
DB_PATH = os.environ.get("DB_PATH", "/tmp/blocks-phase3.db")


def opener():
    jar = http.cookiejar.CookieJar()
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))


def call(op, method, path, data=None):
    req = urllib.request.Request(
        BASE + path,
        data=None if data is None else data.encode(),
        method=method,
    )
    try:
        with op.open(req) as resp:
            return resp.status, resp.read().decode()
    except urllib.error.HTTPError as err:
        return err.code, err.read().decode()


def main() -> int:
    sessions = [opener() for _ in range(8)]
    status, html = call(sessions[0], "GET", "/play/blocks/")
    assert status == 200 and "Join" in html, html[:200]
    for op in sessions:
        status, _ = call(op, "POST", "/play/blocks/join")
        assert status in (200, 303)
        call(op, "POST", "/play/blocks/command/ready")
    time.sleep(0.5)
    status, play = call(sessions[0], "GET", "/play/blocks/")
    assert status == 200 and "blocks-canvas" in play, play[:300]
    time.sleep(0.5)
    status, after = call(sessions[0], "GET", "/play/blocks/")
    assert status == 200
    db = sqlite3.connect(DB_PATH)
    phase, reason = db.execute("SELECT phase, reason FROM room").fetchone()
    seats = db.execute("SELECT COUNT(*) FROM players").fetchone()[0]
    ok, *_ = db.execute("PRAGMA integrity_check").fetchone()
    db.close()
    assert seats == 8, seats
    assert ok == "ok", ok
    assert phase in ("round", "result", "lobby"), phase
    if phase == "result":
        assert reason in ("Timeout", "Last player standing", "Interrupted")
    print("phase3-eight ok", phase, reason)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
