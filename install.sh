#!/usr/bin/env bash
# GeliShell installer for Linux and macOS
#
# IMPORTANT — sqlite-vec is NOT the same as SQLite:
#   SQLite     → standard relational database (likely already installed)
#   sqlite-vec → a SEPARATE vector-search C extension by Alex Garcia
#                https://github.com/asg017/sqlite-vec
#                GeliShell downloads the .so/.dylib from its GitHub releases.
#
# build_docs_db requires Ollama running with nomic-embed-text:
#   https://ollama.com/download
#   ollama pull nomic-embed-text
#
# Usage:
#   ./install.sh                   # interactive install
#   ./install.sh --force           # overwrite all existing files
#   ./install.sh --skip-docs       # skip docs.db generation
#   ./install.sh --bin-dir <path>  # custom binary directory

set -euo pipefail
IFS=$'\n\t'

# ── Colors ────────────────────────────────────────────────────

CYAN='\033[0;36m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
RED='\033[0;31m';  GRAY='\033[0;90m';  MAGENTA='\033[0;35m'
RESET='\033[0m'

step() { echo -e "  ${CYAN}-->${RESET} $1"; }
ok()   { echo -e "   ${GREEN}ok${RESET} $1"; }
warn() { echo -e " ${YELLOW}warn${RESET} $1"; }
info() { echo -e "      ${GRAY}$1${RESET}"; }
fail() { echo -e " ${RED}FAIL${RESET} $1"; exit 1; }

ask_yes_no() {
    local question="$1"
    local default="${2:-y}"   # y or n
    local hint
    hint="$( [[ "$default" == "y" ]] && echo "[Y/n]" || echo "[y/N]" )"
    echo -ne "  ${CYAN}${question} ${hint}${RESET} "
    local answer
    read -r answer
    answer="${answer:-$default}"
    [[ "${answer,,}" == "y" || "${answer,,}" == "yes" ]]
}

# ── Args ──────────────────────────────────────────────────────

FORCE=false
SKIP_DOCS=false
BIN_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --force|-f)       FORCE=true; shift ;;
        --skip-docs)      SKIP_DOCS=true; shift ;;
        --bin-dir)        BIN_DIR="$2"; shift 2 ;;
        --bin-dir=*)      BIN_DIR="${1#*=}"; shift ;;
        -h|--help)
            sed -n '/^# Usage/,/^[^#]/{ /^#/p }' "$0" | sed 's/^# \?//'
            exit 0 ;;
        *) fail "unknown argument: $1 (use --help)" ;;
    esac
done

# ── Banner ────────────────────────────────────────────────────

echo ""
echo -e "  ${MAGENTA}GeliShell Installer  |  Bash — Linux / macOS${RESET}"
echo ""

# ── Detect OS ─────────────────────────────────────────────────

OS="$(uname -s)"
case "$OS" in
    Linux*)  PLATFORM="linux"; VEC0_EXT="so";    BINARY_NAME="geli" ;;
    Darwin*) PLATFORM="macos"; VEC0_EXT="dylib"; BINARY_NAME="geli" ;;
    *)       fail "Unsupported OS: $OS" ;;
esac
info "platform: $PLATFORM"

# ── Project root ──────────────────────────────────────────────

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[[ -f "$PROJECT_ROOT/Cargo.toml" ]] || fail "Run from the GeliShell project root (where Cargo.toml lives)"
info "project root: $PROJECT_ROOT"

# ── Paths ─────────────────────────────────────────────────────

HOME_DIR="${HOME:-$(eval echo ~"$USER")}"
[[ -z "$BIN_DIR" ]] && BIN_DIR="$HOME_DIR/.local/bin"

CONFIG_ROOT="$HOME_DIR/.config/geliShell"
MODELS_DIR="$CONFIG_ROOT/models"
DOCS_DIR="$CONFIG_ROOT/docs"
VEC0_DEST="$MODELS_DIR/vec0.${VEC0_EXT}"
DOCS_DB_PATH="$DOCS_DIR/docs.db"

for DIR in "$BIN_DIR" "$CONFIG_ROOT" "$MODELS_DIR" "$DOCS_DIR"; do
    mkdir -p "$DIR"
done

# ══════════════════════════════════════════════════════════════
# STEP 1 — geli binary
# ══════════════════════════════════════════════════════════════

echo ""
step "installing GeliShell binary..."

BINARY_SOURCE="$PROJECT_ROOT/target/release/$BINARY_NAME"
if [[ ! -f "$BINARY_SOURCE" ]]; then
    echo ""
    warn "Binary not found: $BINARY_SOURCE"
    echo -e "  ${YELLOW}Run first:  cargo build --release${RESET}"
    echo ""
    exit 1
fi

BINARY_DEST="$BIN_DIR/geli"
cp -f "$BINARY_SOURCE" "$BINARY_DEST"
chmod +x "$BINARY_DEST"
ok "geli → $BINARY_DEST"

# PATH
add_to_rc() {
    local rc_file="$1"
    [[ -f "$rc_file" ]] || return
    grep -qF "$BIN_DIR" "$rc_file" 2>/dev/null && return
    printf '\n# GeliShell\nexport PATH="$PATH:%s"\n' "$BIN_DIR" >> "$rc_file"
    ok "added to $rc_file"
}

if echo "$PATH" | tr ':' '\n' | grep -qxF "$BIN_DIR"; then
    info "$BIN_DIR already in PATH"
else
    add_to_rc "$HOME_DIR/.bashrc"
    add_to_rc "$HOME_DIR/.bash_profile"
    add_to_rc "$HOME_DIR/.zshrc"
    add_to_rc "$HOME_DIR/.profile"
    export PATH="$PATH:$BIN_DIR"
    warn "Restart your terminal (or source your rc file) for PATH to take effect"
fi

# ══════════════════════════════════════════════════════════════
# STEP 2 — SQLite (sanity check only)
# ══════════════════════════════════════════════════════════════

echo ""
step "checking SQLite..."

SQLITE_OK=false
if command -v sqlite3 &>/dev/null; then
    SQLITE_VERSION="$(sqlite3 --version 2>&1 | head -1)"
    ok "sqlite3 found: $SQLITE_VERSION"
    SQLITE_OK=true
else
    warn "sqlite3 not found in PATH"
    info "SQLite is a runtime dependency for the GeliShell assistant."
    echo ""

    if ask_yes_no "Install SQLite now?"; then
        SQLITE_INSTALLED=false

        if [[ "$PLATFORM" == "macos" ]] && command -v brew &>/dev/null; then
            info "trying: brew install sqlite ..."
            brew install sqlite && SQLITE_INSTALLED=true && SQLITE_OK=true \
                && ok "SQLite installed via Homebrew" \
                || warn "brew install failed"
        fi

        if [[ "$PLATFORM" == "linux" ]] && ! $SQLITE_INSTALLED; then
            # Detect package manager
            if command -v apt-get &>/dev/null; then
                info "trying: apt-get install sqlite3 ..."
                sudo apt-get install -y sqlite3 \
                    && SQLITE_INSTALLED=true && SQLITE_OK=true \
                    && ok "SQLite installed via apt" \
                    || warn "apt-get failed"
            elif command -v dnf &>/dev/null; then
                info "trying: dnf install sqlite ..."
                sudo dnf install -y sqlite \
                    && SQLITE_INSTALLED=true && SQLITE_OK=true \
                    && ok "SQLite installed via dnf" \
                    || warn "dnf failed"
            elif command -v pacman &>/dev/null; then
                info "trying: pacman -S sqlite ..."
                sudo pacman -S --noconfirm sqlite \
                    && SQLITE_INSTALLED=true && SQLITE_OK=true \
                    && ok "SQLite installed via pacman" \
                    || warn "pacman failed"
            fi
        fi

        if ! $SQLITE_INSTALLED; then
            warn "Automatic install failed. Install manually:"
            if [[ "$PLATFORM" == "macos" ]]; then
                info "  brew install sqlite"
            else
                info "  sudo apt-get install sqlite3  (Debian/Ubuntu)"
                info "  sudo dnf install sqlite       (Fedora/RHEL)"
                info "  sudo pacman -S sqlite         (Arch)"
            fi
        fi
    else
        info "Skipped. GeliShell core works without SQLite; AI assistant will not."
    fi
fi

# ══════════════════════════════════════════════════════════════
# STEP 3 — sqlite-vec
#
# sqlite-vec is a SEPARATE project from SQLite.
# Source: https://github.com/asg017/sqlite-vec
#
# Do NOT copy the .so/.dylib from a SQLite installation folder —
# the correct file comes from sqlite-vec GitHub releases only.
#
# GeliShell loads it at runtime from:
#   ~/.config/geliShell/models/vec0.{so|dylib}
# ══════════════════════════════════════════════════════════════

echo ""
step "checking sqlite-vec extension (vec0.${VEC0_EXT})..."
info "sqlite-vec is NOT part of SQLite — it is a separate vector-search"
info "extension: https://github.com/asg017/sqlite-vec"

VEC0_AVAILABLE=false

if [[ -f "$VEC0_DEST" ]] && ! $FORCE; then
    ok "vec0.${VEC0_EXT} already present: $VEC0_DEST"
    VEC0_AVAILABLE=true
fi

if ! $VEC0_AVAILABLE; then
    LOCAL_CANDIDATES=(
        "$PROJECT_ROOT/assets/vec0.${VEC0_EXT}"
        "$PROJECT_ROOT/models/vec0.${VEC0_EXT}"
        "$PROJECT_ROOT/vec0.${VEC0_EXT}"
    )
    for CANDIDATE in "${LOCAL_CANDIDATES[@]}"; do
        if [[ -f "$CANDIDATE" ]]; then
            cp -f "$CANDIDATE" "$VEC0_DEST"
            ok "vec0.${VEC0_EXT} found locally → copied from: $CANDIDATE"
            VEC0_AVAILABLE=true
            break
        fi
    done
fi

if ! $VEC0_AVAILABLE; then
    echo ""
    warn "vec0.${VEC0_EXT} not found locally."
    echo ""
    info "GeliShell needs vec0.${VEC0_EXT} for the AI assistant RAG engine."
    info "It will be placed at: $VEC0_DEST"
    echo ""

    if ask_yes_no "Download vec0.${VEC0_EXT} from github.com/asg017/sqlite-vec now?"; then
        download_vec0 && VEC0_AVAILABLE=true
    else
        info "Skipped. Install manually:"
        info "  1. https://github.com/asg017/sqlite-vec/releases"
        if [[ "$PLATFORM" == "linux" ]]; then
            info "  2. Download: sqlite-vec-*-loadable-linux-x86_64.tar.gz"
        else
            info "  2. Download: sqlite-vec-*-loadable-macos-aarch64.tar.gz (Apple Silicon)"
            info "     or:       sqlite-vec-*-loadable-macos-x86_64.tar.gz  (Intel)"
        fi
        info "  3. Extract vec0.${VEC0_EXT} and copy to:"
        info "     $VEC0_DEST"
    fi
fi

download_vec0() {
    info "fetching latest release info from GitHub API..."

    if ! command -v curl &>/dev/null; then
        warn "curl not found — cannot download automatically"
        return 1
    fi

    local api_url="https://api.github.com/repos/asg017/sqlite-vec/releases/latest"
    local release_json
    release_json="$(curl -fsSL \
        -H "User-Agent: GeliShell-Installer/0.1" \
        -H "Accept: application/vnd.github+json" \
        --max-time 15 \
        "$api_url")" || { warn "GitHub API request failed"; return 1; }

    local tag
    tag="$(echo "$release_json" | grep '"tag_name"' | head -1 | sed 's/.*: "\(.*\)".*/\1/')"
    info "latest sqlite-vec release: $tag"

    # Build asset name pattern based on platform/arch
    local arch
    arch="$(uname -m)"
    local asset_pattern
    case "$PLATFORM-$arch" in
        macos-arm64|macos-aarch64) asset_pattern="loadable-macos-aarch64" ;;
        macos-x86_64)              asset_pattern="loadable-macos-x86_64"  ;;
        linux-x86_64)              asset_pattern="loadable-linux-x86_64"  ;;
        linux-aarch64|linux-arm64) asset_pattern="loadable-linux-aarch64" ;;
        *)                         asset_pattern="loadable-${PLATFORM}-${arch}" ;;
    esac

    info "looking for asset matching: *${asset_pattern}*"

    local download_url
    download_url="$(echo "$release_json" | \
        grep '"browser_download_url"' | \
        grep "$asset_pattern" | \
        grep -E '\.tar\.gz"|\.zip"' | \
        head -1 | \
        sed 's/.*"browser_download_url": "\(.*\)".*/\1/')"

    if [[ -z "$download_url" ]]; then
        warn "No matching asset found for ${PLATFORM}/${arch} in release $tag"
        info "Available assets:"
        echo "$release_json" | grep '"name"' | grep -v "tag_name" | \
            sed 's/.*"name": "\(.*\)".*/  \1/' | head -20
        info ""
        info "Manual install:"
        info "  https://github.com/asg017/sqlite-vec/releases/tag/$tag"
        info "  Copy vec0.${VEC0_EXT} to: $VEC0_DEST"
        return 1
    fi

    local asset_file
    asset_file="$(basename "$download_url")"
    local tmp_archive="/tmp/sqlite-vec-${tag}-${asset_file}"
    local tmp_dir="/tmp/sqlite-vec-extract-${tag}"

    info "downloading: $asset_file ..."
    curl -fL --max-time 120 --progress-bar \
        -o "$tmp_archive" "$download_url" \
        || { warn "Download failed"; return 1; }

    rm -rf "$tmp_dir"
    mkdir -p "$tmp_dir"

    # Extract (handle both .tar.gz and .zip)
    if [[ "$tmp_archive" == *.zip ]]; then
        command -v unzip &>/dev/null \
            || { warn "unzip not found — install it and retry"; return 1; }
        unzip -q "$tmp_archive" -d "$tmp_dir"
    else
        tar -xzf "$tmp_archive" -C "$tmp_dir"
    fi

    # Find the extension file
    local found_vec0
    found_vec0="$(find "$tmp_dir" -name "vec0.${VEC0_EXT}" | head -1)"

    if [[ -z "$found_vec0" ]]; then
        # Fallback: any .so/.dylib in the archive
        found_vec0="$(find "$tmp_dir" -name "*.${VEC0_EXT}" | head -1)"
    fi

    if [[ -z "$found_vec0" ]]; then
        warn "vec0.${VEC0_EXT} not found in downloaded archive"
        info "Archive contents:"
        find "$tmp_dir" | head -20
        return 1
    fi

    cp -f "$found_vec0" "$VEC0_DEST"
    rm -rf "$tmp_archive" "$tmp_dir"
    ok "vec0.${VEC0_EXT} installed at: $VEC0_DEST"
    return 0
}

# ══════════════════════════════════════════════════════════════
# STEP 4 — Ollama
# ══════════════════════════════════════════════════════════════

echo ""
step "checking Ollama..."

OLLAMA_OK=false
if command -v ollama &>/dev/null; then
    OLLAMA_VERSION="$(ollama --version 2>&1 | head -1)"
    ok "ollama found: $OLLAMA_VERSION"
    OLLAMA_OK=true
else
    warn "ollama not found in PATH"
    info "Ollama is required to generate docs.db (the RAG knowledge base)."
    info "Install from: https://ollama.com/download"
    info "Then pull the embedding model:"
    info "  ollama pull nomic-embed-text"
fi

# ══════════════════════════════════════════════════════════════
# STEP 5 — docs.db via build_docs_db
# ══════════════════════════════════════════════════════════════

echo ""
step "checking docs.db (RAG knowledge base)..."

DOCS_DB_OK=false
if [[ -f "$DOCS_DB_PATH" ]] && ! $FORCE; then
    ok "docs.db already present: $DOCS_DB_PATH"
    DOCS_DB_OK=true
elif $SKIP_DOCS; then
    info "skipping docs.db generation (--skip-docs)"
elif ! $VEC0_AVAILABLE; then
    warn "skipping — vec0.${VEC0_EXT} not available (required by build_docs_db)"
    info "Install sqlite-vec first, then run:"
    info "  cargo run --bin build_docs_db"
elif ! $OLLAMA_OK; then
    warn "skipping — Ollama not available (required to generate embeddings)"
    info "Start Ollama and run:"
    info "  ollama pull nomic-embed-text"
    info "  cargo run --bin build_docs_db"
else
    echo ""
    info "docs.db is generated by embedding your markdown docs with Ollama."
    info "Make sure Ollama is running:  ollama serve"
    echo ""

    if ask_yes_no "Generate docs.db now? (cargo run --bin build_docs_db)"; then
        pushd "$PROJECT_ROOT" > /dev/null
        OLD_VEC0_ENV="${GELI_SQLITE_VEC_PATH:-}"
        export GELI_SQLITE_VEC_PATH="$VEC0_DEST"
        info "GELI_SQLITE_VEC_PATH=$VEC0_DEST"
        echo ""

        if cargo run --bin build_docs_db; then
            export GELI_SQLITE_VEC_PATH="$OLD_VEC0_ENV"
            if [[ -f "$DOCS_DB_PATH" ]]; then
                ok "docs.db generated at: $DOCS_DB_PATH"
                DOCS_DB_OK=true
            else
                warn "build_docs_db finished but docs.db not found at expected path"
            fi
        else
            export GELI_SQLITE_VEC_PATH="$OLD_VEC0_ENV"
            warn "build_docs_db failed. Fix the error and re-run:"
            info "  cargo run --bin build_docs_db"
        fi
        popd > /dev/null
    else
        info "Skipped. Run manually when ready:"
        info "  cargo run --bin build_docs_db"
    fi
fi

# ══════════════════════════════════════════════════════════════
# SUMMARY
# ══════════════════════════════════════════════════════════════

status_line() {
    local ok_flag="$1"; local label="$2"; local detail="$3"
    if [[ "$ok_flag" == "true" ]]; then
        echo -e "  ${GREEN}[OK]${RESET} $label"
    else
        echo -e "  ${GRAY}[--]${RESET} $label"
    fi
    [[ -n "$detail" ]] && echo -e "       ${GRAY}$detail${RESET}"
}

echo ""
echo -e "  ${GRAY}──────────────────────────────────────────${RESET}"
echo -e "  ${MAGENTA}GeliShell Installation Summary${RESET}"
echo -e "  ${GRAY}──────────────────────────────────────────${RESET}"
echo ""
status_line "true"          "geli"        "$BINARY_DEST"
status_line "$SQLITE_OK"    "SQLite"      "sqlite3 in PATH"
status_line "$VEC0_AVAILABLE" "sqlite-vec" "vec0.${VEC0_EXT} — $VEC0_DEST"
status_line "$OLLAMA_OK"    "Ollama"      "ollama in PATH"
status_line "$DOCS_DB_OK"   "docs.db"     "$DOCS_DB_PATH"
echo ""

if $VEC0_AVAILABLE && $OLLAMA_OK && $DOCS_DB_OK; then
    echo -e "  ${GREEN}All components ready.${RESET}"
else
    echo -e "  ${GREEN}GeliShell core is installed and fully functional.${RESET}"
    echo -e "  ${YELLOW}AI assistant features require the missing components above.${RESET}"
fi

echo ""
echo -e "  ${CYAN}Open a new terminal and run: geli${RESET}"
echo ""
