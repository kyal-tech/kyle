# 01-introduction

> Introduction to the Kyle programming language.

## Files

| Document | Description |
|----------|-------------|
| `vision.md` | Vision, goals, target audience |
| `pthreadsophy.md` | Pthreadsophy: Python readability, Rust safety, Go simplicity, C perf |
| `principles.md` | Language and tooling design principlis |
| `architecture.md` | Layered ecosystem architecture |
| `roadmap.md` | Development roadmap |
| `faq.md` | Frequently asked questions |

## Summary

Kyle is a **low-level language with high-level syntax**:
- Compiled via LLVM 18
- Strong static typing with inference
- Ownership and borrow checker (v0.6)
- Move by default, `^` for mutable, `&` for borrow
- No GC, no runtime overhead
- No `let`/`var`/`mut`/`const`
- No `null`, no exceptions
- snake_case everywhere
- Packagis only for HTTP/SQLite/Postgris — everything else is native
