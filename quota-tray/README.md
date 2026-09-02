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
| GitHub Copilot OAuth | `~/.config/github-copilot/apps.json` (`oauth_token`) — sent only to GitHub |

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
| Copilot | `GET https://api.github.com/copilot_internal/user` | Bearer `gho_*` from Copilot `apps.json` |

Using Quota Tray means you accept that these are the same *class* of client calls vendor apps make, with **no warranty**, and account/ToS risk is yours.

Deep reference: [`docs/superpowers/specs/_research-providers.md`](../docs/superpowers/specs/_research-providers.md).

## Requirements (macOS)

- macOS 13+ recommended
- [Rust](https://rustup.rs/) stable
- Node.js 20+ and npm
- Xcode CLT (`xcode-select --install`)

## Run on macOS (dev)

```bash
cd quota-tray
npm install --prefix ui
cargo build -p quota_tray_core
cd src-tauri && cargo tauri dev
```

First launch:

1. Open **Settings** and enable Claude / Cursor / Copilot as needed.
2. Approve Keychain prompts for Claude when asked.
3. Sign in to Cursor and GitHub Copilot in their official apps first so local tokens exist.
4. Quit the older **Claude Tracker** (Laravel/NativePHP) once Quota Tray’s Claude row is healthy to avoid double polling.

Panel: click the menubar icon. Esc / click away dismisses (when wired). Poll interval default **60s** (settings clamp **60–120**).

## Build

```bash
cd quota-tray/ui && npm run build
cd ../src-tauri && cargo tauri build
```

Artifacts land under `src-tauri/target/release/bundle/`.

## Workspace layout

```
quota-tray/
  crates/core/
  crates/provider-claude/
  crates/provider-cursor/
  crates/provider-copilot/
  src-tauri/
  ui/
```

## License

MIT (or match repo root). Open source is part of the credential trust bargain.
