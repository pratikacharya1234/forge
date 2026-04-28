#!/usr/bin/env bash
set -euo pipefail

# ── GeminiX Release Packager ───────────────────────────────────────────────────
# Builds binaries for all platforms and packages them for GitHub releases.
#
# Usage: bash release.sh [VERSION]
#   If VERSION is omitted, reads from Cargo.toml.
#
# Output: packages/geminix-{platform}.tar.gz for each platform
# ────────────────────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    VERSION="$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"
fi

PACKAGES_DIR="$SCRIPT_DIR/packages"
BINARY="geminix"

# Platforms to build
PLATFORMS=(
    "x86_64-unknown-linux-gnu:linux-x86_64"
    "aarch64-unknown-linux-gnu:linux-arm64"
    "x86_64-apple-darwin:macos-x86_64"
    "aarch64-apple-darwin:macos-arm64"
)

# ── Colors ────────────────────────────────────────────────────────────────────

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
info()  { printf "${GREEN}[+]${NC} %s\n" "$1"; }
error() { printf "${RED}[-]${NC} %s\n" "$1"; }
step()  { printf "${CYAN}[*]${NC} %s\n" "$1"; }

# ── Check prerequisites ──────────────────────────────────────────────────────

echo ""
info "GeminiX Release Packager — v${VERSION}"
echo ""

if ! command -v cargo &>/dev/null; then
    error "Rust is not installed"
    exit 1
fi

# Remove old packages
rm -rf "$PACKAGES_DIR"
mkdir -p "$PACKAGES_DIR"

# ── Verify build ─────────────────────────────────────────────────────────────

step "Verifying build..."
if ! cargo build --release 2>&1 | tail -1; then
    error "Build failed"
    exit 1
fi

# ── Native build (current platform) ──────────────────────────────────────────

step "Building native binary..."
cp "target/release/$BINARY" "$PACKAGES_DIR/$BINARY"

local_platform="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
if [ "$(uname -m)" = "x86_64" ]; then local_arch="x86_64"; else local_arch="arm64"; fi
local_name="$(echo "$(uname -s)" | tr '[:upper:]' '[:lower:]')-${local_arch}"

info "Native: ${local_name}"

cd "$PACKAGES_DIR"
tar -czf "geminix-${local_name}.tar.gz" "$BINARY"
rm "$BINARY"
cd "$SCRIPT_DIR"

step "  → packages/geminix-${local_name}.tar.gz"

# ── Cross-compile for other platforms ────────────────────────────────────────

if command -v cross &>/dev/null; then
    for entry in "${PLATFORMS[@]}"; do
        rust_target="${entry%%:*}"
        pkg_name="${entry##*:}"

        if [ "$rust_target" = "$(rustc -vV | grep host | awk '{print $2}')" ]; then
            continue  # Already built native
        fi

        step "Cross-compiling for ${rust_target}..."
        if cross build --release --target "$rust_target" 2>&1 | tail -1; then
            if [ -f "target/${rust_target}/release/${BINARY}" ]; then
                cp "target/${rust_target}/release/${BINARY}" "$PACKAGES_DIR/${BINARY}"
                cd "$PACKAGES_DIR"
                tar -czf "geminix-${pkg_name}.tar.gz" "$BINARY"
                rm "$BINARY"
                cd "$SCRIPT_DIR"
                info "  → packages/geminix-${pkg_name}.tar.gz"
            else
                error "  Binary not found for ${rust_target}"
            fi
        else
            error "  Cross-compile failed for ${rust_target} (skipping)"
        fi
    done
else
    warn="[!]"
    echo ""
    echo -e "  ${CYAN}[*]${NC} cross not installed — building native only."
    echo "      Install: cargo install cross"
    echo "      Then re-run for all platforms."
fi

# ── Checksums ────────────────────────────────────────────────────────────────

echo ""
step "Generating checksums..."
cd "$PACKAGES_DIR"
if command -v sha256sum &>/dev/null; then
    sha256sum *.tar.gz > checksums.sha256
elif command -v shasum &>/dev/null; then
    shasum -a 256 *.tar.gz > checksums.sha256
fi
cd "$SCRIPT_DIR"

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
info "Release packages ready in packages/"
echo ""
ls -lh "$PACKAGES_DIR"/*.tar.gz 2>/dev/null || true
if [ -f "$PACKAGES_DIR/checksums.sha256" ]; then
    echo ""
    cat "$PACKAGES_DIR/checksums.sha256"
fi
echo ""
step "Upload to GitHub:"
echo "  1. Create release: gh release create v${VERSION} packages/geminix-*.tar.gz"
echo "  2. Or manually at:  https://github.com/pratikacharya1234/geminix/releases/new"
echo "  3. Tag: v${VERSION}"
echo "  4. Attach all .tar.gz and checksums.sha256"
echo ""
