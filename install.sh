#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ── GeminiX Install Script ─────────────────────────────────────────────────────
# One-liner: curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/geminix/main/install.sh | bash
#
# Options:
#   GEMINIX_INSTALL_DIR   Install directory (default: /usr/local/bin)
#   GEMINIX_VERSION       Version to install (default: latest)
#   GEMINIX_FROM_SOURCE=1 Force source build
#   --help                Show help
#   --dir /path           Override install directory
#   --version X.Y.Z       Install specific version
# ────────────────────────────────────────────────────────────────────────────────

REPO="pratikacharya1234/geminix"
BINARY="geminix"
INSTALL_DIR="${GEMINIX_INSTALL_DIR:-/usr/local/bin}"
VERSION="${GEMINIX_VERSION:-latest}"

# ── Argument parsing ──────────────────────────────────────────────────────────

while [ $# -gt 0 ]; do
    case "$1" in
        --help|-h)
            echo "GeminiX Installer"
            echo ""
            echo "Usage: curl -fsSL https://raw.githubusercontent.com/pratikacharya1234/geminix/main/install.sh | bash"
            echo "       bash install.sh"
            echo ""
            echo "Options:"
            echo "  --dir PATH        Install directory (default: /usr/local/bin)"
            echo "  --version VER     Install specific version (default: latest)"
            echo "  --help            Show this help"
            echo ""
            echo "Environment:"
            echo "  GEMINIX_INSTALL_DIR   Override install directory"
            echo "  GEMINIX_VERSION       Override version"
            echo "  GEMINIX_FROM_SOURCE=1 Force build from source"
            exit 0
            ;;
        --dir)
            INSTALL_DIR="$2"; shift 2 ;;
        --version)
            VERSION="$1"; shift 2 ;;
        *)
            echo "Unknown option: $1 (use --help)"
            exit 1
            ;;
    esac
done

# ── Colors ────────────────────────────────────────────────────────────────────

if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    CYAN='\033[0;36m'; MAGENTA='\033[0;35m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; CYAN=''; MAGENTA=''; NC=''
fi

info()  { printf "${GREEN}[+]${NC} %s\n" "$1" >&2; }
warn()  { printf "${YELLOW}[!]${NC} %s\n" "$1" >&2; }
error() { printf "${RED}[-]${NC} %s\n" "$1" >&2; }
step()  { printf "${CYAN}[*]${NC} %s\n" "$1" >&2; }

# ── Platform detection ───────────────────────────────────────────────────────

detect_platform() {
    case "$(uname -s)" in
        Linux)  echo "linux" ;;
        Darwin) echo "macos" ;;
        *)      error "Unsupported OS: $(uname -s)"; exit 1 ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "arm64" ;;
        *) error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
}

# ── Install from binary release ──────────────────────────────────────────────

install_from_release() {
    local platform="$1"
    local tmpdir

    if [ "$VERSION" = "latest" ]; then
        download_url="https://github.com/${REPO}/releases/latest/download/geminix-${platform}.tar.gz"
    else
        download_url="https://github.com/${REPO}/releases/download/v${VERSION}/geminix-${platform}.tar.gz"
    fi

    tmpdir="$(mktemp -d)"
    # shellcheck disable=SC2064
    trap "rm -rf '$tmpdir'" EXIT

    step "Downloading GeminiX v${VERSION} for ${platform}..."
    if ! curl -fsSL --connect-timeout 10 --max-time 60 \
        "$download_url" -o "$tmpdir/geminix.tar.gz" 2>/dev/null; then
        return 1
    fi

    step "Extracting..."
    if ! tar -xzf "$tmpdir/geminix.tar.gz" -C "$tmpdir" 2>/dev/null; then
        return 1
    fi

    if [ ! -f "$tmpdir/$BINARY" ]; then
        error "Binary not found in archive"
        return 1
    fi

    install_binary "$tmpdir/$BINARY"
    return 0
}

# ── Install from source ──────────────────────────────────────────────────────

build_from_source() {
    step "Building GeminiX from source..."

    if ! command -v cargo &>/dev/null; then
        error "Rust is not installed."
        echo ""
        echo "  Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "  Then re-run this script."
        exit 1
    fi

    # Check Rust version
    local rust_version
    rust_version="$(rustc --version | grep -oP '\d+\.\d+' | head -1)"
    if [ "$(printf '%s\n' "1.75" "$rust_version" | sort -V | head -1)" != "1.75" ]; then
        error "Rust $rust_version is too old. Rust 1.75+ required."
        echo "  Update: rustup update"
        exit 1
    fi

    local srcdir
    local cleanup_tmp=false
    # Detect if running from the repo itself (development mode)
    if [ -f "${SCRIPT_DIR:-.}/Cargo.toml" ] && [ -f "${SCRIPT_DIR:-.}/src/main.rs" ]; then
        srcdir="${SCRIPT_DIR}"
        step "Using local repository at ${srcdir}"
    elif [ -f "Cargo.toml" ] && [ -f "src/main.rs" ]; then
        srcdir="$(pwd)"
        step "Using current directory as source"
    else
        if ! command -v git &>/dev/null; then
            error "git is required. Install it first: apt install git / brew install git"
            exit 1
        fi
        srcdir="$(mktemp -d)"
        cleanup_tmp=true
        step "Cloning repository..."
        if ! git clone --depth 1 "https://github.com/${REPO}.git" "$srcdir" 2>/dev/null; then
            error "Failed to clone https://github.com/${REPO}.git"
            error "Make sure the repo exists and is public, or run this script from the repo directory."
            exit 1
        fi
    fi

    cd "$srcdir"

    step "Building (this may take 2-5 minutes)..."
    if ! cargo build --release 2>&1 | grep -v "^$" | tail -3; then
        error "Build failed. Check the error output above."
        exit 1
    fi

    install_binary "$srcdir/target/release/$BINARY"
    if [ "$cleanup_tmp" = true ]; then rm -rf "$srcdir"; fi
}

# ── Install the binary ───────────────────────────────────────────────────────

install_binary() {
    local src="$1"

    if [ ! -f "$src" ]; then
        error "Binary '$src' not found"
        return 1
    fi

    step "Installing to ${INSTALL_DIR}..."

    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR" 2>/dev/null || {
            warn "Need sudo to create ${INSTALL_DIR}"
            sudo mkdir -p "$INSTALL_DIR"
        }
    fi

    if [ -w "$INSTALL_DIR" ]; then
        cp "$src" "$INSTALL_DIR/$BINARY"
    else
        warn "Need sudo to install to ${INSTALL_DIR}"
        sudo cp "$src" "$INSTALL_DIR/$BINARY"
    fi

    chmod +x "$INSTALL_DIR/$BINARY" 2>/dev/null || sudo chmod +x "$INSTALL_DIR/$BINARY"

    # Verify
    if ! "$INSTALL_DIR/$BINARY" --version >/dev/null 2>&1; then
        error "Installation verification failed — binary does not run"
        error "Try running directly: $INSTALL_DIR/$BINARY --version"
        exit 1
    fi

    local installed_version
    installed_version="$("$INSTALL_DIR/$BINARY" --version 2>&1 | head -1)"

    echo ""
    info "GeminiX ${installed_version} installed to ${INSTALL_DIR}/${BINARY}"
    echo ""
    step "Next steps:"
    echo "  1. Get a free API key → https://aistudio.google.com/apikey"
    echo "  2. Set it:           export GEMINI_API_KEY=\"your-key-here\""
    echo "  3. Run it:           geminix"
    echo ""
    echo "  With Claude:         geminix --model claude-4-sonnet --anthropic-api-key \"sk-ant-...\""
    echo "  With GPT:            geminix --model gpt-4.1 --openai-api-key \"sk-...\""
    echo ""
}

# ── Main ─────────────────────────────────────────────────────────────────────

main() {
    echo ""
    info "GeminiX Installer"
    echo ""

    local platform
    platform="$(detect_platform)-$(detect_arch)"

    step "System: $(uname -s) $(uname -m)"

    # Check if force source build
    if [ "${GEMINIX_FROM_SOURCE:-}" = "1" ]; then
        step "Source build requested"
        build_from_source
        return
    fi

    # Try prebuilt binary first
    if command -v curl &>/dev/null && command -v tar &>/dev/null; then
        step "Checking for prebuilt binary..."
        if install_from_release "$platform"; then
            return
        fi
        warn "No prebuilt binary for ${platform} (v${VERSION})"
    fi

    # Fall back to source build
    if command -v cargo &>/dev/null; then
        info "Building from source..."
        build_from_source
    else
        echo ""
        error "No prebuilt binary found and Rust is not installed."
        echo ""
        echo "  Option 1: Install Rust and re-run this script"
        echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        echo "    Then: bash install.sh"
        echo ""
        echo "  Option 2: Download a prebuilt binary from GitHub"
        echo "    https://github.com/${REPO}/releases"
        echo ""
        exit 1
    fi
}

main
