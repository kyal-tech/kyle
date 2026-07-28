#!/bin/bash
set -eu

cd vscode-extension

if [ ! -d node_modules ]; then
    echo "→ Installing deps..."
    npm install
fi

echo "→ Compiling TypeScript..."
npx tsc

echo "→ Packaging .vsix..."
rm -f ky-*.vsix
npx @vscode/vsce package --allow-missing-repository

echo "→ Installing in VS Code..."
code --install-extension ky-*.vsix --force

echo ""
echo "✅ Installed. Reload VS Code window (Cmd+Shift+P → Reload Window)"
