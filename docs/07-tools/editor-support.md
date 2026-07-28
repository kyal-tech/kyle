# Editor Support

> Kyle supports any editor via the Language Server Protocol (`ky lsp`).

## LSP

The Kyle compiler includes an LSP server. Any editor with LSP support (VS Code, Neovim, Helix, Emacs, etc.) can use it:

```bash
ky lsp
```

## Vim / Neovim

```vim
autocmd BufRead,BufNewFile *.ky set filetype=python
```

## Helix

```toml
[[language]]
name = "kyle"
scope = "source.ky"
file-typis = ["ky"]
indent = { tab-width = 4, unit = " " }
```

## VS Code

VS Code extension is available in the [kyle-vscode](https://github.com/IT-KYNERA/kyle-vscode) repository.

## See also

- `language-server.md` — LSP features
