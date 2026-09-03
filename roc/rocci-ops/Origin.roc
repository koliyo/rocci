Origin := [].{
    live_app_env_key = do_live_app_env_key
    parse_publish_live = do_parse_publish_live
    resolved_lane = do_resolved_lane
    should_publish_live = do_should_publish_live
    example_public_hosts = do_example_public_hosts
    health_checks = do_health_checks
    compose_argv = do_compose_argv
    origin_publish_cmd = do_origin_publish_cmd
    validate_sha = do_validate_sha
    health_curl_argv = do_health_curl_argv
    parse_origin = do_parse_origin
    parse_deploy = do_parse_deploy
    origin_help = origin_help_text
    deploy_help = deploy_help_text
}

origin_help_text = "usage: rocci-ops origin [-h] {publish,up,backup} ...\n"
deploy_help_text = "usage: rocci-ops deploy [-h] {probe,bootstrap,push} ...\n"

OriginReq : [
    OriginUsage,
    OriginHelp,
    OriginPublish(Str),
    OriginUp({ dist : Str, bin : Str }),
    OriginBackup(Str),
    DeployUsage,
    DeployHelp,
    DeployProbe,
    DeployBootstrap,
    DeployPush({ dir : Str, sha : Str }),
]

trim = |s| {
    bytes = Str.to_utf8(s)
    len = List.len(bytes)
    var $a = 0.U64
    var $b = len
    var $go = Bool.True
    while $go and $a < $b {
        match List.get(bytes, $a) {
            Ok(ch) => {
                if ch == 32 or ch == 9 or ch == 10 or ch == 13 {
                    $a = $a + 1
                } else {
                    $go = Bool.False
                }
            }
            Err(_) => {
                $go = Bool.False
            }
        }
    }
    $go = Bool.True
    while $go and $b > $a {
        match List.get(bytes, $b - 1) {
            Ok(ch) => {
                if ch == 32 or ch == 9 or ch == 10 or ch == 13 {
                    $b = $b - 1
                } else {
                    $go = Bool.False
                }
            }
            Err(_) => {
                $go = Bool.False
            }
        }
    }
    var $i = $a
    var $out = []
    while $i < $b {
        match List.get(bytes, $i) {
            Ok(ch) => {
                $out = List.concat($out, [ch])
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.from_utf8_lossy($out)
}

ascii_lower = |s| {
    bytes = Str.to_utf8(s)
    var $i = 0.U64
    var $out = []
    while $i < List.len(bytes) {
        match List.get(bytes, $i) {
            Ok(ch) => {
                up = if ch >= 65 and ch <= 90 { ch + 32 } else { ch }
                $out = List.concat($out, [up])
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.from_utf8_lossy($out)
}

env_get = |pairs, key, default| {
    var $i = 0.U64
    var $found = Bool.False
    var $val = default
    while !$found and $i < List.len(pairs) {
        match List.get(pairs, $i) {
            Ok(pair) => {
                if pair.key == key {
                    $found = Bool.True
                    $val = pair.val
                } else {
                    $i = $i + 1
                }
            }
            Err(_) => {
                $i = List.len(pairs)
            }
        }
    }
    if $found {
        if $val == "" {
            default
        } else {
            $val
        }
    } else {
        default
    }
}

env_has = |pairs, key| {
    var $i = 0.U64
    var $found = Bool.False
    while !$found and $i < List.len(pairs) {
        match List.get(pairs, $i) {
            Ok(pair) => {
                if pair.key == key {
                    $found = Bool.True
                } else {
                    $i = $i + 1
                }
            }
            Err(_) => {
                $i = List.len(pairs)
            }
        }
    }
    $found
}

env_raw = |pairs, key| {
    var $i = 0.U64
    var $found = Bool.False
    var $val = ""
    while !$found and $i < List.len(pairs) {
        match List.get(pairs, $i) {
            Ok(pair) => {
                if pair.key == key {
                    $found = Bool.True
                    $val = pair.val
                } else {
                    $i = $i + 1
                }
            }
            Err(_) => {
                $i = List.len(pairs)
            }
        }
    }
    { found: $found, val: $val }
}

do_parse_publish_live = |raw| {
    t = ascii_lower(trim(raw))
    if t == "0" {
        Bool.False
    } else if t == "false" {
        Bool.False
    } else if t == "no" {
        Bool.False
    } else {
        Bool.True
    }
}

do_live_app_env_key = |app_id| {
    bytes = Str.to_utf8(app_id)
    var $i = 0.U64
    var $out = Str.to_utf8("ROCCI_")
    while $i < List.len(bytes) {
        match List.get(bytes, $i) {
            Ok(45) => {
                $out = List.concat($out, [95])
            }
            Ok(ch) => {
                up = if ch >= 97 and ch <= 122 { ch - 32 } else { ch }
                $out = List.concat($out, [up])
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $out = List.concat($out, Str.to_utf8("_CONTEXT"))
    Str.from_utf8_lossy($out)
}

empty_lane = {
    err: "",
    name: "",
    origin_root: "/srv/rocci/prod",
    http_port: "8080",
    compose_project: "rocci-prod",
    publish_live: "unset",
    image_tag: "local",
    bootstrap_dest: "/srv/rocci/prod/docker",
}

do_resolved_lane = |pairs| {
    name = trim(env_raw(pairs, "ROCCI_LANE").val)
    match name {
        "production" => {
            origin = env_get(pairs, "ROCCI_ORIGIN_ROOT", "/srv/rocci/prod")
            live = if env_has(pairs, "ROCCI_PUBLISH_LIVE") {
                if do_parse_publish_live(env_raw(pairs, "ROCCI_PUBLISH_LIVE").val) {
                    "true"
                } else {
                    "false"
                }
            } else {
                "false"
            }
            {
                err: "",
                name: name,
                origin_root: origin,
                http_port: env_get(pairs, "ROCCI_HTTP_PORT", "8080"),
                compose_project: env_get(pairs, "COMPOSE_PROJECT_NAME", "rocci-prod"),
                publish_live: live,
                image_tag: env_get(pairs, "ROCCI_IMAGE_TAG", "prod"),
                bootstrap_dest: env_get(pairs, "ROCCI_BOOTSTRAP_DEST", "${origin}/docker"),
            }
        }
        "staging" => {
            origin = env_get(pairs, "ROCCI_ORIGIN_ROOT", "/srv/rocci/staging")
            live = if env_has(pairs, "ROCCI_PUBLISH_LIVE") {
                if do_parse_publish_live(env_raw(pairs, "ROCCI_PUBLISH_LIVE").val) {
                    "true"
                } else {
                    "false"
                }
            } else {
                "true"
            }
            {
                err: "",
                name: name,
                origin_root: origin,
                http_port: env_get(pairs, "ROCCI_HTTP_PORT", "8081"),
                compose_project: env_get(pairs, "COMPOSE_PROJECT_NAME", "rocci-staging"),
                publish_live: live,
                image_tag: env_get(pairs, "ROCCI_IMAGE_TAG", "staging"),
                bootstrap_dest: env_get(pairs, "ROCCI_BOOTSTRAP_DEST", "${origin}/docker"),
            }
        }
        "" => {
            origin = env_get(pairs, "ROCCI_ORIGIN_ROOT", "/srv/rocci/prod")
            live = if env_has(pairs, "ROCCI_PUBLISH_LIVE") {
                if do_parse_publish_live(env_raw(pairs, "ROCCI_PUBLISH_LIVE").val) {
                    "true"
                } else {
                    "false"
                }
            } else {
                "unset"
            }
            {
                err: "",
                name: "",
                origin_root: origin,
                http_port: env_get(pairs, "ROCCI_HTTP_PORT", "8080"),
                compose_project: env_get(pairs, "COMPOSE_PROJECT_NAME", "rocci-prod"),
                publish_live: live,
                image_tag: env_get(pairs, "ROCCI_IMAGE_TAG", "local"),
                bootstrap_dest: env_get(pairs, "ROCCI_BOOTSTRAP_DEST", "${origin}/docker"),
            }
        }
        other => {
            { ..empty_lane, err: "error: unknown ROCCI_LANE='${other}'" }
        }
    }
}

do_should_publish_live = |live_ids, cfg| {
    if List.len(live_ids) == 0 {
        Bool.False
    } else if cfg.publish_live == "unset" {
        Bool.True
    } else if cfg.publish_live == "true" {
        Bool.True
    } else {
        Bool.False
    }
}

do_example_public_hosts = |app_id, cfg| {
    if cfg.name == "production" {
        []
    } else if cfg.publish_live == "false" {
        []
    } else {
        staging = ["${app_id}-example-staging.rocci.dev", "${app_id}.examples.localhost"]
        if cfg.name == "staging" {
            staging
        } else {
            List.concat(staging, ["${app_id}-example.rocci.dev"])
        }
    }
}

do_health_checks = |live_ids, cfg| {
    port = cfg.http_port
    site = "http://127.0.0.1:${port}/health"
    var $i = 0.U64
    var $out = [{ url: site, host: "" }]
    while $i < List.len(live_ids) {
        match List.get(live_ids, $i) {
            Ok(app_id) => {
                $out = List.concat($out, [{ url: "http://127.0.0.1:${port}/play/${app_id}/health", host: "" }])
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $i = 0
    while $i < List.len(live_ids) {
        match List.get(live_ids, $i) {
            Ok(app_id) => {
                hosts = do_example_public_hosts(app_id, cfg)
                var $h = 0.U64
                while $h < List.len(hosts) {
                    match List.get(hosts, $h) {
                        Ok(host) => {
                            $out = List.concat($out, [{ url: site, host: host }])
                        }
                        Err(_) => {}
                    }
                    $h = $h + 1
                }
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $out
}

do_compose_argv = |repo, live_ids, cfg| {
    hybrid = "${repo}/docker/compose.hybrid.yml"
    extra = "${repo}/docker/compose.origin.yml"
    proj = "${repo}/docker"
    base = ["docker", "compose", "-f", hybrid]
    with_live = if do_should_publish_live(live_ids, cfg) {
        List.concat(base, ["-f", extra])
    } else {
        base
    }
    List.concat(with_live, ["--project-directory", proj, "up", "-d", "--build", "--remove-orphans"])
}

do_origin_publish_cmd = |sha, origin_root, cfg| {
    live = if cfg.publish_live == "false" { "0" } else { "1" }
    lane = if cfg.name == "" {
        ""
    } else {
        "ROCCI_LANE='${cfg.name}' "
    }
    "cd '${origin_root}' && ${lane}ROCCI_ORIGIN_ROOT='${origin_root}' ROCCI_HTTP_PORT='${cfg.http_port}' COMPOSE_PROJECT_NAME='${cfg.compose_project}' ROCCI_PUBLISH_LIVE='${live}' ROCCI_IMAGE_TAG='${cfg.image_tag}' uv run --no-dev rocci-ops origin publish '${sha}'"
}

do_validate_sha = |sha| {
    bytes = Str.to_utf8(sha)
    if List.len(bytes) == 0 {
        Bool.False
    } else {
        var $i = 0.U64
        var $ok = Bool.True
        while $ok and $i < List.len(bytes) {
            match List.get(bytes, $i) {
                Ok(ch) => {
                    hex = (ch >= 48 and ch <= 57) or (ch >= 65 and ch <= 70) or (ch >= 97 and ch <= 102)
                    if hex {
                        {}
                    } else {
                        $ok = Bool.False
                    }
                }
                Err(_) => {
                    $ok = Bool.False
                }
            }
            $i = $i + 1
        }
        $ok
    }
}

do_health_curl_argv = |url, host| {
    base = ["curl", "-sS", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "5", "--noproxy", "*"]
    if host == "" {
        List.concat(base, [url])
    } else {
        List.concat(base, ["-H", "Host: ${host}", url])
    }
}

do_parse_origin = |args|
    match List.get(args, 0) {
        Err(_) => OriginUsage
        Ok("-h") => OriginHelp
        Ok("--help") => OriginHelp
        Ok("publish") => {
            match List.get(args, 1) {
                Ok(sha) => {
                    if List.len(args) == 2 {
                        OriginPublish(sha)
                    } else {
                        OriginUsage
                    }
                }
                Err(_) => OriginUsage
            }
        }
        Ok("up") => {
            match (List.get(args, 1), List.get(args, 2)) {
                (Ok(dist), Ok(bin)) => {
                    if List.len(args) == 3 {
                        OriginUp({ dist: dist, bin: bin })
                    } else {
                        OriginUsage
                    }
                }
                _ => OriginUsage
            }
        }
        Ok("backup") => {
            match List.get(args, 1) {
                Err(_) => OriginBackup("/var/backups/rocci")
                Ok(dest) => {
                    if List.len(args) == 2 {
                        OriginBackup(dest)
                    } else {
                        OriginUsage
                    }
                }
            }
        }
        Ok(_) => OriginUsage
    }

do_parse_deploy = |args|
    match List.get(args, 0) {
        Err(_) => DeployUsage
        Ok("-h") => DeployHelp
        Ok("--help") => DeployHelp
        Ok("probe") => {
            if List.len(args) == 1 {
                DeployProbe
            } else {
                DeployUsage
            }
        }
        Ok("bootstrap") => {
            if List.len(args) == 1 {
                DeployBootstrap
            } else {
                DeployUsage
            }
        }
        Ok("push") => {
            match (List.get(args, 1), List.get(args, 2)) {
                (Ok(dir), Ok(sha)) => {
                    if List.len(args) == 3 {
                        DeployPush({ dir: dir, sha: sha })
                    } else {
                        DeployUsage
                    }
                }
                _ => DeployUsage
            }
        }
        Ok(_) => DeployUsage
    }

expect do_live_app_env_key("live-counter") == "ROCCI_LIVE_COUNTER_CONTEXT"
expect do_live_app_env_key("datastar") == "ROCCI_DATASTAR_CONTEXT"
expect do_live_app_env_key("snake") == "ROCCI_SNAKE_CONTEXT"

expect do_validate_sha("abcDEF012") == Bool.True
expect do_validate_sha("not a sha") == Bool.False
expect do_validate_sha("") == Bool.False

expect
    match do_resolved_lane([{ key: "ROCCI_LANE", val: "lab" }]) {
        { err, .. } => err == "error: unknown ROCCI_LANE='lab'"
    }

expect
    match do_resolved_lane([{ key: "ROCCI_LANE", val: "staging" }]) {
        { err: "", http_port: "8081", origin_root: "/srv/rocci/staging", image_tag: "staging", publish_live: "true", .. } => Bool.True
        _ => Bool.False
    }

expect
    match do_resolved_lane([{ key: "ROCCI_LANE", val: "production" }]) {
        { err: "", http_port: "8080", origin_root: "/srv/rocci/prod", image_tag: "prod", publish_live: "false", .. } => Bool.True
        _ => Bool.False
    }

expect
    match do_health_checks(["live-counter", "datastar"], do_resolved_lane([])) {
        checks => {
            match (List.get(checks, 0), List.get(checks, 1), List.get(checks, 2)) {
                (Ok({ url: "http://127.0.0.1:8080/health", host: "" }), Ok({ url: "http://127.0.0.1:8080/play/live-counter/health", host: "" }), Ok({ url: "http://127.0.0.1:8080/play/datastar/health", host: "" })) => {
                    match List.get(checks, 3) {
                        Ok({ host: "live-counter-example-staging.rocci.dev", url: _u }) => Bool.True
                        _ => Bool.False
                    }
                }
                _ => Bool.False
            }
        }
    }

expect
    match do_health_checks(["live-counter"], do_resolved_lane([{ key: "ROCCI_LANE", val: "staging" }])) {
        checks => {
            match List.get(checks, 0) {
                Ok({ url: "http://127.0.0.1:8081/health", host: _h }) => {
                    match List.get(checks, List.len(checks) - 1) {
                        Ok({ host: "live-counter.examples.localhost", url: _u }) => Bool.True
                        _ => Bool.False
                    }
                }
                _ => Bool.False
            }
        }
    }

expect
    match do_health_checks(["live-counter"], do_resolved_lane([{ key: "ROCCI_LANE", val: "production" }])) {
        checks => {
            match (List.len(checks), List.get(checks, 0), List.get(checks, 1)) {
                (2, Ok({ url: "http://127.0.0.1:8080/health", host: "" }), Ok({ url: "http://127.0.0.1:8080/play/live-counter/health", host: "" })) => Bool.True
                _ => Bool.False
            }
        }
    }

expect
    match do_compose_argv("/repo", [], do_resolved_lane([])) {
        argv => {
            match (List.get(argv, 3), List.get(argv, List.len(argv) - 1)) {
                (Ok("/repo/docker/compose.hybrid.yml"), Ok("--remove-orphans")) => Bool.True
                _ => Bool.False
            }
        }
    }

expect
    match do_compose_argv("/repo", ["live-counter"], do_resolved_lane([])) {
        argv => {
            match List.get(argv, 5) {
                Ok("/repo/docker/compose.origin.yml") => Bool.True
                _ => Bool.False
            }
        }
    }

expect
    match do_origin_publish_cmd("deadbeef", "/srv/rocci", do_resolved_lane([])) {
        cmd => {
            has_uv = match List.get(Str.to_utf8(cmd), 0) {
                Ok(_) => Bool.True
                Err(_) => Bool.False
            }
            has_uv and cmd == "cd '/srv/rocci' && ROCCI_ORIGIN_ROOT='/srv/rocci' ROCCI_HTTP_PORT='8080' COMPOSE_PROJECT_NAME='rocci-prod' ROCCI_PUBLISH_LIVE='1' ROCCI_IMAGE_TAG='local' uv run --no-dev rocci-ops origin publish 'deadbeef'"
        }
    }

expect
    match do_origin_publish_cmd("deadbeef", "/srv/rocci/staging", do_resolved_lane([{ key: "ROCCI_LANE", val: "staging" }])) {
        cmd => cmd == "cd '/srv/rocci/staging' && ROCCI_LANE='staging' ROCCI_ORIGIN_ROOT='/srv/rocci/staging' ROCCI_HTTP_PORT='8081' COMPOSE_PROJECT_NAME='rocci-staging' ROCCI_PUBLISH_LIVE='1' ROCCI_IMAGE_TAG='staging' uv run --no-dev rocci-ops origin publish 'deadbeef'"
    }

expect
    match do_health_curl_argv("http://127.0.0.1:8080/health", "") {
        argv => {
            match List.get(argv, 0) {
                Ok("curl") => Bool.True
                _ => Bool.False
            }
        }
    }
