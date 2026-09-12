"""Phase 1 HTML compatibility contract. Imported by run.py; not a product module."""

from html import escape
import json
from pathlib import Path
import shutil


HERE = Path(__file__).resolve().parent
CLASSIFICATIONS = (
    "matches",
    "equivalent_serialization",
    "product_string_drift",
    "restricted_library_limitation",
    "unresolved_contract",
)
SUPPORTED_INTERPOLATION = {
    "text": "ordinary element text",
    "quoted_attribute": "complete double-quoted ordinary attribute values",
}
UNSUPPORTED_INTERPOLATION = {
    "dynamic_tag_name": "template-selected tag names",
    "dynamic_attribute_name": "template-selected attribute names",
    "script_context": "text inside script",
    "style_context": "text inside style",
    "event_handler_attr": "on* or other raw-script attributes",
    "raw_html": "unescaped markup from application strings",
    "roc_expressions": "arbitrary Roc in template text",
}
BOOLEAN_CONTRACT = {
    "rocci_syntax": "valueless attributes lower to boolean_attribute(name, True) only",
    "product_helper": "true and false both construct Attribute.attribute(name, \"\")",
    "string_helper": "true emits a valueless name; false omits the attribute",
    "reachable_from_rocci": False,
    "proposed_repair": {
        "owner": "crates/rocci-platform/platform/Html.roc",
        "also": ["crates/rocci-cli/runtime/Html.roc"],
        "behavior": "false must not serialize a present empty attribute; match HTML boolean presence semantics",
        "tests": "helper expect for True => disabled=\"\" or disabled; False => omitted",
        "not_in_this_plan": True,
    },
}


def require_html5lib():
    try:
        import html5lib
    except ImportError as error:
        raise RuntimeError("Phase 1 needs html5lib==1.1; pip install -r roc/template-preparation-experiment/requirements.txt") from error
    return html5lib


def parse_html5(source, document=False):
    html5lib = require_html5lib()
    if source is None:
        return {"ok": False, "error": "missing source", "events": []}
    if document:
        tree = html5lib.parse(source, treebuilder="etree", namespaceHTMLElements=False)
        events = walk_etree(tree)
    else:
        fragment = html5lib.parseFragment(source, treebuilder="etree", namespaceHTMLElements=False)
        events = []
        for node in fragment:
            events.extend(walk_etree(node))
    return {"ok": True, "error": None, "events": events, "parser": "html5lib"}


def walk_etree(node):
    if not hasattr(node, "tag"):
        text = str(node)
        return [("text", text)] if text else []
    tag = node.tag
    if isinstance(tag, str) and tag.startswith("{"):
        tag = tag.rsplit("}", 1)[-1]
    events = [("open", tag, tuple(sorted((str(k), str(v)) for k, v in node.attrib.items())))]
    if node.text:
        events.append(("text", node.text))
    for child in list(node):
        events.extend(walk_etree(child))
        if child.tail:
            events.append(("text", child.tail))
    events.append(("close", tag))
    return events


def preprocess_newlines(source):
    return source.replace("\r\n", "\n").replace("\r", "\n")


COMPAT_INPUTS = [
    {"name": "ordinary_text", "active": 1, "rows": 2, "text": "plain"},
    {"name": "empty_data", "active": 0, "rows": 0, "text": ""},
    {"name": "quotes_ampersands_angles", "active": 1, "rows": 2, "text": '&<>"\' café 😀'},
    {"name": "html_looking_amp", "active": 0, "rows": 3, "text": "<&amp;>"},
    {"name": "script_looking", "active": 1, "rows": 1, "text": '"><script>alert(1)</script>'},
    {"name": "cr_in_attribute", "active": 0, "rows": 1, "text": "a\nb\rc\td"},
    {"name": "lf_only", "active": 0, "rows": 1, "text": "a\nb"},
    {"name": "crlf", "active": 0, "rows": 1, "text": "a\r\nb"},
    {"name": "tab", "active": 0, "rows": 1, "text": "a\tb"},
    {"name": "unicode", "active": 1, "rows": 1, "text": "café 😀"},
    {"name": "long_string", "active": 0, "rows": 1, "text": "x" * 2048},
    {"name": "nested_and_empty_kids", "active": 1, "rows": 2, "text": "row"},
    {"name": "empty_list_active", "active": 1, "rows": 0, "text": "none"},
    {"name": "template_looking", "active": 1, "rows": 3, "text": "{{ name }} @if {}"},
]


MAIN = r'''
as_text = |arg| match OsStr.to_raw(arg) {
    Utf8(s) => s
    UnixBytes(bytes) => Str.from_utf8_lossy(bytes)
    _ => ""
}

main! = |args| {
    match args {
        [_, mode_arg, reps_arg, rows_arg, text_arg] => {
            mode = as_text(mode_arg)
            reps = U64.from_str(as_text(reps_arg)) ?? 1
            rows = U64.from_str(as_text(rows_arg)) ?? 1
            text = as_text(text_arg)
            items = List.from_iter((0..<rows).iter()).map(|i| {
                name: "${text}${i.to_str()}",
                kids: [{ name: "${text}k${i.to_str()}" }],
            })
            ctx = { title: text, active: reps > 0, items }
            match mode {
                "render" => Stdout.write!(render_page(ctx))?
                _ => return Err(Exit(2))
            }
            Ok({})
        }
        _ => Err(Exit(2))
    }
}
'''

BOOLEAN_MAIN = r'''
main! = |_| {
    true_btn = Html.element("button", [Html.boolean_attribute("disabled", Bool.True)], [Html.text("on")])
    false_btn = Html.element("button", [Html.boolean_attribute("disabled", Bool.False)], [Html.text("off")])
    Stdout.line!(Html.render_without_doc_type(true_btn))?
    Stdout.line!(Html.render_without_doc_type(false_btn))?
    Ok({})
}
'''

FRAGMENT_STRING_MAIN = r'''
main! = |_| {
    node = Html.element("p", [], [Html.text("Hi")])
    Stdout.line!(Html.render_without_doc_type(node))?
    Stdout.line!(Html.render(node))?
    Stdout.line!(Html.render_document(node))?
    Stdout.line!(Html.render_fragment(node))?
    Ok({})
}
'''

FRAGMENT_NODE_MAIN = r'''
main! = |_| {
    node = Html.element("p", [], [Html.text("Hi")])
    Stdout.line!(Html.render_without_doc_type(node))?
    Stdout.line!(Html.render(node))?
    Stdout.line!(Html.render_document(node).to_str())?
    Stdout.line!(Html.render_fragment([node, node]).to_str())?
    Ok({})
}
'''


def classify_pair(left_name, right_name, left_bytes, right_bytes, left_dom, right_dom):
    if left_bytes == right_bytes and left_dom == right_dom:
        return "matches"
    if left_dom == right_dom:
        return "equivalent_serialization"
    if {left_name, right_name} <= {"rocci-string", "rocci-node"}:
        return "product_string_drift"
    if "prepared" in (left_name, right_name):
        return "restricted_library_limitation"
    return "unresolved_contract"


def matrix_row(case, outputs, html5):
    row = {
        "name": case["name"],
        "authored": {"active": case["active"], "rows": case["rows"], "text": case["text"]},
        "bytes": {backend: (outputs.get(backend) or {}).get("stdout_sha256") for backend in outputs},
        "byte_equal": len({(outputs.get(backend) or {}).get("stdout") for backend in outputs}) == 1,
        "parsed": {},
        "classifications": {},
    }
    parsed_events = {}
    for backend, item in outputs.items():
        source = item.get("stdout")
        parsed = html5(source) if source is not None and not item.get("invalid_utf8") else {"ok": False, "events": [], "error": "invalid or missing"}
        parsed_events[backend] = parsed.get("events")
        row["parsed"][backend] = {
            "ok": parsed.get("ok"),
            "error": parsed.get("error"),
            "events": parsed.get("events"),
        }
    backends = list(outputs)
    for i, left in enumerate(backends):
        for right in backends[i + 1 :]:
            row["classifications"][f"{left}_vs_{right}"] = classify_pair(
                left, right,
                (outputs.get(left) or {}).get("stdout"),
                (outputs.get(right) or {}).get("stdout"),
                parsed_events.get(left),
                parsed_events.get(right),
            )
    return row


def write_compat_apps(work, rocci, harness):
    lowered = harness.command([rocci, "build", HERE / "Compat.rocci"], harness.REPO, timeout=60)
    if lowered["exit"] != 0 or lowered["timed_out"]:
        raise harness.ExperimentError("rocci-template failed to lower Compat.rocci")
    inspect = harness.command([rocci, "inspect", "--ast", HERE / "Compat.rocci"], harness.REPO)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    generated = {}
    for backend in harness.BACKENDS:
        target = work / "compat" / backend
        target.mkdir(parents=True)
        if backend == "prepared":
            for name in [*harness.HASHES, "TypedTemplate.roc"]:
                shutil.copyfile(work / name, target / name)
            text = (HERE / "compat.mustache.html").read_text().removesuffix("\n")
            (target / "compat.mustache.html").write_text(text)
            body = (
                'import TypedTemplate\nimport "compat.mustache.html" as source : Str\n'
                'sample : { title : Str, active : Bool, items : List({ name : Str, kids : List({ name : Str }) }) }\n'
                'sample = { title: "", active: Bool.True, items: [{ name: "", kids: [{ name: "" }] }] }\n'
                'render_page = TypedTemplate.prepare("compat.mustache.html", source, sample)\n'
            )
        else:
            body = (lowered["stdout"] or "") + "\nrender_page = |ctx| Html.render_without_doc_type(compat(ctx))\n"
            if backend == "rocci-string":
                shutil.copyfile(harness.REPO / "crates/rocci-ui/runtime/Html.roc", target / "Html.roc")
            else:
                (target / "Runtime").mkdir()
                for name in ["Html.roc", "Attribute.roc"]:
                    shutil.copyfile(harness.REPO / "crates/rocci-platform/platform" / name, target / "Runtime" / name)
                wrapper = (harness.REPO / "crates/rocci-cli/runtime/Html.roc").read_text()
                wrapper = wrapper.replace("import pf.Attribute", "import Runtime/Attribute").replace("import pf.Html", "import Runtime/Html")
                (target / "Html.roc").write_text(wrapper)
        (target / "main.roc").write_text(header + body + MAIN)
        generated[backend] = harness.sha256_file(target / "main.roc")
    return inspect, generated, lowered


def build_backend(target, harness):
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
    return build


def render_cases(target, harness):
    outputs = {}
    for case in COMPAT_INPUTS:
        result = harness.command([target / "app", "render", case["active"], case["rows"], case["text"]], target, timeout=30)
        outputs[case["name"]] = result
    return outputs


def write_helper_app(work, name, harness, body_main, string_runtime):
    target = work / "compat" / name
    target.mkdir(parents=True)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.Stdout\n'
    if string_runtime:
        shutil.copyfile(harness.REPO / "crates/rocci-ui/runtime/Html.roc", target / "Html.roc")
        source = header + "import Html\n" + body_main
    else:
        (target / "Runtime").mkdir()
        for filename in ["Html.roc", "Attribute.roc"]:
            shutil.copyfile(harness.REPO / "crates/rocci-platform/platform" / filename, target / "Runtime" / filename)
        wrapper = (harness.REPO / "crates/rocci-cli/runtime/Html.roc").read_text()
        wrapper = wrapper.replace("import pf.Attribute", "import Runtime/Attribute").replace("import pf.Html", "import Runtime/Html")
        (target / "Html.roc").write_text(wrapper)
        source = header + "import Html\n" + body_main
    (target / "main.roc").write_text(source)
    build = build_backend(target, harness)
    run = harness.command([target / "app"], target, timeout=15) if build["exit"] == 0 and not build["timed_out"] else build
    return {"build": build, "run": run, "stdout": run.get("stdout"), "invalid_utf8": run.get("invalid_utf8")}


def unsupported_rows():
    return [
        {
            "name": name,
            "supported": False,
            "classification": "restricted_library_limitation",
            "detail": detail,
        }
        for name, detail in UNSUPPORTED_INTERPOLATION.items()
    ]


def run_compat(harness, work, report):
    require_html5lib()
    rocci = harness.REPO / "target/debug/rocci-template"
    cargo = harness.command(["cargo", "build", "-q", "-p", "rocci-template"], harness.REPO, timeout=120)
    if cargo["exit"] != 0:
        raise harness.ExperimentError("cargo build -p rocci-template failed")
    inspect, generated, lowered = write_compat_apps(work, rocci, harness)
    report["compat_inspect"] = inspect
    report["compat_generated_hashes"] = generated
    report["compat_lowered_ok"] = lowered["exit"] == 0
    comparisons = {}
    for backend in harness.BACKENDS:
        target = work / "compat" / backend
        build = build_backend(target, harness)
        print(f"compat build {backend}: exit={build['exit']} ({build['seconds']:.2f}s)", flush=True)
        comparisons[backend] = {"build": build}
        if build["exit"] != 0 or build["timed_out"]:
            continue
        comparisons[backend]["outputs"] = render_cases(target, harness)
    report["compat_comparisons"] = comparisons
    matrix = []
    for case in COMPAT_INPUTS:
        outputs = {}
        for backend in harness.BACKENDS:
            item = (comparisons.get(backend) or {}).get("outputs", {}).get(case["name"])
            if item:
                outputs[backend] = item
        matrix.append(matrix_row(case, outputs, parse_html5))
    cr = next(row for row in matrix if row["name"] == "cr_in_attribute")
    quotes = next(row for row in matrix if row["name"] == "quotes_ampersands_angles")
    for row in (cr, quotes):
        for backend, item in (comparisons.items()):
            output = (item.get("outputs") or {}).get(row["name"], {})
            source = output.get("stdout") or ""
            raw_events = harness.HtmlEvents(source).events if source else []
            preprocessed = harness.HtmlEvents(preprocess_newlines(source)).events if source else []
            row.setdefault("newline_preprocessing_distinct", {})[backend] = {
                "raw_python_events": raw_events,
                "preprocessed_python_events": preprocessed,
                "html5_events": row["parsed"].get(backend, {}).get("events"),
            }
    report["compatibility_matrix"] = matrix
    phase0_path = harness.REPO / "knowledge/research/rocci/compile-time-template-preparation-phase-0-results.json"
    report["benchmarked_matrix"] = classify_phase0(phase0_path, harness)
    report["html5lib_version"] = require_html5lib().__version__
    report["boolean_probes"] = {
        "string": write_helper_app(work, "boolean-string", harness, BOOLEAN_MAIN, True),
        "node": write_helper_app(work, "boolean-node", harness, BOOLEAN_MAIN, False),
        "contract": BOOLEAN_CONTRACT,
    }
    report["document_fragment_probes"] = {
        "string": write_helper_app(work, "fragment-string", harness, FRAGMENT_STRING_MAIN, True),
        "node": write_helper_app(work, "fragment-node", harness, FRAGMENT_NODE_MAIN, False),
    }
    classify_boolean_and_fragments(report)
    report["supported_interpolation"] = SUPPORTED_INTERPOLATION
    report["unsupported_cases"] = unsupported_rows()
    report["unexplained_benchmark_differences"] = unexplained(
        report["benchmarked_matrix"],
        expected_dom_drift=("cr_in_attribute",),
    )
    report["unexplained_compat_differences"] = unexplained(
        matrix,
        expected_dom_drift=("cr_in_attribute", "crlf"),
    )
    return report


def classify_phase0(path, harness):
    data = json.loads(path.read_text())
    matrix = []
    for case in harness.RENDER_CASES:
        outputs = {}
        for backend in harness.BACKENDS:
            item = data["comparisons"][backend]["named_outputs"][case["name"]]
            outputs[backend] = item
        matrix.append(matrix_row(case, outputs, parse_html5))
    return matrix


def classify_boolean_and_fragments(report):
    string_out = ((report.get("boolean_probes") or {}).get("string") or {}).get("stdout") or ""
    node_out = ((report.get("boolean_probes") or {}).get("node") or {}).get("stdout") or ""
    string_lines = [line for line in string_out.splitlines() if line]
    node_lines = [line for line in node_out.splitlines() if line]
    report["boolean_probes"]["observed"] = {
        "string_true": string_lines[0] if string_lines else None,
        "string_false": string_lines[1] if len(string_lines) > 1 else None,
        "node_true": node_lines[0] if node_lines else None,
        "node_false": node_lines[1] if len(node_lines) > 1 else None,
        "classification": "product_string_drift",
        "reachable_from_rocci": False,
    }
    string_frag = ((report.get("document_fragment_probes") or {}).get("string") or {}).get("stdout") or ""
    node_frag = ((report.get("document_fragment_probes") or {}).get("node") or {}).get("stdout") or ""
    report["document_fragment_probes"]["observed"] = {
        "string_lines": string_frag.splitlines(),
        "node_lines": node_frag.splitlines(),
        "classification": "product_string_drift" if string_frag != node_frag else "matches",
    }


def unexplained(matrix, expected_dom_drift=()):
    unexplained_rows = []
    for row in matrix:
        classes = row.get("classifications") or {}
        if "unresolved_contract" in classes.values():
            unexplained_rows.append(row["name"])
            continue
        node_string = classes.get("rocci-string_vs_rocci-node")
        if node_string == "product_string_drift" and row["name"] not in expected_dom_drift:
            unexplained_rows.append(row["name"])
    return unexplained_rows
