# Quota Tray

Local macOS-first system tray app that shows AI coding quota for **Claude Code**, **Cursor**, and **GitHub Copilot** in one place.

Stack: **Tauri 2 + Rust + Svelte**. No Quota Tray servers. No telemetry. No accounts with us.

Design: [`docs/superpowers/specs/2026-09-02-quota-tray-design.md`](../docs/superpowers/specs/2026-09-02-quota-tray-design.md)

## Privacy

Quota Tray is **local-only**:

| What we read | Where it stays |
| --- | --- |
| Claude Code OAuth token | macOS Keychain service `Claude Code-credentials` (or `~/.claude/.credentials.json`) — sent only to Anthropic |
| Cursor access token | `state.vscdb` key `cursorAuth/accessToken` — sent only to Cursor |
| GitHub Copilot OAuth | `~/.config/github-copilot/apps.json` or legacy `hosts.json` (`oauth_token`) — sent only to GitHub |

Rules:

- No Quota Tray backend, cloud sync, or “send diagnostics” toggle.
- Secrets are never written to logs, crash reports, or UI copy.
- On-disk cache (if any) stores **usage numbers**, not raw tokens.
- Network: HTTPS only to the vendor endpoints listed below.
- Open source so you can audit credential and HTTP call sites before granting Keychain / file access.

## Unofficial APIs (may break)

These endpoints are **not** public SLA APIs. Vendors can change or remove them. When a call fails we mark that provider **stale** and keep the last good reading — we never invent live zeros.

| Provider | Endpoint | Auth |
| --- | --- | --- |
| Claude Code | `GET https://api.anthropic.com/api/oauth/usage` | Bearer OAuth + `anthropic-beta: oauth-2025-04-20` |
| Cursor | `POST https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage` | Bearer JWT from `state.vscdb` |
| Copilot | `GET https://api.github.com/copilot_internal/user` | Bearer `gho_*` from Copilot `apps.json` / `hosts.json` |

Using Quota Tray means you accept that these are the same *class* of client calls vendor apps make, with **no warranty**, and account/ToS risk is yours.

Deep reference: [`docs/superpowers/specs/_research-providers.md`](../docs/superpowers/specs/_research-providers.md).

## Requirements (macOS)

- macOS 13+ recommended
- [Rust](https://rustup.rs/) stable
- Node.js 20+ and npm
- Xcode CLT (`xcode-select --install`)

## Requirements (Windows)

- Windows 10 1809+ or Windows 11 (x64)
- Edge WebView2 Runtime (the NSIS setup.exe embeds the bootstrapper if it is missing)
- Sign in to Claude Code, Cursor, and/or GitHub Copilot in their official apps first so local tokens exist

## Run on macOS (dev)

```bash
cd quota-tray
npm install --prefix ui
cargo build -p quota_tray_core
cd src-tauri && cargo tauri dev
# beforeDevCommand runs npm from the quota-tray workspace root (`ui/`, not `../ui`)
```

First launch:

1. Open **Settings** and enable Claude / Cursor / Copilot as needed.
2. Approve Keychain prompts for Claude when asked.
3. Sign in to Cursor and GitHub Copilot in their official apps first so local tokens exist.
4. Quit the older **Claude Tracker** (Laravel/NativePHP) once Quota Tray’s Claude row is healthy to avoid double polling.

Panel: click the menubar icon. Esc hides the panel. Poll interval default **60s** (settings clamp **60–120**).

## Build

```bash
cd quota-tray
npm install
npm run build --prefix ui
npx tauri build
```

Artifacts land under `target/release/bundle/` (or `target/<triple>/release/bundle/` when cross-compiling).

### Windows installer (NSIS setup.exe)

The installable file is an NSIS wizard: `Quota Tray_<version>_x64-setup.exe`. It lets you install for the current user or all users, creates Start Menu shortcuts, and can uninstall from Settings.

On a Windows machine:

```bash
cd quota-tray
npm install
npm ci --prefix ui
npx tauri build --bundles nsis
```

Output: `target/release/bundle/nsis/Quota Tray_0.1.0_x64-setup.exe`

From Linux/macOS (NSIS + LLVM + [cargo-xwin](https://github.com/rust-cross/cargo-xwin)):

```bash
# Ubuntu: sudo apt install nsis lld llvm clang
chmod +x scripts/build-windows-installer.sh
./scripts/build-windows-installer.sh
```

Output: `target/x86_64-pc-windows-msvc/release/bundle/nsis/Quota Tray_0.1.0_x64-setup.exe`

GitHub Actions also builds this installer on every change under `quota-tray/` (workflow **Quota Tray Windows installer**; download the `quota-tray-windows-setup` artifact).

### Chrome extension

The desktop tray cannot run inside Chrome. There is a Manifest V3 popup that reuses the same panel UI:

```bash
cd quota-tray/ui
npm ci
npm run build:extension
```

Then `chrome://extensions` → Developer mode → **Load unpacked** → `quota-tray/ui/dist-extension`.

Details: [`extension/README.md`](extension/README.md). Cursor can use your `cursor.com` browser login; Claude and Copilot need a credentials file import (Chrome cannot read Keychain or `state.vscdb`).

## Workspace layout

```
quota-tray/
  crates/core/
  crates/provider-claude/
  crates/provider-cursor/
  crates/provider-copilot/
  src-tauri/
  ui/
  extension/
```

## License

MIT (or match repo root). Open source is part of the credential trust bargain.
