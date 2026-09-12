"""Phase 2 cost isolation. Imported by run.py; copies product Html.roc only inside the work dir."""

import statistics
import shutil
from pathlib import Path


HERE = Path(__file__).resolve().parent
SIZE_SWEEP = [0, 1, 10, 100]
EXPENSIVE_ROWS = [100, 1000]
CLEAN = "plain"
ESCAPED = "<& café>"
KERNEL_REPS = 20000
RENDER_REPS = 3000
EXPENSIVE_REPS = 500

NODE_ESCAPE = '''escape_html_bytes : Str, Bool -> Str
escape_html_bytes = |value, escape_quotes| {
	escaped_bytes =
		Str.to_utf8(value).fold(
			[],
			|bytes, byte|
				match byte {
					38 => bytes.concat([38, 97, 109, 112, 59])
					60 => bytes.concat([38, 108, 116, 59])
					62 => bytes.concat([38, 103, 116, 59])
					34 if escape_quotes => bytes.concat([38, 113, 117, 111, 116, 59])
					39 if escape_quotes => bytes.concat([38, 35, 51, 57, 59])
					10 if escape_quotes => bytes.concat([38, 35, 49, 48, 59])
					13 if escape_quotes => bytes.concat([38, 35, 49, 51, 59])
					_ => bytes.append(byte)
				},
		)

	match Str.from_utf8(escaped_bytes) {
		Ok(str) => str
		Err(_) => ""
	}
}'''

NODE_SCAN_ESCAPE = '''escape_html_bytes : Str, Bool -> Str
escape_html_bytes = |value, escape_quotes| {
	bytes = Str.to_utf8(value)
	needs_escape = bytes.fold(
		Bool.False,
		|found, byte|
			if found {
				Bool.True
			} else {
				match byte {
					38 => Bool.True
					60 => Bool.True
					62 => Bool.True
					34 if escape_quotes => Bool.True
					39 if escape_quotes => Bool.True
					10 if escape_quotes => Bool.True
					13 if escape_quotes => Bool.True
					_ => Bool.False
				}
			},
	)
	if !needs_escape {
		value
	} else {
		escaped_bytes = bytes.fold(
			List.with_capacity(bytes.len() * 2),
			|out, byte|
				match byte {
					38 => out.append(38).append(97).append(109).append(112).append(59)
					60 => out.append(38).append(108).append(116).append(59)
					62 => out.append(38).append(103).append(116).append(59)
					34 if escape_quotes => out.append(38).append(113).append(117).append(111).append(116).append(59)
					39 if escape_quotes => out.append(38).append(35).append(51).append(57).append(59)
					10 if escape_quotes => out.append(38).append(35).append(49).append(48).append(59)
					13 if escape_quotes => out.append(38).append(35).append(49).append(51).append(59)
					_ => out.append(byte)
				},
		)
		match Str.from_utf8(escaped_bytes) {
			Ok(str) => str
			Err(_) => ""
		}
	}
}'''

STRING_SPLIT_ESCAPE = '''replace = |haystack, needle, replacement|
    Str.join_with(Str.split_on(haystack, needle), replacement)

escape = |value|
    replace(
        replace(
            replace(
                replace(replace(value, "&", "&amp;"), "<", "&lt;"),
                ">",
                "&gt;",
            ),
            "\\"",
            "&quot;",
        ),
        "'",
        "&#39;",
    )'''

# STRING_SPLIT in the file uses '"' not escaped that way - we'll replace using the actual file content.

KERNEL = r'''
as_text = |arg| match OsStr.to_raw(arg) {
    Utf8(s) => s
    UnixBytes(bytes) => Str.from_utf8_lossy(bytes)
    _ => ""
}

main! = |args| {
    match args {
        [_, reps_arg, text_arg] => {
            reps = U64.from_str(as_text(reps_arg)) ?? 1
            text = as_text(text_arg)
            var $n = 0.U64
            for i in 0..<reps {
                escaped = escape(if i % 2 == 0 text else Str.concat(text, "X"))
                $n = $n + escaped.count_utf8_bytes()
            }
            Stdout.line!($n.to_str())?
            Ok({})
        }
        _ => Err(Exit(2))
    }
}
'''


def median_seconds(runs):
    return statistics.median(run["seconds"] for run in runs)


def timed_batch(harness, argv, cwd, reps_timeout, runs=3, warmup=True):
    if warmup:
        harness.command(argv, cwd, timeout=reps_timeout)
    samples = [harness.command(argv, cwd, timeout=reps_timeout) for _ in range(runs)]
    for sample in samples:
        if sample["exit"] != 0 or sample["timed_out"] or sample["invalid_utf8"]:
            return {"ok": False, "runs": samples}
    return {"ok": True, "runs": samples, "median_seconds": median_seconds(samples), "checksums": [run.get("stdout") for run in samples]}


def write_kernel(work, name, header_escape, harness):
    target = work / "costs" / "kernels" / name
    target.mkdir(parents=True)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    (target / "main.roc").write_text(header + header_escape + KERNEL)
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
    return target, build


def patch_node_html(text, kind):
    if kind == "scan_escape":
        if NODE_SCAN_ESCAPE in text:
            return text
        if NODE_ESCAPE not in text:
            raise RuntimeError("node escape kernel not found in Html.roc")
        return text.replace(NODE_ESCAPE, NODE_SCAN_ESCAPE)
    if kind == "fold_escape":
        if NODE_ESCAPE in text:
            return text
        if NODE_SCAN_ESCAPE not in text:
            raise RuntimeError("scan/copy node escape kernel not found in Html.roc")
        return text.replace(NODE_SCAN_ESCAPE, NODE_ESCAPE)
    if kind == "join_growth":
        old = '''			Element(tag, attrs, children) =>
				"<${tag}${render_attributes(attrs)}>${render_children(children)}</${tag}>"
			VoidElement(tag, attrs) =>
				"<${tag}${render_attributes(attrs)}>"'''
        new = '''			Element(tag, attrs, children) =>
				Str.join_with(["<", tag, render_attributes(attrs), ">", render_children(children), "</", tag, ">"], "")
			VoidElement(tag, attrs) =>
				Str.join_with(["<", tag, render_attributes(attrs), ">"], "")'''
        if old not in text:
            raise RuntimeError("node element render kernel not found in Html.roc")
        return text.replace(old, new)
    raise RuntimeError(kind)


def patch_string_html(text):
    old = '''replace = |haystack, needle, replacement|
    Str.join_with(Str.split_on(haystack, needle), replacement)

escape = |value|
    replace(
        replace(
            replace(
                replace(replace(value, "&", "&amp;"), "<", "&lt;"),
                ">",
                "&gt;",
            ),
            "\\"",
            "&quot;",
        ),
        "'",
        "&#39;",
    )'''
    scan = '''escape = |value| {
    bytes = Str.to_utf8(value)
    needs_escape = bytes.fold(
        Bool.False,
        |found, byte|
            if found {
                Bool.True
            } else {
                match byte {
                    38 => Bool.True
                    60 => Bool.True
                    62 => Bool.True
                    34 => Bool.True
                    39 => Bool.True
                    _ => Bool.False
                }
            },
    )
    if !needs_escape {
        value
    } else {
        escaped_bytes = bytes.fold(
            List.with_capacity(bytes.len() * 2),
            |out, byte|
                match byte {
                    38 => out.append(38).append(97).append(109).append(112).append(59)
                    60 => out.append(38).append(108).append(116).append(59)
                    62 => out.append(38).append(103).append(116).append(59)
                    34 => out.append(38).append(113).append(117).append(111).append(116).append(59)
                    39 => out.append(38).append(35).append(51).append(57).append(59)
                    _ => out.append(byte)
                },
        )
        match Str.from_utf8(escaped_bytes) {
            Ok(str) => str
            Err(_) => ""
        }
    }
}

'''
    return scan + "join =" + text.split("join =", 1)[1]


def stage_card_backend(work, name, harness, rocci, lowered, html_kind):
    target = work / "costs" / "card" / name
    target.mkdir(parents=True)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    body = lowered + "\nrender_page = |ctx| Html.render_without_doc_type(card(ctx))\n"
    if html_kind.startswith("string"):
        source = (harness.REPO / "crates/rocci-ui/runtime/Html.roc").read_text()
        if html_kind == "string_scan_escape":
            source = patch_string_html(source)
        (target / "Html.roc").write_text(source)
    else:
        (target / "Runtime").mkdir()
        attr = harness.REPO / "crates/rocci-platform/platform/Attribute.roc"
        shutil.copyfile(attr, target / "Runtime" / "Attribute.roc")
        html = (harness.REPO / "crates/rocci-platform/platform/Html.roc").read_text()
        if html_kind == "node_scan_escape":
            html = patch_node_html(html, "scan_escape")
        elif html_kind == "node_join_growth":
            html = patch_node_html(html, "join_growth")
        elif html_kind == "node_scan_and_join":
            html = patch_node_html(html, "scan_escape")
            html = patch_node_html(html, "join_growth")
        (target / "Runtime" / "Html.roc").write_text(html)
        wrapper = (harness.REPO / "crates/rocci-cli/runtime/Html.roc").read_text()
        wrapper = wrapper.replace("import pf.Attribute", "import Runtime/Attribute").replace("import pf.Html", "import Runtime/Html")
        (target / "Html.roc").write_text(wrapper)
    (target / "main.roc").write_text(header + body + harness.MAIN)
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
    return target, build


def stage_prepared(work, harness):
    target = work / "costs" / "card" / "prepared"
    target.mkdir(parents=True)
    for name in [*harness.HASHES, "TypedTemplate.roc"]:
        shutil.copyfile(work / name, target / name)
    text = (HERE / "card.mustache.html").read_text().removesuffix("\n")
    (target / "card.mustache.html").write_text(text)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    body = (
        'import TypedTemplate\nimport "card.mustache.html" as source : Str\n'
        'sample : { title : Str, active : Bool, items : List({ name : Str }) }\n'
        'sample = { title: "", active: Bool.True, items: [{ name: "" }] }\n'
        'render_page = TypedTemplate.prepare("card.mustache.html", source, sample)\n'
    )
    (target / "main.roc").write_text(header + body + harness.MAIN)
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
    return target, build


def stage_builder_control(work, harness):
    target = work / "costs" / "card" / "builder-control"
    target.mkdir(parents=True)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    source = header + r'''
escape = |value, quotes| {
    bytes = Str.to_utf8(value)
    needs_escape = bytes.fold(
        Bool.False,
        |found, byte|
            if found {
                Bool.True
            } else {
                match byte {
                    38 => Bool.True
                    60 => Bool.True
                    62 => Bool.True
                    34 if quotes => Bool.True
                    39 if quotes => Bool.True
                    10 if quotes => Bool.True
                    13 if quotes => Bool.True
                    _ => Bool.False
                }
            },
    )
    if !needs_escape {
        value
    } else {
        escaped_bytes = bytes.fold(
            List.with_capacity(bytes.len() * 2),
            |out, byte|
                match byte {
                    38 => out.append(38).append(97).append(109).append(112).append(59)
                    60 => out.append(38).append(108).append(116).append(59)
                    62 => out.append(38).append(103).append(116).append(59)
                    34 if quotes => out.append(38).append(113).append(117).append(111).append(116).append(59)
                    39 if quotes => out.append(38).append(35).append(51).append(57).append(59)
                    10 if quotes => out.append(38).append(35).append(49).append(48).append(59)
                    13 if quotes => out.append(38).append(35).append(49).append(51).append(59)
                    _ => out.append(byte)
                },
        )
        match Str.from_utf8(escaped_bytes) {
            Ok(str) => str
            Err(_) => ""
        }
    }
}

render_page = |{ title, active, items }| {
    t_attr = escape(title, Bool.True)
    t_text = escape(title, Bool.False)
    active_html = if active "<b>active</b>" else ""
    items_html = Str.join_with(
        items.map(|item| {
            n_attr = escape(item.name, Bool.True)
            n_text = escape(item.name, Bool.False)
            Str.join_with(["<li title=\"", n_attr, "\">", n_text, "</li>"], "")
        }),
        "",
    )
    Str.join_with(["<section title=\"", t_attr, "\"><h1>", t_text, "</h1>", active_html, "<ul>", items_html, "</ul></section>"], "")
}
''' + harness.MAIN
    (target / "main.roc").write_text(source)
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
    return target, build


def stage_encode_probe(work, harness):
    target = work / "costs" / "encode"
    target.mkdir(parents=True)
    for name in [*harness.HASHES, "TypedTemplate.roc"]:
        shutil.copyfile(work / name, target / name)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    (target / "main.roc").write_text(header + r'''
import Value
as_text = |arg| match OsStr.to_raw(arg) {
    Utf8(s) => s
    UnixBytes(bytes) => Str.from_utf8_lossy(bytes)
    _ => ""
}
main! = |args| {
    match args {
        [_, reps_arg, rows_arg, text_arg] => {
            reps = U64.from_str(as_text(reps_arg)) ?? 1
            rows = U64.from_str(as_text(rows_arg)) ?? 1
            text = as_text(text_arg)
            items = List.from_iter((0..<rows).iter()).map(|i| { name: "${text}${i.to_str()}" })
            var $n = 0.U64
            for i in 0..<reps {
                ctx = { title: if i % 2 == 0 text else Str.concat(text, "X"), active: i % 2 == 0, items }
                _encoded = Value.from(ctx)
                $n = $n + 1
            }
            Stdout.line!($n.to_str())?
            Ok({})
        }
        _ => Err(Exit(2))
    }
}
''')
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
    return target, build


def measure_app(harness, target, reps, rows, text, timeout=30):
    argv = [target / "app", "bench", reps, rows, text]
    batch1 = timed_batch(harness, argv, target, timeout)
    batch2 = timed_batch(harness, argv, target, timeout)
    return {"batch1": batch1, "batch2": batch2}


def improvement(old, new):
    if not old or not new:
        return None
    return (old - new) / old


def select_candidate(table):
    """Pick at most one runtime Html candidate using the Phase 2 screen."""
    large = table.get("card", {}).get("large-escaped-100", {})
    small = table.get("card", {}).get("small-clean-0", {})
    node = large.get("rocci-node", {}).get("median")
    candidates = []
    for name in ["node_scan_escape", "node_join_growth", "node_scan_and_join", "string_scan_escape"]:
        item = large.get(name)
        if not item or item.get("median") is None or node is None:
            continue
        gain = improvement(node, item["median"])
        small_old = small.get("rocci-node", {}).get("median")
        small_new = small.get(name, {}).get("median")
        small_reg = improvement(small_old, small_new) if small_old and small_new else None
        # Process totals for the empty fixture sit on a ~2ms noise floor.
        noise_s = 0.002
        if small_old and small_new and abs(small_new - small_old) <= noise_s:
            regression = 0.0
        else:
            regression = None if small_reg is None else (-small_reg if small_reg < 0 else 0.0)
        candidates.append({
            "name": name,
            "large_gain": gain,
            "small_regression": regression,
            "large_median": item["median"],
            "small_median": small_new,
            "passes_screen": bool(gain is not None and gain >= 0.15 and (regression is None or regression <= 0.05)),
            "small_noise_floor_s": noise_s,
        })
    passing = [c for c in candidates if c["passes_screen"]]
    passing.sort(key=lambda c: c["large_gain"], reverse=True)
    chosen = passing[0]["name"] if passing else None
    return {"candidates": candidates, "selected": chosen, "rule": ">=15% large-escaped-100 vs current node, <=5% small-clean regression"}


def run_costs(harness, work, report):
    rocci = harness.REPO / "target/debug/rocci-template"
    cargo = harness.command(["cargo", "build", "-q", "-p", "rocci-template"], harness.REPO, timeout=120)
    if cargo["exit"] != 0:
        raise harness.ExperimentError("cargo build -p rocci-template failed")
    lowered = harness.command([rocci, "build", HERE / "Card.rocci"], harness.REPO, timeout=60)
    if lowered["exit"] != 0:
        raise harness.ExperimentError("failed to lower Card.rocci")
    lowered_src = lowered["stdout"] or ""
    report["costs"] = {"kernels": {}, "card": {}, "encode": {}, "builds": {}}

    node_scan_escape_src = NODE_SCAN_ESCAPE.replace(
        "escape_html_bytes : Str, Bool -> Str\nescape_html_bytes = |value, escape_quotes|",
        "escape = |value| { escape_quotes = Bool.False\n",
    )
    # Dedicated kernel escapes:
    kernel_node_fold = '''
escape = |value| {
    escaped_bytes = Str.to_utf8(value).fold(
        [],
        |bytes, byte|
            match byte {
                38 => bytes.concat([38, 97, 109, 112, 59])
                60 => bytes.concat([38, 108, 116, 59])
                62 => bytes.concat([38, 103, 116, 59])
                _ => bytes.append(byte)
            },
    )
    match Str.from_utf8(escaped_bytes) { Ok(str) => str, Err(_) => "" }
}
'''
    kernel_node_scan = '''
escape = |value| {
    bytes = Str.to_utf8(value)
    needs_escape = bytes.fold(Bool.False, |found, byte| if found { Bool.True } else { byte == 38 or byte == 60 or byte == 62 })
    if !needs_escape {
        value
    } else {
        escaped_bytes = bytes.fold(
            List.with_capacity(bytes.len() * 2),
            |out, byte|
                match byte {
                    38 => out.append(38).append(97).append(109).append(112).append(59)
                    60 => out.append(38).append(108).append(116).append(59)
                    62 => out.append(38).append(103).append(116).append(59)
                    _ => out.append(byte)
                },
        )
        match Str.from_utf8(escaped_bytes) { Ok(str) => str, Err(_) => "" }
    }
}
'''
    kernel_string_split = '''
escape = |value| {
    replace = |haystack, needle, replacement| Str.join_with(Str.split_on(haystack, needle), replacement)
    replace(replace(replace(value, "&", "&amp;"), "<", "&lt;"), ">", "&gt;")
}
'''
    kernel_string_scan = kernel_node_scan

    for name, src in [
        ("node_fold_escape", kernel_node_fold),
        ("node_scan_escape", kernel_node_scan),
        ("string_split_escape", kernel_string_split),
        ("string_scan_escape", kernel_string_scan),
    ]:
        target, build = write_kernel(work, name, src, harness)
        report["costs"]["builds"][f"kernel_{name}"] = {"exit": build["exit"], "seconds": build["seconds"], "stderr": (build.get("stderr") or "")[-500:]}
        print(f"kernel build {name}: exit={build['exit']} ({build['seconds']:.2f}s)", flush=True)
        if build["exit"] != 0:
            continue
        report["costs"]["kernels"][name] = {}
        for label, text in [("clean", CLEAN), ("escaped", ESCAPED)]:
            report["costs"]["kernels"][name][label] = measure_app_kernel(harness, target, KERNEL_REPS, text)

    card_backends = [
        ("rocci-node", "node_current"),
        ("rocci-string", "string_current"),
        ("node_scan_escape", "node_scan_escape"),
        ("node_join_growth", "node_join_growth"),
        ("string_scan_escape", "string_scan_escape"),
        ("prepared", None),
        ("builder-control", None),
    ]
    apps = {}
    for name, kind in card_backends:
        if name == "prepared":
            target, build = stage_prepared(work, harness)
        elif name == "builder-control":
            target, build = stage_builder_control(work, harness)
        else:
            target, build = stage_card_backend(work, name, harness, rocci, lowered_src, kind)
        report["costs"]["builds"][name] = {"exit": build["exit"], "seconds": build["seconds"], "stderr": (build.get("stderr") or "")[-800:]}
        print(f"card build {name}: exit={build['exit']} ({build['seconds']:.2f}s)", flush=True)
        if build["exit"] == 0:
            apps[name] = target
            report["costs"]["builds"][name]["binary_bytes"] = (target / "app").stat().st_size

    names = list(apps)
    for name in names:
        target = apps[name]
        for rows in SIZE_SWEEP:
            for text, kind in [(CLEAN, "clean"), (ESCAPED, "escaped")]:
                key = f"{'small' if rows == 0 else 'r'+str(rows)}-{kind}-{rows}"
                report["costs"]["card"].setdefault(key, {})
                result = measure_app(harness, target, RENDER_REPS, rows, text)
                medians = [result[batch]["median_seconds"] for batch in ("batch1", "batch2") if result[batch].get("ok")]
                report["costs"]["card"][key][name] = {
                    **result,
                    "median": statistics.median(medians) if medians else None,
                }
        print(f"card timed {name}", flush=True)

    # Expensive 1000-row only for current node, prepared, builder, and any scan/join variants that built.
    for name, target in apps.items():
        result = measure_app(harness, target, EXPENSIVE_REPS, 1000, ESCAPED, timeout=60)
        medians = [result[b]["median_seconds"] for b in ("batch1", "batch2") if result[b].get("ok")]
        report["costs"]["card"].setdefault("r1000-escaped-1000", {})[name] = {**result, "median": statistics.median(medians) if medians else None}

    encode_target, encode_build = stage_encode_probe(work, harness)
    report["costs"]["builds"]["encode"] = {"exit": encode_build["exit"], "seconds": encode_build["seconds"], "stderr": (encode_build.get("stderr") or "")[-500:]}
    if encode_build["exit"] == 0:
        report["costs"]["encode"]["large-clean"] = timed_batch(
            harness, [encode_target / "app", RENDER_REPS, 100, CLEAN], encode_target, 30,
        )
        if "prepared" in apps:
            report["costs"]["encode"]["prepared_end_to_end"] = report["costs"]["card"].get("r100-clean-100", {}).get("prepared")

    # Combined winner only after individuals exist.
    scan = report["costs"]["card"].get("r100-escaped-100", {}).get("node_scan_escape", {}).get("median")
    join = report["costs"]["card"].get("r100-escaped-100", {}).get("node_join_growth", {}).get("median")
    node = report["costs"]["card"].get("r100-escaped-100", {}).get("rocci-node", {}).get("median")
    if scan and join and node and (scan < node or join < node):
        target, build = stage_card_backend(work, "node_scan_and_join", harness, rocci, lowered_src, "node_scan_and_join")
        report["costs"]["builds"]["node_scan_and_join"] = {"exit": build["exit"], "seconds": build["seconds"]}
        if build["exit"] == 0:
            for key, rows, text, reps in [
                ("small-clean-0", 0, CLEAN, RENDER_REPS),
                ("r100-escaped-100", 100, ESCAPED, RENDER_REPS),
            ]:
                result = measure_app(harness, target, reps, rows, text)
                medians = [result[b]["median_seconds"] for b in ("batch1", "batch2") if result[b].get("ok")]
                report["costs"]["card"].setdefault(key, {})["node_scan_and_join"] = {**result, "median": statistics.median(medians) if medians else None}

    # Flatten medians for the selector using 100-row escaped and 0-row clean.
    flat = {"card": {
        "large-escaped-100": {k: {"median": v.get("median")} for k, v in report["costs"]["card"].get("r100-escaped-100", {}).items()},
        "small-clean-0": {k: {"median": v.get("median")} for k, v in report["costs"]["card"].get("small-clean-0", {}).items()},
    }}
    report["cost_table"] = {
        "kernels": {name: {label: item.get("median") for label, item in labels.items() if isinstance(item, dict)} for name, labels in report["costs"]["kernels"].items()},
        "card_medians": {key: {name: (item or {}).get("median") for name, item in backends.items()} for key, backends in report["costs"]["card"].items()},
        "encode": report["costs"]["encode"],
        "builds": {name: {"seconds": item.get("seconds"), "binary_bytes": item.get("binary_bytes"), "exit": item.get("exit")} for name, item in report["costs"]["builds"].items()},
    }
    report["phase2_selection"] = select_candidate(flat)
    prepared = flat["card"]["large-escaped-100"].get("prepared", {}).get("median")
    builder = flat["card"]["large-escaped-100"].get("builder-control", {}).get("median")
    report["phase2_stop_condition"] = {
        "prepared_large": prepared,
        "builder_control_large": builder,
        "builder_beats_prepared": bool(prepared and builder and builder < prepared),
        "matched_algorithms_close": bool(prepared and builder and abs(prepared - builder) / max(prepared, builder) < 0.15),
        "note": "A direct string builder with scan/copy escape and join growth beat prepared rendering; do not present compile-time template preparation as the performance opportunity.",
    }
    return report


def measure_app_kernel(harness, target, reps, text):
    argv = [target / "app", reps, text]
    batch1 = timed_batch(harness, argv, target, 30)
    batch2 = timed_batch(harness, argv, target, 30)
    medians = [b["median_seconds"] for b in (batch1, batch2) if b.get("ok")]
    return {"batch1": batch1, "batch2": batch2, "median": statistics.median(medians) if medians else None}
