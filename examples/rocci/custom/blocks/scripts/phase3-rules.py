#!/usr/bin/env python3
"""Deterministic Phase 3 attack, target, garbage, and serialize checks."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
WIDTH = 10
HEIGHT = 20
INSERT_CAP = 8
SEATS = 8


def load_family(name: str) -> list[dict]:
    return json.loads((FIXTURES / name).read_text())


def select_target(living: list[int], self_seat: int, cursor: int) -> int:
    for i in range(SEATS):
        seat = (cursor + i) % SEATS
        if seat != self_seat and seat in living:
            return seat
    return -1


def advance_cursor(living: list[int], self_seat: int, target: int) -> int:
    return select_target(living, self_seat, (target + 1) % SEATS)


def cancel_incoming(incoming: list[dict], attack: int) -> dict:
    leftover = attack
    kept: list[dict] = []
    for packet in sorted(incoming, key=lambda item: item["order"]):
        if leftover <= 0:
            kept.append(packet)
        elif leftover >= packet["rows"]:
            leftover -= packet["rows"]
        else:
            kept.append({**packet, "rows": packet["rows"] - leftover})
            leftover = 0
    return {
        "incoming": kept,
        "residual": leftover,
        "cursor_advanced": leftover > 0,
    }


def resolve_residual(cancelled: dict, self_seat: int, cursor: int, living: list[int]) -> dict:
    if cancelled["residual"] <= 0:
        return {
            "incoming": cancelled["incoming"],
            "residual": 0,
            "target": -1,
            "cursor": cursor,
            "writes": 0,
        }
    target = select_target(living, self_seat, cursor)
    if target < 0:
        return {
            "incoming": cancelled["incoming"],
            "residual": cancelled["residual"],
            "target": -1,
            "cursor": cursor,
            "writes": 0,
        }
    return {
        "incoming": cancelled["incoming"],
        "residual": cancelled["residual"],
        "target": target,
        "cursor": advance_cursor(living, self_seat, target),
        "writes": 1,
    }


def apply_ready(packets: list[dict], now: int) -> dict:
    ready = [p for p in packets if p["ready_at_ms"] <= now]
    applied = 0
    taken: list[dict] = []
    rest: list[dict] = []
    for packet in sorted(
        packets,
        key=lambda item: (item.get("order", 0), item.get("ready_at_ms", 0)),
    ):
        if packet["ready_at_ms"] > now or applied >= INSERT_CAP:
            rest.append(packet)
            continue
        room = INSERT_CAP - applied
        if packet["rows"] <= room:
            applied += packet["rows"]
            taken.append(packet)
        else:
            taken.append({**packet, "rows": room})
            rest.append({**packet, "rows": packet["rows"] - room})
            applied = INSERT_CAP
    return {
        "ready_rows": sum(p["rows"] for p in ready),
        "applied_rows": applied,
        "remaining": rest,
        "applied": taken,
    }


def garbage_row(hole: int) -> str:
    return "".join("." if x == hole else "G" for x in range(WIDTH))


def insert_garbage(board: str, packets: list[dict]) -> str:
    rows: list[str] = []
    for packet in packets:
        rows.extend(garbage_row(packet["hole"]) for _ in range(packet["rows"]))
    drop = len(rows) * WIDTH
    return board[drop:] + "".join(rows)


def next_hole(last: int) -> int:
    return 3 if last < 0 else (last + 3) % WIDTH


def spawn_ok(board: str, piece: str) -> bool:
    # I rot0 at (3,0) uses row 1; O rot0 uses rows 0-1 at x+1/+2.
    cells = {
        "I": [(3, 1), (4, 1), (5, 1), (6, 1)],
        "O": [(4, 0), (5, 0), (4, 1), (5, 1)],
    }[piece]
    for x, y in cells:
        if board[y * WIDTH + x] != ".":
            return False
    return True


def occupied_rows(board: str) -> int:
    return sum(
        any(board[y * WIDTH + x] != "." for x in range(WIDTH)) for y in range(HEIGHT)
    )


def packets_equal(actual: list[dict], expected: list[dict]) -> bool:
    if len(actual) != len(expected):
        return False
    for got, want in zip(actual, expected):
        for key, value in want.items():
            if got.get(key) != value:
                return False
    return True


def test_cancellation() -> None:
    for case in load_family("cancellation.json"):
        incoming = case["input"]["incoming"]
        cancelled = cancel_incoming(incoming, case["input"]["attack"])
        living = case["input"].get("living", [])
        self_seat = case["input"].get("self", 0)
        cursor = case["input"]["cursor"]
        resolved = resolve_residual(cancelled, self_seat, cursor, living)
        out = case["output"]
        assert packets_equal(resolved["incoming"], out["incoming"]), case["id"]
        assert resolved["residual"] == out["residual"], case["id"]
        assert resolved["cursor"] == out["cursor"], (case["id"], resolved["cursor"])
        assert resolved["writes"] <= 1, case["id"]
        if "target" in out:
            assert resolved["target"] == out["target"], case["id"]
        assert resolved["writes"] == (1 if out["residual"] > 0 and living else 0), case["id"]


def test_targets() -> None:
    for case in load_family("target-rotation.json"):
        inp = case["input"]
        living = inp.get("living") or inp["living_at_commit"]
        self_seat = inp["self"]
        cursor = inp["cursor"]
        target = select_target(living, self_seat, cursor)
        cursor_after = advance_cursor(living, self_seat, target)
        assert target == case["output"]["target"], case["id"]
        assert cursor_after == case["output"]["cursor_after_residual"], case["id"]
        assert (1 if target >= 0 else 0) <= 1


def test_garbage() -> None:
    for case in load_family("garbage-holes.json"):
        inp = case["input"]
        out = case["output"]
        if "packets" in inp:
            result = apply_ready(inp["packets"], inp["now_ms"])
            assert result["ready_rows"] == out["ready_rows"], case["id"]
            assert result["applied_rows"] == out["applied_rows"], case["id"]
            assert packets_equal(result["remaining"], out["remaining"]), case["id"]
        elif "previous_hole" in inp:
            assert next_hole(inp["previous_hole"]) == out["hole"], case["id"]
        else:
            assert garbage_row(inp["hole"]) == out["row"], case["id"]


def test_top_out() -> None:
    for case in load_family("top-out.json"):
        inp = case["input"]
        board = "".join(inp["board"])
        if "piece" in inp:
            x, y = inp["x"], inp["y"]
            # O at (3,0) overlaps the fixture T stack.
            blocked = board[y * WIDTH + (x + 1)] != "."
            assert blocked is True
            assert case["output"]["error"] == "InvalidGeometry"
        elif "apply_garbage_rows" in inp:
            packets = [
                {
                    "rows": inp["apply_garbage_rows"],
                    "ready_at_ms": 0,
                    "hole": 3,
                    "order": 0,
                }
            ]
            boarded = insert_garbage(board, packets)
            assert occupied_rows(boarded) == case["output"]["occupied_rows_before_spawn"]
            # Frozen I geometry occupies row 1, which stays empty at 16 garbage rows.
            assert spawn_ok(boarded, inp["next_piece"]) is True
            filled = insert_garbage(boarded, packets)
            assert spawn_ok(filled, inp["next_piece"]) is False
        else:
            assert spawn_ok(board, inp["next_piece"]) is False


def test_serialize_one_write() -> None:
    living = list(range(8))
    first = resolve_residual(cancel_incoming([], 4), 0, 0, [s for s in living if s != 0])
    queues = {seat: [] for seat in living}
    assert first["writes"] == 1
    queues[first["target"]].append({"rows": first["residual"], "order": 0})
    second = resolve_residual(cancel_incoming([], 4), 1, 1, [s for s in living if s != 1])
    assert second["writes"] == 1
    queues[second["target"]].append({"rows": second["residual"], "order": 1})
    total = sum(len(items) for items in queues.values())
    assert total == 2
    assert max(len(items) for items in queues.values()) >= 1


def test_three_and_eight_living() -> None:
    three = resolve_residual(cancel_incoming([], 2), 0, 1, [0, 2, 5])
    assert three["target"] == 2
    assert three["writes"] == 1
    eight = resolve_residual(cancel_incoming([], 4), 0, 0, list(range(8)))
    assert eight["target"] == 1
    assert eight["cursor"] == 2
    assert eight["writes"] == 1


def main() -> int:
    test_cancellation()
    test_targets()
    test_garbage()
    test_top_out()
    test_serialize_one_write()
    test_three_and_eight_living()
    print("phase3-rules ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
