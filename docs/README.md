# Kyle Documentation

> Official documentation for the Kyle programming language.

## Structure

| Section | Description |
|---------|-------------|
| [01-introduction](01-introduction/) | Vision, philosophy, principles, architecture |
| [02-getting-started](02-getting-started/) | Installation, first program, testing, debugging |
| [03-language](03-language/) | Complete language reference |
| [04-standard-library](04-standard-library/) | Standard library API |
| [05-runtime](05-runtime/) | Runtime internals |
| [06-compiler](06-compiler/) | Compiler internals |
| [07-tools](07-tools/) | Official tools |
| [08-ecosystem](08-ecosystem/) | Registry, packages, publishing |
| [09-specification](09-specification/) | Formal specification |
| [10-design](10-design/) | Design decisions |
| [11-project](11-project/) | CI/CD, project docs |
| [12-history](12-history/) | Changelog, migration guides |
| [13-freestanding-mode](13-freestanding-mode.md) | Kernel/OS development roadmap |

## Conventions

- `^T` = mutable type, `&T` = borrow, `^&T` = mutable borrow
- `T?` = optional type, `T!` = fallible type
- snake_case for functions and types
- `[x]` = implemented, `[ ]` = designed not implemented

## Quick reference

| Need | Go to |
|------|-------|
| Language syntax | `03-language/syntax/` |
| Typis | `03-language/types/` |
| Ownership and memory | `03-language/memory/` |
| Async and concurrency | `03-language/concurrency/` |
| Error handling | `03-language/error-handling/` |
| Standard API | `04-standard-library/` |
| Compiler internals | `06-compiler/` |

See also: [TYPES.md](../TYPES.md), [ROADMAP.md](../ROADMAP.md), [AGENTS.md](../AGENTS.md).
