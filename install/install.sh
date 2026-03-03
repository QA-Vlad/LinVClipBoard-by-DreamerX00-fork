#!/usr/bin/env bash
set -euo pipefail

# LinVClipBoard Installer
# Installs clipd (daemon), clipctl (CLI), and linvclip-ui (overlay) as a native app.

INSTALL_DIR="${HOME}/.local/bin"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"
DESKTOP_DIR="${HOME}/.local/share/applications"
SYSTEMD_DIR="${HOME}/.config/systemd/user"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🔧 LinVClipBoard Installer"
echo "=========================="

# Ensure directories exist
mkdir -p "$INSTALL_DIR" "$ICON_DIR" "$DESKTOP_DIR" "$SYSTEMD_DIR"

# Build release binaries if not already built
if [ ! -f "$PROJECT_DIR/target/release/clipd" ] || \
   [ ! -f "$PROJECT_DIR/target/release/clipctl" ] || \
   [ ! -f "$PROJECT_DIR/target/release/linvclip-ui" ]; then
    echo "📦 Building release binaries..."
    cd "$PROJECT_DIR"
    cargo build --release --workspace
    cd "$PROJECT_DIR/crates/linvclip-ui"
    npx tauri build
fi

# Copy binaries
echo "📋 Installing binaries to $INSTALL_DIR..."
cp "$PROJECT_DIR/target/release/clipd" "$INSTALL_DIR/"
cp "$PROJECT_DIR/target/release/clipctl" "$INSTALL_DIR/"
cp "$PROJECT_DIR/target/release/linvclip-ui" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/clipd" "$INSTALL_DIR/clipctl" "$INSTALL_DIR/linvclip-ui"

# Copy icon
echo "🎨 Installing icon..."
cp "$SCRIPT_DIR/../crates/linvclip-ui/src-tauri/icons/icon.png" "$ICON_DIR/linvclipboard.png"

# Install .desktop file
echo "🖥️  Installing desktop entry..."
cp "$SCRIPT_DIR/linvclipboard.desktop" "$DESKTOP_DIR/"
# Update the desktop database
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

# Install systemd service
echo "⚙️  Installing systemd user service..."
cat > "$SYSTEMD_DIR/clipd.service" << EOF
[Unit]
Description=LinVClipBoard Clipboard Daemon
Documentation=https://github.com/LinVClipBoard
After=graphical-session.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/clipd
Restart=on-failure
RestartSec=3
MemoryMax=50M
CPUQuota=5%
Environment=RUST_LOG=info

[Install]
WantedBy=default.target
EOF

# Enable and start the daemon
systemctl --user daemon-reload
systemctl --user enable clipd.service
systemctl --user start clipd.service
systemctl --user restart clipd.service
systemctl --user status clipd.service --no-pager
systemctl --user is-active --quiet clipd.service && echo "✅ clipd service is active and running." || echo "⚠️  clipd service failed to start."

echo ""
echo "✅ LinVClipBoard installed successfully!"
echo ""
echo "📍 Binaries:    $INSTALL_DIR/{clipd,clipctl,linvclip-ui}"
echo "📍 Desktop:     $DESKTOP_DIR/linvclipboard.desktop"
echo "📍 Service:     $SYSTEMD_DIR/clipd.service"
echo ""
echo "🚀 Usage:"
echo "   • The clipboard daemon (clipd) is now running as a systemd service"
echo "   • Launch the overlay UI from your app menu or run: linvclip-ui"
echo "   • Use Super+V to toggle the overlay"
echo "   • CLI: clipctl list | clipctl search <query> | clipctl status"
echo ""
echo "📝 To check daemon status: systemctl --user status clipd"
echo "📝 To view logs: journalctl --user -u clipd -f"
echo ""

# Check for typing tools needed for emoji/symbol insertion
if ! command -v wtype >/dev/null 2>&1 && ! command -v xdotool >/dev/null 2>&1 && ! command -v ydotool >/dev/null 2>&1; then
    echo "💡 Optional: For emoji/symbol insertion into text fields (like the"
    echo "   Windows emoji panel), install a typing tool:"
    if [ "${XDG_SESSION_TYPE:-}" = "wayland" ]; then
        echo "     sudo apt install wtype    # Wayland (recommended)"
    else
        echo "     sudo apt install xdotool  # X11 (recommended)"
    fi
    echo "   Without it, emojis will be copied to clipboard instead."
    echo ""
fi

echo "Make sure $INSTALL_DIR is in your PATH. Add this to ~/.bashrc if needed:"
echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
