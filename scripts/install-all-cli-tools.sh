#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST="${HOME}/.local/bin"

CRATES=(
    "rocci-cli:rocci"
    "rocci-rocdown-cli:rocdown"
    "rocci-okf:rocci-okf"
)

echo "Rocci CLI installer"
echo "  Source: ${ROOT}"
echo "  Destination: ${DEST}"
echo ""

if [ ! -d "${DEST}" ]; then
    printf "  '%s' does not exist. Create it? [y/N] " "${DEST}"
    read -r answer
    case "${answer}" in
        [yY][eE][sS]|[yY])
            mkdir -p "${DEST}"
            echo "  Created ${DEST}"
            ;;
        *)
            echo "  Aborted."
            exit 1
            ;;
    esac
fi

if [ ! -w "${DEST}" ]; then
    echo "  Error: '${DEST}' is not writable." >&2
    exit 1
fi

echo "Building release binaries..."
echo ""

for entry in "${CRATES[@]}"; do
    crate="${entry%%:*}"
    bin="${entry##*:}"
    echo "  cargo build --release -p ${crate}"
    cargo build --release -p "${crate}"
    src="${ROOT}/target/release/${bin}"
    if [ ! -f "${src}" ]; then
        echo "  Error: expected binary not found at '${src}'" >&2
        exit 1
    fi
    echo "  Installing ${bin} -> ${DEST}/${bin}"
    cp "${src}" "${DEST}/${bin}"
    chmod 755 "${DEST}/${bin}"
done

echo ""
echo "Installed:"
for entry in "${CRATES[@]}"; do
    bin="${entry##*:}"
    echo "  ${DEST}/${bin}"
done

if [[ ":${PATH}:" != *":${DEST}:"* ]]; then
    echo ""
    echo "  Note: '${DEST}' is not on your PATH."
    echo "  Add the following to your shell profile:"
    echo "    export PATH=\"\${HOME}/bin:\${PATH}\""
fi
