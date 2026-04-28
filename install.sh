#!/usr/bin/env bash
set -euo pipefail

# GeminiX Install Script
# Downloads and installs the latest GeminiX binary.
# Usage: curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/geminix/main/install.sh | bash

REPO="pratikacharya1234/geminix"
BINARY="geminix"
INSTALL_DIR="${GEMINIX_INSTALL_DIR:-/usr/local/bin}"
VERSION="${GEMINIX_VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}[+]${NC} $1"; }
warn()  { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[-]${NC} $1"; }
step()  { echo -e "${CYAN}[*]${NC} $1"; }

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        *)
            error "Unsupported OS: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Check prerequisites
check_prerequisites() {
    if ! command -v curl &>/dev/null; then
        error "curl is required but not installed"
        exit 1
    fi

    if ! command -v tar &>/dev/null; then
        error "tar is required but not installed"
        exit 1
    fi
}

# Download and install
install_geminix() {
    local platform="$1"
    local download_url
    local tmpdir

    step "Detected platform: ${platform}"

    if [ "$VERSION" = "latest" ]; then
        download_url="https://github.com/${REPO}/releases/latest/download/geminix-${platform}.tar.gz"
    else
        download_url="https://github.com/${REPO}/releases/download/${VERSION}/geminix-${platform}.tar.gz"
    fi

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    step "Downloading GeminiX..."
    if ! curl -fsSL --progress-bar "$download_url" -o "$tmpdir/geminix.tar.gz"; then
        error "Failed to download from $download_url"
        error "Check that a release exists for your platform"
        exit 1
    fi

    step "Extracting..."
    tar -xzf "$tmpdir/geminix.tar.gz" -C "$tmpdir"

    step "Installing to ${INSTALL_DIR}..."
    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR"
    fi

    if [ -w "$INSTALL_DIR" ]; then
        cp "$tmpdir/$BINARY" "$INSTALL_DIR/$BINARY"
    else
        warn "Need sudo to install to ${INSTALL_DIR}"
        sudo cp "$tmpdir/$BINARY" "$INSTALL_DIR/$BINARY"
    fi

    chmod +x "$INSTALL_DIR/$BINARY"

    info "GeminiX installed to ${INSTALL_DIR}/${BINARY}"
    echo
    step "Setup complete! Next steps:"
    echo "  1. Get a free API key: https://aistudio.google.com/apikey"
    echo "  2. Set it: export GEMINI_API_KEY=\"your-key-here\""
    echo "  3. Run: geminix"
    echo
}

# Build from source
build_from_source() {
    step "Building from source..."

    if ! command -v cargo &>/dev/null; then
        error "Rust is not installed. Install it from https://rustup.rs"
        exit 1
    fi

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    git clone "https://github.com/${REPO}.git" "$tmpdir"
    cd "$tmpdir"
    cargo build --release

    if [ -w "$INSTALL_DIR" ]; then
        cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
    else
        sudo cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
    fi

    info "GeminiX built and installed to ${INSTALL_DIR}/${BINARY}"
}

# Main
main() {
    echo
    info "GeminiX Installer"
    echo

    check_prerequisites

    local platform
    platform="$(detect_platform)"

    # Check if we have Rust/Cargo available
    if command -v cargo &>/dev/null && [ "${GEMINIX_FROM_SOURCE:-}" = "1" ]; then
        build_from_source
    else
        # Try binary download first, fall back to source build
        if install_geminix "$platform" 2>/dev/null; then
            :
        else
            warn "No prebuilt binary found for ${platform}"
            if command -v cargo &>/dev/null; then
                info "Falling back to building from source..."
                build_from_source
            else
                error "No prebuilt binary and no Rust toolchain available"
                error "Install Rust: https://rustup.rs"
                exit 1
            fi
        fi
    fi

    info "Done. Run 'geminix' to start."
}

main
