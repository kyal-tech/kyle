# Kyle Packages

Pure Kyle libraries in `packages/`. Publishable via registry.

## Available Packages

| Package | Path | Status | Description |
|---------|------|--------|-------------|
| **ui** | `packages/ui/` | ✅ | 30 .kyx components, themes, desktop renderer |
| **http** | `packages/http/` | ✅ | HTTP client + server + websocket |
| **json** | `packages/json/` | ✅ | JSON parser/generator (registry tarball) |
| **sqlite** | `packages/sqlite/` | ✅ | SQLite bindings |
| **postgres** | `packages/postgres/` | ✅ | PostgreSQL client (WIP) |
| **env** | `packages/env/` | ✅ | Environment variables via libc `getenv` |
| **webapp** | `packages/webapp/` | ✅ | Project template for `ky new` |

## Using a Package

```bash
# Install from registry
ky add http

# Or use directly from packages/ directory
use http.{get, post}
```

## Package Structure

Every package has:

```
packages/<name>/
├── ky.toml              # Package manifest (name, version, deps)
└── src/
    └── lib.ky           # Entry point
```

Example `ky.toml`:
```toml
[package]
name = "mylib"
version = "0.1.0"
```

## Creating a Package

```bash
mkdir -p packages/myapp/src
cat > packages/myapp/ky.toml << 'EOF'
[package]
name = "myapp"
version = "0.1.0"
EOF

cat > packages/myapp/src/lib.ky << 'EOF'
# myapp v0.1.0
fn hello() str:
    "hello from myapp"
EOF

# Verify
ky check packages/myapp/src/lib.ky
```

## Testing a Package

```bash
# Type-check
ky check packages/<name>/src/lib.ky

# Run tests (if #[test] functions exist)
ky test
```

## Publishing

```bash
# Login to registry
ky login

# Publish
ky publish

# Check for outdated deps
ky outdated
```

> Package registry files at `docs/packages/` (see `http.json`, `sqlite.json`, etc.).
