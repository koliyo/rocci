"""HTTP-origin host coverage. Imported by run.py; copies platform Html only in the work dir."""

import hashlib
import os
import platform as py_platform
import re
import shutil
import signal
import socket
import subprocess
import time
from pathlib import Path


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
PLATFORM_SRC = REPO / "crates/rocci-platform/platform"
PIN = "../platform/main.roc"


def cargo_target_dir():
    return Path(os.environ.get("CARGO_TARGET_DIR") or (REPO / "target"))


def rocci_bin():
    return cargo_target_dir() / "debug" / "rocci"


def load_costs():
    import importlib.util
    spec = importlib.util.spec_from_file_location("costs", HERE / "costs.py")
    costs = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(costs)
    return costs


def sha256_file(path):
    digest = hashlib.sha256()
    digest.update(path.read_bytes())
    return digest.hexdigest()


def native_libhost_target():
    system = py_platform.system()
    machine = py_platform.machine()
    if system == "Darwin":
        return "arm64mac" if machine == "arm64" else "x64mac"
    if system == "Linux":
        return "arm64musl" if machine in ("aarch64", "arm64") else "x64musl"
    return None


def libhost_path():
    target = native_libhost_target()
    if not target:
        return None
    return PLATFORM_SRC / "targets" / target / "libhost.a"


def ensure_libhost(harness):
    path = libhost_path()
    record = {
        "target": native_libhost_target(),
        "path": str(path) if path else None,
        "existed": bool(path and path.is_file()),
    }
    if path is None:
        record["ok"] = False
        record["error"] = f"unsupported host {py_platform.system()} {py_platform.machine()}"
        return record
    if path.is_file():
        record["ok"] = True
        record["bytes"] = path.stat().st_size
        return record
    build = harness.command(
        ["bash", str(REPO / "crates/rocci-platform/build.sh")],
        REPO,
        timeout=600,
    )
    record["build"] = {
        "exit": build["exit"],
        "seconds": build["seconds"],
        "timed_out": build.get("timed_out"),
        "stderr": (build.get("stderr") or "")[-800:],
    }
    record["ok"] = path.is_file()
    if path.is_file():
        record["bytes"] = path.stat().st_size
    else:
        record["error"] = "build.sh did not write native libhost.a"
    return record


def wait_for_staged(tmp, timeout=120):
    start = time.time()
    last = None
    stable = 0
    while time.time() - start < timeout:
        found = None
        for path in tmp.glob("rocci-islands-build-*"):
            main = path / "main.roc"
            page = path / "HostPage.roc"
            if main.is_file() and page.is_file() and main.stat().st_size > 0:
                found = path
                break
        if found:
            size = (found / "main.roc").stat().st_size + (found / "HostPage.roc").stat().st_size
            marker = (found, size)
            if marker == last:
                stable += 1
                if stable >= 4:
                    return found
            else:
                stable = 0
                last = marker
        time.sleep(0.05)
    return None


def isolate_hostpage(work):
    source = work / "host" / "input"
    if source.exists():
        shutil.rmtree(source)
    source.mkdir(parents=True)
    shutil.copyfile(HERE / "HostPage.rocci", source / "HostPage.rocci")
    return source / "HostPage.rocci"


def capture_staged_workspace(work):
    tmp = work / "host" / "stage-tmp"
    if tmp.exists():
        shutil.rmtree(tmp)
    tmp.mkdir(parents=True)
    dest = work / "host" / "staged"
    if dest.exists():
        shutil.rmtree(dest)
    dummy = work / "host" / "product-dummy"
    env = os.environ.copy()
    env["TMPDIR"] = str(tmp)
    process = subprocess.Popen(
        [
            str(rocci_bin()),
            "build",
            "--platform",
            "rocci",
            "--output",
            str(dummy),
            str(isolate_hostpage(work)),
        ],
        cwd=REPO,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    record = {"cli": str(rocci_bin()), "tmp": str(tmp)}
    try:
        found = wait_for_staged(tmp)
        if found is None:
            stop_process(process)
            stdout, stderr = process.communicate()
            record["ok"] = False
            record["error"] = "staged islands-build workspace did not appear"
            record["cli_exit"] = process.returncode
            record["cli_stderr"] = (stderr or b"").decode("utf-8", "replace")[-800:]
            return dest, record
        shutil.copytree(found, dest)
        record["ok"] = True
        record["source"] = str(found)
        record["files"] = sorted(
            str(path.relative_to(dest)) for path in dest.rglob("*") if path.is_file()
        )
        record["main_sha256"] = sha256_file(dest / "main.roc")
        record["hostpage_sha256"] = sha256_file(dest / "HostPage.roc")
        record["in_tree_pin"] = "pf: platform" in (dest / "main.roc").read_text()
        return dest, record
    finally:
        stop_process(process)


def copy_platform(dest, html_kind):
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(PLATFORM_SRC, dest, ignore=shutil.ignore_patterns(".DS_Store"))
    html_path = dest / "Html.roc"
    html = html_path.read_text()
    costs = load_costs()
    html_path.write_text(costs.patch_node_html(html, html_kind))
    return {
        "html_kind": html_kind,
        "html_sha256": sha256_file(html_path),
        "libhost": bool(libhost_path() and (dest / "targets" / native_libhost_target() / "libhost.a").is_file()),
    }


def rewrite_main_pin(app_dir, pin):
    main = app_dir / "main.roc"
    text = main.read_text()
    rewritten, count = re.subn(r'pf: platform "[^"]+"', f'pf: platform "{pin}"', text, count=1)
    if count != 1:
        raise RuntimeError(f"expected one platform pin in {main}, found {count}")
    if pin.startswith("/") or pin.startswith("file:"):
        raise RuntimeError(f"refusing absolute platform pin {pin}")
    main.write_text(rewritten)
    return rewritten


def stage_variant(work, staged, name, html_kind):
    root = work / "host" / name
    if root.exists():
        shutil.rmtree(root)
    platform = root / "platform"
    app = root / "app"
    copied = copy_platform(platform, html_kind)
    shutil.copytree(staged, app)
    rewrite_main_pin(app, PIN)
    return root, copied


def generated_files(app):
    return {
        path.relative_to(app).as_posix(): sha256_file(path)
        for path in sorted(app.rglob("*"))
        if path.is_file()
    }


def build_variant(harness, root):
    output = root / "server"
    build = harness.command(
        ["roc", "build", "--no-cache", "--opt=speed", f"--output={output}", "main.roc"],
        root / "app",
        timeout=240,
    )
    stderr = build.get("stderr") or ""
    executable = output.is_file() and os.access(output, os.X_OK) and output.stat().st_size > 0
    return {
        "exit": build["exit"],
        "seconds": build["seconds"],
        "timed_out": build.get("timed_out"),
        "stderr_head": stderr[:800],
        "stderr_tail": stderr[-800:],
        "binary": str(output),
        "binary_exists": output.is_file(),
        "binary_executable": executable,
        "binary_bytes": output.stat().st_size if output.is_file() else 0,
        "unused_variable_exit": build["exit"] == 2 and "unused variable" in stderr,
    }


def free_port():
    sock = socket.socket()
    sock.bind(("127.0.0.1", 0))
    port = sock.getsockname()[1]
    sock.close()
    return port


def http_get(url, timeout=5):
    import urllib.error
    import urllib.request
    try:
        with urllib.request.urlopen(url, timeout=timeout) as response:
            body = response.read()
            return {
                "ok": True,
                "status": getattr(response, "status", None),
                "len": len(body),
                "sha256": hashlib.sha256(body).hexdigest(),
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
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            pass


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
    cargo_cli = harness.command(["cargo", "build", "-q", "-p", "rocci-cli"], REPO, timeout=180)
    report["host"] = {
        "os": py_platform.system(),
        "architecture": py_platform.machine(),
        "candidate": "node_scan_escape",
        "original": "node_fold_escape",
        "linux_coverage": py_platform.system() == "Linux",
        "cargo_cli": {"exit": cargo_cli["exit"], "seconds": cargo_cli["seconds"]},
    }
    if cargo_cli["exit"] != 0:
        report["error"] = "cargo build -p rocci-cli failed"
        report["host_staging"] = {"ok": False, "error": "cargo build -p rocci-cli failed"}
        return
    binary = rocci_bin()
    if not binary.is_file():
        report["error"] = f"missing {binary}"
        report["host_staging"] = {"ok": False, "error": report["error"]}
        return

    print("host capture staged workspace", flush=True)
    staged, capture = capture_staged_workspace(work)
    report["host"]["capture"] = capture
    if not capture.get("ok"):
        report["error"] = capture.get("error") or "staged workspace capture failed"
        report["host_staging"] = {"ok": False, "error": report["error"], "capture": capture}
        return

    print("host ensure libhost", flush=True)
    libhost = ensure_libhost(harness)
    report["host"]["libhost"] = libhost
    if not libhost.get("ok"):
        report["error"] = libhost.get("error") or "native libhost.a missing"
        report["host_staging"] = {"ok": False, "error": report["error"], "libhost": libhost}
        return

    print("host stage orig fold platform", flush=True)
    orig_root, orig_platform = stage_variant(work, staged, "orig", "fold_escape")
    print("host stage scan platform", flush=True)
    scan_root, scan_platform = stage_variant(work, staged, "scan", "scan_escape")
    orig_generated = generated_files(orig_root / "app")
    scan_generated = generated_files(scan_root / "app")
    generated_identical = orig_generated == scan_generated
    html_differs = orig_platform["html_sha256"] != scan_platform["html_sha256"]

    print("host roc build orig", flush=True)
    orig_build = build_variant(harness, orig_root)
    print("host roc build scan", flush=True)
    scan_build = build_variant(harness, scan_root)
    builds_ok = bool(orig_build.get("binary_executable") and scan_build.get("binary_executable"))
    staging = {
        "ok": bool(builds_ok and generated_identical and html_differs),
        "pin": PIN,
        "generated_identical": generated_identical,
        "html_differs": html_differs,
        "orig_platform": orig_platform,
        "scan_platform": scan_platform,
        "orig_build": orig_build,
        "scan_build": scan_build,
        "orig_generated_files": orig_generated,
        "scan_generated_files": scan_generated,
    }
    if not generated_identical:
        staging["error"] = "generated Roc differed between orig and scan after pin rewrite"
    elif not html_differs:
        staging["error"] = "orig and scan Html.roc hashes matched; fold versus scan was not applied"
    elif not builds_ok:
        staging["error"] = "roc build failed for orig or scan"
    report["host_staging"] = staging
    report["host"]["staging"] = staging
    if not staging["ok"]:
        report["error"] = staging.get("error")
