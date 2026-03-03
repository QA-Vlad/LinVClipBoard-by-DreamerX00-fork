#!/usr/bin/env bash
# LinVClipBoard — weekly update checker (invoked by systemd timer).
# Fetches the latest GitHub release tag and sends a desktop notification
# if a newer version is available.

set -euo pipefail

REPO="DreamerX00/LinVClipBoard"
CURRENT=$(/usr/bin/linvclip-ui --version 2>/dev/null | grep -oP '[\d.]+' || echo "0.0.0")

# Fallback: read from dpkg
if [ "$CURRENT" = "0.0.0" ]; then
    CURRENT=$(dpkg-query -W -f='${Version}' linvclipboard 2>/dev/null | sed 's/-.*//' || echo "0.0.0")
fi

LATEST=$(curl -sf "https://api.github.com/repos/${REPO}/releases/latest" \
    -H "Accept: application/vnd.github+json" \
    | grep -oP '"tag_name"\s*:\s*"v?\K[^"]+' || echo "")

[ -z "$LATEST" ] && exit 0   # network failed, silently skip

# Compare versions numerically
ver_gt() {
    local IFS=.
    local i a=($1) b=($2)
    for ((i=0; i<${#a[@]} || i<${#b[@]}; i++)); do
        local x=${a[i]:-0} y=${b[i]:-0}
        ((x > y)) && return 0
        ((x < y)) && return 1
    done
    return 1
}

if ver_gt "$LATEST" "$CURRENT"; then
    notify-send -i linvclipboard \
        "LinVClipBoard Update Available" \
        "Version ${LATEST} is available (you have ${CURRENT}).\nVisit https://github.com/${REPO}/releases to download." \
        2>/dev/null || true
fi
