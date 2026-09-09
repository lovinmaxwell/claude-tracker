# Quota Tray — Chrome extension

Manifest V3 toolbar popup with the same cream/terracotta panel as the desktop tray app.

Chrome **cannot** read macOS Keychain, Cursor `state.vscdb`, or Copilot `apps.json` from disk. This build:

- **Cursor:** reads the `WorkosCursorSessionToken` cookie if you are signed in at [cursor.com](https://cursor.com) in this Chrome profile (or you import a JWT).
- **Claude / Copilot:** import `.credentials.json` / `apps.json` (or paste the token) in Settings. Tokens stay in `chrome.storage.local` for this profile only — never synced, never sent to a Quota Tray server.

## Load unpacked

1. `cd quota-tray/ui && npm ci && npm run build:extension`
2. Open `chrome://extensions`
3. Enable **Developer mode**
4. **Load unpacked** → select `quota-tray/ui/dist-extension`

Or unzip `quota-tray/extension/QuotaTray-chrome.zip` and load that folder.

Do **not** attach the zip to Microsoft Teams — Defender often flags binaries and packed zips. Share https://github.com/lovinmaxwell/claude-tracker/releases/latest (or clone the repo) instead.

## First run

1. Pin the Quota Tray icon.
2. Open **Settings** in the popup.
3. Enable providers.
4. For Cursor, sign in at cursor.com then **Recheck login**.
5. For Claude, import `%USERPROFILE%\.claude\.credentials.json` (Windows) or `~/.claude/.credentials.json`.
6. For Copilot, import `%LOCALAPPDATA%\github-copilot\apps.json`.

The toolbar badge shows combined % used (worst healthy provider). Polling uses Chrome alarms (minimum 1 minute).
