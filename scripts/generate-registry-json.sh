#!/bin/bash
set -eu
# Generate static JSON files + tarballs for GitHub Pages registry
# Reads from registry/ and writes to docs/packages/

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
REGISTRY_DIR="$ROOT_DIR/registry"
OUT_DIR="$ROOT_DIR/docs/packages"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

for pkg_dir in "$REGISTRY_DIR"/*/; do
  pkg_name=$(basename "$pkg_dir")
  echo "Processing $pkg_name ..."

  # Read ky.toml for dependencies
  versions_json="{\"versions\":["
  first_ver=true

  for ver_dir in "$pkg_dir"*/; do
    ver=$(basename "$ver_dir")
    [ "$ver" = "*.tar.gz" ] && continue
    [ ! -d "$ver_dir" ] && continue

    # Add to versions list
    if [ "$first_ver" = true ]; then
      first_ver=false
    else
      versions_json="$versions_json,"
    fi
    versions_json="$versions_json{\"version\":\"$ver\",\"yanked\":false}"

    # Create version directory with deps.json
    ver_out="$OUT_DIR/$pkg_name/$ver"
    mkdir -p "$ver_out"

    # Extract dependencies from ky.toml
    deps_json="{\"dependencies\":["
    first_dep=true
    if [ -f "$ver_dir/ky.toml" ]; then
      # Parse [dependencies] section
      in_deps=false
      while IFS= read -r line; do
        if [[ "$line" =~ ^\[dependencies\]$ ]]; then
          in_deps=true
        elif [[ "$line" =~ ^\[.*\]$ ]]; then
          in_deps=false
        elif [ "$in_deps" = true ] && [[ "$line" =~ ^([a-zA-Z0-9_-]+)\ *=\ *\"([^\"]+)\"$ ]]; then
          dep_name="${BASH_REMATCH[1]}"
          dep_ver="${BASH_REMATCH[2]}"
          if [ "$first_dep" = true ]; then
            first_dep=false
          else
            deps_json="$deps_json,"
          fi
          deps_json="$deps_json{\"name\":\"$dep_name\",\"version\":\"$dep_ver\"}"
        fi
      done < "$ver_dir/ky.toml"
    fi
    deps_json="$deps_json]}"
    echo "$deps_json" > "$ver_out/deps.json"

    # Copy tarball
    tarball="$REGISTRY_DIR/$pkg_name/$ver.tar.gz"
    if [ -f "$tarball" ]; then
      cp "$tarball" "$ver_out/download.tar.gz"
    fi
  done

  versions_json="$versions_json]}"
  echo "$versions_json" > "$OUT_DIR/$pkg_name.json"

  echo "  Wrote $OUT_DIR/$pkg_name.json"
done

echo "Done."
