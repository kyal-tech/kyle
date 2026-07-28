#!/bin/bash
# Redirect to the actual install script
SCRIPT_URL="https://raw.githubusercontent.com/IT-KYNERA/KYLE/main/scripts/install.sh"
echo "Downloading from $SCRIPT_URL..."
curl -fsSL "$SCRIPT_URL" | sh
