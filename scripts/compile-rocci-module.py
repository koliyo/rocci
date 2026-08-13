#!/usr/bin/env python3
"""Compile a .rocci file and wrap the generated Roc as a type module."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def wrap_type_module(src: str, type_name: str) -> str:
    imports: list[str] = []
    body: list[str] = []
    for line in src.splitlines():
        if line.startswith("import "):
            imports.append(line)
        else:
            body.append(line)
    while body and not body[0].strip():
        body.pop(0)
    while body and not body[-1].strip():
        body.pop()
    indented = "\n".join(f"    {line}" if line else "" for line in body)
    header = "\n".join(imports)
    prefix = f"{header}\n\n" if header else ""
    return f"{prefix}{type_name} := [].{{\n{indented}\n}}\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("-o", "--output", required=True)
    parser.add_argument("--type-name", default="Counter")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    compiled = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "rocci-cli",
            "--",
            "compile",
            args.input,
        ],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if compiled.returncode != 0:
        sys.stderr.write(compiled.stderr)
        sys.stderr.write(compiled.stdout)
        return compiled.returncode

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(wrap_type_module(compiled.stdout, args.type_name))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
