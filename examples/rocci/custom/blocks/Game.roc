Game := [].{
    width = 10.I64
    height = 20.I64
    spawn_x = 3.I64
    spawn_y = 0.I64
    empty_cell = 46
    garbage_cell = 71

    empty_board =
        repeat_char(empty_cell, width * height)

    valid_piece = |piece|
        match piece {
            "I" => True
            "J" => True
            "L" => True
            "O" => True
            "S" => True
            "T" => True
            "Z" => True
            _ => False
        }

    piece_byte = |piece|
        match piece {
            "I" => 73
            "J" => 74
            "L" => 76
            "O" => 79
            "S" => 83
            "T" => 84
            "Z" => 90
            "G" => garbage_cell
            _ => empty_cell
        }

    from_rows = |rows|
        List.fold(rows, "", |acc, row| "${acc}${row}")

    next_seed = |seed|
        I64.bitwise_and(I64.plus_saturated(I64.times(seed, 1103515245.I64), 12345.I64), 2147483647.I64)

    rand_range = |seed, lo, hi| {
        span = hi - lo + 1
        n = next_seed(seed)
        if span <= 0 {
            { value: lo, seed: n }
        } else {
            { value: lo + n.rem_by(span), seed: n }
        }
    }

    offsets = |piece, rot|
        match piece {
            "I" => i_offsets(rot)
            "O" => [{ x: 1, y: 0 }, { x: 2, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 1 }]
            "T" => t_offsets(rot)
            "S" => s_offsets(rot)
            "Z" => z_offsets(rot)
            "J" => j_offsets(rot)
            "L" => l_offsets(rot)
            _ => []
        }

    i_offsets = |rot|
        match rot {
            0 => [{ x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 3, y: 1 }]
            1 => [{ x: 2, y: 0 }, { x: 2, y: 1 }, { x: 2, y: 2 }, { x: 2, y: 3 }]
            2 => [{ x: 0, y: 2 }, { x: 1, y: 2 }, { x: 2, y: 2 }, { x: 3, y: 2 }]
            3 => [{ x: 1, y: 0 }, { x: 1, y: 1 }, { x: 1, y: 2 }, { x: 1, y: 3 }]
            _ => []
        }

    t_offsets = |rot|
        match rot {
            0 => [{ x: 1, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }]
            1 => [{ x: 1, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 1, y: 2 }]
            2 => [{ x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 1, y: 2 }]
            3 => [{ x: 1, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 1, y: 2 }]
            _ => []
        }

    s_offsets = |rot|
        match rot {
            0 => [{ x: 1, y: 0 }, { x: 2, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }]
            1 => [{ x: 1, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 2, y: 2 }]
            2 => [{ x: 1, y: 1 }, { x: 2, y: 1 }, { x: 0, y: 2 }, { x: 1, y: 2 }]
            3 => [{ x: 0, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 1, y: 2 }]
            _ => []
        }

    z_offsets = |rot|
        match rot {
            0 => [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 1 }]
            1 => [{ x: 2, y: 0 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 1, y: 2 }]
            2 => [{ x: 0, y: 1 }, { x: 1, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 2 }]
            3 => [{ x: 1, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 0, y: 2 }]
            _ => []
        }

    j_offsets = |rot|
        match rot {
            0 => [{ x: 0, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }]
            1 => [{ x: 1, y: 0 }, { x: 2, y: 0 }, { x: 1, y: 1 }, { x: 1, y: 2 }]
            2 => [{ x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 2, y: 2 }]
            3 => [{ x: 1, y: 0 }, { x: 1, y: 1 }, { x: 0, y: 2 }, { x: 1, y: 2 }]
            _ => []
        }

    l_offsets = |rot|
        match rot {
            0 => [{ x: 2, y: 0 }, { x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }]
            1 => [{ x: 1, y: 0 }, { x: 1, y: 1 }, { x: 1, y: 2 }, { x: 2, y: 2 }]
            2 => [{ x: 0, y: 1 }, { x: 1, y: 1 }, { x: 2, y: 1 }, { x: 0, y: 2 }]
            3 => [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }, { x: 1, y: 2 }]
            _ => []
        }

    kicks = [
        { x: 0, y: 0 },
        { x: -1, y: 0 },
        { x: 1, y: 0 },
        { x: 0, y: 1 },
        { x: -1, y: 1 },
        { x: 1, y: 1 },
        { x: 0, y: -1 },
    ]

    nth = |list, index, current|
        if current == index {
            match list {
                [head, ..] => Ok(head)
                [] => Err(OutOfBounds)
            }
        } else {
            match list {
                [_, .. as tail] => nth(tail, index, current + 1)
                [] => Err(OutOfBounds)
            }
        }

    drop_n = |list, n|
        if n <= 0 {
            list
        } else {
            match list {
                [] => []
                [_, .. as tail] => drop_n(tail, n - 1)
            }
        }

    piece_cells = |piece, rot, x, y|
        if valid_piece(piece) == False {
            Err(UnknownPiece)
        } else if rot < 0 or rot > 3 {
            Err(BadRotation)
        } else {
            cells = List.map(offsets(piece, rot), |off| { x: x + off.x, y: y + off.y })
            if List.any(cells, |cell| cell.x < 0 or cell.x >= width or cell.y < 0 or cell.y >= height) {
                Err(InvalidGeometry)
            } else {
                Ok(cells)
            }
        }

    cell_byte = |board, x, y| {
        bytes = Str.to_utf8(board)
        match nth(bytes, y * width + x, 0) {
            Ok(byte) => byte
            Err(_) => empty_cell
        }
    }

    can_place = |board, cells|
        List.all(
            cells,
            |cell| cell_byte(board, cell.x, cell.y) == empty_cell,
        )

    replace_byte = |bytes, target, byte, index, acc|
        match bytes {
            [] => acc
            [head, .. as tail] => {
                next = if index == target { byte } else { head }
                replace_byte(tail, target, byte, index + 1, List.append(acc, next))
            }
        }

    merge = |board, cells, piece| {
        fill = piece_byte(piece)
        List.fold(
            cells,
            board,
            |acc, cell| {
                bytes = Str.to_utf8(acc)
                Str.from_utf8_lossy(replace_byte(bytes, cell.y * width + cell.x, fill, 0, []))
            },
        )
    }

    row_slice = |board, y| {
        bytes = Str.to_utf8(board)
        start = y * width
        take_bytes(drop_n(bytes, start), width, [])
    }

    take_bytes = |bytes, n, acc|
        if n <= 0 {
            acc
        } else {
            match bytes {
                [] => acc
                [head, .. as tail] => take_bytes(tail, n - 1, List.append(acc, head))
            }
        }

    row_full = |row|
        List.all(row, |byte| byte != empty_cell)

    repeat_char = |byte, n|
        Str.from_utf8_lossy(repeat_byte(byte, n, []))

    repeat_byte = |byte, n, acc|
        if n <= 0 {
            acc
        } else {
            repeat_byte(byte, n - 1, List.append(acc, byte))
        }

    clear_lines = |board| {
        rows = range(height)
        kept = List.keep_if(rows, |y| row_full(row_slice(board, y)) == False)
        cleared = height - List.len(kept).to_i64_wrap()
        pad = repeat_char(empty_cell, cleared * width)
        kept_text = List.fold(kept, "", |acc, y| "${acc}${Str.from_utf8_lossy(row_slice(board, y))}")
        { board: "${pad}${kept_text}", cleared }
    }

    range = |n|
        range_help(n, [])

    range_help = |i, acc|
        if i <= 0 {
            acc
        } else {
            range_help(i - 1, List.prepend(acc, i - 1))
        }

    attack_for = |cleared, back_to_back| {
        base =
            match cleared {
                2 => 1
                3 => 2
                4 => 4
                _ => 0
            }
        bonus = if back_to_back and cleared == 4 { 1 } else { 0 }
        next_b2b =
            if cleared == 4 {
                True
            } else if cleared == 0 {
                back_to_back
            } else {
                False
            }
        { attack: base + bonus, back_to_back: next_b2b }
    }

    lock = |board, piece, rot, x, y, back_to_back|
        match piece_cells(piece, rot, x, y) {
            Err(err) => Err(err)
            Ok(cells) if can_place(board, cells) == False => Err(InvalidGeometry)
            Ok(cells) => {
                merged = merge(board, cells, piece)
                cleared = clear_lines(merged)
                attack = attack_for(cleared.cleared, back_to_back)
                Ok({
                    board: cleared.board,
                    cleared: cleared.cleared,
                    attack: attack.attack,
                    back_to_back: attack.back_to_back,
                })
            }
        }

    spawn_cells = |piece|
        piece_cells(piece, 0, spawn_x, spawn_y)

    spawn_ok = |board, piece|
        match spawn_cells(piece) {
            Ok(cells) => can_place(board, cells)
            Err(_) => False
        }

    seven = ["I", "J", "L", "O", "S", "T", "Z"]

    replace_at = |items, target, value, index, acc|
        match items {
            [] => acc
            [head, .. as tail] => {
                next = if index == target { value } else { head }
                replace_at(tail, target, value, index + 1, List.append(acc, next))
            }
        }

    swap_index = |items, i, j|
        if i == j {
            items
        } else {
            match (nth(items, i, 0), nth(items, j, 0)) {
                (Ok(a), Ok(b)) => replace_at(replace_at(items, i, b, 0, []), j, a, 0, [])
                _ => items
            }
        }

    scramble = |items, seed| {
        n = List.len(items).to_i64_wrap()
        fisher(items, seed, n - 1)
    }

    fisher = |items, seed, i|
        if i <= 0 {
            { pieces: items, seed }
        } else {
            rolled = rand_range(seed, 0, i)
            fisher(swap_index(items, i, rolled.value), rolled.seed, i - 1)
        }

    refill_bag = |seed|
        scramble(seven, seed)

    encode_bag = |pieces|
        List.fold(pieces, "", |acc, piece| "${acc}${piece}")

    decode_bag = |text|
        List.keep_if(
            List.map(Str.to_utf8(text), |byte| Str.from_utf8_lossy([byte])),
            |piece| valid_piece(piece),
        )

    draw_piece = |bag, seed|
        match bag {
            [piece, .. as rest] => { piece, bag: rest, seed }
            [] => {
                filled = refill_bag(seed)
                match filled.pieces {
                    [piece, .. as rest] => { piece, bag: rest, seed: filled.seed }
                    [] => { piece: "I", bag: [], seed: filled.seed }
                }
            }
        }

    occupied_rows = |board|
        List.len(List.keep_if(range(height), |y| List.any(row_slice(board, y), |byte| byte != empty_cell))).to_i64_wrap()

    try_kicks = |board, piece, rot, x, y|
        try_kicks_help(board, piece, rot, x, y, kicks)

    try_kicks_help = |board, piece, rot, x, y, remaining|
        match remaining {
            [] => Err(InvalidGeometry)
            [kick, .. as rest] =>
                match piece_cells(piece, rot, x + kick.x, y + kick.y) {
                    Ok(cells) if can_place(board, cells) => Ok({ x: x + kick.x, y: y + kick.y, rot })
                    _ => try_kicks_help(board, piece, rot, x, y, rest)
                }
        }

    garbage_delay_ms = 600.I64
    insert_cap = 8.I64
    seat_count = 8.I64
    first_hole = 3.I64

    next_hole = |last|
        if last < 0 {
            first_hole
        } else {
            (last + 3).rem_by(width)
        }

    garbage_row = |hole| {
        bytes = range(width)
        Str.from_utf8_lossy(
            List.map(
                bytes,
                |x| if x == hole { empty_cell } else { garbage_cell },
            ),
        )
    }

    select_target = |living, self, cursor|
        List.fold(
            range(seat_count),
            -1,
            |acc, i|
                if acc >= 0 {
                    acc
                } else {
                    seat = (cursor + i).rem_by(seat_count)
                    if seat != self and List.contains(living, seat) {
                        seat
                    } else {
                        acc
                    }
                },
        )

    advance_cursor = |living, self, target|
        select_target(living, self, (target + 1).rem_by(seat_count))

    insert_ordered = |acc, packet| {
        before = List.keep_if(
            acc,
            |item| item.order < packet.order or (item.order == packet.order and item.ready_at_ms <= packet.ready_at_ms),
        )
        after = List.keep_if(
            acc,
            |item| item.order > packet.order or (item.order == packet.order and item.ready_at_ms > packet.ready_at_ms),
        )
        List.concat(List.append(before, packet), after)
    }

    sort_packets = |packets|
        List.fold(packets, [], insert_ordered)

    cancel_incoming = |incoming, attack| {
        sorted = sort_packets(incoming)
        folded =
            List.fold(
                sorted,
                { leftover: attack, kept: [] },
                |acc, packet|
                    if acc.leftover <= 0 {
                        { leftover: 0, kept: List.append(acc.kept, packet) }
                    } else if acc.leftover >= packet.rows {
                        { leftover: acc.leftover - packet.rows, kept: acc.kept }
                    } else {
                        {
                            leftover: 0,
                            kept: List.append(acc.kept, { ..packet, rows: packet.rows - acc.leftover }),
                        }
                    },
            )
        { incoming: folded.kept, residual: folded.leftover, cursor_advanced: folded.leftover > 0 }
    }

    resolve_residual = |cancelled, self, cursor, living| {
        if cancelled.residual <= 0 {
            {
                incoming: cancelled.incoming,
                residual: 0,
                target: -1,
                cursor,
                writes: 0,
            }
        } else {
            target = select_target(living, self, cursor)
            if target < 0 {
                {
                    incoming: cancelled.incoming,
                    residual: cancelled.residual,
                    target: -1,
                    cursor,
                    writes: 0,
                }
            } else {
                {
                    incoming: cancelled.incoming,
                    residual: cancelled.residual,
                    target,
                    cursor: advance_cursor(living, self, target),
                    writes: 1,
                }
            }
        }
    }

    packet_rows = |packets|
        List.fold(packets, 0, |acc, packet| acc + packet.rows)

    apply_ready = |packets, now| {
        sorted = sort_packets(packets)
        ready = List.keep_if(sorted, |packet| packet.ready_at_ms <= now)
        folded =
            List.fold(
                sorted,
                { applied: 0.I64, taken: [], rest: [] },
                |acc, packet|
                    if packet.ready_at_ms > now {
                        { applied: acc.applied, taken: acc.taken, rest: List.append(acc.rest, packet) }
                    } else if acc.applied >= insert_cap {
                        { applied: acc.applied, taken: acc.taken, rest: List.append(acc.rest, packet) }
                    } else {
                        room = insert_cap - acc.applied
                        if packet.rows <= room {
                            {
                                applied: acc.applied + packet.rows,
                                taken: List.append(acc.taken, packet),
                                rest: acc.rest,
                            }
                        } else {
                            {
                                applied: insert_cap,
                                taken: List.append(acc.taken, { ..packet, rows: room }),
                                rest: List.append(acc.rest, { ..packet, rows: packet.rows - room }),
                            }
                        }
                    },
            )
        {
            ready_rows: packet_rows(ready),
            applied_rows: folded.applied,
            remaining: folded.rest,
            applied: folded.taken,
        }
    }

    insert_garbage = |board, packets| {
        rows =
            List.fold(
                packets,
                [],
                |acc, packet|
                    List.concat(acc, List.map(range(packet.rows), |_| garbage_row(packet.hole))),
            )
        drop = List.len(rows).to_i64_wrap() * width
        bytes = drop_n(Str.to_utf8(board), drop)
        extra = List.fold(rows, [], |acc, row| List.concat(acc, Str.to_utf8(row)))
        Str.from_utf8_lossy(List.concat(bytes, extra))
    }

    next_order = |packets|
        List.fold(packets, -1, |acc, packet| if packet.order > acc { packet.order } else { acc }) + 1

    encode_queue = |packets|
        List.fold(
            packets,
            "",
            |acc, packet| {
                part = "${packet.rows.to_str()},${packet.ready_at_ms.to_str()},${packet.hole.to_str()},${packet.order.to_str()}"
                if acc == "" {
                    part
                } else {
                    "${acc}|${part}"
                }
            },
        )

    parse_i64 = |text|
        match I64.from_str(text) {
            Ok(n) => n
            Err(_) => 0
        }

    decode_packet = |text| {
        parts = Str.split_on(text, ",")
        match (nth(parts, 0, 0), nth(parts, 1, 0), nth(parts, 2, 0), nth(parts, 3, 0)) {
            (Ok(rows), Ok(ready_at), Ok(hole), Ok(order)) =>
                Ok({
                    rows: parse_i64(rows),
                    ready_at_ms: parse_i64(ready_at),
                    hole: parse_i64(hole),
                    order: parse_i64(order),
                })
            _ => Err(InvalidGeometry)
        }
    }

    decode_queue = |text|
        if text == "" {
            []
        } else {
            List.fold(
                Str.split_on(text, "|"),
                [],
                |acc, part|
                    match decode_packet(part) {
                        Ok(packet) if packet.rows > 0 => List.append(acc, packet)
                        _ => acc
                    },
            )
        }

    queue_rows = |text|
        packet_rows(decode_queue(text))

    ready_rows_now = |text, now|
        packet_rows(List.keep_if(decode_queue(text), |packet| packet.ready_at_ms <= now))
}
