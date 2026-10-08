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
    ICON_DIR="/usr/local/share/icons/hicolor/scalable/apps"
    META_DIR="/usr/local/share/metainfo"
else
    BIN_DIR="${HOME}/.local/bin"
    APP_DIR="${HOME}/.local/share/applications"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
    META_DIR="${HOME}/.local/share/metainfo"
fi

if [ "$1" = "--uninstall" ] || [ "$1" = "uninstall" ]; then
    info "Uninstalling Glance..."
    rm -f "${BIN_DIR}/glance" "${BIN_DIR}/glance-tree"
    rm -f "${APP_DIR}/io.github.maycon.Glance.desktop"
    rm -f "${ICON_DIR}/io.github.maycon.Glance.svg" "${ICON_DIR}/glance.svg"
    rm -f "${META_DIR}/io.github.maycon.Glance.metainfo.xml"
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -f -t "${ICON_DIR%/*/*}" >/dev/null 2>&1 || true
    fi
    success "Glance has been uninstalled successfully from ${BIN_DIR}."
    exit 0
fi

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR" "$META_DIR"

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
    # Ensure desktop entry and icon are present (download from main branch if missing from older release archive)
    if [ ! -f "${TMP_DIR}/io.github.maycon.Glance.desktop" ]; then
        curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/data/io.github.maycon.Glance.desktop" -o "${TMP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
    fi
    if [ ! -f "${TMP_DIR}/io.github.maycon.Glance.svg" ]; then
        curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/data/icons/hicolor/scalable/apps/io.github.maycon.Glance.svg" -o "${TMP_DIR}/io.github.maycon.Glance.svg" 2>/dev/null || true
    fi

    if [ -f "${TMP_DIR}/io.github.maycon.Glance.desktop" ]; then
        install -m 644 "${TMP_DIR}/io.github.maycon.Glance.desktop" "${APP_DIR}/io.github.maycon.Glance.desktop"
        sed -i "s|^Exec=.*|Exec=${BIN_DIR}/glance|" "${APP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
        sed -i "s|^Icon=.*|Icon=io.github.maycon.Glance|" "${APP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
        sed -i "s|^StartupWMClass=.*|StartupWMClass=io.github.maycon.Glance|" "${APP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
    fi
    if [ -f "${TMP_DIR}/io.github.maycon.Glance.svg" ]; then
        install -m 644 "${TMP_DIR}/io.github.maycon.Glance.svg" "${ICON_DIR}/io.github.maycon.Glance.svg"
        cp -f "${ICON_DIR}/io.github.maycon.Glance.svg" "${ICON_DIR}/glance.svg"
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
        sed -i "s|^Exec=.*|Exec=${BIN_DIR}/glance|" "${APP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
        sed -i "s|^Icon=.*|Icon=io.github.maycon.Glance|" "${APP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
        sed -i "s|^StartupWMClass=.*|StartupWMClass=io.github.maycon.Glance|" "${APP_DIR}/io.github.maycon.Glance.desktop" 2>/dev/null || true
        if [ -f "${TMP_DIR}/repo/data/icons/hicolor/scalable/apps/io.github.maycon.Glance.svg" ]; then
            install -m 644 "${TMP_DIR}/repo/data/icons/hicolor/scalable/apps/io.github.maycon.Glance.svg" "${ICON_DIR}/io.github.maycon.Glance.svg"
            cp -f "${ICON_DIR}/io.github.maycon.Glance.svg" "${ICON_DIR}/glance.svg"
        fi
        install -m 644 "${TMP_DIR}/repo/data/io.github.maycon.Glance.metainfo.xml" "${META_DIR}/io.github.maycon.Glance.metainfo.xml"
    else
        error "Could not download ${DOWNLOAD_URL} and Cargo is not installed to compile from source."
    fi
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -f -t "${ICON_DIR%/*/*}" >/dev/null 2>&1 || true
fi

success "Glance successfully installed!"

# Anonymous installation telemetry ping (respects DO_NOT_TRACK=1 and GLANCE_NO_TELEMETRY=1)
if [ -z "$DO_NOT_TRACK" ] && [ -z "$GLANCE_NO_TELEMETRY" ]; then
    if command -v curl >/dev/null 2>&1; then
        (curl -fsSL -X POST "https://www.maycondouglas.work/api/glance-telemetry" \
            -H "Content-Type: application/json" \
            -d "{\"version\":\"${LATEST_TAG}\",\"arch\":\"${TARGET_ARCH}\",\"os\":\"${OS}\",\"method\":\"script\"}" \
            --max-time 3 >/dev/null 2>&1 || true) &
    elif command -v wget >/dev/null 2>&1; then
        (wget -q -O- --post-data="{\"version\":\"${LATEST_TAG}\",\"arch\":\"${TARGET_ARCH}\",\"os\":\"${OS}\",\"method\":\"script\"}" \
            --header="Content-Type: application/json" \
            --timeout=3 "https://www.maycondouglas.work/api/glance-telemetry" >/dev/null 2>&1 || true) &
    fi
fi

printf "\n${GREEN}==>${NC} ${BOLD}Launch from your Desktop:${NC}\n"
printf "  • Press ${BOLD}Super${NC} (Windows key) and type ${BOLD}Glance${NC} to launch\n"
printf "  • Or click Glance in your application menu / dock\n"
printf "  • Right-click the Glance dock icon to ${BOLD}Pin to Dash / Add to Favorites${NC}\n"

case ":$PATH:" in
    *":${BIN_DIR}:"*)
        ;;
    *)
        printf "\n${BLUE}Note:${NC} ${BIN_DIR} is not in your terminal PATH.\n"
        printf "To also run from terminal, add it to your ~/.bashrc or ~/.zshrc:\n"
        printf "  export PATH=\"%s:\$PATH\"\n" "$BIN_DIR"
        ;;
esac

printf "\n${BLUE}==>${NC} Terminal launch (optional):\n"
printf "  • Run ${BOLD}glance${NC}      (GUI monitor)\n"
printf "  • Run ${BOLD}glance-tree${NC} (terminal tree view)\n\n"
