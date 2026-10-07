#!/bin/sh
# Glance Installer — Lightweight Linux Application & Process Monitor
# Usage: curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh

set -e

REPO="maycondouglascc/glance"
GITHUB_URL="https://github.com/${REPO}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

info() {
    printf "${BLUE}==>${NC} ${BOLD}%s${NC}\n" "$1"
}

success() {
    printf "${GREEN}==>${NC} ${BOLD}%s${NC}\n" "$1"
}

error() {
    printf "${RED}Error:${NC} %s\n" "$1" >&2
    exit 1
}

# 1. Check OS
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    error "Glance is only supported on Linux (detected: $OS)."
fi

# 2. Check Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        error "Unsupported architecture: $ARCH (currently x86_64 is provided)."
        ;;
esac

# 3. Determine Installation Directories
if [ "$(id -u)" -eq 0 ]; then
    BIN_DIR="/usr/local/bin"
    APP_DIR="/usr/local/share/applications"
    META_DIR="/usr/local/share/metainfo"
else
    BIN_DIR="${HOME}/.local/bin"
    APP_DIR="${HOME}/.local/share/applications"
    META_DIR="${HOME}/.local/share/metainfo"
fi

mkdir -p "$BIN_DIR" "$APP_DIR" "$META_DIR"

# 4. Check for downloader (curl or wget)
if command -v curl >/dev/null 2>&1; then
    FETCH_CMD="curl -fsSL"
elif command -v wget >/dev/null 2>&1; then
    FETCH_CMD="wget -qO-"
else
    error "Neither curl nor wget was found. Please install curl or wget first."
fi

info "Detecting latest release for Glance..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)

if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v0.1.0"
fi

TARBALL_NAME="glance-linux-${TARGET_ARCH}.tar.gz"
DOWNLOAD_URL="${GITHUB_URL}/releases/download/${LATEST_TAG}/${TARBALL_NAME}"

TMP_DIR="$(mktemp -d /tmp/glance-install-XXXXXX)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

info "Downloading Glance (${LATEST_TAG}) for ${TARGET_ARCH}..."

if curl -sLf "$DOWNLOAD_URL" -o "${TMP_DIR}/${TARBALL_NAME}" 2>/dev/null || wget -q "$DOWNLOAD_URL" -O "${TMP_DIR}/${TARBALL_NAME}" 2>/dev/null; then
    info "Extracting ${TARBALL_NAME}..."
    tar -xzf "${TMP_DIR}/${TARBALL_NAME}" -C "$TMP_DIR"

    install -m 755 "${TMP_DIR}/glance" "${BIN_DIR}/glance"
    if [ -f "${TMP_DIR}/glance-tree" ]; then
        install -m 755 "${TMP_DIR}/glance-tree" "${BIN_DIR}/glance-tree"
    fi
    if [ -f "${TMP_DIR}/io.github.maycon.Glance.desktop" ]; then
        install -m 644 "${TMP_DIR}/io.github.maycon.Glance.desktop" "${APP_DIR}/io.github.maycon.Glance.desktop"
    fi
    if [ -f "${TMP_DIR}/io.github.maycon.Glance.metainfo.xml" ]; then
        install -m 644 "${TMP_DIR}/io.github.maycon.Glance.metainfo.xml" "${META_DIR}/io.github.maycon.Glance.metainfo.xml"
    fi
else
    # Fallback: if release asset is not yet available, build from repository if cargo is installed
    if command -v cargo >/dev/null 2>&1; then
        info "Release binary not yet available on GitHub. Building latest from source via Cargo..."
        git clone --depth 1 "https://github.com/${REPO}.git" "${TMP_DIR}/repo"
        cargo build --release --manifest-path "${TMP_DIR}/repo/Cargo.toml"
        install -m 755 "${TMP_DIR}/repo/target/release/glance" "${BIN_DIR}/glance"
        install -m 755 "${TMP_DIR}/repo/target/release/glance-tree" "${BIN_DIR}/glance-tree"
        install -m 644 "${TMP_DIR}/repo/data/io.github.maycon.Glance.desktop" "${APP_DIR}/io.github.maycon.Glance.desktop"
        install -m 644 "${TMP_DIR}/repo/data/io.github.maycon.Glance.metainfo.xml" "${META_DIR}/io.github.maycon.Glance.metainfo.xml"
    else
        error "Could not download ${DOWNLOAD_URL} and Cargo is not installed to compile from source."
    fi
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
fi

success "Glance successfully installed to ${BIN_DIR}/glance"

case ":$PATH:" in
    *":${BIN_DIR}:"*)
        ;;
    *)
        printf "\n${BLUE}Note:${NC} ${BIN_DIR} is not in your PATH.\n"
        printf "Add it by running:\n"
        printf "  export PATH=\"%s:\$PATH\"\n" "$BIN_DIR"
        printf "Or add the line above to your ~/.bashrc or ~/.zshrc\n\n"
        ;;
esac

printf "Run ${BOLD}glance${NC} to start the application, or ${BOLD}glance-tree${NC} for terminal view.\n"
