"""Phase 1 string/theme escape. Copies string Html.roc only inside the work dir."""

import hashlib
import shutil
import statistics
from pathlib import Path


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
KERNEL_REPS = 20000
PAINTER_REPS = 800
NOISE_S = 0.005
CLEAN = "plain"
ESCAPED = "<&\"' café>"
CSS_TEXT = "body { color: #333; } a[href*=\"&\"] { content: \"<&'>\" }"

PAINTERS = [
    (
        "NavList",
        REPO / "crates/rocci-ui/templates/chrome/NavList.rocci",
    ),
    (
        "Breadcrumbs",
        REPO / "crates/rocci-ui/templates/chrome/Breadcrumbs.rocci",
    ),
    (
        "PageOutline",
        REPO / "crates/rocci-ui/templates/chrome/PageOutline.rocci",
    ),
    (
        "RocdownTheme",
        REPO / "crates/rocci-rocdown/templates/RocdownTheme.rocci",
    ),
]

KERNEL_STRING_SPLIT = '''
escape = |value| {
    replace = |haystack, needle, replacement| Str.join_with(Str.split_on(haystack, needle), replacement)
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
    )
}
'''

KERNEL_STRING_SCAN = '''
escape = |value| {
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

PAINTER_ROC = r'''
empty_item = { title: "", href: "", class_name: "" }

small_view = {
    site: {
        title: "Rocci",
        description: "",
        base_url: "",
        language: "en",
        repository: "",
        social_image: "",
        favicon: "",
        apple_touch_icon: "",
        subtitle: "",
        footer: "",
    },
    lanes: [],
    sidebar: [],
    route: "/",
    title: "Hi",
    document_title: "Hi",
    description: "",
    layout: "plain",
    published: "",
    updated: "",
    authors: [],
    tags: [],
    collection: "",
    collection_items: [],
    outline: [],
    breadcrumbs: [],
    previous: empty_item,
    next: empty_item,
    resources: {
        stylesheet: "/theme.css",
        csp: "",
        canonical: "",
        module_script: "",
        chrome_script: "",
        playground_css: "",
        playground_session: "",
    },
}

painter_view = {
    site: {
        title: "Rocci",
        description: "Docs",
        base_url: "https://rocci.dev",
        language: "en",
        repository: "https://github.com/koliyo/rocci",
        social_image: "",
        favicon: "/favicon.svg",
        apple_touch_icon: "",
        subtitle: "hypermedia",
        footer: "Apache-2.0",
    },
    lanes: [
        { label: "Docs", href: "/docs/", current: Bool.True },
        { label: "Status", href: "/status/", current: Bool.False },
    ],
    sidebar: [
        {
            title: "Reference",
            href: "/docs/reference/",
            open: Bool.True,
            items: [
                { title: "Overview", href: "/docs/reference/", class_name: "nav-link nav-child" },
            ],
            children: [
                {
                    title: "Rocci language reference",
                    href: "/docs/reference/language/",
                    open: Bool.True,
                    items: [
                        { title: "Overview", href: "/docs/reference/language/", class_name: "nav-link nav-child" },
                        { title: "File structure", href: "/docs/reference/language/file-structure/", class_name: "nav-link nav-child" },
                    ],
                    children: [],
                },
                {
                    title: "Runtime and HTTP",
                    href: "/docs/reference/runtime/",
                    open: Bool.False,
                    items: [],
                    children: [],
                },
            ],
        },
    ],
    route: "/docs/reference/language/",
    title: title_text,
    document_title: title_text,
    description: css_text,
    layout: "docs",
    published: "2026-09-12",
    updated: "",
    authors: ["nils"],
    tags: ["rocci"],
    collection: "",
    collection_items: [],
    outline: [
        { id: "goal", title: "Goal", level: "2" },
        { id: "bound", title: "Bound", level: "3" },
    ],
    breadcrumbs: [
        { title: "Rocci", href: "/" },
        { title: "Reference", href: "/docs/reference/" },
        { title: title_text, href: "/docs/reference/language/" },
    ],
    previous: { title: "Overview", href: "/docs/reference/", class_name: "" },
    next: { title: "Runtime", href: "/docs/reference/runtime/", class_name: "" },
    resources: {
        stylesheet: "/theme.css",
        csp: "default-src 'none'",
        canonical: "https://rocci.dev/docs/reference/language/",
        module_script: "",
        chrome_script: "/goto.js",
        playground_css: "",
        playground_session: "",
    },
}

view_for = |kind, text| {
    base = if kind == "small" small_view else painter_view
    {
        site: base.site,
        lanes: base.lanes,
        sidebar: base.sidebar,
        route: base.route,
        title: text,
        document_title: text,
        description: if kind == "small" "" else Str.concat(base.description, text),
        layout: base.layout,
        published: base.published,
        updated: base.updated,
        authors: base.authors,
        tags: base.tags,
        collection: base.collection,
        collection_items: base.collection_items,
        outline: base.outline,
        breadcrumbs: base.breadcrumbs,
        previous: base.previous,
        next: base.next,
        resources: base.resources,
    }
}

render_page = |kind, text| Html.render(RocdownTheme.siteShell(view_for(kind, text), Html.text(text)))
'''

PAINTER_MAIN = r'''
as_text = |arg| match OsStr.to_raw(arg) {
    Utf8(s) => s
    UnixBytes(bytes) => Str.from_utf8_lossy(bytes)
    _ => ""
}

main! = |args| {
    match args {
        [_, mode_arg, kind_arg, text_arg] => {
            mode = as_text(mode_arg)
            kind = as_text(kind_arg)
            text = as_text(text_arg)
            match mode {
                "digest" => {
                    html = render_page(kind, text)
                    Stdout.line!(html)?
                    Ok({})
                }
                _ => Err(Exit(2))
            }
        }
        [_, mode_arg, reps_arg, kind_arg, text_arg] => {
            mode = as_text(mode_arg)
            reps = U64.from_str(as_text(reps_arg)) ?? 1
            kind = as_text(kind_arg)
            text = as_text(text_arg)
            match mode {
                "bench" => {
                    var $n = 0.U64
                    for i in 0..<reps {
                        html = render_page(kind, if i % 2 == 0 text else Str.concat(text, "X"))
                        $n = $n + html.count_utf8_bytes()
                    }
                    Stdout.line!($n.to_str())?
                    Ok({})
                }
                _ => Err(Exit(2))
            }
        }
        _ => Err(Exit(2))
    }
}
'''


def wrap_type_module(src, type_name):
    lines = src.splitlines()
    imports = []
    body = []
    for line in lines:
        if line.startswith("module ") and " exposing " in line:
            continue
        if line.startswith("import "):
            imports.append(line)
        else:
            body.append(line)
    while body and not body[0].strip():
        body.pop(0)
    while body and not body[-1].strip():
        body.pop()
    out = []
    if imports:
        out.extend(imports)
        out.append("")
    out.append(f"{type_name} := [].{{")
    for line in body:
        out.append(f"    {line}" if line else "")
    out.append("}")
    return "\n".join(out) + "\n"


def rewrite_html_annos(roc):
    return roc.replace(", Html -> Html", ", Str -> Str").replace(" -> Html\n", " -> Str\n")


def improvement(old, new):
    if not old or not new:
        return None
    return (old - new) / old


def load_costs():
    import importlib.util

    spec = importlib.util.spec_from_file_location("costs", HERE / "costs.py")
    costs = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(costs)
    return costs


def compile_painters(harness, rocci, dest):
    generated = {}
    dest.mkdir(parents=True, exist_ok=True)
    for type_name, path in PAINTERS:
        lowered = harness.command([rocci, "build", path], harness.REPO, timeout=60)
        if lowered["exit"] != 0:
            return {"ok": False, "failed": type_name, "stderr": (lowered.get("stderr") or "")[-800:]}, {}
        wrapped = rewrite_html_annos(wrap_type_module(lowered["stdout"] or "", type_name))
        (dest / f"{type_name}.roc").write_text(wrapped)
        generated[type_name] = {
            "bytes": len(wrapped.encode()),
            "sha256": hashlib.sha256(wrapped.encode()).hexdigest(),
        }
    return {"ok": True}, generated


def stage_painter(work, name, harness, html_text, lowered_dir):
    target = work / "theme-escape" / "painter" / name
    target.mkdir(parents=True)
    for type_name, _path in PAINTERS:
        shutil.copyfile(lowered_dir / f"{type_name}.roc", target / f"{type_name}.roc")
    (target / "Html.roc").write_text(html_text)
    header = (
        f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\n'
        "import pf.OsStr\nimport pf.Stdout\nimport Html\nimport RocdownTheme\n"
    )
    (target / "main.roc").write_text(
        header
        + "title_text = \"Docs\"\ncss_text = "
        + roc_string(CSS_TEXT)
        + "\n"
        + PAINTER_ROC
        + PAINTER_MAIN
    )
    build = harness.command(
        ["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"],
        target,
        timeout=180,
    )
    return target, build


def roc_string(value):
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def run_theme_escape(harness, work, report):
    costs = load_costs()
    rocci = harness.REPO / "target/debug/rocci-template"
    cargo = harness.command(["cargo", "build", "-q", "-p", "rocci-template"], harness.REPO, timeout=120)
    if cargo["exit"] != 0:
        raise harness.ExperimentError("cargo build -p rocci-template failed")
    report["theme_escape"] = {"kernels": {}, "painter": {}, "builds": {}, "bytes_equal": {}, "screen": {}}
    report["theme_escape_requested"] = True

    rocdown_html = (REPO / "crates/rocci-rocdown/runtime/Html.roc").read_text()
    ui_html = (REPO / "crates/rocci-ui/runtime/Html.roc").read_text()
    report["theme_escape"]["html_copies_identical"] = rocdown_html == ui_html
    report["product_html_hashes"]["crates/rocci-rocdown/runtime/Html.roc"] = hashlib.sha256(
        rocdown_html.encode()
    ).hexdigest()

    for name, src in [
        ("string_split_escape", KERNEL_STRING_SPLIT),
        ("string_scan_escape", KERNEL_STRING_SCAN),
    ]:
        target, build = costs.write_kernel(work / "theme-escape", name, src, harness)
        report["theme_escape"]["builds"][f"kernel_{name}"] = {
            "exit": build["exit"],
            "seconds": build["seconds"],
            "stderr": (build.get("stderr") or "")[-500:],
        }
        print(f"theme kernel build {name}: exit={build['exit']} ({build['seconds']:.2f}s)", flush=True)
        if build["exit"] != 0:
            continue
        report["theme_escape"]["kernels"][name] = {}
        for label, text in [("clean", CLEAN), ("escaped", ESCAPED)]:
            report["theme_escape"]["kernels"][name][label] = costs.measure_app_kernel(
                harness, target, KERNEL_REPS, text
            )

    html_split = rocdown_html
    html_scan = costs.patch_string_html(rocdown_html)
    lowered_dir = work / "theme-escape" / "lowered"
    compiled, generated = compile_painters(harness, rocci, lowered_dir)
    report["theme_escape"]["builds"]["lower"] = compiled
    report["theme_escape"]["generated"] = generated
    if not compiled["ok"]:
        raise harness.ExperimentError(f"failed to lower {compiled.get('failed')}")
    apps = {}
    for name, html in [("string_split", html_split), ("string_scan", html_scan)]:
        target, build = stage_painter(work, name, harness, html, lowered_dir)
        report["theme_escape"]["builds"][name] = {
            "exit": build["exit"],
            "seconds": build["seconds"],
            "stderr": (build.get("stderr") or "")[-1200:],
        }
        print(f"theme painter build {name}: exit={build['exit']} ({build['seconds']:.2f}s)", flush=True)
        if build["exit"] == 0:
            apps[name] = target
            report["theme_escape"]["builds"][name]["binary_bytes"] = (target / "app").stat().st_size

    if "string_split" in apps and "string_scan" in apps:
        for kind, text in [
            ("small", CLEAN),
            ("painter", ESCAPED),
            ("cr", "a\rb"),
        ]:
            left = harness.command(
                [apps["string_split"] / "app", "digest", "painter" if kind == "cr" else kind, text],
                apps["string_split"],
                timeout=30,
            )
            right = harness.command(
                [apps["string_scan"] / "app", "digest", "painter" if kind == "cr" else kind, text],
                apps["string_scan"],
                timeout=30,
            )
            report["theme_escape"]["bytes_equal"][kind] = {
                "equal": left.get("stdout") == right.get("stdout")
                and left.get("exit") == 0
                and right.get("exit") == 0,
                "split_exit": left.get("exit"),
                "scan_exit": right.get("exit"),
                "split_bytes": len((left.get("stdout") or "").encode()),
                "scan_bytes": len((right.get("stdout") or "").encode()),
            }

    for name, target in apps.items():
        report["theme_escape"]["painter"][name] = {}
        for kind, text, reps in [("small", CLEAN, PAINTER_REPS), ("painter", ESCAPED, PAINTER_REPS)]:
            argv = [target / "app", "bench", reps, kind, text]
            batch1 = costs.timed_batch(harness, argv, target, 90)
            batch2 = costs.timed_batch(harness, argv, target, 90)
            medians = [b["median_seconds"] for b in (batch1, batch2) if b.get("ok")]
            report["theme_escape"]["painter"][name][kind] = {
                "batch1": batch1,
                "batch2": batch2,
                "median": statistics.median(medians) if medians else None,
                "reps": reps,
            }
            print(f"theme painter timed {name} {kind}", flush=True)

    split_painter = ((report["theme_escape"]["painter"].get("string_split") or {}).get("painter") or {}).get("median")
    scan_painter = ((report["theme_escape"]["painter"].get("string_scan") or {}).get("painter") or {}).get("median")
    split_small = ((report["theme_escape"]["painter"].get("string_split") or {}).get("small") or {}).get("median")
    scan_small = ((report["theme_escape"]["painter"].get("string_scan") or {}).get("small") or {}).get("median")
    gain = improvement(split_painter, scan_painter)
    if split_small and scan_small and abs(scan_small - split_small) <= NOISE_S:
        small_regression = 0.0
    else:
        small_reg = improvement(split_small, scan_small)
        small_regression = None if small_reg is None else (-small_reg if small_reg < 0 else 0.0)
    passes = bool(gain is not None and gain >= 0.15 and (small_regression is None or small_regression <= 0.05))
    report["theme_escape"]["screen"] = {
        "painter_gain": gain,
        "small_regression": small_regression,
        "small_noise_floor_s": NOISE_S,
        "painter_split_median": split_painter,
        "painter_scan_median": scan_painter,
        "small_split_median": split_small,
        "small_scan_median": scan_small,
        "passes_screen": passes,
        "rule": ">=15% painter-shaped vs current string split/join, <=5% small-case regression after noise",
        "decision": "string_kernel" if passes else "status_quo",
        "embed_css_note": "rocci-template default embed_css true puts scoped CSS through Html.text; product theme compile sets embed_css false.",
    }
    report["theme_escape"]["ok"] = bool(
        report["theme_escape"]["bytes_equal"].get("painter", {}).get("equal")
        and report["theme_escape"]["bytes_equal"].get("small", {}).get("equal")
        and split_painter
        and scan_painter
        and split_small
        and scan_small
        and not report.get("error")
    )
    return report
