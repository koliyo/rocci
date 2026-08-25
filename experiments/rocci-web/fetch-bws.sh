#!/usr/bin/env bash
set -euo pipefail

# Same pin as crates/rocci-cli/src/dispatch.rs PLATFORM.
TARBALL_URL='https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst'
TARBALL_STEM='42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw'

ROOT="$(cd "$(dirname "$0")" && pwd)"
VENDOR="${ROOT}/vendor"
OVERLAY="${ROOT}/overlay"
CACHE_TAR="${HOME}/.cache/roc/packages/${TARBALL_STEM}/${TARBALL_STEM}.tar.zst"

fetch_vendor() {
    if [[ -f "${VENDOR}/main.roc" ]]; then
        echo "already fetched: ${VENDOR}"
        return 0
    fi

    TMP="$(mktemp -d "${TMPDIR:-/tmp}/rocci-web-bws.XXXXXX")"
    cleanup() { rm -rf "${TMP}"; }
    trap cleanup EXIT

    TAR="${TMP}/${TARBALL_STEM}.tar.zst"
    if [[ -f "${CACHE_TAR}" ]]; then
        cp "${CACHE_TAR}" "${TAR}"
    else
        curl -fsSL "${TARBALL_URL}" -o "${TAR}"
    fi

    mkdir -p "${TMP}/extract"
    if tar --zstd -xf "${TAR}" -C "${TMP}/extract" 2>/dev/null; then
        :
    elif command -v zstd >/dev/null; then
        zstd -dc "${TAR}" | tar -xf - -C "${TMP}/extract"
    else
        echo "need tar --zstd or zstd to unpack ${TARBALL_URL}" >&2
        exit 1
    fi

    if [[ ! -f "${TMP}/extract/main.roc" ]]; then
        echo "tarball did not contain main.roc" >&2
        exit 1
    fi

    rm -rf "${VENDOR}"
    mv "${TMP}/extract" "${VENDOR}"
    trap - EXIT
    rm -rf "${TMP}"
    echo "fetched basic-webserver 0.16.0 -> ${VENDOR}"
}

apply_overlay() {
    if [[ ! -f "${OVERLAY}/Rocci.roc" ]]; then
        return 0
    fi
    cp "${OVERLAY}/Rocci.roc" "${VENDOR}/Rocci.roc"
    python3 - "${VENDOR}/main.roc" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text()
changed = False
if "\t\tRocci,\n" not in text:
    needle = "\t\tPath,\n"
    if needle not in text:
        raise SystemExit("vendor/main.roc: expected Path in exposes")
    text = text.replace(needle, needle + "\t\tRocci,\n", 1)
    changed = True
if "import Rocci\n" not in text:
    needle = "import Server\n"
    if needle not in text:
        raise SystemExit("vendor/main.roc: expected import Server")
    text = text.replace(needle, needle + "import Rocci\n", 1)
    changed = True
if changed:
    path.write_text(text)
    print(f"patched {path}")
else:
    print(f"overlay already applied: {path}")
PY
}

fetch_vendor
apply_overlay
