#!/usr/bin/env bash
set -euo pipefail

# Build a single .deb package containing clipd, clipctl, and linvclip-ui.

VERSION="1.0.1"
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
cat > "${PKG_DIR}/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e

# Update icon cache and desktop database
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi

# Free up Super+. from IBus emoji picker so LinVClipBoard can use it.
# Save the original value so we can restore on uninstall.
free_shortcut() {
    USER="$1"
    HOME_DIR="$2"
    # Run gsettings as the actual user, not root
    ORIG=$(su - "$USER" -c "gsettings get org.freedesktop.ibus.panel.emoji hotkey" 2>/dev/null) || return 0
    # Save original if not already saved
    SAVE_FILE="${HOME_DIR}/.config/linvclip/.ibus-emoji-hotkey-backup"
    if [ ! -f "$SAVE_FILE" ]; then
        mkdir -p "${HOME_DIR}/.config/linvclip"
        echo "$ORIG" > "$SAVE_FILE"
        chown "$USER":"$USER" "$SAVE_FILE"
    fi
    su - "$USER" -c "gsettings set org.freedesktop.ibus.panel.emoji hotkey '[]'" 2>/dev/null || true
}

# Apply to all logged-in users with a graphical session, or SUDO_USER
if [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    SUDO_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    free_shortcut "$SUDO_USER" "$SUDO_HOME"
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
POSTINST
chmod 755 "${PKG_DIR}/DEBIAN/postinst"

# DEBIAN/prerm
cat > "${PKG_DIR}/DEBIAN/prerm" <<'PRERM'
#!/bin/sh
set -e

echo "Stopping clipd service (if running)..."
systemctl --user stop clipd.service 2>/dev/null || true
systemctl --user disable clipd.service 2>/dev/null || true

# Kill linvclip-ui if running
pkill linvclip-ui 2>/dev/null || true
PRERM
chmod 755 "${PKG_DIR}/DEBIAN/prerm"

# DEBIAN/postrm
cat > "${PKG_DIR}/DEBIAN/postrm" <<'POSTRM'
#!/bin/sh
set -e

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi

# Restore the original IBus emoji hotkey if we saved a backup
restore_shortcut() {
    USER="$1"
    HOME_DIR="$2"
    SAVE_FILE="${HOME_DIR}/.config/linvclip/.ibus-emoji-hotkey-backup"
    if [ -f "$SAVE_FILE" ]; then
        ORIG=$(cat "$SAVE_FILE")
        su - "$USER" -c "gsettings set org.freedesktop.ibus.panel.emoji hotkey \"$ORIG\"" 2>/dev/null || true
        rm -f "$SAVE_FILE"
    fi
}

if [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
    SUDO_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    restore_shortcut "$SUDO_USER" "$SUDO_HOME"
fi
POSTRM
chmod 755 "${PKG_DIR}/DEBIAN/postrm"

# Build the .deb
mkdir -p "$(dirname "$OUT")"
dpkg-deb --build --root-owner-group "${PKG_DIR}" "${OUT}"

echo "==> Created: ${OUT}"
echo "    Size: $(du -h "${OUT}" | cut -f1)"
echo ""
echo "Install with: sudo dpkg -i ${OUT}"
