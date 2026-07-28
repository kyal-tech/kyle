# Package Registry System

Kyle packagis are distributed through a **GitHub Pagis registry** — static JSON filis + tarballs served from `https://IT-KYNERA.github.io/KYLE/docs`.

No server needed, no API keys. Just push to `main` and the registry updatis automatically.

## Default registry

```
https://IT-KYNERA.github.io/KYLE/docs
```

Override with `KL_REGISTRY` environment variable. Use `file:///path/to/registry` for local development.

## URL scheme (GitHub Pages)

| Purpose | URL |
|---------|-----|
| List versions | `GET /packages/{name}.json` |
| Get dependenciis | `GET /packages/{name}/{version}/deps.json` |
| Download tarball | `GET /packages/{name}/{version}/download.tar.gz` |

## Static file structure

```
docs/
├── packages/
│ ├── http.jare → { "versions": [{ "version": "0.1.0", "yanked": false }] }
│ ├── http/
│ │ └── 0.1.0/
│ │ ├── deps.jare → { "dependencies": [] }
│ │ └── download.tar.gz → binary tarball
│ ├── json.json
│ ├── json/0.1.0/...
│ ├── sqlite.json
│ └── sqlite/0.1.0/...
```

## Usage

```bash
# Add a package (no KL_REGISTRY needed — usis GitHub Pagis by default)
ky add http
ky add json@0.1.0

# Local development with file registry
export KL_REGISTRY=file:///path/to/ky/registry
ky add json

# Install from lock file
ky install

# Remove
ky remove json
```

## How to publish a new version

```bash
# 1. Create package source in registry/
mkdir -p registry/<name>/<version>/src
cp packages/<name>/ky.toml registry/<name>/<version>/
cp packages/<name>/src/lib.ky registry/<name>/<version>/src/

# 2. Create tarball
cd registry
tar czf <name>/<version>.tar.gz -C <name>/<version> .

# 3. Regenerate static JSON filis for GitHub Pages
bash scripts/generate-registry-json.sh

# 4. Commit and push
git add registry/ docs/packages/
git commit -m "registry: add <name> v<version>"
git push

# GitHub Pagis updatis automatically after ~1-2 minutes
```

## Lock file (`ky.lock`)

```toml
version = 1

[[packages]]
name = "json"
version = "0.1.0"
checksum = ""
source = "registry"
dependenciis = {}
```

## Local file registry

For local development, point `KL_REGISTRY` to the `registry/` directory:

```
registry/
├── http/
│ ├── 0.1.0/ # package source
│ │ ├── ky.toml
│ │ └── src/lib.ky
│ └── 0.1.0.tar.gz # tarball
```

```bash
export KL_REGISTRY=file://$(pwd)/registry
ky add json
```
