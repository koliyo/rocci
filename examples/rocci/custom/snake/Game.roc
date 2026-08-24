Game := [].{
    max_players = 8.I64
    tick_ms = 125.I64
    respawn_ticks = 16.I64
    grow_every = 16.I64
    view_width = 31.I64
    view_height = 21.I64
    cam_margin = 3.I64
    world_size = 100.I64
    food_target = 10.I64
    init_length = 3.I64
    spawn_lo = 20.I64
    spawn_hi = 80.I64

    valid_dir = |s|
        s == "up" or s == "down" or s == "left" or s == "right"

    color_label = |color|
        match color {
            "c1" => "Jade"
            "c2" => "Mint"
            "c3" => "Gold"
            "c4" => "Violet"
            "c5" => "Coral"
            "c6" => "Lime"
            "c7" => "Rose"
            "c8" => "Sky"
            _ => "Snake"
        }

    next_color = |used| {
        palette = ["c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8"]
        List.fold(
            palette,
            "",
            |acc, color|
                if acc != "" {
                    acc
                } else if List.contains(used, color) {
                    acc
                } else {
                    color
                },
        )
    }

    encode_body = |body|
        List.fold(
            body,
            "",
            |acc, point| {
                part = "${point.x.to_str()},${point.y.to_str()}"
                if acc == "" {
                    part
                } else {
                    "${acc}|${part}"
                }
            },
        )

    decode_body = |text| {
        if text == "" {
            []
        } else {
            List.fold(
                Str.split_on(text, "|"),
                [],
                |acc, part| {
                    coords = Str.split_on(part, ",")
                    match (List.get(coords, 0), List.get(coords, 1)) {
                        (Ok(xs), Ok(ys)) =>
                            match (I64.from_str(xs), I64.from_str(ys)) {
                                (Ok(x), Ok(y)) => List.append(acc, { x, y })
                                _ => acc
                            }
                        _ => acc
                    }
                },
            )
        }
    }

    from_row = |row| {
        {
            id: row.id,
            name: row.name,
            color: row.color,
            dir: row.dir,
            pending_dir: row.pending_dir,
            alive: row.alive != 0.I64,
            respawn_in: row.respawn_in,
            body: decode_body(row.body),
            score: row.score,
            cam: { x: row.cam_x, y: row.cam_y },
        }
    }

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

    clamp = |n, lo, hi|
        if n < lo {
            lo
        } else if n > hi {
            hi
        } else {
            n
        }

    in_bounds = |point|
        point.x >= 0
        and point.x < world_size
        and point.y >= 0
        and point.y < world_size

    world_center = ||
        { x: world_size.div_trunc_by(2), y: world_size.div_trunc_by(2) }

    translate = |point, dir|
        match dir {
            "up" => { x: point.x, y: point.y - 1 }
            "down" => { x: point.x, y: point.y + 1 }
            "left" => { x: point.x - 1, y: point.y }
            _ => { x: point.x + 1, y: point.y }
        }

    opposite = |dir|
        match dir {
            "up" => "down"
            "down" => "up"
            "left" => "right"
            _ => "left"
        }

    is_reverse = |a, b|
        (a == "up" and b == "down")
        or (a == "down" and b == "up")
        or (a == "left" and b == "right")
        or (a == "right" and b == "left")

    same_point = |a, b|
        a.x == b.x and a.y == b.y

    drop_last = |list|
        List.drop_last(list, 1)

    drop_first = |list|
        List.drop_first(list, 1)

    range = |n|
        range_help(n, [])

    range_help = |i, acc|
        if i <= 0 {
            acc
        } else {
            range_help(i - 1, List.prepend(acc, i - 1))
        }

    dir_at = |seed| {
        rolled = rand_range(seed, 0, 3)
        dir =
            match rolled.value {
                0 => "up"
                1 => "down"
                2 => "left"
                _ => "right"
            }
        { dir, seed: rolled.seed }
    }

    occupied_points = |snakes, food|
        List.concat(
            List.fold(
                snakes,
                [],
                |acc, snake|
                    if snake.alive {
                        List.concat(acc, snake.body)
                    } else {
                        acc
                    },
            ),
            food,
        )

    is_free = |snakes, food, point|
        in_bounds(point)
        and !List.any(occupied_points(snakes, food), |other| same_point(other, point))

    centroid = |snakes| {
        living = List.keep_if(snakes, |snake| snake.alive)
        if List.is_empty(living) {
            world_center()
        } else {
            sum = List.fold(
                living,
                { x: 0.I64, y: 0.I64 },
                |acc, snake|
                    match List.get(snake.body, 0) {
                        Ok(head) => { x: acc.x + head.x, y: acc.y + head.y }
                        Err(_) => acc
                    },
            )
            n = List.len(living).to_i64_wrap()
            if n == 0.I64 {
                world_center()
            } else {
                { x: sum.x.div_trunc_by(n), y: sum.y.div_trunc_by(n) }
            }
        }
    }

    body_from_head = |head, dir, length| {
        back = opposite(dir)
        List.fold(
            range(length),
            [],
            |acc, i| {
                List.append(acc, translate_n(head, back, i))
            },
        )
    }

    translate_n = |point, dir, n|
        if n <= 0 {
            point
        } else {
            translate_n(translate(point, dir), dir, n - 1)
        }

    find_player_spawn = |snakes, food, seed, attempts| {
        if attempts <= 0 {
            picked = dir_at(seed)
            { head: world_center(), dir: picked.dir, seed: picked.seed }
        } else {
            x = rand_range(seed, spawn_lo, spawn_hi)
            y = rand_range(x.seed, spawn_lo, spawn_hi)
            picked = dir_at(y.seed)
            head = { x: x.value, y: y.value }
            if is_free(snakes, food, head) {
                { head, dir: picked.dir, seed: picked.seed }
            } else {
                find_player_spawn(snakes, food, picked.seed, attempts - 1)
            }
        }
    }

    find_food_spawn = |snakes, food, seed, attempts| {
        last = world_size - 1
        if attempts <= 0 {
            { head: world_center(), seed: next_seed(seed) }
        } else {
            x = rand_range(seed, 0, last)
            y = rand_range(x.seed, 0, last)
            head = { x: x.value, y: y.value }
            if is_free(snakes, food, head) {
                { head, seed: y.seed }
            } else {
                find_food_spawn(snakes, food, y.seed, attempts - 1)
            }
        }
    }

    spawn_player = |world, id, name, color| {
        found = find_player_spawn(world.snakes, world.food, world.seed, 32)
        snake = {
            id,
            name,
            color,
            dir: found.dir,
            pending_dir: found.dir,
            alive: True,
            respawn_in: 0,
            body: body_from_head(found.head, found.dir, init_length),
            score: init_length,
            cam: centered_origin(found.head),
        }
        {
            ..world,
            snakes: List.append(world.snakes, snake),
            seed: found.seed,
        }
    }

    camera = |world, player_id| {
        if player_id == "" {
            centered_origin(centroid(world.snakes))
        } else {
            match player_snake(world.snakes, player_id) {
                Some(snake) => snake.cam
                _ => centered_origin(centroid(world.snakes))
            }
        }
    }

    centered_origin = |focus| {
        {
            x: clamp(focus.x - view_width.div_trunc_by(2), 0, world_size - view_width),
            y: clamp(focus.y - view_height.div_trunc_by(2), 0, world_size - view_height),
        }
    }

    follow_axis = |origin, focus, view| {
        last = view - 1
        max_origin = world_size - view
        if focus < origin + cam_margin {
            clamp(focus - cam_margin, 0, max_origin)
        } else if focus > origin + last - cam_margin {
            clamp(focus - (last - cam_margin), 0, max_origin)
        } else {
            origin
        }
    }

    follow_origin = |origin, focus| {
        {
            x: follow_axis(origin.x, focus.x, view_width),
            y: follow_axis(origin.y, focus.y, view_height),
        }
    }

    player_snake = |snakes, player_id|
        List.fold(
            snakes,
            None,
            |acc, snake|
                match acc {
                    Some(_) => acc
                    None if snake.id == player_id => Some(snake)
                    None => acc
                },
        )

    living_head_color = |snakes, x, y|
        List.fold(
            snakes,
            "",
            |acc, snake|
                if acc != "" or !snake.alive {
                    acc
                } else {
                    match List.get(snake.body, 0) {
                        Ok(head) if head.x == x and head.y == y => snake.color
                        _ => acc
                    }
                },
        )

    living_body_color = |snakes, x, y|
        List.fold(
            snakes,
            "",
            |acc, snake|
                if acc != "" or !snake.alive {
                    acc
                } else {
                    List.fold(
                        drop_first(snake.body),
                        acc,
                        |inner, point|
                            if inner != "" {
                                inner
                            } else if point.x == x and point.y == y {
                                snake.color
                            } else {
                                inner
                            },
                    )
                },
        )

    food_at = |food, x, y|
        List.any(food, |point| point.x == x and point.y == y)

    cell_class = |world, x, y| {
        if !in_bounds({ x, y }) {
            "cell wall"
        } else {
            head = living_head_color(world.snakes, x, y)
            if head != "" {
                "cell snake ${head} head"
            } else {
                body = living_body_color(world.snakes, x, y)
                if body != "" {
                    "cell snake ${body}"
                } else if food_at(world.food, x, y) {
                    "cell food"
                } else {
                    "cell"
                }
            }
        }
    }

    cells = |world, cam| {
        origin = cam
        List.fold(
            range(view_height),
            [],
            |acc, row|
                List.concat(
                    acc,
                    List.map(
                        range(view_width),
                        |col| { class: cell_class(world, origin.x + col, origin.y + row) },
                    ),
                ),
        )
    }

    mark_style = |x, y|
        "left:${x.to_str()}%;top:${y.to_str()}%"

    mark_kind = |class|
        if Str.starts_with(class, "mark food") {
            0.I64
        } else if Str.starts_with(class, "mark body") {
            1.I64
        } else {
            2.I64
        }

    mark_before = |a, b| {
        ka = mark_kind(a.class)
        kb = mark_kind(b.class)
        if ka < kb {
            True
        } else if ka > kb {
            False
        } else if a.x < b.x {
            True
        } else if a.x > b.x {
            False
        } else {
            a.y <= b.y
        }
    }

    insert_mark = |list, mark|
        insert_mark_help(list, mark, [])

    insert_mark_help = |rest, mark, acc|
        match rest {
            [] => List.append(acc, mark)
            [head, .. as tail] =>
                if mark_before(mark, head) {
                    List.concat(List.append(acc, mark), rest)
                } else {
                    insert_mark_help(tail, mark, List.append(acc, head))
                }
        }

    add_mark = |marks, class, point|
        insert_mark(
            marks,
            {
                class,
                style: mark_style(point.x, point.y),
                x: point.x,
                y: point.y,
            },
        )

    minimap = |world, cam| {
        origin = cam
        food_marks = List.fold(
            world.food,
            [],
            |acc, point| add_mark(acc, "mark food", point),
        )
        snake_marks = List.fold(
            world.snakes,
            food_marks,
            |acc, snake|
                if !snake.alive {
                    acc
                } else {
                    match List.get(snake.body, 0) {
                        Err(_) => acc
                        Ok(head) => {
                            with_body = List.fold(
                                drop_first(snake.body),
                                acc,
                                |inner, point| add_mark(inner, "mark body ${snake.color}", point),
                            )
                            add_mark(with_body, "mark head ${snake.color}", head)
                        }
                    }
                },
        )
        List.append(
            snake_marks,
            {
                class: "view",
                style: "left:${origin.x.to_str()}%;top:${origin.y.to_str()}%;width:${view_width.to_str()}%;height:${view_height.to_str()}%",
                x: origin.x,
                y: origin.y,
            },
        )
    }

    apply_dir = |current, pending|
        if is_reverse(current, pending) {
            current
        } else if valid_dir(pending) {
            pending
        } else {
            current
        }

    move_snake = |snake, food, grow| {
        if !snake.alive {
            snake
        } else {
            dir = apply_dir(snake.dir, snake.pending_dir)
            match List.get(snake.body, 0) {
                Err(_) => snake
                Ok(head) => {
                    next_head = translate(head, dir)
                    eating = food_at(food, next_head.x, next_head.y)
                    growing = eating or grow
                    rest = if growing { snake.body } else { drop_last(snake.body) }
                    {
                        ..snake,
                        dir,
                        pending_dir: dir,
                        body: List.prepend(rest, next_head),
                        score: if growing {
                            snake.score + 1
                        } else {
                            snake.score
                        },
                        cam: follow_origin(snake.cam, next_head),
                    }
                }
            }
        }
    }

    head_of = |snake|
        List.get(snake.body, 0)

    head_conflicts = |snakes, snake|
        match head_of(snake) {
            Err(_) => False
            Ok(head) =>
                List.any(
                    snakes,
                    |other|
                        other.id != snake.id
                        and other.alive
                        and (
                            match head_of(other) {
                                Ok(other_head) => same_point(head, other_head)
                                Err(_) => False
                            }
                        ),
                )
        }

    hits_body = |snakes, snake|
        match head_of(snake) {
            Err(_) => False
            Ok(head) =>
                List.any(
                    snakes,
                    |other| {
                        segments =
                            if other.id == snake.id {
                                drop_first(other.body)
                            } else if other.alive {
                                other.body
                            } else {
                                []
                            }
                        List.any(segments, |point| same_point(head, point))
                    },
                )
        }

    hits_wall = |snake|
        match head_of(snake) {
            Err(_) => True
            Ok(head) => !in_bounds(head)
        }

    kill = |snake|
        { ..snake, alive: False, respawn_in: respawn_ticks, body: [] }

    resolve_deaths = |world| {
        snakes = List.map(
            world.snakes,
            |snake|
                if !snake.alive {
                    snake
                } else if hits_wall(snake) or head_conflicts(world.snakes, snake) or hits_body(world.snakes, snake) {
                    kill(snake)
                } else {
                    snake
                },
        )
        { ..world, snakes }
    }

    eat_food = |world| {
        heads = List.fold(
            world.snakes,
            [],
            |acc, snake|
                if snake.alive {
                    match head_of(snake) {
                        Ok(head) => List.append(acc, head)
                        Err(_) => acc
                    }
                } else {
                    acc
                },
        )
        remaining = List.keep_if(
            world.food,
            |point| !List.any(heads, |head| same_point(head, point)),
        )
        { ..world, food: remaining }
    }

    refill_food = |world, needed| {
        if needed <= 0 {
            world
        } else {
            found = find_food_spawn(world.snakes, world.food, world.seed, 32)
            next = {
                ..world,
                food: List.append(world.food, found.head),
                seed: found.seed,
            }
            refill_food(next, needed - 1)
        }
    }

    respawn_ready = |world| {
        living = List.keep_if(world.snakes, |snake| snake.alive)
        List.fold(
            world.snakes,
            { ..world, snakes: [] },
            |acc, snake|
                if snake.alive {
                    { ..acc, snakes: List.append(acc.snakes, snake) }
                } else if snake.respawn_in <= 1 {
                    occupancy = List.concat(living, acc.snakes)
                    found = find_player_spawn(occupancy, acc.food, acc.seed, 32)
                    spawned = {
                        ..snake,
                        alive: True,
                        respawn_in: 0,
                        dir: found.dir,
                        pending_dir: found.dir,
                        body: body_from_head(found.head, found.dir, init_length),
                        score: init_length,
                        cam: centered_origin(found.head),
                    }
                    {
                        ..acc,
                        snakes: List.append(acc.snakes, spawned),
                        seed: found.seed,
                    }
                } else {
                    {
                        ..acc,
                        snakes: List.append(acc.snakes, { ..snake, respawn_in: snake.respawn_in - 1 }),
                    }
                },
        )
    }

    step = |world| {
        tick = world.tick + 1
        grow = tick.rem_by(grow_every) == 0
        after_respawn = respawn_ready(world)
        moved = {
            ..after_respawn,
            snakes: List.map(after_respawn.snakes, |snake| move_snake(snake, after_respawn.food, grow)),
            tick,
        }
        resolved = resolve_deaths(moved)
        eaten = eat_food(resolved)
        missing = food_target - List.len(eaten.food).to_i64_wrap()
        refill_food(eaten, missing)
    }
}
