#!/usr/bin/env bash
# GeliShell installer for Linux and macOS
#
# Installs:
#   1. geli + gerisabet binaries → ~/.local/bin/
#   2. docs.db (if found in assets/) → config dir
#
# NOTE: sqlite-vec and docs.db are downloaded automatically at first run
# if not found locally.  This installer only handles repo-local assets.
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
  --force | -f)
    FORCE=true
    shift
    ;;
  --skip-docs)
    SKIP_DOCS=true
    shift
    ;;
  --bin-dir)
    BIN_DIR="$2"
    shift 2
    ;;
  --bin-dir=*)
    BIN_DIR="${1#*=}"
    shift
    ;;
  -h | --help)
    sed -n '/^# Usage/,/^[^#]/{ /^#/p }' "$0" | sed 's/^# \?//'
    exit 0
    ;;
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
Linux*) PLATFORM="linux" ;;
Darwin*) PLATFORM="macos" ;;
*) fail "Unsupported OS: $OS" ;;
esac

ARCH="$(detect_arch)"
info "platform: $PLATFORM  arch: $ARCH"

# ── Project root ──────────────────────────────────────────────
PROJECT_ROOT="$SCRIPT_DIR"
[[ -f "$PROJECT_ROOT/Cargo.toml" ]] ||
  fail "Run from the GeliShell project root (where Cargo.toml lives)"
info "project root: $PROJECT_ROOT"

# ── Pre-flight: require pre-compiled binaries ─────────────────
# This installer copies pre-built binaries — it never invokes cargo.
for BIN in geli gerisabet; do
  [[ -f "$PROJECT_ROOT/target/release/$BIN" ]] ||
    fail "Binary not found: target/release/$BIN"$'\n'"       Run first: cargo build --release"
done

# ── Resolve paths ─────────────────────────────────────────────
HOME_DIR="${HOME:-}"
[[ -z "$HOME_DIR" ]] && fail "\$HOME is not set"
[[ -z "$BIN_DIR" ]] && BIN_DIR="$HOME_DIR/.local/bin"

CONFIG_ROOT="$HOME_DIR/.config/geliShell"
MODELS_DIR="$CONFIG_ROOT/models"
DOCS_DIR="$CONFIG_ROOT/docs"
DOCS_DB_DEST="$DOCS_DIR/docs.db"

# ── Create directories (idempotent) ───────────────────────────
for DIR in "$BIN_DIR" "$CONFIG_ROOT" "$MODELS_DIR" "$DOCS_DIR"; do
  mkdir -p "$DIR"
done

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
  printf '\n# GeliShell\nexport PATH="$PATH:%s"\n' "$BIN_DIR" >>"$rc_file"
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
    printf '\n# GeliShell\nfish_add_path "%s"\n' "$BIN_DIR" >>"$FISH_RC"
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
    "$PROJECT_ROOT/docs/docs.db"; do
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
# STEP 3 — Post-installation verification
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
_status_line "true" "geli" "$GELI_DEST"
_status_line "true" "gerisabet" "$BIN_DIR/gerisabet"
_status_line "$DOCS_DB_OK" "docs.db" "$DOCS_DB_DEST"
echo ""

if $DOCS_DB_OK; then
  echo -e "  ${GREEN}All components ready.${RESET}"
else
  echo -e "  ${GREEN}GeliShell core is installed and fully functional.${RESET}"
  echo -e "  ${YELLOW}sqlite-vec and docs.db are downloaded automatically at first run.${RESET}"
fi

echo ""
echo -e "  ${CYAN}Open a new terminal and run: geli${RESET}"
echo ""
