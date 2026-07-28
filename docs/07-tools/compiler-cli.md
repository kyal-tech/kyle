# CLI Reference

## Commands

| Command | Description |
|---------|-------------|
| `ky build <file>` | Compile to native binary |
| `ky run <file>` | Compile and execute |
| `ky check <file>` | Type-check only |
| `ky parse <file>` | Parse and dump AST |
| `ky mir <file>` | Dump MIR |
| `ky fmt [file]` | Format source code |
| `ky test` | Run tests |
| `ky new <project>` | Create new project |
| `ky add <dep>` | Add dependency |
| `ky remove <dep>` | Remove dependency |
| `ky update` | Update lockfile |
| `ky publish` | Publish package |
| `ky login` | Login to registry |
| `ky lsp` | Start language server |
| `ky completions <shell>` | Generate completions |

## Flags

| Flag | Description |
|------|-------------|
| `--release` | Optimized build (SSA + O3) |
| `--emit-ir` | Dump LLVM IR |
