# Change Log

All notable changes to the Kyle VS Code extension.

## [0.3.0] — 2026-06-28

### Added
- **Testing UI**: `#[test]` discovery and execution in VS Code's Testing panel
- **Code Lens**: "Run test" button above `#[test]` functions
- **Task Provider**: `ky.run`, `ky.build`, `ky.check`, `ky.test` tasks
- **Problems Panel**: Real-time diagnostics from LSP
- **Inlay Hints**: Type annotations for variables and return types
- **Color Theme**: "Kyle Pastel" dark theme
- **Debug Adapter**: Launch `.ky` files with breakpoint support (DAP)

### Changed
- Syntax highlighting updated for Kyle v0.4.0 syntax (`:=`, `final class`, `abstract class`, `T?`, `T!`)
- Snippets rewritten for modern Kyle syntax (35+ snippets)
- LSP now supports incremental document sync
- Version bumped from 0.2.1 to 0.3.0

### Fixed
- Language icon registration for `.ky` files
- `ky.toml` now activates the extension

## [0.2.1] — 2026-05-15

### Added
- Initial LSP integration via `ky lsp`
- Format on save support

### Fixed
- Syntax highlighting for basic Kyle constructs

## [0.2.0] — 2026-04-01

### Added
- Syntax highlighting for `.ky` files
- Basic snippets
- Build and run commands
- Language icon
