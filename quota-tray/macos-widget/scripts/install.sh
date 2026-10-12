#!/bin/bash
# Builds Claude Widget, installs it to ~/Applications and launches it.
# On first launch the app registers itself as a login item, so it comes back
# after every restart. Re-run this script any time to update.
#
#   ./scripts/install.sh
#
# Needs macOS 13+ and the Swift toolchain (Xcode or `xcode-select --install`).
set -euo pipefail

cd "$(dirname "$0")/.."

swift build -c release
BIN="$(swift build -c release --show-bin-path)/ClaudeWidget"
APP="$HOME/Applications/Claude Widget.app"

pkill -x ClaudeWidget 2>/dev/null || true

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/ClaudeWidget"
cp Resources/Info.plist "$APP/Contents/Info.plist"

# Ad-hoc signature: enough for a locally built app and for the login item.
codesign --force --sign - "$APP"

open "$APP"
echo "Installed to $APP"
echo "Right-click the widget (or use the gauge icon in the menu bar) for options."
