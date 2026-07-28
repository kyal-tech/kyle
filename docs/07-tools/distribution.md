# Distribution — Multi-Platform Install

**Status:** Implementation Plan (Fase 0)
**Date:** 2026-07-09
**Documentaci&oacute;n relacionada:**
- [`docs/10-design/rfc/0002-ui-architecture.md`](../10-design/rfc/0002-ui-architecture.md) — UI framework roadmap
- [`docs/02-getting-started/installation.md`](../02-getting-started/installation.md) — Instalaci&oacute;n desde c&oacute;digo fuente

---

## 1. Objetivo

Que cualquier persona pueda instalar Kyle en **cualquier sistema operativo y arquitectura**
con **un solo comando**, y que al hacerlo reciba:

- El compilador `ky` (o `ky.exe`) para su plataforma
- El runtime `libkyc_runtime.a` para su plataforma
- Configuraci&oacute;n autom&aacute;tica del PATH

Sin necesidad de tener Rust, LLVM, ni ning&uacute;n otro requisito previo instalado.

### 1.1 Plataformas objetivo

| Plataforma | Arquitectura | Triplete LLVM | Bundle |
|------------|-------------|---------------|--------|
| macOS | ARM64 (Apple Silicon) | `aarch64-apple-darwin` | `ky-macos-arm64.tar.gz` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | `ky-linux-arm64.tar.gz` |
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | `ky-linux-x64.tar.gz` |
| Windows | x86_64 | `x86_64-pc-windows-gnu` | `ky-windows-x64.zip` |

---

## 2. Experiencia de instalaci&oacute;n

### 2.1 macOS / Linux

El usuario ejecuta un solo comando:

```bash
curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh | sh
```

El script:

1. Detecta SO y arquitectura (`uname -s` + `uname -m`)
2. Descarga el bundle correcto desde GitHub Releases
3. Extrae `ky` y `libkyc_runtime.a`
4. Instala en `~/.ky/bin/ky` + `~/.ky/lib/libkyc_runtime.a`
5. Si hay permisos de sudo, pregunta y puede instalar en `/usr/local/`
6. Agrega `~/.ky/bin` al PATH en `.zshrc` / `.bashrc`
7. Muestra mensaje de &eacute;xito

Si el usuario ya tiene Kyle, el script detecta la versi&oacute;n actual y pregunta si desea actualizar.

### 2.2 Windows

El usuario ejecuta en PowerShell:

```powershell
iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" | iex
```

El script:

1. Detecta arquitectura (x64 o ARM64)
2. Descarga el ZIP correcto desde GitHub Releases
3. Extrae `ky.exe` y `libkyc_runtime.a`
4. Instala en `%USERPROFILE%\.ky\bin\ky.exe` + `%USERPROFILE%\.ky\lib\libkyc_runtime.a`
5. Agrega `%USERPROFILE%\.ky\bin` al PATH de usuario
6. Muestra mensaje de &eacute;xito

### 2.3 Descarga directa (avanzado)

Usuarios avanzados pueden descargar manualmente desde GitHub Releases:

```
https://github.com/IT-KYNERA/KYLE/releases/download/v0.6.0/ky-macos-arm64.tar.gz
https://github.com/IT-KYNERA/KYLE/releases/download/v0.6.0/ky-windows-x64.zip
...
```

---

## 3. Estructura de los bundles

### 3.1 macOS / Linux (.tar.gz)

```
ky-macos-arm64.tar.gz
  ky                    # Binario compilado (executable)
  libkyc_runtime.a      # Runtime est&aacute;tico
  ky.toml               # Template opcional de proyecto
  LICENSE               # Licencia MIT
```

### 3.2 Windows (.zip)

```
ky-windows-x64.zip
  ky.exe                # Binario compilado (executable)
  libkyc_runtime.a      # Runtime est&aacute;tico
  ky.toml               # Template opcional de proyecto
  LICENSE               # Licencia MIT
```

---

## 4. Script de instalaci&oacute;n: `install.sh`

### 4.1 Detecci&oacute;n de plataforma

```bash
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Darwin) os="macos" ;;
        Linux)  os="linux" ;;
        *)      echo "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac

    case "$(uname -m)" in
        arm64|aarch64) arch="arm64" ;;
        x86_64|amd64)  arch="x64" ;;
        *)      echo "Unsupported arch: $(uname -m)"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}
```

### 4.2 Flujo completo

```bash
#!/bin/bash
set -eu

REPO="IT-KYNERA/KYLE"
VERSION="v0.6.0"
PLATFORM=$(detect_platform)

# Preguntar si requiere sudo (solo para /usr/local/)
if [ -w /usr/local/bin ]; then
    INSTALL_PREFIX="/usr/local"
    USE_SUDO=false
else
    echo "Install to /usr/local requires sudo."
    echo "Install to ~/.ky/ instead? [Y/n]"
    read -r answer
    if [ "$answer" != "n" ]; then
        INSTALL_PREFIX="$HOME/.ky"
        USE_SUDO=false
    else
        echo "Please run with sudo or install manually."
        exit 1
    fi
fi

# Descargar bundle
BUNDLE="ky-${PLATFORM}.tar.gz"
URL="https://github.com/$REPO/releases/download/$VERSION/$BUNDLE"

echo "Downloading Kyle $VERSION for $PLATFORM..."
curl -fsSL "$URL" -o "/tmp/$BUNDLE"

# Extraer
tar xzf "/tmp/$BUNDLE" -C /tmp/

# Instalar
mkdir -p "$INSTALL_PREFIX/bin" "$INSTALL_PREFIX/lib/ky"
cp /tmp/ky "$INSTALL_PREFIX/bin/ky"
cp /tmp/libkyc_runtime.a "$INSTALL_PREFIX/lib/ky/libkyc_runtime.a"
chmod +x "$INSTALL_PREFIX/bin/ky"

# Agregar al PATH
# ...

echo "✅ Kyle $VERSION installed to $INSTALL_PREFIX"
```

### 4.3 Parámetros adicionales

| Variable | Efecto |
|----------|--------|
| `KY_VERSION=v0.7.0` | Instalar versión específica |
| `KY_PREFIX=/custom/path` | Directorio de instalación |
| `KY_NO_MODIFY_PATH=1` | No modificar PATH automáticamente |

---

## 5. Script de instalación: `install.ps1`

```powershell
# install.ps1
param(
    [string]$Version = "v0.6.0",
    [string]$Prefix = "$env:USERPROFILE\.ky"
)

$arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "arm64" }
$bundle = "ky-windows-$arch.zip"
$url = "https://github.com/IT-KYNERA/KYLE/releases/download/$Version/$bundle"

Write-Host "Downloading Kyle $Version for Windows..."
Invoke-WebRequest -Uri $url -OutFile "$env:TEMP\$bundle"

Expand-Archive "$env:TEMP\$bundle" -DestinationPath $Prefix

# Add to PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$Prefix\bin*") {
    [Environment]::SetEnvironmentVariable(
        "PATH", "$userPath;$Prefix\bin", "User"
    )
}

Write-Host "✅ Kyle $Version installed to $Prefix"
```

---

## 6. Build/Release script: `scripts/build-release.sh`

### 6.1 Cross-compilación

Cada release ejecuta:

```bash
#!/bin/bash
set -eu

VERSION="v0.6.0"
TARGETS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."

    # 1. Compilar kyc_runtime para $target
    cargo build --release --target "$target" -p kyc_runtime

    # 2. Compilar ky para $target
    cargo build --release --target "$target" --bin ky

    # 3. Empaquetar
    local bundle_name
    case "$target" in
        aarch64-apple-darwin)       bundle_name="ky-macos-arm64" ;;
        x86_64-unknown-linux-gnu)   bundle_name="ky-linux-x64" ;;
        aarch64-unknown-linux-gnu)  bundle_name="ky-linux-arm64" ;;
        x86_64-pc-windows-gnu)      bundle_name="ky-windows-x64" ;;  # or msvc
    esac

    mkdir -p "dist/$bundle_name"
    cp "target/$target/release/ky" "dist/$bundle_name/ky"
    cp "target/$target/release/libkyc_runtime.a" "dist/$bundle_name/"
    cp LICENSE "dist/$bundle_name/"

    if [[ "$target" == *"windows"* ]]; then
        (cd dist && zip -r "$bundle_name.zip" "$bundle_name/")
    else
        (cd dist && tar czf "$bundle_name.tar.gz" "$bundle_name/")
    fi
done

echo "✅ All bundles built in dist/"
```

### 6.2 Toolchains necesarias

| Target | Toolchain | Instalación |
|--------|-----------|-------------|
| `aarch64-apple-darwin` | Nativo (macOS ARM64) | — |
| `x86_64-apple-darwin` | Rust target | `rustup target add x86_64-apple-darwin` |
| `x86_64-unknown-linux-gnu` | Rust target + cross-linker | `rustup target add ...` + `brew install x86_64-linux-gnu-gcc` |
| `aarch64-unknown-linux-gnu` | Rust target + cross-linker | `rustup target add ...` + `brew install aarch64-linux-gnu-gcc` |
| `x86_64-pc-windows-gnu` | Rust target + mingw | `rustup target add ...` + `brew install mingw-w64` |

---

## 7. Cambios necesarios en el compilador

### 7.1 Flag `--target` en `ky build`

El compilador debe aceptar un target triple opcional:

```bash
ky build app.ky                          # usa target del host
ky build --target wasm32 app.ky          # WebAssembly
ky build --target x86_64-pc-windows-gnu  # Windows cross
```

### 7.2 codegen.rs — Target triple configurable

Actualmente:

```rust
let triple = inkwell::targets::TargetMachine::get_default_triple();
module.set_triple(&triple);
```

Debe cambiar a:

```rust
let triple = self.target_triple.clone()
    .unwrap_or_else(|| inkwell::targets::TargetMachine::get_default_triple());
module.set_triple(&triple);
```

### 7.3 linker.rs — Linker por target

Actualmente usa `cfg!(target_os)` para elegir linker.
Debe seleccionar el linker según el target triple:

| Triplete | Linker |
|----------|--------|
| `aarch64-apple-darwin` | `clang -arch arm64` |
| `x86_64-apple-darwin` | `clang -arch x86_64` |
| `x86_64-unknown-linux-gnu` | `x86_64-linux-gnu-gcc` |
| `aarch64-unknown-linux-gnu` | `aarch64-linux-gnu-gcc` |
| `x86_64-pc-windows-gnu` | `x86_64-w64-mingw32-gcc` |
| `wasm32-unknown-unknown` | `wasm-ld` |

### 7.4 linker.rs — Búsqueda de runtime

Actualmente busca `libkyc_runtime.a` en varias ubicaciones. Para soportar
múltiples targets, debe buscar también en:

```
~/.ky/lib/targets/<triple>/libkyc_runtime.a
```

---

## 8. Estructura de directorios instalada

```
~/.ky/                          # (o /usr/local/lib/ky/)
  bin/
    ky                          # Compilador
  lib/
    libkyc_runtime.a            # Runtime para el host
    targets/
      aarch64-apple-darwin/
        libkyc_runtime.a
      x86_64-apple-darwin/
        libkyc_runtime.a
      x86_64-unknown-linux-gnu/
        libkyc_runtime.a
      aarch64-unknown-linux-gnu/
        libkyc_runtime.a
      x86_64-pc-windows-gnu/
        libkyc_runtime.a
      wasm32-unknown-unknown/
        libkyc_runtime.a
```

---

## 9. Seguridad

| Medida | Descripción |
|--------|-------------|
| **HTTPS** | Todas las descargas via HTTPS desde GitHub |
| **Checksums** | Cada release incluye SHA-256 del bundle |
| **GPG** | Opcional: firma GPG del install.sh |
| **Verificación** | `install.sh` verifica checksum después de descargar |
| **Minimal** | Solo se descarga lo necesario para la plataforma |

---

## 10. Mantenimiento

### Por release

1. Ejecutar `scripts/build-release.sh` para generar los 4 bundles
2. Subir bundles a GitHub Releases
3. (Opcional) Actualizar install.sh si cambia la URL base

### Nueva plataforma

Para agregar un nuevo target (ej: Linux RISC-V):

1. Agregar el target a `TARGETS` en `build-release.sh`
2. Agregar el triplete al mapa de bundles
3. Agregar detección en `install.sh` y `install.ps1`

---

## 11. Referencias

- [`docs/10-design/rfc/0002-ui-architecture.md`](../10-design/rfc/0002-ui-architecture.md) — UI framework roadmap
- [`docs/02-getting-started/installation.md`](../02-getting-started/installation.md) — Instalación desde fuente
- [`docs/07-tools/compiler-cli.md`](./compiler-cli.md) — CLI del compilador
- [`docs/07-tools/package-manager.md`](./package-manager.md) — Package manager
