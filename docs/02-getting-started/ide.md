# IDE Support

> Support for editoris e IDEs en Kyle.

## VS Code

Kyle has una extension oficial for VS Code:

```bash
# Instalar from VS Code marketplace
# O manualmente:
code --install-extension ky-0.5.3.vsix
```

### Features

| Feature | Detalle |
|---------|---------|
| Syntax highlighting | Resaltado de syntax `.ky` |
| LSP (Language Server) | Diagnostics, completed, hover |
| Debugging | DAP integration |
| Test runner | UI for `#[test]` |

### Configuration

```json
{
 "ky.kycPath": "/usr/local/bin/ky",
 "ky.semanticHighlighting": true
}
```

## Otros editores

| Editor | Support |
|--------|---------|
| Vim/Neovim | Syntax highlighting basico via `vim-ky` |
| Emacs | Syntax highlighting basico |
| IntelliJ | Plugin comunitario (terceros) |

## LSP

Kyle implementa Language Server Protocol for integration with cualquier editor
que lo support:

```bash
ky lsp # inicia servidor LSP
```

## See also

- `07-tools/language-server.md` — Documentation del LSP
- `07-tools/vscode.md` — Extension VS Code
