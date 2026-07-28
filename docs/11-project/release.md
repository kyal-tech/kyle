# Release Guide — Kyle Multi-Platform Distribution

## Plataformas objetivo

| Bundle | Plataforma | Origen |
|--------|------------|--------|
| `ky-macos-arm64.tar.gz` | macOS Apple Silicon | `macos-latest` (CI) o nativo |
| `ky-macos-x64.tar.gz` | macOS Intel | `macos-13` (CI) |
| `ky-linux-arm64.tar.gz` | Linux ARM64 | `ubuntu-24.04-arm` (CI) |
| `ky-linux-x64.tar.gz` | Linux x86_64 | `ubuntu-24.04` (CI) |
| `ky-windows-x64.zip` | Windows x86_64 | `windows-2025` (CI) |

Cada bundle contiene:
```
ky (o ky.exe)
libkyc_runtime.a
LICENSE
```

---

## Opción A: GitHub Actions (automática, recomendada)

Solo hacer push del tag:

```bash
git tag -a v0.6.0 -m "v0.6.0"
git push origin v0.6.0
```

Esto dispara `.github/workflows/release.yml` que:
1. Crea el release en GitHub
2. Compila `ky` + `kyc_runtime` para las 5 plataformas (en paralelo)
3. Genera los bundles + checksums
4. Sube los assets al release

**No requiere nada local.** Esperar ~30 min a que termine CI.

---

## Opción B: Local + CI (híbrida)

Compilar localmente solo macOS ARM64 y dejar que CI genere el resto:

```bash
# 1. Compilar local (macOS ARM64)
brew install llvm@18
export LLVM_SYS_181_PREFIX=$(brew --prefix llvm@18)
./scripts/build-release.sh

# 2. Push tag → CI genera el resto
git tag -a v0.7.0 -m "v0.7.0"
git push origin v0.7.0
```

---

## Opción C: Todo local (Windows requiere mingw)

```bash
# Prerrequisitos (una vez)
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
brew install x86_64-linux-gnu-gcc   # Linux x64 cross-linker

# Compilar todo
./scripts/build-release.sh

# Los bundles quedan en dist/
ls dist/*.tar.gz dist/*.zip
```

---

## Script de build local

```bash
./scripts/build-release.sh
```

Variables de entorno:

| Variable | Default | Descripción |
|----------|---------|-------------|
| `KY_VERSION` | (de Cargo.toml) | Versión para nombrar bundles |
| `CARGO_PROFILE` | `release` | Perfil de compilación |

Solo compila targets con el toolchain instalado. Omite los que faltan.

---

## Verificar bundles localmente

```bash
# Extraer y probar
mkdir -p /tmp/test-ky
tar xzf dist/ky-macos-arm64.tar.gz -C /tmp/test-ky
/tmp/test-ky/ky --version
/tmp/test-ky/ky build examples/hello.ky
```

---

## Scripts de instalación

| Script | Plataforma | Comando |
|--------|------------|---------|
| `install.sh` | macOS / Linux | `curl -fsSL https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.sh \| sh` |
| `install.ps1` | Windows | `iwr -Uri "https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/install.ps1" \| iex` |

Ambos scripts:
1. Detectan SO + arquitectura automáticamente
2. Descargan el bundle correcto desde GitHub Releases
3. Verifican checksum SHA-256
4. Instalan `ky` + `libkyc_runtime.a`
5. Configuran PATH automáticamente

---

## Notas técnicas

### Linker por plataforma

El linker se selecciona en tiempo de compilación del compilador (`cfg!(target_os)`):

| Binario compilado para | Usa linker |
|------------------------|------------|
| macOS | `clang` |
| Linux | `clang` (fallback `cc`) |
| Windows | `link.exe` |

Esto es correcto: cada binario `ky` usa el linker nativo de su plataforma.

### Windows + LLVM 18

GitHub Actions `windows-2025` tiene LLVM 20 pre-instalado, pero inkwell requiere LLVM 18.
El workflow descarga el zip portable de LLVM 18.1.8 y lo extrae (más confiable que el installer MSI).

### Checksums

Cada bundle tiene su SHA-256 (`bundle.tar.gz.sha256`). `install.sh` verifica automáticamente.

### Directorio de instalación

```
~/.ky/                       # (o /usr/local/)
  bin/ky                     # Compilador
  lib/libkyc_runtime.a       # Runtime estático
```

El linker `ky` busca `libkyc_runtime.a` automáticamente en orden:
1. `../lib/kl/libkyc_runtime.a` (junto al binario)
2. `../lib/libkyc_runtime.a`  ← este es el que usamos
3. Mismo directorio que el binario
4. `./target/debug/` y `./target/release/`
