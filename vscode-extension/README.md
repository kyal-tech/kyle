# Kyle Language Support for Visual Studio Code

Syntax highlighting, LSP integration, snippets, and language support for the [Kyle programming language](https://github.com/kyal-tech/kyle).

## Features

- **Syntax Highlighting** — Full syntax highlighting for `.ky` files using TextMate grammar
- **Language Server Protocol** — Diagnostics, completions, go-to-definition, hover, inlay hints, code lens via `ky lsp`
- **Snippets** — 35+ snippets for Kyle constructs (`fn`, `final class`, `match`, etc.)
- **Testing UI** — Discover and run `#[test]` functions from VS Code's Testing panel
- **Tasks** — Run, build, check, and test commands via VS Code tasks
- **Format on Save** — Auto-format via `ky fmt` on save
- **Color Theme** — "Kyle Pastel" dark theme included

## Requirements

- **Kyle compiler** (`ky`) must be installed and available in PATH, or configured via `ky.path`
- **LLVM 18** runtime libraries for compiled binaries

## Extension Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `ky.path` | `"ky"` | Path to the Kyle compiler binary |
| `ky.semanticHighlighting` | `true` | Enable semantic highlighting |

## Commands

| Command | Description |
|---------|-------------|
| `Kyle: Run current file` | Compile and run the active `.ky` file |
| `Kyle: Build current file` | Compile to native binary |
| `Kyle: Type-check current file` | Type-check without codegen |
| `Kyle: Run tests in current file` | Run `#[test]` functions |
| `Kyle: Run specific test` | Run a specific test function |

## Known Issues

- Full step-through debugging requires runtime debugger support (in development)

## Release Notes

See [CHANGELOG.md](CHANGELOG.md) for version history.
