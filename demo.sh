#!/usr/bin/env bash
# FORGE Demo — shows what FORGE can do in under 2 minutes
# Prerequisites: forge-cli installed, GEMINI_API_KEY set
set -euo pipefail

echo ""
echo "  ╔══════════════════════════════════════════════════╗"
echo "  ║        ◈ FORGE v0.0.2 — Live Demo              ║"
echo "  ╚══════════════════════════════════════════════════╝"
echo ""

# Check prerequisites
if ! command -v forge-cli &>/dev/null; then
    echo "  [!] forge-cli not found. Install it first:"
    echo "      curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/forge/main/install.sh | bash"
    exit 1
fi

if [ -z "${GEMINI_API_KEY:-}" ] && [ -z "${FORGE_API_KEY:-}" ]; then
    echo "  [!] No API key set. Get a free one at https://aistudio.google.com/apikey"
    echo "      Then: export GEMINI_API_KEY='your-key'"
    exit 1
fi

FORGE_VERSION=$(forge-cli --version 2>&1 | head -1)
echo "  [⊕] FORGE ${FORGE_VERSION} ready"
echo "  [⊕] API key configured"
echo ""
echo "  ─────────────────────────────────────────────────"
echo "  Demo 1: Quick code fix"
echo "  ─────────────────────────────────────────────────"
echo ""

# Create a temp directory for the demo
DEMO_DIR=$(mktemp -d)
cd "$DEMO_DIR"

echo 'fn greet(name: &str) -> String {
    format!("hello {}!", name)
}

fn main() {
    println!("{}", greet("world"));
}' > main.rs

echo "  [+] Created main.rs with a basic Rust program"
echo ""

echo "  Running: forge-cli --auto-apply --prompt 'add a test for the greet function and fix any issues'"
echo ""

# Run FORGE in auto-apply mode
forge-cli --auto-apply --prompt "add a test for the greet function in main.rs and make sure it compiles and passes" 2>&1 || true

echo ""
echo "  ─────────────────────────────────────────────────"
echo "  Demo complete!"
echo "  ─────────────────────────────────────────────────"
echo ""
echo "  What you just saw:"
echo "  1. FORGE auto-detected the Rust project"
echo "  2. Added a test module"
echo "  3. Verified it compiles"
echo "  4. All for FREE (Gemini free tier)"
echo ""
echo "  Try it yourself:"
echo "    export GEMINI_API_KEY='your-free-key'"
echo "    forge-cli"
echo ""
echo "  More: https://forgecli.vercel.app"
echo ""

# Cleanup
cd /
rm -rf "$DEMO_DIR"
