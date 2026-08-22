Game := [].{
    width = 10.I64
    height = 20.I64
    spawn_x = 3.I64
    spawn_y = 0.I64
    empty_cell = 46
    gravity_ms = 800.I64

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
            _ => empty_cell
        }

    letter_of_byte = |byte|
        match byte {
            73 => "I"
            74 => "J"
            76 => "L"
            79 => "O"
            83 => "S"
            84 => "T"
            90 => "Z"
            _ => ""
        }

    next_seed = |seed| {
        n = I64.bitwise_and(seed, 2147483647.I64)
        I64.bitwise_and(I64.plus_saturated(I64.times(n, 1103515245.I64), 12345.I64), 2147483647.I64)
    }

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

    range = |n|
        range_help(n, [])

    range_help = |i, acc|
        if i <= 0 {
            acc
        } else {
            range_help(i - 1, List.prepend(acc, i - 1))
        }

    clear_lines = |board| {
        rows = range(height)
        kept = List.keep_if(rows, |y| row_full(row_slice(board, y)) == False)
        cleared = height - List.len(kept).to_i64_wrap()
        pad = repeat_char(empty_cell, cleared * width)
        kept_text = List.fold(kept, "", |acc, y| "${acc}${Str.from_utf8_lossy(row_slice(board, y))}")
        { board: "${pad}${kept_text}", cleared }
    }

    score_for = |cleared|
        match cleared {
            1 => 100
            2 => 300
            3 => 500
            4 => 800
            _ => 0
        }

    lock = |board, piece, rot, x, y|
        match piece_cells(piece, rot, x, y) {
            Err(err) => Err(err)
            Ok(cells) if can_place(board, cells) == False => Err(InvalidGeometry)
            Ok(cells) => {
                merged = merge(board, cells, piece)
                cleared = clear_lines(merged)
                Ok({
                    board: cleared.board,
                    cleared: cleared.cleared,
                    score: score_for(cleared.cleared),
                })
            }
        }

    spawn_ok = |board, piece|
        match piece_cells(piece, 0, spawn_x, spawn_y) {
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

    occupies = |cells, x, y|
        List.any(cells, |cell| cell.x == x and cell.y == y)

    start = |seed| {
        drawn = draw_piece([], seed)
        {
            board: empty_board,
            piece: drawn.piece,
            rot: 0.I64,
            x: spawn_x,
            y: spawn_y,
            bag: encode_bag(drawn.bag),
            seed: drawn.seed,
            score: 0.I64,
            status: "playing",
        }
    }

    peek_next = |bag, seed|
        draw_piece(decode_bag(bag), seed).piece

    playing = |state|
        state.status == "playing"

    ghost_y = |state|
        ghost_y_help(state, state.y)

    ghost_y_help = |state, y|
        match piece_cells(state.piece, state.rot, state.x, y + 1) {
            Ok(cells) if can_place(state.board, cells) => ghost_y_help(state, y + 1)
            _ => y
        }

    spawn_after = |board, bag, seed, score| {
        drawn = draw_piece(decode_bag(bag), seed)
        if spawn_ok(board, drawn.piece) {
            {
                board,
                piece: drawn.piece,
                rot: 0.I64,
                x: spawn_x,
                y: spawn_y,
                bag: encode_bag(drawn.bag),
                seed: drawn.seed,
                score,
                status: "playing",
            }
        } else {
            {
                board,
                piece: drawn.piece,
                rot: 0.I64,
                x: spawn_x,
                y: spawn_y,
                bag: encode_bag(drawn.bag),
                seed: drawn.seed,
                score,
                status: "topped",
            }
        }
    }

    commit_lock = |state|
        match lock(state.board, state.piece, state.rot, state.x, state.y) {
            Err(_) => { ..state, status: "topped" }
            Ok(result) => spawn_after(result.board, state.bag, state.seed, state.score + result.score)
        }

    shift = |state, dx, dy|
        if playing(state) == False {
            state
        } else {
            match piece_cells(state.piece, state.rot, state.x + dx, state.y + dy) {
                Ok(cells) if can_place(state.board, cells) =>
                    { ..state, x: state.x + dx, y: state.y + dy }
                _ => state
            }
        }

    rotate = |state, dir|
        if playing(state) == False {
            state
        } else {
            next_rot = (state.rot + dir + 4).rem_by(4)
            match try_kicks(state.board, state.piece, next_rot, state.x, state.y) {
                Ok(placed) => { ..state, rot: placed.rot, x: placed.x, y: placed.y }
                Err(_) => state
            }
        }

    tick = |state|
        if playing(state) == False {
            state
        } else {
            moved = shift(state, 0, 1)
            if moved.y == state.y {
                commit_lock(state)
            } else {
                moved
            }
        }

    apply_gravity = |state, steps|
        if steps <= 0 {
            state
        } else {
            apply_gravity(tick(state), steps - 1)
        }

    hard_drop = |state|
        if playing(state) == False {
            state
        } else {
            commit_lock({ ..state, y: ghost_y(state) })
        }

    overlay_rows = |state| {
        active =
            match piece_cells(state.piece, state.rot, state.x, state.y) {
                Ok(cells) => cells
                Err(_) => []
            }
        ghost =
            match piece_cells(state.piece, state.rot, state.x, ghost_y(state)) {
                Ok(cells) => cells
                Err(_) => []
            }
        List.map(
            range(height),
            |y| {
                y: y,
                cells: List.map(
                    range(width),
                    |x| {
                        x: x,
                        y: y,
                        class: overlay_class(state.board, state.piece, active, ghost, x, y),
                    },
                ),
            },
        )
    }

    overlay_class = |board, piece, active, ghost, x, y| {
        locked = letter_of_byte(cell_byte(board, x, y))
        if occupies(active, x, y) {
            "cell active piece-${piece}"
        } else if locked != "" {
            "cell locked piece-${locked}"
        } else if occupies(ghost, x, y) {
            "cell ghost piece-${piece}"
        } else {
            "cell empty"
        }
    }
}
