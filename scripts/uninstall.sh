#!/usr/bin/env bash
set -eu
if command -v bash >/dev/null 2>&1 && [ -n "${BASH_VERSION:-}" ]; then
    set -o pipefail 2>/dev/null || true
fi

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

error() { printf "${RED}%s${NC}\n" "$*"; exit 1; }
info()  { printf "${YELLOW}%s${NC}\n" "$*"; }
ok()    { printf "${GREEN}%s${NC}\n" "$*"; }

# Buscar ky en PATH
KY_PATH="$(command -v ky 2>/dev/null || true)"
if [ -z "$KY_PATH" ]; then
    error "ky no está instalado en el PATH"
fi

# Resolver directorio de instalación
if [ -L "$KY_PATH" ]; then
    KY_REAL="$(readlink -f "$KY_PATH" 2>/dev/null || readlink "$KY_PATH")"
    KY_DIR="$(dirname "$(dirname "$KY_REAL")")"
else
    KY_DIR="$(dirname "$(dirname "$KY_PATH")")"
fi

echo "Se eliminará ky de:"
echo "  Binario:  $KY_PATH"
echo "  Directorio: $KY_DIR"
echo ""
echo -n "Continuar? [s/N] "
read -r CONFIRM
if [ "$CONFIRM" != "s" ] && [ "$CONFIRM" != "S" ]; then
    echo "Cancelado."
    exit 0
fi

# Eliminar binarios
rm -f "$KY_DIR/bin/ky"
ok "Binarios eliminados: ky"

# Eliminar runtime library
if [ -d "$KY_DIR/lib/ky" ]; then
    rm -rf "$KY_DIR/lib/ky"
    ok "Runtime library eliminada: $KY_DIR/lib/ky"
fi

# Limpiar directorios vacíos (solo si no es /usr/local)
if [ "$KY_DIR" != "/usr/local" ]; then
    rmdir "$KY_DIR/bin" 2>/dev/null || true
    rmdir "$KY_DIR/lib" 2>/dev/null || true
    rmdir "$KY_DIR" 2>/dev/null || true
    ok "Directorios eliminados: $KY_DIR"
fi

echo ""
ok "ky ha sido eliminado completamente."
echo ""
echo "Si agregaste ~/.ky/bin al PATH, elimina esa línea de ~/.zshrc o ~/.bashrc"
