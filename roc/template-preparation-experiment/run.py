#!/usr/bin/env python3
"""Explicit, isolated experiment. No product integration or default CI tests."""

import argparse
from datetime import datetime, timezone
import hashlib
from html import escape
from html.parser import HTMLParser
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import time
import urllib.request


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
REVISION = "e13a34b63ee466a03588e1e3cba85fcb4d045f71"
HASHES = {
    "Template.roc": "c4c65a2a15553e2c805d9c5258e315b044535fd18d4bc299d67161c1868bc78f",
    "Value.roc": "ce0029d866f81f0f7979ce87908c91106be3628892de42074fe42dcf8b315790",
    "Formatters.roc": "f52550ec833b5f62807ae9d1a3175f1e20e9891ce3042b125cd6a278f0d71060",
    "RuntimeFormatters.roc": "743063f3dd49195bc06968a7c339f6ec5333c1e1a99085ac1754ddd4a2142439",
}
PLATFORM = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst"
PLATFORM_CACHE = Path.home() / ".cache/roc/packages/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst"
BACKENDS = ["prepared", "rocci-string", "rocci-node"]
WORKLOADS = [
    {"name": "small-clean", "rows": 0, "text": "plain"},
    {"name": "large-clean", "rows": 100, "text": "plain"},
    {"name": "large-escaped", "rows": 100, "text": "<& café>"},
]
RENDER_CASES = [
    {"name": "ordinary_text", "active": 1, "rows": 2, "text": "plain"},
    {"name": "empty_data", "active": 0, "rows": 0, "text": ""},
    {"name": "quotes_ampersands_angles", "active": 1, "rows": 2, "text": '&<>"\' café 😀'},
    {"name": "html_looking_amp", "active": 0, "rows": 3, "text": "<&amp;>"},
    {"name": "script_looking", "active": 1, "rows": 1, "text": '"><script>alert(1)</script>'},
    {
        "name": "cr_in_attribute",
        "active": 0,
        "rows": 1,
        "text": "a\nb\rc\td",
        "expected_compatible": {"prepared": False, "rocci-string": False, "rocci-node": True},
    },
    {"name": "template_looking", "active": 1, "rows": 3, "text": "{{ name }} @if {}"},
]
PROBE_CASES = [
    {"name": "same", "typ": "{ name : Str }", "sample": '{ name: "" }', "source": "{{ name }}", "actual": '{ name: "Ada" }', "output": "Ada", "diagnostic": None},
    {"name": "reordered", "typ": "{ name : Str, title : Str }", "sample": '{ name: "", title: "" }', "source": "{{ name }}:{{ title }}", "actual": '{ title: "Dr", name: "Ada" }', "output": "Ada:Dr", "diagnostic": None},
    {"name": "extra", "typ": "{ name : Str }", "sample": '{ name: "" }', "source": "{{ name }}", "actual": '{ aaa: "WRONG", name: "Ada" }', "output": None, "diagnostic": "type mismatch"},
    {"name": "missing", "typ": "{ name : Str, title : Str }", "sample": '{ name: "", title: "" }', "source": "{{ name }}", "actual": '{ name: "Ada" }', "output": None, "diagnostic": "type mismatch"},
    {"name": "scalar_type", "typ": "{ name : Str }", "sample": '{ name: "" }', "source": "{{ name }}", "actual": '{ name: 1.I64 }', "output": None, "diagnostic": "type mismatch"},
    {"name": "nested_extra", "typ": "{ user : { name : Str } }", "sample": '{ user: { name: "" } }', "source": "{{ user.name }}", "actual": '{ user: { aaa: "WRONG", name: "Ada" } }', "output": None, "diagnostic": "type mismatch"},
    {"name": "nested_type", "typ": "{ user : { name : Str } }", "sample": '{ user: { name: "" } }', "source": "{{ user.name }}", "actual": '{ user: { name: 1.I64 } }', "output": None, "diagnostic": "type mismatch"},
    {"name": "list_extra", "typ": "{ items : List({ name : Str }) }", "sample": '{ items: [{ name: "" }] }', "source": "{{#items}}{{ name }}{{/items}}", "actual": '{ items: [{ aaa: "WRONG", name: "Ada" }] }', "output": None, "diagnostic": "type mismatch"},
    {"name": "list_type", "typ": "{ items : List({ name : Str }) }", "sample": '{ items: [{ name: "" }] }', "source": "{{#items}}{{ name }}{{/items}}", "actual": '{ items: [{ name: 1.I64 }] }', "output": None, "diagnostic": "type mismatch"},
    {"name": "empty_runtime", "typ": "{ items : List({ name : Str }) }", "sample": '{ items: [{ name: "" }] }', "source": "{{#items}}{{ name }}{{/items}}{{^items}}empty{{/items}}", "actual": '{ items: [] }', "output": "empty", "diagnostic": None},
    {"name": "empty_sample", "typ": "{ items : List({ name : Str }) }", "sample": '{ items: [] }', "source": "{{#items}}{{ name }}{{/items}}", "actual": '{ items: [] }', "output": None, "diagnostic": "sample list `items` is empty"},
    {"name": "empty_static_body", "typ": "{ items : List(Str) }", "sample": '{ items: [] }', "source": "{{#items}}x{{/items}}", "actual": '{ items: ["a", "b"] }', "output": "xx", "diagnostic": None},
    {"name": "bad_field", "typ": "{ name : Str }", "sample": '{ name: "" }', "source": "first\n{{ missing }}", "actual": '{ name: "Ada" }', "output": None, "diagnostic": "bad_field.mustache: line 2"},
    {"name": "bad_syntax", "typ": "{ name : Str }", "sample": '{ name: "" }', "source": "first\n{{#name}}", "actual": '{ name: "Ada" }', "output": None, "diagnostic": "bad_syntax.mustache"},
    {"name": "unsupported_u8", "typ": "{ name : Str, unused : U8 }", "sample": '{ name: "", unused: 1.U8 }', "source": "{{ name }}", "actual": '{ name: "Ada", unused: 2.U8 }', "output": None, "diagnostic": "encode_u8"},
    {"name": "unsupported_union", "typ": "{ state : [Ready, Busy] }", "sample": '{ state: Ready }', "source": "constant", "actual": '{ state: Busy }', "output": None, "diagnostic": "encode_tag"},
]
EXPECTED_PROBE_NAMES = [case["name"] for case in PROBE_CASES]
ROC_FLAGS = {
    "check_no_cache": ["roc", "check", "--no-cache"],
    "check_warm": ["roc", "check"],
    "build_no_cache": ["roc", "build", "--no-cache", "--opt=speed"],
    "build_warm": ["roc", "build", "--opt=speed"],
    "test_default": ["roc", "test"],
    "test_no_cache": ["roc", "test", "--no-cache"],
}
REQUIRED_KEYS = [
    "timestamp", "timestamp_end", "engine_revision", "engine_sha256", "os",
    "architecture", "cpu", "machine", "python", "roc", "roc_flags",
    "rocci_revision", "input_hashes", "product_html_hashes", "generated_hashes",
    "platform_artifact", "dirty_tracked", "untracked_experimental", "work_dir",
    "repetitions", "workloads", "render_cases", "probes", "probes_no_cache",
    "upstream_tests", "comparisons", "harness_ok", "type_contract_ok",
    "html_compatible", "html_expected_findings_confirmed", "error",
]
EXPERIMENTAL_PREFIXES = (
    "roc/template-preparation-experiment/",
    "knowledge/plans/rocci/compile-time-template-preparation",
    "knowledge/research/rocci/compile-time-template-preparation",
)


class ExperimentError(Exception):
    """Harness or input failure that must still yield a complete receipt."""


class HtmlEvents(HTMLParser):
    """Compare this small fixture's parsed text and attributes, not entity spelling."""

    def __init__(self, source):
        super().__init__(convert_charrefs=True)
        self.events = []
        self.feed(source.replace("\r\n", "\n").replace("\r", "\n"))
        self.close()

    def handle_starttag(self, tag, attrs):
        self.events.append(("open", tag, sorted(attrs)))

    def handle_endtag(self, tag):
        self.events.append(("close", tag))

    def handle_data(self, data):
        if self.events and self.events[-1][0] == "text":
            self.events[-1] = ("text", self.events[-1][1] + data)
        else:
            self.events.append(("text", data))


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha256_file(path):
    return sha256_bytes(path.read_bytes())


def expected_compatible(case, backend):
    configured = case.get("expected_compatible")
    if configured is None:
        return True
    return configured[backend]


def reference_html(active, rows, text):
    def attr(value):
        return escape(value, quote=True).replace("\r", "&#13;").replace("\n", "&#10;")

    items = "".join(
        f'<li title="{attr(text + str(i))}">{escape(text + str(i), quote=True)}</li>'
        for i in range(rows)
    )
    return (
        f'<section title="{attr(text)}"><h1>{escape(text, quote=True)}</h1>'
        f'{"<b>active</b>" if active else ""}<ul>{items}</ul></section>'
    )


def decode_utf8(data):
    try:
        return data.decode("utf-8"), False
    except UnicodeDecodeError:
        return None, True


def command_record(argv, exit_code, seconds, stdout, stderr, timed_out):
    stdout_text, stdout_invalid = decode_utf8(stdout)
    stderr_text, stderr_invalid = decode_utf8(stderr)
    return {
        "command": [str(a) for a in argv],
        "exit": exit_code,
        "seconds": seconds,
        "timed_out": timed_out,
        "stdout": stdout_text,
        "stderr": stderr_text,
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
        "stdout_len": len(stdout),
        "stderr_len": len(stderr),
        "invalid_utf8": stdout_invalid or stderr_invalid,
    }


def terminate_group(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=1)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            return
        process.wait(timeout=5)


def command(argv, cwd, timeout=15, env=None):
    start = time.perf_counter()
    try:
        process = subprocess.Popen(
            [str(a) for a in argv],
            cwd=cwd,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
    except OSError as error:
        return command_record(argv, None, time.perf_counter() - start, b"", str(error).encode(), False)
    try:
        stdout, stderr = process.communicate(timeout=timeout)
        return command_record(argv, process.returncode, time.perf_counter() - start, stdout, stderr, False)
    except subprocess.TimeoutExpired:
        terminate_group(process)
        stdout, stderr = process.communicate()
        return command_record(argv, None, time.perf_counter() - start, stdout, stderr, True)


def probe_outcome(result, diagnostic, output=None, execution=None):
    if result.get("timed_out") or result.get("exit") is None:
        return False
    if diagnostic:
        combined = f"{result.get('stdout') or ''}{result.get('stderr') or ''}"
        return result["exit"] != 0 and diagnostic in combined
    if result["exit"] != 0:
        return False
    if execution is None:
        return True
    if execution.get("timed_out") or execution.get("invalid_utf8"):
        return False
    return execution.get("exit") == 0 and (execution.get("stdout") or "").strip() == output


def cpu_label():
    if platform.system() == "Darwin":
        result = command(["sysctl", "-n", "machdep.cpu.brand_string"], REPO)
        if result["exit"] == 0 and result["stdout"]:
            return result["stdout"].strip()
    return platform.processor() or platform.machine()


def git_lines(args):
    result = command(["git", *args], REPO)
    text = result["stdout"] or ""
    return [line for line in text.splitlines() if line]


def parse_porcelain_line(line):
    if len(line) < 4:
        return None, None
    status = line[:2]
    path = line[3:]
    if " -> " in path:
        path = path.split(" -> ", 1)[1]
    return status, path.strip().strip('"')


def porcelain_paths(kind):
    paths = []
    for line in git_lines(["status", "--porcelain"]):
        status, path = parse_porcelain_line(line)
        if not path:
            continue
        if kind == "dirty_tracked" and status != "??":
            paths.append(path)
        if kind == "untracked" and status == "??":
            paths.append(path)
    return paths


def experimental_untracked(paths):
    return [path for path in paths if path.startswith(EXPERIMENTAL_PREFIXES)]


def hash_named(files):
    hashed = {}
    missing = []
    for name, path in files.items():
        if path.is_file():
            hashed[name] = sha256_file(path)
        else:
            missing.append(name)
    return hashed, missing


def platform_artifact():
    record = {"url": PLATFORM, "path": str(PLATFORM_CACHE), "sha256": None, "present": PLATFORM_CACHE.is_file()}
    if record["present"]:
        record["sha256"] = sha256_file(PLATFORM_CACHE)
        record["bytes"] = PLATFORM_CACHE.stat().st_size
    return record


def blank_receipt(**updates):
    receipt = {
        "timestamp": utc_now(),
        "timestamp_end": None,
        "engine_revision": REVISION,
        "engine_sha256": HASHES,
        "os": platform.system(),
        "architecture": platform.machine(),
        "cpu": None,
        "machine": platform.platform(),
        "python": sys.version,
        "roc": None,
        "roc_flags": ROC_FLAGS,
        "rocci_revision": None,
        "input_hashes": {},
        "product_html_hashes": {},
        "generated_hashes": {},
        "platform_artifact": platform_artifact(),
        "dirty_tracked": [],
        "untracked_experimental": [],
        "work_dir": None,
        "repetitions": None,
        "workloads": WORKLOADS,
        "render_cases": RENDER_CASES,
        "probes": [],
        "probes_no_cache": [],
        "upstream_tests": {},
        "comparisons": {},
        "bench_requested": False,
        "allocations_requested": False,
        "allocations": {},
        "harness_ok": False,
        "type_contract_ok": False,
        "html_compatible": False,
        "html_expected_findings_confirmed": False,
        "error": None,
    }
    receipt.update(updates)
    return receipt


def capture_environment(work, repetitions, bench, allocations):
    roc = command(["roc", "version"], REPO)
    head = command(["git", "rev-parse", "HEAD"], REPO)
    inputs = {
        "run.py": HERE / "run.py",
        "TypedTemplate.roc": HERE / "TypedTemplate.roc",
        "Card.rocci": HERE / "Card.rocci",
        "card.mustache.html": HERE / "card.mustache.html",
        "allocations.c": HERE / "allocations.c",
        "README.md": HERE / "README.md",
        "Compat.rocci": HERE / "Compat.rocci",
        "compat.mustache.html": HERE / "compat.mustache.html",
        "compat.py": HERE / "compat.py",
        "costs.py": HERE / "costs.py",
    }
    product = {
        "crates/rocci-ui/runtime/Html.roc": REPO / "crates/rocci-ui/runtime/Html.roc",
        "crates/rocci-platform/platform/Html.roc": REPO / "crates/rocci-platform/platform/Html.roc",
        "crates/rocci-platform/platform/Attribute.roc": REPO / "crates/rocci-platform/platform/Attribute.roc",
        "crates/rocci-cli/runtime/Html.roc": REPO / "crates/rocci-cli/runtime/Html.roc",
    }
    input_hashes, missing_inputs = hash_named(inputs)
    product_hashes, missing_product = hash_named(product)
    receipt = blank_receipt(
        cpu=cpu_label(),
        roc=(roc["stdout"] or "").strip() if roc["exit"] == 0 else None,
        rocci_revision=(head["stdout"] or "").strip() if head["exit"] == 0 else None,
        input_hashes=input_hashes,
        product_html_hashes=product_hashes,
        dirty_tracked=porcelain_paths("dirty_tracked"),
        untracked_experimental=experimental_untracked(porcelain_paths("untracked")),
        work_dir=str(work),
        repetitions=repetitions,
        bench_requested=bench,
        allocations_requested=allocations,
    )
    missing = missing_inputs + missing_product
    if missing:
        receipt["error"] = f"Missing hashed inputs: {missing}"
    if roc["exit"] != 0:
        receipt["error"] = f"roc version failed: {roc}"
    return receipt


def receipt_complete(report):
    return all(key in report for key in REQUIRED_KEYS)


def summarize(report):
    if report.get("self_test"):
        faults = report.get("harness_fault_probes") or []
        report["harness_ok"] = bool(faults) and all(item.get("passed") for item in faults)
        report["type_contract_ok"] = False
        report["html_compatible"] = False
        report["html_expected_findings_confirmed"] = False
        report["harness_problems"] = [] if report["harness_ok"] else ["self_test_failed"]
        report["timestamp_end"] = utc_now()
        return report
    probes = report.get("probes") or []
    probes_no_cache = report.get("probes_no_cache") or []
    probe_names = [item.get("name") for item in probes]
    no_cache_names = [item.get("name") for item in probes_no_cache]
    missing_probes = [
        name for name in EXPECTED_PROBE_NAMES
        if name not in probe_names or (probes_no_cache and name not in no_cache_names)
    ]
    type_ok = (
        not report.get("compat_requested")
        and not report.get("costs_requested")
        and not missing_probes
        and probes
        and all(item.get("passed") for item in probes)
        and all(item.get("passed") for item in probes_no_cache)
        and not any(item.get("timed_out") or item.get("invalid_utf8") for item in probes + probes_no_cache)
    )
    upstream = report.get("upstream_tests") or {}
    upstream_ok = all(
        (upstream.get(mode) or {}).get("exit") == 0 and not (upstream.get(mode) or {}).get("timed_out")
        for mode in ("default", "no_cache")
        if mode in upstream
    ) if upstream else False
    report["type_contract_ok"] = bool(type_ok and upstream_ok and not missing_probes)
    report["missing_probes"] = missing_probes

    comparisons = report.get("comparisons") or {}
    html_compatible = True
    findings_ok = True
    harness_problems = []
    if report.get("error"):
        harness_problems.append("error")
    if report.get("harness_fault_probes") and not all(item.get("passed") for item in report["harness_fault_probes"]):
        harness_problems.append("harness_fault_probe")
    if not receipt_complete(report):
        harness_problems.append("incomplete_receipt")
    if missing_probes and not report.get("compat_requested") and not report.get("costs_requested"):
        harness_problems.append("missing_probes")
    if report.get("bench_requested"):
        for backend in BACKENDS:
            if backend not in comparisons:
                harness_problems.append(f"absent_backend:{backend}")
                html_compatible = False
                findings_ok = False
                continue
            comparison = comparisons[backend]
            if comparison.get("build", {}).get("exit") != 0 or comparison.get("build", {}).get("timed_out"):
                harness_problems.append(f"failed_build:{backend}")
                html_compatible = False
                findings_ok = False
                continue
            cases = comparison.get("named_outputs") or {}
            for case in RENDER_CASES:
                entry = cases.get(case["name"])
                if not entry:
                    harness_problems.append(f"missing_variant:{backend}:{case['name']}")
                    html_compatible = False
                    findings_ok = False
                    continue
                if entry.get("invalid_utf8"):
                    harness_problems.append(f"invalid_utf8:{backend}:{case['name']}")
                    html_compatible = False
                    findings_ok = False
                    continue
                expected = expected_compatible(case, backend)
                actual = bool(entry.get("matches_reference"))
                if not actual:
                    html_compatible = False
                if actual != expected:
                    findings_ok = False
                    harness_problems.append(f"unexpected_html:{backend}:{case['name']}")
            timings = comparison.get("timings") or {}
            for workload in WORKLOADS:
                item = timings.get(workload["name"])
                if not item:
                    harness_problems.append(f"missing_timing:{backend}:{workload['name']}")
                    continue
                checksums = [run.get("stdout") for run in item.get("runs") or []]
                if len(checksums) != 3 or any(value is None for value in checksums):
                    harness_problems.append(f"missing_timing_sample:{backend}:{workload['name']}")
                elif len(set(checksums)) != 1:
                    harness_problems.append(f"timing_checksum_mismatch:{backend}:{workload['name']}")
            digest = comparison.get("digest")
            if not digest or digest.get("invalid_utf8") or digest.get("exit") != 0:
                harness_problems.append(f"missing_digest:{backend}")
        digests = [
            (comparisons.get(backend) or {}).get("digest", {}).get("stdout_sha256")
            for backend in BACKENDS
            if backend in comparisons
        ]
        length_checksums = {}
        for workload in WORKLOADS:
            values = []
            for backend in BACKENDS:
                runs = ((comparisons.get(backend) or {}).get("timings") or {}).get(workload["name"], {}).get("runs") or []
                values.extend(run.get("stdout") for run in runs)
            length_checksums[workload["name"]] = values
        report["benchmark_length_checksums_equal"] = all(
            values and len(set(values)) == 1 for values in length_checksums.values()
        )
        report["output_digests_equal"] = bool(digests) and len(set(digests)) == 1
        if report["benchmark_length_checksums_equal"] and not report["output_digests_equal"]:
            harness_problems.append("same_length_wrong_output")
        if report.get("allocations_requested"):
            allocations = report.get("allocations") or {}
            if allocations.get("skipped"):
                if platform.system() == "Darwin":
                    harness_problems.append("missing_allocation_records")
            else:
                for backend in BACKENDS:
                    if backend not in allocations:
                        harness_problems.append(f"missing_allocation_records:{backend}")
                        continue
                    for workload in WORKLOADS:
                        item = allocations[backend].get(workload["name"])
                        if not item:
                            harness_problems.append(f"missing_allocation_records:{backend}:{workload['name']}")
                            continue
                        checksums = [run.get("checksum") for run in item.get("runs") or [] if run.get("repetitions")]
                        if len(checksums) != 2 or any(value is None for value in checksums):
                            harness_problems.append(f"allocation_output_unchecked:{backend}:{workload['name']}")
    if report.get("compat_requested"):
        if not report.get("compatibility_matrix"):
            harness_problems.append("missing_compat_matrix")
        if report.get("unexplained_benchmark_differences"):
            harness_problems.append("unexplained_html")
        if not report.get("unsupported_cases"):
            harness_problems.append("missing_unsupported_cases")
        boolean = (report.get("boolean_probes") or {}).get("observed") or {}
        if not boolean:
            harness_problems.append("missing_boolean_probe")
    if report.get("costs_requested"):
        if not report.get("cost_table") or "selected" not in (report.get("phase2_selection") or {}):
            harness_problems.append("missing_cost_table")
    report["html_compatible"] = bool(html_compatible and report.get("bench_requested") and not report.get("error"))
    report["html_expected_findings_confirmed"] = bool(
        findings_ok and report.get("bench_requested") and "error" not in harness_problems
        and not any(item.startswith("absent_backend") or item.startswith("failed_build") or item.startswith("missing_variant") for item in harness_problems)
    )
    report["harness_ok"] = not harness_problems
    report["harness_problems"] = harness_problems
    report["timestamp_end"] = utc_now()
    return report


def write_receipt(path, report):
    summarize(report)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Results: {path}\nWork directory: {report.get('work_dir')}", flush=True)


def engine_files(work, source_dir):
    for name, expected in HASHES.items():
        if source_dir:
            data = (source_dir / name).read_bytes()
        else:
            url = f"https://raw.githubusercontent.com/y2kbugger/roc-templegen/{REVISION}/{name}"
            with urllib.request.urlopen(url, timeout=20) as response:
                data = response.read()
        digest = sha256_bytes(data)
        if digest != expected:
            raise ExperimentError(
                f"Unexpected content for {name}",
            )
        (work / name).write_bytes(data)
    shutil.copyfile(HERE / "TypedTemplate.roc", work / "TypedTemplate.roc")


def write_probe_module(work, case):
    (work / f"{case['name']}.mustache").write_text(case["source"])
    (work / f"{case['name']}.roc").write_text(
        f'import TypedTemplate\nimport "{case["name"]}.mustache" as source : Str\n'
        f'sample : {case["typ"]}\nsample = {case["sample"]}\n'
        f'render = TypedTemplate.prepare("{case["name"]}.mustache", source, sample)\n'
        f'main! = |_| {{\n echo!(render({case["actual"]}))\n Ok({{}})\n}}\n'
    )


def type_probes(work, cache_mode):
    results = []
    check = ["roc", "check", "--no-cache"] if cache_mode == "no-cache" else ["roc", "check"]
    for case in PROBE_CASES:
        write_probe_module(work, case)
        result = command([*check, f"{case['name']}.roc"], work)
        result["cache_mode"] = cache_mode
        execution = None
        if case["diagnostic"] is None and result["exit"] == 0 and not result["timed_out"]:
            execution = command(["roc", f"{case['name']}.roc"], work)
            result["execution"] = execution
        passed = probe_outcome(result, case["diagnostic"], case["output"], execution)
        result.update(name=case["name"], passed=passed, expected_diagnostic=case["diagnostic"])
        results.append(result)
        print(f"probe {cache_mode} {case['name']}: {'PASS' if passed else 'FAIL'}", flush=True)
    return results


MAIN = '''
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
            items = List.from_iter((0..<rows).iter()).map(|i| { name: "${text}${i.to_str()}" })
            match mode {
                "render" => Stdout.write!(render_page({ title: text, active: reps > 0, items }))?
                "bench" => {
                    var $checksum = 0.U64
                    for i in 0..<reps {
                        html = render_page({ title: if i % 2 == 0 text else Str.concat(text, "X"), active: i % 2 == 0, items })
                        $checksum = $checksum + html.count_utf8_bytes()
                    }
                    Stdout.line!($checksum.to_str())?
                }
                "digest" => {
                    html0 = render_page({ title: text, active: Bool.True, items })
                    html1 = render_page({ title: Str.concat(text, "X"), active: Bool.False, items })
                    Stdout.write!(html0)?
                    Stdout.write!("\\n")?
                    Stdout.write!(html1)?
                }
                _ => return Err(Exit(2))
            }
            Ok({})
        }
        _ => Err(Exit(2))
    }
}
'''


def hash_tree(root):
    hashed = {}
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "app":
            hashed[str(path.relative_to(root.parent))] = sha256_file(path)
    return hashed


def comparison_apps(work, rocci):
    lowered = command([rocci, "build", HERE / "Card.rocci"], REPO, timeout=60)
    if lowered["exit"] != 0 or lowered["timed_out"] or lowered["invalid_utf8"]:
        raise ExperimentError("rocci-template failed to lower Card.rocci")
    inspect = command([rocci, "inspect", "--ast", HERE / "Card.rocci"], REPO)
    (work / "card-inspect.txt").write_text(inspect["stdout"] or "")
    header = f'app [main!] {{ pf: platform "{PLATFORM}" }}\nimport pf.OsStr\nimport pf.Stdout\n'
    generated = {}
    for backend in BACKENDS:
        target = work / backend
        target.mkdir()
        if backend == "prepared":
            for name in [*HASHES, "TypedTemplate.roc"]:
                shutil.copyfile(work / name, target / name)
            text = (HERE / "card.mustache.html").read_text().removesuffix("\n")
            (target / "card.mustache.html").write_text(text)
            body = (
                'import TypedTemplate\nimport "card.mustache.html" as source : Str\n'
                'sample : { title : Str, active : Bool, items : List({ name : Str }) }\n'
                'sample = { title: "", active: Bool.True, items: [{ name: "" }] }\n'
                'render_page = TypedTemplate.prepare("card.mustache.html", source, sample)\n'
            )
        else:
            body = (lowered["stdout"] or "") + "\nrender_page = |ctx| Html.render_without_doc_type(card(ctx))\n"
            if backend == "rocci-string":
                shutil.copyfile(REPO / "crates/rocci-ui/runtime/Html.roc", target / "Html.roc")
            else:
                (target / "Runtime").mkdir()
                for name in ["Html.roc", "Attribute.roc"]:
                    shutil.copyfile(REPO / "crates/rocci-platform/platform" / name, target / "Runtime" / name)
                wrapper = (REPO / "crates/rocci-cli/runtime/Html.roc").read_text()
                wrapper = wrapper.replace("import pf.Attribute", "import Runtime/Attribute").replace("import pf.Html", "import Runtime/Html")
                (target / "Html.roc").write_text(wrapper)
        (target / "main.roc").write_text(header + body + MAIN)
        generated.update(hash_tree(target))
    return inspect, generated


def named_render_outputs(target):
    outputs = {}
    for case in RENDER_CASES:
        result = command([target / "app", "render", case["active"], case["rows"], case["text"]], target)
        source = result["stdout"] if not result["invalid_utf8"] else ""
        matches = False
        if not result["timed_out"] and result["exit"] == 0 and not result["invalid_utf8"]:
            matches = HtmlEvents(source).events == HtmlEvents(reference_html(case["active"], case["rows"], case["text"])).events
        outputs[case["name"]] = {
            "stdout": result["stdout"],
            "stdout_sha256": result["stdout_sha256"],
            "exit": result["exit"],
            "timed_out": result["timed_out"],
            "invalid_utf8": result["invalid_utf8"],
            "matches_reference": matches,
            "expected_compatible": expected_compatible(case, target.name),
        }
    return outputs


def measure_apps(work, repetitions, results):
    for backend in BACKENDS:
        target = work / backend
        no_cache_checks = [command(["roc", "check", "--no-cache", "main.roc"], target) for _ in range(3)]
        warm_checks = [command(["roc", "check", "main.roc"], target) for _ in range(3)]
        build = command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=60)
        warm_build = command(["roc", "build", "--opt=speed", "--output=app-warm", "main.roc"], target, timeout=60)
        results[backend] = {
            "checks_no_cache": no_cache_checks,
            "checks_warm": warm_checks,
            "build": build,
            "warm_build": warm_build,
        }
        print(f"build {backend}: exit={build['exit']} ({build['seconds']:.2f}s)", flush=True)
        if build["exit"] != 0 or build["timed_out"] or build["invalid_utf8"]:
            continue
        results[backend]["binary_bytes"] = (target / "app").stat().st_size
        results[backend]["named_outputs"] = named_render_outputs(target)
        results[backend]["matches_reference"] = all(item["matches_reference"] for item in results[backend]["named_outputs"].values())
        timings = {}
        for workload in WORKLOADS:
            runs = [
                command([target / "app", "bench", repetitions, workload["rows"], workload["text"]], target, timeout=30)
                for _ in range(3)
            ]
            timings[workload["name"]] = {
                "runs": runs,
                "median_seconds": statistics.median(run["seconds"] for run in runs),
                "checksums": [run.get("stdout") for run in runs],
            }
        results[backend]["timings"] = timings
        digest = command([target / "app", "digest", 1, 100, "plain"], target, timeout=30)
        results[backend]["digest"] = digest
    return results


def measure_allocations(work, repetitions, comparisons):
    if platform.system() != "Darwin":
        return {"skipped": "The optional libc interposer is macOS-only"}
    dylib = work / "allocations.dylib"
    compiled = command(["cc", "-dynamiclib", "-O2", "-Wall", "-Wextra", HERE / "allocations.c", "-o", dylib], work)
    if compiled["exit"] != 0:
        return {"skipped": "failed to compile allocations.c", "compile": compiled}
    env = {**os.environ, "DYLD_INSERT_LIBRARIES": str(dylib)}
    results = {}
    for backend, comparison in comparisons.items():
        if comparison.get("build", {}).get("exit") != 0:
            continue
        results[backend] = {}
        for workload in WORKLOADS:
            runs = []
            for reps in [0, repetitions, repetitions * 2]:
                execution = command(
                    [work / backend / "app", "bench", reps, workload["rows"], workload["text"]],
                    work, env=env, timeout=30,
                )
                line = next(
                    (item for item in (execution["stderr"] or "").splitlines() if item.startswith("ALLOCATIONS ")),
                    None,
                )
                counts = json.loads(line.removeprefix("ALLOCATIONS ")) if line else None
                if counts is None:
                    runs.append({"repetitions": reps, "counts": None, "checksum": execution.get("stdout"), "missing_record": True, "run": execution})
                    continue
                if execution.get("invalid_utf8"):
                    runs.append({"repetitions": reps, "counts": counts, "checksum": None, "invalid_utf8": True, "run": execution})
                    continue
                runs.append({"repetitions": reps, "counts": counts, "checksum": (execution.get("stdout") or "").strip(), "run": execution})
            measured = next((item["counts"] for item in runs if item["repetitions"] == repetitions), None)
            doubled = next((item["counts"] for item in runs if item["repetitions"] == repetitions * 2), None)
            baseline = next((item["counts"] for item in runs if item["repetitions"] == 0), None)
            per_render = None
            linear_calls = False
            if measured and doubled and baseline:
                per_render = {key: (measured[key] - baseline[key]) / repetitions for key in baseline}
                linear_calls = all(
                    doubled[key] - baseline[key] == 2 * (measured[key] - baseline[key])
                    for key in ["allocations", "reallocations"]
                )
            results[backend][workload["name"]] = {
                "runs": runs,
                "per_render": per_render,
                "linear_calls": linear_calls,
            }
    return results


def speed_ordering(comparisons):
    ordering = {}
    for workload in WORKLOADS:
        named = []
        for backend in BACKENDS:
            timings = (comparisons.get(backend) or {}).get("timings") or {}
            item = timings.get(workload["name"])
            if item and "median_seconds" in item:
                named.append((item["median_seconds"], backend))
        named.sort()
        ordering[workload["name"]] = [backend for _, backend in named]
    large_prepared_win = (
        ordering.get("large-clean", [None])[0] == "prepared"
        and ordering.get("large-escaped", [None])[0] == "prepared"
    )
    small_node_win = (ordering.get("small-clean") or [None])[0] == "rocci-node"
    return {
        "fastest_to_slowest": ordering,
        "reproduced_claimed_large_prepared_win": large_prepared_win,
        "reproduced_claimed_small_node_win": small_node_win,
    }


def complete_failure_ok(report, name):
    complete = receipt_complete(report)
    harness = report.get("harness_ok")
    return {
        "name": name,
        "passed": complete and harness is False,
        "complete": complete,
        "harness_ok": harness,
        "type_contract_ok": report.get("type_contract_ok"),
    }


def self_test_absent_backend():
    report = blank_receipt(bench_requested=True, work_dir="self-test", repetitions=1, error=None)
    report["comparisons"] = {
        "prepared": {"build": {"exit": 0}, "named_outputs": {}, "timings": {}, "digest": {"exit": 0, "stdout_sha256": "a"}},
        "rocci-string": {"build": {"exit": 0}, "named_outputs": {}, "timings": {}, "digest": {"exit": 0, "stdout_sha256": "a"}},
    }
    summarize(report)
    result = complete_failure_ok(report, "absent_backend")
    result["problems"] = report.get("harness_problems")
    return result, report


def self_test_same_length_wrong_output():
    left, right = b"aaaa", b"bbbb"
    report = blank_receipt(bench_requested=True, work_dir="self-test", repetitions=1)
    fake_runs = [{"stdout": "4", "seconds": 0.1} for _ in range(3)]
    fake_outputs = {
        case["name"]: {
            "stdout": "x",
            "stdout_sha256": "0",
            "exit": 0,
            "timed_out": False,
            "invalid_utf8": False,
            "matches_reference": expected_compatible(case, "rocci-node"),
            "expected_compatible": expected_compatible(case, "rocci-node"),
        }
        for case in RENDER_CASES
    }
    report["comparisons"] = {
        "prepared": {
            "build": {"exit": 0},
            "named_outputs": {
                case["name"]: {**fake_outputs[case["name"]], "matches_reference": expected_compatible(case, "prepared")}
                for case in RENDER_CASES
            },
            "timings": {workload["name"]: {"runs": fake_runs} for workload in WORKLOADS},
            "digest": {"exit": 0, "stdout_sha256": sha256_bytes(left), "invalid_utf8": False},
        },
        "rocci-string": {
            "build": {"exit": 0},
            "named_outputs": {
                case["name"]: {**fake_outputs[case["name"]], "matches_reference": expected_compatible(case, "rocci-string")}
                for case in RENDER_CASES
            },
            "timings": {workload["name"]: {"runs": fake_runs} for workload in WORKLOADS},
            "digest": {"exit": 0, "stdout_sha256": sha256_bytes(right), "invalid_utf8": False},
        },
        "rocci-node": {
            "build": {"exit": 0},
            "named_outputs": fake_outputs,
            "timings": {workload["name"]: {"runs": fake_runs} for workload in WORKLOADS},
            "digest": {"exit": 0, "stdout_sha256": sha256_bytes(left), "invalid_utf8": False},
        },
    }
    summarize(report)
    detected = "same_length_wrong_output" in report["harness_problems"]
    result = complete_failure_ok(report, "same_length_wrong_output")
    result["passed"] = result["complete"] and detected and report["harness_ok"] is False
    result["length_equal"] = len(left) == len(right)
    result["digest_equal"] = sha256_bytes(left) == sha256_bytes(right)
    return result, report


def self_test_wrong_expected_error():
    wrong = command_record(["roc", "check", "extra.roc"], 1, 0.01, b"compiler crash", b"", False)
    passed = probe_outcome(wrong, "type mismatch")
    report = blank_receipt(work_dir="self-test", repetitions=1, error="wrong expected error reproduced")
    report["probes"] = [{"name": name, "passed": name != "extra", "timed_out": False, "invalid_utf8": False} for name in EXPECTED_PROBE_NAMES]
    report["probes"][EXPECTED_PROBE_NAMES.index("extra")].update(passed=passed, expected_diagnostic="type mismatch")
    report["probes_no_cache"] = list(report["probes"])
    report["upstream_tests"] = {"default": {"exit": 0}, "no_cache": {"exit": 0}}
    summarize(report)
    result = complete_failure_ok(report, "wrong_expected_error")
    result["probe_outcome_rejected"] = passed is False
    result["passed"] = result["complete"] and passed is False and report["type_contract_ok"] is False
    return result, report


def self_test_corrupted_source_hash(work):
    source_dir = work / "bad-engine"
    source_dir.mkdir(exist_ok=True)
    for name in HASHES:
        (source_dir / name).write_text("corrupted")
    report = blank_receipt(work_dir=str(work), repetitions=1)
    try:
        engine_files(work, source_dir)
        report["error"] = "corrupted hash was not detected"
    except ExperimentError as error:
        report["error"] = str(error)
    summarize(report)
    result = complete_failure_ok(report, "corrupted_source_hash")
    result["passed"] = result["complete"] and report["error"] == "Unexpected content for Template.roc" and report["harness_ok"] is False
    return result, report


def self_test_timed_out_child(work):
    result = command(["sleep", "5"], work, timeout=0.3)
    report = blank_receipt(work_dir=str(work), repetitions=1, error="timed-out child")
    report["timed_out_child"] = result
    summarize(report)
    negative = probe_outcome(result, "type mismatch")
    outcome = complete_failure_ok(report, "timed_out_child")
    outcome["timed_out"] = result["timed_out"]
    outcome["exit"] = result["exit"]
    outcome["negative_test_rejected"] = negative is False
    outcome["passed"] = (
        outcome["complete"]
        and result["timed_out"] is True
        and result["exit"] is None
        and negative is False
        and report["harness_ok"] is False
    )
    return outcome, report


def self_test_porcelain_paths():
    dirty = []
    untracked = []
    for line in [" M docs/appendix/glossary.rocdown", "M  src/lib.rs", "?? roc/template-preparation-experiment/"]:
        status, path = parse_porcelain_line(line)
        if status != "??":
            dirty.append(path)
        else:
            untracked.append(path)
    passed = dirty == ["docs/appendix/glossary.rocdown", "src/lib.rs"] and untracked == ["roc/template-preparation-experiment/"]
    report = blank_receipt(work_dir="self-test", repetitions=1, error=None if passed else "porcelain parse failed")
    if not passed:
        summarize(report)
    else:
        report["harness_ok"] = True
        report["timestamp_end"] = utc_now()
    result = {
        "name": "porcelain_paths",
        "passed": passed,
        "complete": True,
        "dirty": dirty,
        "untracked": untracked,
    }
    return result, report


def collect_harness_faults(work):
    faults = []
    for name, fn in [
        ("absent_backend", self_test_absent_backend),
        ("same_length_wrong_output", self_test_same_length_wrong_output),
        ("wrong_expected_error", self_test_wrong_expected_error),
        ("corrupted_source_hash", lambda: self_test_corrupted_source_hash(work)),
        ("timed_out_child", lambda: self_test_timed_out_child(work)),
        ("porcelain_paths", self_test_porcelain_paths),
    ]:
        result, _report = fn()
        faults.append(result)
        print(f"fault {name}: {'PASS' if result['passed'] else 'FAIL'}", flush=True)
    return faults


def run_self_test(output):
    work = Path(tempfile.mkdtemp(prefix="rocci-template-preparation-self-test-"))
    faults = []
    receipts = {}
    for name, fn in [
        ("absent_backend", lambda: self_test_absent_backend()),
        ("same_length_wrong_output", lambda: self_test_same_length_wrong_output()),
        ("wrong_expected_error", lambda: self_test_wrong_expected_error()),
        ("corrupted_source_hash", lambda: self_test_corrupted_source_hash(work)),
        ("timed_out_child", lambda: self_test_timed_out_child(work)),
        ("porcelain_paths", lambda: self_test_porcelain_paths()),
    ]:
        result, report = fn()
        faults.append(result)
        receipts[name] = {
            "harness_ok": report.get("harness_ok"),
            "type_contract_ok": report.get("type_contract_ok"),
            "html_compatible": report.get("html_compatible"),
            "error": report.get("error"),
            "harness_problems": report.get("harness_problems"),
            "complete": receipt_complete(report),
        }
        print(f"fault {name}: {'PASS' if result['passed'] else 'FAIL'}", flush=True)
    report = blank_receipt(work_dir=str(work), repetitions=0)
    report["self_test"] = True
    report["harness_fault_probes"] = faults
    report["harness_fault_receipts"] = receipts
    report["error"] = None if all(item["passed"] for item in faults) else "harness fault probe failed"
    write_receipt(output, report)
    passed = all(item["passed"] for item in faults) and receipt_complete(report)
    return 0 if passed else 1


def run_experiment(options):
    work = options.work_dir or Path(tempfile.mkdtemp(prefix="rocci-template-preparation-"))
    if options.work_dir:
        work.mkdir(parents=True, exist_ok=False)
    report = capture_environment(work, options.repetitions, options.bench, options.allocations)
    try:
        if report.get("error"):
            raise ExperimentError(report["error"])
        report["harness_fault_probes"] = collect_harness_faults(work)
        if not all(item["passed"] for item in report["harness_fault_probes"]):
            raise ExperimentError("harness fault probe failed")
        engine_files(work, options.engine_dir)
        if options.compat:
            import importlib.util
            spec = importlib.util.spec_from_file_location("compat", HERE / "compat.py")
            compat = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(compat)
            report["compat_requested"] = True
            compat.run_compat(sys.modules[__name__], work, report)
        elif options.costs:
            import importlib.util
            spec = importlib.util.spec_from_file_location("costs", HERE / "costs.py")
            costs = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(costs)
            report["costs_requested"] = True
            costs.run_costs(sys.modules[__name__], work, report)
        else:
            report["probes"] = type_probes(work, "default")
            report["probes_no_cache"] = type_probes(work, "no-cache")
            report["upstream_tests"] = {
                "default": command(["roc", "test", "Template.roc"], work, timeout=120),
                "no_cache": command(["roc", "test", "--no-cache", "Template.roc"], work, timeout=120),
            }
            if options.bench:
                rocci = REPO / "target/debug/rocci-template"
                cargo = command(["cargo", "build", "-q", "-p", "rocci-template"], REPO, timeout=120)
                if cargo["exit"] != 0:
                    raise ExperimentError("cargo build -p rocci-template failed")
                inspect, generated = comparison_apps(work, rocci)
                report["inspect"] = inspect
                report["generated_hashes"] = generated
                report["comparisons"] = {}
                measure_apps(work, options.repetitions, report["comparisons"])
                report["speed_ordering"] = speed_ordering(report["comparisons"])
                if options.allocations:
                    report["allocations"] = measure_allocations(work, options.repetitions, report["comparisons"])
    except Exception as error:
        report["error"] = f"{type(error).__name__}: {error}"
    write_receipt(options.output, report)
    if options.compat:
        ok = (
            report["harness_ok"]
            and not report.get("unexplained_benchmark_differences")
            and report.get("compatibility_matrix")
        )
    elif options.costs:
        ok = report["harness_ok"] and report.get("phase2_selection") is not None
    else:
        ok = (
            report["harness_ok"]
            and report["type_contract_ok"]
            and (not options.bench or report["html_expected_findings_confirmed"])
        )
    return 0 if ok else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine-dir", type=Path, help="Use already downloaded, hash-verified engine modules")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--bench", action="store_true", help="Build optimized basic-cli comparison apps")
    parser.add_argument("--allocations", action="store_true", help="With --bench, count intercepted malloc-family calls on macOS")
    parser.add_argument("--repetitions", type=int, default=5000)
    parser.add_argument("--work-dir", type=Path, help="Keep generated files in a NEW directory")
    parser.add_argument("--self-test", action="store_true", help="Run harness fault probes only")
    parser.add_argument("--compat", action="store_true", help="Build the Phase 1 HTML compatibility matrix")
    parser.add_argument("--costs", action="store_true", help="Run Phase 2 isolated escape/growth cost experiments")
    options = parser.parse_args()
    if options.repetitions <= 0:
        parser.error("--repetitions must be positive")
    if options.allocations and not options.bench:
        parser.error("--allocations requires --bench")
    if options.compat and (options.bench or options.self_test or options.costs):
        parser.error("--compat cannot be combined with --bench, --self-test, or --costs")
    if options.costs and (options.bench or options.self_test):
        parser.error("--costs cannot be combined with --bench or --self-test")
    if options.self_test:
        raise SystemExit(run_self_test(options.output))
    raise SystemExit(run_experiment(options))


if __name__ == "__main__":
    main()
