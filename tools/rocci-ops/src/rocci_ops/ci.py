from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

from rocci_ops.paths import repo_root

JOB_NAMES = (
    "lint",
    "test",
    "fixtures-and-docs",
    "editors",
    "knowledge",
)


@dataclass(frozen=True)
class Step:
    argv: tuple[str, ...]
    stdout_path: str | None = None


def okmate_dir(root: Path) -> Path:
    if env := os.environ.get("OKMATE_DIR"):
        return Path(env).expanduser().resolve()
    sibling = (root / ".." / "okmate").resolve()
    if (sibling / "Cargo.toml").is_file():
        return sibling
    return (root / ".okmate-tool").resolve()


def okmate_argv(root: Path, *args: str) -> tuple[str, ...]:
    return (
        "cargo",
        "run",
        "-q",
        "--no-default-features",
        "--manifest-path",
        str(okmate_dir(root) / "Cargo.toml"),
        "-p",
        "okmate",
        "--",
        *args,
    )


def _rustup_available() -> bool:
    return shutil.which("rustup") is not None


def steps_for(job: str, root: Path) -> list[Step]:
    if job == "lint":
        steps = [
            Step(("uv", "run", "--no-dev", "rocci-ops", "check", "deps")),
            Step(("cargo", "run", "-q", "-p", "rocci-ungram", "--", "check")),
            Step(("cargo", "fmt", "--all", "--", "--check")),
            Step(("cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings")),
        ]
        if _rustup_available():
            return [
                Step(("rustup", "component", "add", "rustfmt", "clippy")),
                *steps,
            ]
        return steps
    if job == "test":
        return [
            Step(("cargo", "test", "--workspace")),
            Step(("cargo", "test", "--workspace", "--doc")),
        ]
    if job == "fixtures-and-docs":
        return [
            Step(("cargo", "run", "-q", "-p", "rocci-cli", "--", "inspect", "--ast", "test/AllSyntax.rocci")),
            Step(
                (
                    "cargo",
                    "run",
                    "-q",
                    "-p",
                    "rocci-rocdown-cli",
                    "--",
                    "inspect",
                    "ast",
                    "test/AllSyntax.rocdown",
                )
            ),
            Step(
                (
                    "cargo",
                    "run",
                    "-q",
                    "-p",
                    "rocci-cli",
                    "--",
                    "inspect",
                    "--ast",
                    "test/EmbeddedLanguages.rocci",
                )
            ),
            Step(
                (
                    "cargo",
                    "run",
                    "-q",
                    "-p",
                    "rocci-rocdown-cli",
                    "--",
                    "inspect",
                    "ast",
                    "test/EmbeddedLanguages.rocdown",
                )
            ),
            Step(("uv", "run", "--no-dev", "rocci-ops", "check", "docs")),
            Step(("cargo", "test", "-p", "rocci-docs")),
            Step(
                (
                    "cargo",
                    "run",
                    "-q",
                    "-p",
                    "rocci-docs",
                    "--",
                    "--catalog",
                    "examples/rocci/apps.toml",
                    "--output",
                    "dist/example-docs",
                )
            ),
            Step(("cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "check", "site")),
            Step(("cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "check", "docs")),
        ]
    if job == "editors":
        steps: list[Step] = []
        if _rustup_available():
            steps.append(Step(("rustup", "target", "add", "wasm32-wasip1", "wasm32-wasip2")))
        steps.extend(
            [
                Step(("cargo", "build", "-p", "rocci-rocdown-lsp")),
                Step(("npm", "--prefix", "editors/vscode", "ci")),
                Step(("npm", "--prefix", "editors/vscode", "run", "lint")),
                Step(("npm", "--prefix", "editors/vscode", "run", "compile")),
                Step(("npm", "--prefix", "editors/vscode", "run", "vscode:prepublish")),
                Step(("npm", "--prefix", "editors/vscode", "test")),
                Step(
                    (
                        "cargo",
                        "check",
                        "--manifest-path",
                        "editors/zed/Cargo.toml",
                        "--target",
                        "wasm32-wasip1",
                    )
                ),
                Step(("cargo", "check", "--manifest-path", "editors/zed/Cargo.toml")),
                Step(("uv", "run", "--no-dev", "rocci-ops", "check", "zed")),
            ]
        )
        return steps
    if job == "knowledge":
        out = Path("target/knowledge-ci")
        return [
            Step(("mkdir", "-p", str(out))),
            Step(
                okmate_argv(
                    root,
                    "check",
                    "knowledge",
                    "--profile",
                    "rocci",
                    "--format",
                    "json",
                ),
                stdout_path=str(out / "validation.json"),
            ),
            Step(
                okmate_argv(
                    root,
                    "inspect",
                    "--profile",
                    "rocci",
                    "graph",
                    "knowledge",
                ),
                stdout_path=str(out / "graph.json"),
            ),
            Step(
                okmate_argv(
                    root,
                    "benchmark",
                    "knowledge/retrieval-benchmark.toml",
                    "knowledge",
                    "--profile",
                    "rocci",
                ),
                stdout_path=str(out / "retrieval.json"),
            ),
            Step(
                okmate_argv(
                    root,
                    "build",
                    "knowledge",
                    "--output",
                    str(out / "build-a"),
                    "--profile",
                    "rocci",
                )
            ),
            Step(
                okmate_argv(
                    root,
                    "build",
                    "knowledge",
                    "--output",
                    str(out / "build-b"),
                    "--profile",
                    "rocci",
                )
            ),
            Step(
                (
                    "diff",
                    "-qr",
                    "-x",
                    "*.html",
                    str(out / "build-a"),
                    str(out / "build-b"),
                )
            ),
        ]
    raise ValueError(f"unknown job: {job}")


def run_step(step: Step, cwd: Path) -> int:
    stdout_file = None
    try:
        kwargs: dict = {
            "cwd": cwd,
            "check": False,
        }
        if step.stdout_path:
            path = cwd / step.stdout_path
            path.parent.mkdir(parents=True, exist_ok=True)
            stdout_file = path.open("w", encoding="utf-8")
            kwargs["stdout"] = stdout_file
        result = subprocess.run(list(step.argv), **kwargs)
        return result.returncode
    finally:
        if stdout_file is not None:
            stdout_file.close()


def run_job(job: str, cwd: Path) -> int:
    print(f"==> {job}", flush=True)
    for step in steps_for(job, cwd):
        print("+ " + " ".join(step.argv), flush=True)
        code = run_step(step, cwd)
        if code != 0:
            if step.stdout_path:
                captured = cwd / step.stdout_path
                if captured.is_file():
                    print(captured.read_text(encoding="utf-8"), flush=True)
            return code
    return 0


def parse_ci_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="rocci-ops ci")
    parser.add_argument("-k", "--keep-going", action="store_true")
    parser.add_argument("-l", "--list", action="store_true")
    parser.add_argument("jobs", nargs="*", choices=JOB_NAMES)
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_ci_args(argv)
    if args.list:
        for name in JOB_NAMES:
            print(name)
        return 0
    jobs = args.jobs or list(JOB_NAMES)
    cwd = repo_root()
    failed: list[str] = []
    for job in jobs:
        code = run_job(job, cwd)
        if code != 0:
            failed.append(job)
            if not args.keep_going:
                return code
    return 1 if failed else 0
