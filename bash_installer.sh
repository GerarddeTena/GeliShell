#!/usr/bin/env bash
# GeliShell installer for Linux and macOS
#
# Usage:
#   ./install.sh              # standard install
#   ./install.sh --force      # overwrite assets even if up-to-date
#   ./install.sh --bin-dir /usr/local/bin   # custom bin directory
#
# Must be run from the GeliShell project root (where Cargo.toml lives).

set -euo pipefail

# ── Color helpers ────────────────────────────────────────────

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
GRAY='\033[0;90m'
MAGENTA='\033[0;35m'
RESET='\033[0m'

step() { echo -e "  ${CYAN}-->${RESET} $1"; }
ok()   { echo -e "   ${GREEN}ok${RESET} $1"; }
warn() { echo -e " ${YELLOW}warn${RESET} $1"; }
fail() { echo -e " ${RED}FAIL${RESET} $1"; exit 1; }
info() { echo -e "      ${GRAY}$1${RESET}"; }

# ── Parse arguments ──────────────────────────────────────────

FORCE=false
BIN_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --force|-f)   FORCE=true; shift ;;
        --bin-dir)    BIN_DIR="$2"; shift 2 ;;
        --bin-dir=*)  BIN_DIR="${1#*=}"; shift ;;
        -h|--help)
            grep '^#' "$0" | grep -v '#!/' | sed 's/^# \?//'
            exit 0 ;;
        *) fail "unknown argument: $1" ;;
    esac
done

# ── Banner ───────────────────────────────────────────────────

echo ""
echo -e "  ${MAGENTA}GeliShell Installer${RESET}"
echo -e "  ${GRAY}v0.1.0  |  Bash — Linux / macOS${RESET}"
echo ""

# ── Validate project root ────────────────────────────────────

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
    fail "Run this script from the GeliShell project root (where Cargo.toml lives)"
fi
info "project root: $PROJECT_ROOT"

# ── Detect OS and binary name ────────────────────────────────

OS="$(uname -s)"
case "$OS" in
    Linux*)   PLATFORM="linux";  LIB_EXT="so";    BINARY_NAME="geli_shell" ;;
    Darwin*)  PLATFORM="macos";  LIB_EXT="dylib";  BINARY_NAME="geli_shell" ;;
    *)        fail "Unsupported OS: $OS" ;;
esac

info "platform: $PLATFORM"

# ── Locate binary ────────────────────────────────────────────

BINARY_SOURCE="$PROJECT_ROOT/target/release/$BINARY_NAME"

if [[ ! -f "$BINARY_SOURCE" ]]; then
    echo ""
    warn "Binary not found at: $BINARY_SOURCE"
    echo -e "  ${YELLOW}Run first:  cargo build --release${RESET}"
    echo ""
    exit 1
fi
ok "binary found: $BINARY_SOURCE"

# ── Resolve install paths ────────────────────────────────────

HOME_DIR="${HOME:-$( eval echo ~"$USER" )}"

if [[ -z "$BIN_DIR" ]]; then
    BIN_DIR="$HOME_DIR/.local/bin"
fi

CONFIG_ROOT="$HOME_DIR/.config/geliShell"
MODELS_DIR="$CONFIG_ROOT/models"
DOCS_DIR="$CONFIG_ROOT/docs"
BINARY_DEST="$BIN_DIR/geli"

info "install dir:  $BIN_DIR"
info "config root:  $CONFIG_ROOT"

# ── Create directory layout ──────────────────────────────────

step "creating directory layout..."

for DIR in "$BIN_DIR" "$CONFIG_ROOT" "$MODELS_DIR" "$DOCS_DIR"; do
    if [[ ! -d "$DIR" ]]; then
        mkdir -p "$DIR"
        ok "created: $DIR"
    else
        info "exists:  $DIR"
    fi
done

# ── Install binary ───────────────────────────────────────────

step "installing binary..."

if [[ -f "$BINARY_DEST" ]] && [[ "$FORCE" != "true" ]]; then
    EXISTING_DATE="$(date -r "$BINARY_DEST" '+%Y-%m-%d %H:%M' 2>/dev/null || echo 'unknown')"
    warn "geli already installed (modified: $EXISTING_DATE)"
    warn "overwriting with new build..."
fi

cp -f "$BINARY_SOURCE" "$BINARY_DEST"
chmod +x "$BINARY_DEST"
ok "installed: $BINARY_DEST"

# ── Copy runtime assets ──────────────────────────────────────

step "copying runtime assets..."

# copy_asset <label> <dest> <source1> [<source2> ...]
copy_asset() {
    local LABEL="$1"
    local DEST="$2"
    shift 2
    local SOURCES=("$@")
    local COPIED=false

    for SRC in "${SOURCES[@]}"; do
        if [[ -f "$SRC" ]]; then
            local SHOULD_COPY=false
            if [[ "$FORCE" == "true" ]]; then
                SHOULD_COPY=true
            elif [[ ! -f "$DEST" ]]; then
                SHOULD_COPY=true
            elif [[ "$SRC" -nt "$DEST" ]]; then
                SHOULD_COPY=true
            fi

            if [[ "$SHOULD_COPY" == "true" ]]; then
                mkdir -p "$(dirname "$DEST")"
                cp -f "$SRC" "$DEST"
                ok "copied:  $LABEL"
            else
                info "skip:    $LABEL (already up-to-date)"
            fi
            COPIED=true
            break
        fi
    done

    if [[ "$COPIED" != "true" ]]; then
        warn "not found: $LABEL — assistant features may be limited"
        info "  expected at one of:"
        for SRC in "${SOURCES[@]}"; do
            info "    $SRC"
        done
    fi
}

# sqlite-vec extension (platform specific)
copy_asset \
    "sqlite-vec extension (.${LIB_EXT})" \
    "$MODELS_DIR/vec0.${LIB_EXT}" \
    "$PROJECT_ROOT/assets/vec0.${LIB_EXT}" \
    "$PROJECT_ROOT/models/vec0.${LIB_EXT}" \
    "$PROJECT_ROOT/vec0.${LIB_EXT}"

# RAG knowledge base
copy_asset \
    "docs.db (RAG knowledge base)" \
    "$DOCS_DIR/docs.db" \
    "$PROJECT_ROOT/assets/docs.db" \
    "$PROJECT_ROOT/docs/docs.db" \
    "$PROJECT_ROOT/docs.db"

# Assistant index
copy_asset \
    "dbjson (assistant index)" \
    "$MODELS_DIR/dbjson" \
    "$PROJECT_ROOT/assets/dbjson" \
    "$PROJECT_ROOT/models/dbjson" \
    "$PROJECT_ROOT/dbjson"

# ── Update shell PATH ────────────────────────────────────────

step "checking PATH..."

path_contains_dir() {
    echo "$PATH" | tr ':' '\n' | grep -qxF "$1"
}

add_to_shell_rc() {
    local RC_FILE="$1"
    local EXPORT_LINE="export PATH=\"\$PATH:$BIN_DIR\""

    if [[ -f "$RC_FILE" ]]; then
        if grep -qF "$BIN_DIR" "$RC_FILE" 2>/dev/null; then
            info "$BIN_DIR already in $RC_FILE"
            return
        fi
        echo "" >> "$RC_FILE"
        echo "# GeliShell" >> "$RC_FILE"
        echo "$EXPORT_LINE" >> "$RC_FILE"
        ok "added to $RC_FILE"
    fi
}

if path_contains_dir "$BIN_DIR"; then
    info "$BIN_DIR is already in PATH"
else
    # Try to add to all common shell rc files that exist
    add_to_shell_rc "$HOME_DIR/.bashrc"
    add_to_shell_rc "$HOME_DIR/.bash_profile"
    add_to_shell_rc "$HOME_DIR/.zshrc"
    add_to_shell_rc "$HOME_DIR/.profile"

    # Also update the current session
    export PATH="$PATH:$BIN_DIR"
    warn "Restart your terminal (or run: source ~/.bashrc) for PATH to take effect"
fi

# ── Verify installation ──────────────────────────────────────

step "verifying installation..."

if [[ -x "$BINARY_DEST" ]]; then
    SIZE_KB=$(( $(stat -c%s "$BINARY_DEST" 2>/dev/null || stat -f%z "$BINARY_DEST" 2>/dev/null || echo 0) / 1024 ))
    ok "geli is executable (${SIZE_KB} KB) at $BINARY_DEST"
else
    fail "binary is not executable at $BINARY_DEST"
fi

# ── Print summary ─────────────────────────────────────────────

echo ""
echo -e "  ${GREEN}GeliShell installed successfully!${RESET}"
echo ""
echo -e "  ${GRAY}Binary  : $BINARY_DEST${RESET}"
echo -e "  ${GRAY}Config  : $CONFIG_ROOT${RESET}"
echo -e "  ${GRAY}Models  : $MODELS_DIR${RESET}"
echo ""
echo -e "  ${CYAN}To start GeliShell, open a new terminal and run:${RESET}"
echo -e "    geli"
echo ""
echo -e "  ${GRAY}First run will launch the setup wizard automatically.${RESET}"
echo -e "  ${GRAY}To reset config at any time, run inside GeliShell:${RESET}"
echo -e "    geli-reset-config"
echo ""