#!/usr/bin/env bash
set -euo pipefail

# Build an AppImage for LinVClipBoard.
# Requires: appimagetool (https://github.com/AppImage/appimagetool)

VERSION=$(grep '^version' "$(dirname "$0")/../Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
ARCH="x86_64"
APP_NAME="LinVClipBoard"
APP_DIR="$(mktemp -d)/AppDir"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="${PROJECT_DIR}/target/release"
OUT="${PROJECT_DIR}/target/${APP_NAME}-${VERSION}-${ARCH}.AppImage"

trap 'rm -rf "$(dirname "$APP_DIR")"' EXIT

echo "==> Building AppImage ${APP_NAME} ${VERSION}"

for bin in clipd clipctl linvclip-ui; do
    if [ ! -f "${RELEASE_DIR}/${bin}" ]; then
        echo "ERROR: ${RELEASE_DIR}/${bin} not found. Run 'make build-all' first." >&2
        exit 1
    fi
done

# Create AppDir structure
mkdir -p "${APP_DIR}/usr/bin"
mkdir -p "${APP_DIR}/usr/share/applications"
mkdir -p "${APP_DIR}/usr/share/icons/hicolor/128x128/apps"
mkdir -p "${APP_DIR}/usr/lib/systemd/user"

# Copy binaries
install -m755 "${RELEASE_DIR}/clipd"        "${APP_DIR}/usr/bin/"
install -m755 "${RELEASE_DIR}/clipctl"      "${APP_DIR}/usr/bin/"
install -m755 "${RELEASE_DIR}/linvclip-ui"  "${APP_DIR}/usr/bin/"

# Desktop entry
install -m644 "${PROJECT_DIR}/install/linvclipboard.desktop" \
    "${APP_DIR}/usr/share/applications/"
install -m644 "${PROJECT_DIR}/crates/linvclip-ui/src-tauri/icons/icon.png" \
    "${APP_DIR}/usr/share/icons/hicolor/128x128/apps/linvclipboard.png"

# Systemd service
install -m644 "${PROJECT_DIR}/install/clipd.service" \
    "${APP_DIR}/usr/lib/systemd/user/"

# AppDir top-level symlinks
ln -sf usr/share/applications/linvclipboard.desktop "${APP_DIR}/linvclipboard.desktop"
ln -sf usr/share/icons/hicolor/128x128/apps/linvclipboard.png "${APP_DIR}/linvclipboard.png"

# AppRun entry point
cat > "${APP_DIR}/AppRun" << 'APPRUN'
#!/bin/bash
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="${HERE}/usr/bin:$PATH"
exec linvclip-ui "$@"
APPRUN
chmod +x "${APP_DIR}/AppRun"

# Build the AppImage
if ! command -v appimagetool &>/dev/null; then
    echo "ERROR: appimagetool not found. Install from https://github.com/AppImage/appimagetool" >&2
    exit 1
fi

mkdir -p "$(dirname "$OUT")"
ARCH="${ARCH}" appimagetool "${APP_DIR}" "${OUT}"

echo "==> Created: ${OUT}"
echo "    Size: $(du -h "${OUT}" | cut -f1)"
