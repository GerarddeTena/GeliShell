#!/usr/bin/env bash
# GeliShell installer for Linux and macOS
#
# IMPORTANT — sqlite-vec is NOT the same as SQLite:
#   SQLite     -> standard relational database (likely already installed)
#   sqlite-vec -> a SEPARATE vector-search C extension by Alex Garcia
#                 https://github.com/asg017/sqlite-vec
#
# docs.db is a pre-generated release artifact — it is NOT built at install
# time.  Place it at assets/docs.db before shipping a release.
# See: cargo run --bin build_docs_db  (developer build step only)
#
# Usage:
#   ./install.sh                   # interactive install
#   ./install.sh --force           # overwrite all existing files
#   ./install.sh --skip-docs       # skip docs.db seeding from assets/
#   ./install.sh --bin-dir <path>  # custom binary directory

set -euo pipefail
IFS=$'\n\t'

# ── Load shared library ───────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=installer/lib/common.sh
source "$SCRIPT_DIR/installer/lib/common.sh"

# Activate rollback on any unexpected non-zero exit.
trap 'do_rollback' ERR

# ── Parse arguments ───────────────────────────────────────────
FORCE=false
SKIP_DOCS=false
BIN_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --force|-f)   FORCE=true; shift ;;
        --skip-docs)  SKIP_DOCS=true; shift ;;
        --bin-dir)    BIN_DIR="$2"; shift 2 ;;
        --bin-dir=*)  BIN_DIR="${1#*=}"; shift ;;
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

# ── Detect platform and architecture ─────────────────────────
OS="$(uname -s)"
case "$OS" in
    Linux*)  PLATFORM="linux";  VEC0_EXT="so"    ;;
    Darwin*) PLATFORM="macos";  VEC0_EXT="dylib"  ;;
    *)       fail "Unsupported OS: $OS" ;;
esac

ARCH="$(detect_arch)"
info "platform: $PLATFORM  arch: $ARCH"

# ── Project root ──────────────────────────────────────────────
PROJECT_ROOT="$SCRIPT_DIR"
[[ -f "$PROJECT_ROOT/Cargo.toml" ]] || \
    fail "Run from the GeliShell project root (where Cargo.toml lives)"
info "project root: $PROJECT_ROOT"

# ── Pre-flight: require pre-compiled binaries ─────────────────
# This installer copies pre-built binaries — it never invokes cargo.
for BIN in geli gerisabet; do
    [[ -f "$PROJECT_ROOT/target/release/$BIN" ]] || \
        fail "Binary not found: target/release/$BIN"$'\n'"       Run first: cargo build --release"
done

# ── Resolve paths ─────────────────────────────────────────────
HOME_DIR="${HOME:-}"
[[ -z "$HOME_DIR" ]] && fail "\$HOME is not set"
[[ -z "$BIN_DIR" ]] && BIN_DIR="$HOME_DIR/.local/bin"

CONFIG_ROOT="$HOME_DIR/.config/geliShell"
MODELS_DIR="$CONFIG_ROOT/models"
DOCS_DIR="$CONFIG_ROOT/docs"
VEC0_DEST="$MODELS_DIR/vec0.${VEC0_EXT}"
DOCS_DB_DEST="$DOCS_DIR/docs.db"

# ── Create directories (idempotent) ───────────────────────────
for DIR in "$BIN_DIR" "$CONFIG_ROOT" "$MODELS_DIR" "$DOCS_DIR"; do
    mkdir -p "$DIR"
done

# ── sqlite-vec downloader ─────────────────────────────────────
# Defined here so VEC0_EXT, VEC0_DEST, PLATFORM, ARCH are in scope.
# Returns 0 on success, 1 on any recoverable failure (warns, does not abort).
download_vec0() {
    info "fetching latest release info from GitHub API..."

    local api_url="https://api.github.com/repos/asg017/sqlite-vec/releases/latest"
    local release_json
    if ! release_json="$(download_text "$api_url" 15)"; then
        warn "GitHub API request failed — cannot auto-download sqlite-vec"
        return 1
    fi

    local tag
    tag="$(echo "$release_json" | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4)"
    [[ -z "$tag" ]] && { warn "Could not parse release tag from GitHub API response"; return 1; }
    info "latest sqlite-vec release: $tag"

    # Map platform+arch to the upstream asset name fragment
    local asset_pattern
    case "$PLATFORM-$ARCH" in
        macos-aarch64) asset_pattern="loadable-macos-aarch64" ;;
        macos-x86_64)  asset_pattern="loadable-macos-x86_64"  ;;
        linux-x86_64)  asset_pattern="loadable-linux-x86_64"  ;;
        linux-aarch64) asset_pattern="loadable-linux-aarch64" ;;
        *)
            warn "No sqlite-vec loadable asset for ${PLATFORM}/${ARCH}"
            return 1 ;;
    esac
    info "looking for asset: *${asset_pattern}*.tar.gz"

    local download_url
    download_url="$(echo "$release_json" | \
        grep -o '"browser_download_url":"[^"]*"' | cut -d'"' -f4 | \
        grep "$asset_pattern" | grep '\.tar\.gz$' | head -1)"

    if [[ -z "$download_url" ]]; then
        warn "No matching asset found for ${PLATFORM}/${ARCH} in release $tag"
        info "Available assets:"
        echo "$release_json" | grep -o '"browser_download_url":"[^"]*"' | \
            cut -d'"' -f4 | sed 's|.*/||' | grep '^sqlite-vec' | head -20
        info ""
        info "Manual install: https://github.com/asg017/sqlite-vec/releases/tag/$tag"
        info "Copy vec0.${VEC0_EXT} to: $VEC0_DEST"
        return 1
    fi

    local asset_file
    asset_file="$(basename "$download_url")"
    local tmp_archive="/tmp/sqlite-vec-${tag}-${asset_file}"
    local tmp_dir="/tmp/sqlite-vec-extract-${tag}"

    # ── Fetch checksums.txt from the same release ─────────────
    # sqlite-vec publishes a checksums.txt alongside every release.
    local checksums_url="https://github.com/asg017/sqlite-vec/releases/download/${tag}/checksums.txt"
    local expected_hash=""
    local checksums_raw
    if checksums_raw="$(download_text "$checksums_url" 15)"; then
        expected_hash="$(echo "$checksums_raw" | grep -F "$asset_file" | awk '{print $1}' | head -1)"
        if [[ -n "$expected_hash" ]]; then
            info "found SHA-256 for $asset_file in checksums.txt"
        else
            warn "asset not found in checksums.txt — SHA-256 verification will be skipped"
        fi
    else
        warn "could not fetch checksums.txt — SHA-256 verification will be skipped"
    fi

    # ── Download archive ──────────────────────────────────────
    info "downloading: $asset_file ..."
    if ! download_file "$download_url" "$tmp_archive" 120; then
        warn "Download failed"
        rm -f "$tmp_archive" || true
        return 1
    fi

    # ── Verify SHA-256 (skipped if expected_hash is empty) ────
    verify_sha256 "$tmp_archive" "$expected_hash"

    # ── Extract ───────────────────────────────────────────────
    rm -rf "$tmp_dir"
    mkdir -p "$tmp_dir"
    if ! tar -xzf "$tmp_archive" -C "$tmp_dir"; then
        warn "Extraction failed"
        rm -rf "$tmp_archive" "$tmp_dir" || true
        return 1
    fi

    local found_vec0
    found_vec0="$(find "$tmp_dir" -name "vec0.${VEC0_EXT}" | head -1)"
    [[ -z "$found_vec0" ]] && \
        found_vec0="$(find "$tmp_dir" -name "*.${VEC0_EXT}" | head -1)"

    if [[ -z "$found_vec0" ]]; then
        warn "vec0.${VEC0_EXT} not found in downloaded archive"
        info "Archive contents:"
        find "$tmp_dir" | head -20
        rm -rf "$tmp_archive" "$tmp_dir" || true
        return 1
    fi

    cp -f "$found_vec0" "$VEC0_DEST"
    register_rollback "$VEC0_DEST"
    rm -rf "$tmp_archive" "$tmp_dir" || true
    ok "vec0.${VEC0_EXT} installed at: $VEC0_DEST"
    return 0
}

# ══════════════════════════════════════════════════════════════
# STEP 1 — geli + gerisabet binaries
# ══════════════════════════════════════════════════════════════
echo ""
step "installing GeliShell binaries..."

for BINARY in geli gerisabet; do
    SRC="$PROJECT_ROOT/target/release/$BINARY"
    DST="$BIN_DIR/$BINARY"
    cp -f "$SRC" "$DST"
    chmod +x "$DST"
    register_rollback "$DST"
    ok "$BINARY -> $DST"
done

GELI_DEST="$BIN_DIR/geli"

# PATH injection (idempotent — grep before append, never duplicates)
_add_to_rc() {
    local rc_file="$1"
    [[ -f "$rc_file" ]] || return 0
    grep -qF "$BIN_DIR" "$rc_file" 2>/dev/null && return 0
    printf '\n# GeliShell\nexport PATH="$PATH:%s"\n' "$BIN_DIR" >> "$rc_file"
    ok "PATH added to $rc_file"
}

if echo "$PATH" | tr ':' '\n' | grep -qxF "$BIN_DIR"; then
    info "$BIN_DIR already in PATH"
else
    _add_to_rc "$HOME_DIR/.bashrc"
    _add_to_rc "$HOME_DIR/.bash_profile"
    _add_to_rc "$HOME_DIR/.zshrc"
    _add_to_rc "$HOME_DIR/.profile"

    # fish — uses fish_add_path (idempotent by design in fish 3.2+)
    FISH_RC="$HOME_DIR/.config/fish/config.fish"
    if [[ -f "$FISH_RC" ]] && ! grep -qF "$BIN_DIR" "$FISH_RC" 2>/dev/null; then
        printf '\n# GeliShell\nfish_add_path "%s"\n' "$BIN_DIR" >> "$FISH_RC"
        ok "PATH added to $FISH_RC"
    fi

    export PATH="$PATH:$BIN_DIR"
    warn "Restart your terminal (or source your rc file) for PATH to take effect"
fi

# ══════════════════════════════════════════════════════════════
# STEP 2 — docs.db  (pre-generated release artifact; no cargo)
# ══════════════════════════════════════════════════════════════
echo ""
step "seeding docs.db (RAG knowledge base)..."

DOCS_DB_OK=false

if [[ -f "$DOCS_DB_DEST" ]] && ! $FORCE; then
    ok "docs.db already present: $DOCS_DB_DEST"
    DOCS_DB_OK=true
elif $SKIP_DOCS; then
    info "skipping docs.db seeding (--skip-docs)"
else
    for CANDIDATE in \
        "$PROJECT_ROOT/assets/docs.db" \
        "$PROJECT_ROOT/docs.db" \
        "$PROJECT_ROOT/docs/docs.db"
    do
        if [[ -f "$CANDIDATE" ]]; then
            cp -f "$CANDIDATE" "$DOCS_DB_DEST"
            register_rollback "$DOCS_DB_DEST"
            ok "docs.db seeded from: $CANDIDATE"
            DOCS_DB_OK=true
            break
        fi
    done

    if ! $DOCS_DB_OK; then
        warn "docs.db not found in release assets."
        info "The AI assistant will not work until docs.db is distributed."
        info "Expected location : $PROJECT_ROOT/assets/docs.db"
        info "Developer build   : cargo run --bin build_docs_db"
    fi
fi

# ══════════════════════════════════════════════════════════════
# STEP 3 — SQLite (sanity check only — GeliShell does not install SQLite)
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
    info "SQLite is a runtime dependency for the GeliShell AI assistant."
    echo ""

    if ask_yes_no "Install SQLite now?"; then
        SQLITE_INSTALLED=false

        if [[ "$PLATFORM" == "macos" ]] && command -v brew &>/dev/null; then
            info "trying: brew install sqlite ..."
            if brew install sqlite; then
                SQLITE_INSTALLED=true; SQLITE_OK=true
                ok "SQLite installed via Homebrew"
            else
                warn "brew install failed"
            fi
        fi

        if [[ "$PLATFORM" == "linux" ]] && ! $SQLITE_INSTALLED; then
            if command -v apt-get &>/dev/null; then
                info "trying: apt-get install sqlite3 ..."
                if sudo apt-get install -y sqlite3; then
                    SQLITE_INSTALLED=true; SQLITE_OK=true
                    ok "SQLite installed via apt"
                else
                    warn "apt-get failed"
                fi
            elif command -v dnf &>/dev/null; then
                info "trying: dnf install sqlite ..."
                if sudo dnf install -y sqlite; then
                    SQLITE_INSTALLED=true; SQLITE_OK=true
                    ok "SQLite installed via dnf"
                else
                    warn "dnf failed"
                fi
            elif command -v pacman &>/dev/null; then
                info "trying: pacman -S sqlite ..."
                if sudo pacman -S --noconfirm sqlite; then
                    SQLITE_INSTALLED=true; SQLITE_OK=true
                    ok "SQLite installed via pacman"
                else
                    warn "pacman failed"
                fi
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
# STEP 4 — sqlite-vec
#
# sqlite-vec is a SEPARATE project from SQLite.
# Source: https://github.com/asg017/sqlite-vec
#
# GeliShell loads it at runtime from:
#   ~/.config/geliShell/models/vec0.{so|dylib}
# ══════════════════════════════════════════════════════════════
echo ""
step "checking sqlite-vec extension (vec0.${VEC0_EXT})..."
info "sqlite-vec is NOT part of SQLite — separate vector-search extension"
info "source: https://github.com/asg017/sqlite-vec"

VEC0_AVAILABLE=false

if [[ -f "$VEC0_DEST" ]] && ! $FORCE; then
    ok "vec0.${VEC0_EXT} already present: $VEC0_DEST"
    VEC0_AVAILABLE=true
fi

if ! $VEC0_AVAILABLE; then
    for CANDIDATE in \
        "$PROJECT_ROOT/assets/vec0.${VEC0_EXT}" \
        "$PROJECT_ROOT/models/vec0.${VEC0_EXT}" \
        "$PROJECT_ROOT/vec0.${VEC0_EXT}"
    do
        if [[ -f "$CANDIDATE" ]]; then
            cp -f "$CANDIDATE" "$VEC0_DEST"
            register_rollback "$VEC0_DEST"
            ok "vec0.${VEC0_EXT} found locally -> copied from: $CANDIDATE"
            VEC0_AVAILABLE=true
            break
        fi
    done
fi

if ! $VEC0_AVAILABLE; then
    echo ""
    warn "vec0.${VEC0_EXT} not found locally."
    info "GeliShell needs vec0.${VEC0_EXT} for the AI assistant RAG engine."
    info "It will be placed at: $VEC0_DEST"
    echo ""

    if ask_yes_no "Download vec0.${VEC0_EXT} from github.com/asg017/sqlite-vec now?"; then
        if download_vec0; then
            VEC0_AVAILABLE=true
        fi
    else
        info "Skipped. Install manually:"
        info "  1. https://github.com/asg017/sqlite-vec/releases"
        case "$PLATFORM-$ARCH" in
            linux-x86_64)  info "  2. Download: sqlite-vec-*-loadable-linux-x86_64.tar.gz" ;;
            linux-aarch64) info "  2. Download: sqlite-vec-*-loadable-linux-aarch64.tar.gz" ;;
            macos-aarch64) info "  2. Download: sqlite-vec-*-loadable-macos-aarch64.tar.gz" ;;
            macos-x86_64)  info "  2. Download: sqlite-vec-*-loadable-macos-x86_64.tar.gz" ;;
        esac
        info "  3. Extract vec0.${VEC0_EXT} and copy to:"
        info "     $VEC0_DEST"
    fi
fi

# ══════════════════════════════════════════════════════════════
# STEP 5 — Post-installation verification
# ══════════════════════════════════════════════════════════════
echo ""
step "verifying installation..."

GELI_VERSION_OK=false
if "$GELI_DEST" --version &>/dev/null; then
    GELI_VER="$("$GELI_DEST" --version 2>&1 | head -1)"
    ok "geli --version: $GELI_VER"
    GELI_VERSION_OK=true
else
    warn "geli --version failed — binary may need additional system libraries"
    info "Try running: $GELI_DEST --version"
fi

# ══════════════════════════════════════════════════════════════
# SUMMARY
# ══════════════════════════════════════════════════════════════
_status_line() {
    local ok_flag="$1" label="$2" detail="$3"
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
_status_line "true"            "geli"       "$GELI_DEST"
_status_line "true"            "gerisabet"  "$BIN_DIR/gerisabet"
_status_line "$SQLITE_OK"      "SQLite"     "sqlite3 in PATH"
_status_line "$VEC0_AVAILABLE" "sqlite-vec" "vec0.${VEC0_EXT} — $VEC0_DEST"
_status_line "$DOCS_DB_OK"     "docs.db"    "$DOCS_DB_DEST"
echo ""

if $VEC0_AVAILABLE && $DOCS_DB_OK; then
    echo -e "  ${GREEN}All components ready.${RESET}"
else
    echo -e "  ${GREEN}GeliShell core is installed and fully functional.${RESET}"
    echo -e "  ${YELLOW}AI assistant features require the missing components above.${RESET}"
fi

echo ""
echo -e "  ${CYAN}Open a new terminal and run: geli${RESET}"
echo ""