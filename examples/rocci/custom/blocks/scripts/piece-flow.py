#!/usr/bin/env python3
"""Verify solo join, legal locks, and one complete seven-bag against a running Blocks app."""

from __future__ import annotations

import http.cookiejar
import json
import os
import re
import sqlite3
import urllib.error
import urllib.request

BASE = os.environ.get("BLOCKS_BASE", "http://127.0.0.1:8000")
DB_PATH = os.environ.get("DB_PATH", "/tmp/blocks-piece-flow.db")
WIDTH = 10
HEIGHT = 20
OFFSETS = {
    "I": [(0, 1), (1, 1), (2, 1), (3, 1)],
    "O": [(1, 0), (2, 0), (1, 1), (2, 1)],
    "T": [(1, 0), (0, 1), (1, 1), (2, 1)],
    "S": [(1, 0), (2, 0), (0, 1), (1, 1)],
    "Z": [(0, 0), (1, 0), (1, 1), (2, 1)],
    "J": [(0, 0), (0, 1), (1, 1), (2, 1)],
    "L": [(2, 0), (0, 1), (1, 1), (2, 1)],
}


def opener():
    jar = http.cookiejar.CookieJar()
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))


def call(op, method: str, path: str, data: str | None = None):
    request = urllib.request.Request(
        BASE + path,
        data=None if data is None else data.encode(),
        method=method,
        headers={"content-type": "application/json"} if data is not None else {},
    )
    try:
        with op.open(request) as response:
            return response.status, response.read().decode()
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode()


def player():
    db = sqlite3.connect(DB_PATH)
    row = db.execute(
        "SELECT board, piece, board_revision, sequence, status FROM players ORDER BY seat LIMIT 1"
    ).fetchone()
    db.close()
    assert row is not None
    return row


def fits(board: str, piece: str, x: int, y: int) -> bool:
    return all(
        0 <= x + dx < WIDTH
        and 0 <= y + dy < HEIGHT
        and board[(y + dy) * WIDTH + x + dx] == "."
        for dx, dy in OFFSETS[piece]
    )


def landing(board: str, piece: str) -> tuple[int, int]:
    candidates = []
    for x in range(WIDTH):
        for y in range(HEIGHT):
            if fits(board, piece, x, y):
                candidates.append((y, x))
    assert candidates, f"no legal placement for {piece}: board is unexpectedly topped out"
    y, x = max(candidates)
    return x, y


def main() -> int:
    op = opener()
    status, lobby = call(op, "GET", "/play/blocks/")
    assert status == 200 and "Join" in lobby, lobby[:300]
    status, _ = call(op, "POST", "/play/blocks/join")
    assert status in (200, 303), status
    status, page = call(op, "GET", "/play/blocks/")
    assert status == 200 and 'data-phase="round"' in page, page[:500]
    initial = re.search(r'data-piece="([IJLOSTZ])"[^>]*data-board-revision="0"', page)
    assert initial, "joined player has no initial piece or board revision"

    played = []
    for expected_sequence in range(1, 8):
        board, piece, board_revision, sequence, status = player()
        assert status == "playing", status
        assert piece in OFFSETS, piece
        assert sequence == expected_sequence - 1, (sequence, expected_sequence)
        x, y = landing(board, piece)
        request = json.dumps(
            {
                "piece": piece,
                "rotation": 0,
                "x": x,
                "y": y,
                "board_revision": board_revision,
                "sequence": expected_sequence,
            }
        )
        status_code, body = call(op, "POST", "/play/blocks/command/lock", request)
        ack = json.loads(body)
        assert status_code == 200 and ack["ok"] == 1, (status_code, ack)
        assert ack["sequence"] == expected_sequence, ack
        assert ack["revision"] == board_revision + 1, ack
        assert ack["piece"] in OFFSETS, ack
        assert ack["board"] != board, ack
        played.append(piece)

    assert set(played) == set(OFFSETS), played
    print("piece-flow ok", "".join(played))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
