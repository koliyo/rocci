"""Phase 4 representative host checks. Imported by run.py; copies platform Html only in the work dir."""

import os
import shutil
import signal
import socket
import subprocess
import time
import urllib.error
import urllib.request
from pathlib import Path


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]


def load_costs():
    import importlib.util
    spec = importlib.util.spec_from_file_location("costs", HERE / "costs.py")
    costs = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(costs)
    return costs


def stage_node_app(work, name, harness, lowered, render_page, scan):
    costs = load_costs()
    target = work / "host" / "cli" / name / ("scan" if scan else "orig")
    target.mkdir(parents=True)
    (target / "Runtime").mkdir()
    shutil.copyfile(REPO / "crates/rocci-platform/platform/Attribute.roc", target / "Runtime" / "Attribute.roc")
    html = (REPO / "crates/rocci-platform/platform/Html.roc").read_text()
    if scan:
        html = costs.patch_node_html(html, "scan_escape")
    (target / "Runtime" / "Html.roc").write_text(html)
    wrapper = (REPO / "crates/rocci-cli/runtime/Html.roc").read_text()
    wrapper = wrapper.replace("import pf.Attribute", "import Runtime/Attribute").replace("import pf.Html", "import Runtime/Html")
    (target / "Html.roc").write_text(wrapper)
    header = f'app [main!] {{ pf: platform "{harness.PLATFORM}" }}\nimport pf.Stdout\n'
    body = lowered + f"\nrender_page = {render_page}\n"
    main = """
main! = |_| {
    Stdout.write!(render_page({}))?
    Ok({})
}
"""
    (target / "main.roc").write_text(header + body + main)
    build = harness.command(["roc", "build", "--no-cache", "--opt=speed", "--output=app", "main.roc"], target, timeout=90)
    return target, build


def render_once(harness, target):
    return harness.command([target / "app"], target, timeout=20)


def compare_pair(harness, work, name, rocci, source, render_page):
    lowered = harness.command([rocci, "build", source], REPO, timeout=60)
    record = {
        "source": str(source),
        "lower_exit": lowered["exit"],
        "lower_stderr": (lowered.get("stderr") or "")[-500:],
    }
    if lowered["exit"] != 0:
        record["ok"] = False
        return record
    orig, orig_build = stage_node_app(work, name, harness, lowered["stdout"] or "", render_page, False)
    scan, scan_build = stage_node_app(work, name, harness, lowered["stdout"] or "", render_page, True)
    record["orig_build"] = {"exit": orig_build["exit"], "seconds": orig_build["seconds"]}
    record["scan_build"] = {"exit": scan_build["exit"], "seconds": scan_build["seconds"]}
    if orig_build["exit"] != 0 or scan_build["exit"] != 0:
        record["ok"] = False
        record["orig_stderr"] = (orig_build.get("stderr") or "")[-500:]
        record["scan_stderr"] = (scan_build.get("stderr") or "")[-500:]
        return record
    orig_run = render_once(harness, orig)
    scan_run = render_once(harness, scan)
    record["orig_run"] = {"exit": orig_run["exit"], "sha256": orig_run.get("stdout_sha256"), "len": orig_run.get("stdout_len")}
    record["scan_run"] = {"exit": scan_run["exit"], "sha256": scan_run.get("stdout_sha256"), "len": scan_run.get("stdout_len")}
    record["bytes_equal"] = (
        orig_run["exit"] == 0
        and scan_run["exit"] == 0
        and orig_run.get("stdout") == scan_run.get("stdout")
    )
    record["ok"] = bool(record["bytes_equal"])
    return record


def inspect_theme(harness, report):
    theme_rs = REPO / "crates/rocci-rocdown/src/plan/theme.rs"
    text = theme_rs.read_text()
    uses_str = 'html_type: "Str".to_string()' in text
    painter = REPO / "crates/rocci-rocdown/templates/DocsComponents.rocci"
    rocci = REPO / "target/debug/rocci-template"
    inspect = harness.command([rocci, "inspect", "--ast", painter], REPO, timeout=30)
    report["theme"] = {
        "source": str(theme_rs.relative_to(REPO)),
        "html_type_str": uses_str,
        "painter": str(painter.relative_to(REPO)),
        "inspect_exit": inspect["exit"],
        "candidate_applies": False,
        "note": "Theme painters select Str signatures; node_scan_escape lives on platform Node Html and does not apply.",
    }


def free_port():
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    port = sock.getsockname()[1]
    sock.close()
    return port


def http_get(url, timeout=5):
    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            body = response.read()
            return {
                "ok": True,
                "status": getattr(response, "status", None),
                "len": len(body),
                "sha256": __import__("hashlib").sha256(body).hexdigest(),
                "head": body[:200].decode("utf-8", "replace"),
            }
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        return {"ok": False, "error": str(error)}


def wait_http(url, timeout=40):
    start = time.time()
    last = None
    while time.time() - start < timeout:
        last = http_get(url, timeout=2)
        if last.get("ok"):
            return last
        time.sleep(0.15)
    return last or {"ok": False, "error": "no attempt"}


def stop_process(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait(timeout=5)


def product_origin_smoke(work, harness):
    dest = work / "host" / "http-product"
    dest.mkdir(parents=True)
    binary = dest / "server"
    build = harness.command(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "rocci-cli",
            "--",
            "build",
            "--platform",
            "rocci",
            "--output",
            str(binary),
            str(HERE / "HostPage.rocci"),
        ],
        REPO,
        timeout=180,
    )
    record = {
        "cli_build": {
            "exit": build["exit"],
            "seconds": build["seconds"],
            "stderr": (build.get("stderr") or "")[-800:],
        },
        "binary_exists": binary.is_file(),
        "candidate_http": False,
        "note": (
            "rocci build --output writes a process binary against the in-tree platform "
            "pin and drops the staged workspace. --platform only accepts `rocci`. "
            "A copied Html.roc therefore cannot be substituted for the candidate. "
            "This smoke is the product origin only, not candidate HTTP performance."
        ),
    }
    if not binary.is_file():
        record["ok"] = False
        record["error"] = "rocci-cli did not write a server binary"
        return record
    try:
        record["served"] = serve_and_fetch(binary, dest)
        record["ok"] = bool(record["served"].get("ok"))
    except Exception as error:
        record["ok"] = False
        record["error"] = f"{type(error).__name__}: {error}"
    return record


def serve_and_fetch(binary, cwd):
    port = free_port()
    env = os.environ.copy()
    env["ROC_BASIC_WEBSERVER_HOST"] = "127.0.0.1"
    env["ROC_BASIC_WEBSERVER_PORT"] = str(port)
    process = subprocess.Popen(
        [str(binary)],
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    origin = f"http://127.0.0.1:{port}"
    try:
        view = wait_http(f"{origin}/")
        fragment = http_get(f"{origin}/card") if view.get("ok") else {"ok": False, "error": "view failed"}
        return {
            "port": port,
            "view": view,
            "fragment": fragment,
            "ok": bool(view.get("ok") and fragment.get("ok")),
        }
    finally:
        stop_process(process)


def run_host(harness, work, report):
    report["host_requested"] = True
    cargo_template = harness.command(["cargo", "build", "-q", "-p", "rocci-template"], REPO, timeout=120)
    cargo_cli = harness.command(["cargo", "build", "-q", "-p", "rocci-cli"], REPO, timeout=180)
    rocci = REPO / "target/debug/rocci-template"
    report["host"] = {
        "os": harness.platform.system() if hasattr(harness, "platform") else __import__("platform").system(),
        "architecture": __import__("platform").machine(),
        "linux_coverage": False,
        "cargo_template": {"exit": cargo_template["exit"], "seconds": cargo_template["seconds"]},
        "cargo_cli": {"exit": cargo_cli["exit"], "seconds": cargo_cli["seconds"]},
        "candidate": "node_scan_escape",
        "fixtures": {},
        "http": {},
    }
    if cargo_template["exit"] != 0:
        report["error"] = "cargo build -p rocci-template failed"
        report["phase4_coverage"] = {"http": False, "linux": False, "theme_painter_node": False}
        return
    fixtures = [
        (
            "hello",
            REPO / "examples/rocci/template/Hello.rocci",
            '|_| Html.render_without_doc_type(hello({ name: "Ada & <Co>" }))',
        ),
        (
            "card",
            HERE / "Card.rocci",
            '|_| Html.render_without_doc_type(card({ title: "Ada & <Co>", active: Bool.True, items: [{ name: "one" }, { name: "two" }] }))',
        ),
        (
            "compat",
            HERE / "Compat.rocci",
            '|_| Html.render_without_doc_type(compat({ title: "Ada & <Co>", active: Bool.True, items: [{ name: "one", kids: [{ name: "k" }] }] }))',
        ),
        (
            "callout",
            REPO / "test/Callout.rocci",
            '|_| Html.render_without_doc_type(callout({ tone: "info" }, Html.text("Ada & <Co>")))',
        ),
    ]
    for name, source, render_page in fixtures:
        print(f"host fixture {name}", flush=True)
        report["host"]["fixtures"][name] = compare_pair(harness, work, name, rocci, source, render_page)
    inspect_theme(harness, report["host"])
    http = {"attempted": True, "candidate_http": False}
    if cargo_cli["exit"] != 0:
        http["ok"] = False
        http["error"] = "cargo build -p rocci-cli failed"
        http["cli_stderr"] = (cargo_cli.get("stderr") or "")[-500:]
    else:
        print("host product origin smoke", flush=True)
        try:
            http["product_origin_smoke"] = product_origin_smoke(work, harness)
        except Exception as error:
            http["product_origin_smoke"] = {"ok": False, "error": f"{type(error).__name__}: {error}"}
        http["ok"] = False
        http["error"] = (
            "candidate HTML cannot be served through rocci-cli: the CLI pin is in-tree "
            "rocci-platform only, and --output is a process binary rather than a kept workspace"
        )
    report["host"]["http"] = http
    equal = all(item.get("ok") for item in report["host"]["fixtures"].values())
    report["phase4_coverage"] = {
        "cli_fixtures_equal": equal,
        "http": False,
        "linux": False,
        "theme_painter_node": False,
        "product_origin_smoke": bool((http.get("product_origin_smoke") or {}).get("ok")),
        "note": "Linux target absent on this Darwin host. Theme painters use Str, so the node candidate is not applied there. Candidate HTTP coverage is absent: rocci-cli cannot keep a staged workspace whose platform pin points at a copied Html.roc.",
    }
    report["phase4_selection"] = "node_scan_escape" if equal else None
    if not equal:
        report["error"] = report.get("error") or "Phase 4 representative fixtures were not byte-identical"
