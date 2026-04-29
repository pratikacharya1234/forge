#!/usr/bin/env bash
# FORGE Replit setup script — downloads the prebuilt binary
set -euo pipefail

FORGE_VERSION="0.0.1"
FORGE_URL="https://github.com/pratikacharya1234/forge/releases/download/v${FORGE_VERSION}/forge-cli-linux-x86_64.tar.gz"

echo "[+] Setting up FORGE v${FORGE_VERSION} for Replit..."
echo ""

if [ -f "./forge-cli" ]; then
    echo "[*] FORGE already installed: $(./forge-cli --version 2>&1 | head -1)"
    echo ""
    exit 0
fi

echo "[*] Downloading FORGE..."
curl -fsSL "$FORGE_URL" -o /tmp/forge.tar.gz

echo "[*] Extracting..."
tar -xzf /tmp/forge.tar.gz -C .
chmod +x forge-cli
rm /tmp/forge.tar.gz

echo ""
echo "[+] FORGE $(./forge-cli --version 2>&1 | head -1) installed!"
echo ""
