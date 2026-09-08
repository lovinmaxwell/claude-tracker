#!/usr/bin/env bash
# Cross-compile Quota Tray's Windows NSIS setup.exe from Linux/macOS.
# Requires: rustup, Node 20+, NSIS, lld, llvm, clang, cargo-xwin
# Ubuntu also needs libayatana-appindicator3-dev so the Tauri CLI host check passes.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v nsis >/dev/null 2>&1 && ! command -v makensis >/dev/null 2>&1; then
  echo "NSIS (makensis) is required. On Ubuntu: sudo apt install nsis" >&2
  exit 1
fi

python3 "$ROOT/scripts/generate-app-icon.py"
npm ci --prefix ui
if [[ -f "$ROOT/package-lock.json" ]]; then
  npm ci
else
  npm install
fi

rustup target add x86_64-pc-windows-msvc
if ! command -v cargo-xwin >/dev/null 2>&1; then
  cargo install --locked cargo-xwin
fi

npx tauri build \
  --runner cargo-xwin \
  --target x86_64-pc-windows-msvc \
  --bundles nsis

echo
echo "Installer:"
ls -lh "$ROOT/target/x86_64-pc-windows-msvc/release/bundle/nsis/"*-setup.exe
