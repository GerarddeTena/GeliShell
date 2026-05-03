#!/usr/bin/env bash
# installer/lib/common.sh — Shared functions for GeliShell install scripts.
#
# Source this file from install.sh — do NOT execute it directly.
# Requires: bash 4.0+  (bash 3.2 on macOS is NOT supported)
#
# Provides:
#   Logging   : step, ok, warn, info, fail
#   Prompts   : ask_yes_no
#   Rollback  : register_rollback, do_rollback   (ROLLBACK_FILES array)
#   Checksums : verify_sha256
#   Downloads : download_file, download_text
#   Platform  : detect_arch

# ── Colors (disabled when stdout is not a terminal) ───────────
if [[ -t 1 ]]; then
    CYAN='\033[0;36m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    RED='\033[0;31m'
    GRAY='\033[0;90m'
    MAGENTA='\033[0;35m'
    RESET='\033[0m'
else
    CYAN=''
    GREEN=''
    YELLOW=''
    RED=''
    GRAY=''
    MAGENTA=''
    RESET=''
fi

# ── Logging ───────────────────────────────────────────────────
step() { echo -e "  ${CYAN}-->${RESET} $1"; }
ok()   { echo -e "   ${GREEN}[OK]${RESET} $1"; }
warn() { echo -e " ${YELLOW}[WARN]${RESET} $1"; }
info() { echo -e "       ${GRAY}$1${RESET}"; }

# fail <message>
# Prints the error, triggers rollback, then exits 1.
fail() {
    echo -e "  ${RED}[ERROR]${RESET} $1" >&2
    do_rollback
    exit 1
}

# ── Interactive prompt ────────────────────────────────────────
# ask_yes_no <question> [default: y|n]
# Returns 0 (yes) or 1 (no).
ask_yes_no() {
    local question="$1"
    local default="${2:-y}"
    local hint
    hint="$([[ "$default" == "y" ]] && echo "[Y/n]" || echo "[y/N]")"
    echo -ne "  ${CYAN}${question} ${hint}${RESET} "
    local answer
    read -r answer
    answer="${answer:-$default}"
    [[ "${answer,,}" == "y" || "${answer,,}" == "yes" ]]
}

# ── Rollback tracker ─────────────────────────────────────────
# Files registered here are deleted by do_rollback().
# Register every file copied during installation with register_rollback.
ROLLBACK_FILES=()
ROLLBACK_RESTORE_SRC=()
ROLLBACK_RESTORE_DST=()
_ROLLBACK_DONE=false

# register_rollback <file_path>
register_rollback() {
    ROLLBACK_FILES+=("$1")
}

# register_restore <backup_path> <destination_path>
register_restore() {
    ROLLBACK_RESTORE_SRC+=("$1")
    ROLLBACK_RESTORE_DST+=("$2")
}

# do_rollback
# Idempotent — safe to call multiple times (only acts once).
do_rollback() {
    if $_ROLLBACK_DONE; then return 0; fi
    _ROLLBACK_DONE=true
    [[ ${#ROLLBACK_FILES[@]} -eq 0 && ${#ROLLBACK_RESTORE_SRC[@]} -eq 0 ]] && return 0
    warn "Rolling back installation..."
    for f in "${ROLLBACK_FILES[@]}"; do
        if [[ -f "$f" ]]; then
            rm -f "$f" || true   # never let rollback itself fail
            info "removed: $f"
        fi
    done
    local i
    for ((i = 0; i < ${#ROLLBACK_RESTORE_SRC[@]}; i++)); do
        local src="${ROLLBACK_RESTORE_SRC[$i]}"
        local dst="${ROLLBACK_RESTORE_DST[$i]}"
        if [[ -f "$src" ]]; then
            cp -f "$src" "$dst" || true
            info "restored: $dst"
        fi
    done
}

# ── SHA-256 verification ──────────────────────────────────────
# verify_sha256 <file> <expected_hash>
#
# If expected_hash is empty the function prints a warning and returns 0
# (skip mode).  This happens when checksums.txt could not be fetched.
#
# TODO: populate expected_hash from the release pipeline so the empty-string
#       path is never taken in production.
verify_sha256() {
    local file="$1"
    local expected="$2"

    if [[ -z "$expected" ]]; then
        warn "SHA-256 checksum not available — skipping verification"
        # TODO: populate from release pipeline
        return 0
    fi

    local actual
    if command -v sha256sum &>/dev/null; then
        actual="$(sha256sum "$file" | awk '{print $1}')"
    elif command -v shasum &>/dev/null; then
        actual="$(shasum -a 256 "$file" | awk '{print $1}')"
    else
        warn "No SHA-256 tool found (sha256sum / shasum) — skipping verification"
        return 0
    fi

    # Both sha256sum and shasum emit lowercase; expected from checksums.txt is
    # also lowercase, so a direct comparison is safe.
    if [[ "$actual" == "$expected" ]]; then
        ok "SHA-256 verified: $(basename "$file")"
        return 0
    else
        # fail() calls do_rollback + exit 1
        fail "SHA-256 mismatch for $(basename "$file")"$'\n'"  expected: $expected"$'\n'"  actual:   $actual"
    fi
}

# ── Download helpers ──────────────────────────────────────────
# download_file <url> <dest_file> [timeout_secs=120]
# Downloads a binary/archive to a file. Tries curl, then wget.
download_file() {
    local url="$1"
    local dest="$2"
    local timeout="${3:-120}"

    if command -v curl &>/dev/null; then
        curl -fSL --max-time "$timeout" --progress-bar -o "$dest" "$url"
    elif command -v wget &>/dev/null; then
        wget -q --timeout="$timeout" -O "$dest" "$url"
    else
        fail "Neither curl nor wget is available — cannot download files"
    fi
}

# download_text <url> [timeout_secs=15]
# Fetches a URL to stdout (JSON / plain text).
# Sends GitHub API headers when using curl so rate-limit headroom is wider.
download_text() {
    local url="$1"
    local timeout="${2:-15}"

    if command -v curl &>/dev/null; then
        curl -fsSL --max-time "$timeout" \
            -H "User-Agent: GeliShell-Installer/1.0" \
            -H "Accept: application/vnd.github+json" \
            "$url"
    elif command -v wget &>/dev/null; then
        wget -q --timeout="$timeout" \
            --header="User-Agent: GeliShell-Installer/1.0" \
            -O - "$url"
    else
        fail "Neither curl nor wget is available — cannot fetch remote resources"
    fi
}

# ── Architecture detection ────────────────────────────────────
# detect_arch
# Normalises uname -m to a canonical token.
# Outputs: x86_64 | aarch64 | x86 | unknown
detect_arch() {
    local raw
    raw="$(uname -m)"
    case "$raw" in
        x86_64)        echo "x86_64"  ;;
        aarch64|arm64) echo "aarch64" ;;
        i386|i686)     echo "x86"     ;;
        *)             echo "unknown" ;;
    esac
}
