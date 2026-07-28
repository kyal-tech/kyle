# VS Code Extension

The Kyle VS Code extension is at [`vscode-extension/`](vscode-extension/) in this monorepo.

## Features

- **Syntax Highlighting** — `.ky` and `.kyx` files via TextMate grammar
- **Language Server** — Diagnostics, completions, go-to-definition via `ky lsp`
- **Snippets** — 60+ snippets for Kyle constructs
- **Testing UI** — Discover and run `#[test]` functions
- **Debugger** — Step-through debugging support
- **Format on Save** — Auto-format via `ky fmt`
- **Color Theme** — "Kyle Pastel" dark theme

## Quick Install

```bash
# From VS Code Marketplace
code --install-extension kynera.ky

# Or from the .vsix file
code --install-extension ky-0.8.7.vsix
```

## Extension Structure

```
vscode-extension/
├── syntaxes/
│   ├── ky.tmLanguage.json       # Grammar for .ky files
│   └── kyx.tmLanguage.json      # Grammar for .kyx files
├── snippets/
│   ├── ky.json                  # 35+ snippets for .ky
│   └── kyx.json                 # 25+ snippets for .kyx
├── src/
│   ├── extension.ts             # Main entry + LSP client
│   ├── debugger.ts              # Debug adapter
│   ├── tasks.ts                 # VS Code tasks (run, build, check)
│   └── testUI.ts                # Testing panel integration
├── themes/
│   └── kl-color-theme.json      # "Kyle Pastel" dark theme
├── icons/                       # Extension icons
├── out/                         # Compiled JS
├── package.json                 # Extension manifest (v0.8.7)
└── language-configuration.json  # Comment toggles, brackets, folding
```

## Development

```bash
cd vscode-extension

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package .vsix
npx @vscode/vsce package

# Install locally
code --install-extension ky-*.vsix
```

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `ky.path` | `"ky"` | Path to the Kyle compiler binary |
| `ky.semanticHighlighting` | `true` | Enable semantic highlighting |

## Commands

| Command | Description |
|---------|-------------|
| `Kyle: Run current file` | Compile and run `.ky` file |
| `Kyle: Build current file` | Compile to native binary |
| `Kyle: Type-check current file` | Type-check without codegen |
| `Kyle: Run tests in current file` | Run `#[test]` functions |
| `Kyle: Run specific test` | Run a specific test function |

## Grammar / Theme Updates

When the Kyle language syntax changes:

1. Update `syntaxes/ky.tmLanguage.json` and `syntaxes/kyx.tmLanguage.json`
2. Update color scopes in `themes/kl-color-theme.json` if needed
3. Bump version in `package.json`
4. Rebuild: `npm run compile && npx @vscode/vsce package`
