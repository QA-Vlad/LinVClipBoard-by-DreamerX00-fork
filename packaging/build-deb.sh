#!/usr/bin/env bash
set -euo pipefail

# Build a single .deb package containing clipd, clipctl, and linvclip-ui.

VERSION="1.0.0"
ARCH="amd64"
PKG_NAME="linvclipboard"
PKG_DIR="$(mktemp -d)"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="${PROJECT_DIR}/target/release"
OUT="${PROJECT_DIR}/target/debian/${PKG_NAME}_${VERSION}-1_${ARCH}.deb"

trap 'rm -rf "$PKG_DIR"' EXIT

echo "==> Packaging ${PKG_NAME} ${VERSION}"

# Verify binaries exist
for bin in clipd clipctl linvclip-ui; do
    if [ ! -f "${RELEASE_DIR}/${bin}" ]; then
        echo "ERROR: ${RELEASE_DIR}/${bin} not found. Run 'make build' and build the UI first." >&2
        exit 1
    fi
done

# Create directory structure
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/lib/systemd/user"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps"
mkdir -p "${PKG_DIR}/usr/share/doc/${PKG_NAME}"

# Copy binaries
install -Dm755 "${RELEASE_DIR}/clipd"        "${PKG_DIR}/usr/bin/clipd"
install -Dm755 "${RELEASE_DIR}/clipctl"      "${PKG_DIR}/usr/bin/clipctl"
install -Dm755 "${RELEASE_DIR}/linvclip-ui"  "${PKG_DIR}/usr/bin/linvclip-ui"

# Copy assets
install -Dm644 "${PROJECT_DIR}/install/clipd.service" \
    "${PKG_DIR}/usr/lib/systemd/user/clipd.service"
install -Dm644 "${PROJECT_DIR}/install/linvclipboard.desktop" \
    "${PKG_DIR}/usr/share/applications/linvclipboard.desktop"
install -Dm644 "${PROJECT_DIR}/crates/linvclip-ui/src-tauri/icons/icon.png" \
    "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps/linvclipboard.png"

# Copyright
cat > "${PKG_DIR}/usr/share/doc/${PKG_NAME}/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: LinVClipBoard
License: MIT
Copyright: 2025-2026 LinVClipBoard Contributors
EOF

# Calculate installed size (in KB)
INSTALLED_SIZE=$(du -sk "${PKG_DIR}" | cut -f1)

# DEBIAN/control
cat > "${PKG_DIR}/DEBIAN/control" <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}-1
Section: utils
Priority: optional
Architecture: ${ARCH}
Installed-Size: ${INSTALLED_SIZE}
Maintainer: LinVClipBoard Contributors
Description: Clipboard history manager for Linux
 A Win+V style clipboard manager that captures text and images,
 with full-text search, pinning, and a beautiful overlay UI.
 Includes clipd (daemon), clipctl (CLI), and linvclip-ui (GUI).
Homepage: https://github.com/akash-singh8/LinVClipBoard
EOF

# DEBIAN/postinst
cat > "${PKG_DIR}/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e

# Update icon cache and desktop database
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi

echo ""
echo "LinVClipBoard installed successfully!"
echo ""
echo "To start the clipboard daemon, run:"
echo "  systemctl --user daemon-reload"
echo "  systemctl --user enable --now clipd.service"
echo ""
echo "Then press Super+. (Win+Period) to open the overlay,"
echo "or use 'clipctl' from the terminal."
echo ""
EOF
chmod 755 "${PKG_DIR}/DEBIAN/postinst"

# DEBIAN/prerm
cat > "${PKG_DIR}/DEBIAN/prerm" <<'EOF'
#!/bin/sh
set -e
echo "Stopping clipd service (if running)..."
systemctl --user stop clipd.service 2>/dev/null || true
systemctl --user disable clipd.service 2>/dev/null || true
EOF
chmod 755 "${PKG_DIR}/DEBIAN/prerm"

# DEBIAN/postrm
cat > "${PKG_DIR}/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -e
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi
EOF
chmod 755 "${PKG_DIR}/DEBIAN/postrm"

# Build the .deb
mkdir -p "$(dirname "$OUT")"
dpkg-deb --build --root-owner-group "${PKG_DIR}" "${OUT}"

echo "==> Created: ${OUT}"
echo "    Size: $(du -h "${OUT}" | cut -f1)"
echo ""
echo "Install with: sudo dpkg -i ${OUT}"
