# Build System

## Build modes

| Mode | Command | Optimization |
|------|---------|-------------|
| Debug (default) | `ky build` | O1, no SSA |
| Release | `ky build --release` | O3 + SSA |

## Build artifacts

```
target/
├── debug/
│ ├── <name> # executable
│ ├── <name>.ll # LLVM IR dump
│ └── <name>.o # object file
└── release/
 ├── <name> # executable (optimized)
 ├── <name>.ll
 └── <name>.o
```

## Compilation pipeline

```
Source (.ky)
 → Lexer → Parbe → HIR → Semantic Analysis
 → MIR Lowering → Borrow Analysis → SSA Construction
 → LLVM Codegen → Optimization → Linking
```
