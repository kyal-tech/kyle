# VS Code Extension

**Status:** Cross-platform — macOS, Linux, Windows

## Install

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.sh | sh
```

**Windows (PowerShell):**
```powershell
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/vscode-ky/install-extension.ps1" | iex
```

Or install from the VS Code Marketplace: search "Kyle Language Support".

## Features

- **Syntax highlighting** — TextMate grammar for `.ky` files
- **LSP integration** — diagnostics, autocomplete, go-to-def, hover, find refs, rename
- **Inlay hints** — inferred types shown inline
- **Code lens** — "Run test" button above `#[test]` functions
- **Task provider** — auto-discovers `.ky` files for run/build/check
- **Debug adapter** — DAP-based debugging with breakpoints, stack traces, scopes
- **Test UI** — VS Code TestController integration for `#[test]` functions
- **Problems panel** — compiler diagnostics shown as problems
- **Color theme** — "Kyle Pastel" dark theme
- **Snippets** — code snippets for common patterns

## Configuration

| Setting | Description |
|---------|-------------|
| `ky.kycPath` | path to `ky` binary (auto-detected) |
| `ky.semanticHighlighting` | Enable experimental semantic highlighting |

## Binary search order

1. `ky.kycPath` setting
2. PATH environment
3. macOS/Linux: `~/.ky/bin/ky`, `~/.cargo/bin/ky`, `/usr/local/bin/ky`, `/opt/homebrew/bin/ky`
4. Windows: `%USERPROFILE%\.ky\bin\ky.exe`
5. `where` (Windows) / `which` (Unix) command
