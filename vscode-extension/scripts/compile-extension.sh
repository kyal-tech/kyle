#!/usr/bin/env bash
set -euo pipefail

# Compile TypeScript sources for the Kyle VS Code extension

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXT_DIR="$SCRIPT_DIR/.."

echo "==> Installing dependencies..."
cd "$EXT_DIR"
npm install --omit=dev

echo "==> Compiling TypeScript..."
npx tsc -p tsconfig.json

echo "==> Compilation complete. JS output in $EXT_DIR/out/"
