#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

code --extensions-dir ~/.cursor/extensions --install-extension editors/vscode/rocci-*.vsix
