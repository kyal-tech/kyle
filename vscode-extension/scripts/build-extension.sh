#!/usr/bin/env bash
set -euo pipefail

# Build the Kyle VS Code extension package (.vsix)
# Prerequisites: npm, @vscode/vsce

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXT_DIR="$SCRIPT_DIR/.."

echo "==> Compiling TypeScript..."
cd "$EXT_DIR"
npx tsc --noEmit 2>/dev/null || true  # just type-check, we commit .js

echo "==> Packaging extension..."
npx @vscode/vsce package --out "kl-$(node -e "console.log(require('./package.json').version)").vsix"

echo "==> Done! VSIX created in $EXT_DIR"
