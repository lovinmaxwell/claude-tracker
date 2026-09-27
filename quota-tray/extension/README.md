# Quota Tray — Chrome extension

Manifest V3 toolbar popup with the same cream/terracotta panel as the desktop tray app.

Chrome **cannot** read macOS Keychain, Cursor `state.vscdb`, or Copilot `apps.json` from disk. This build:

- **Claude:** uses your existing [claude.ai](https://claude.ai) login session in this Chrome profile (reads the `sessionKey` cookie, calls `claude.ai`'s own usage API with `credentials: "include"` — no token ever touches extension storage), or you can import `.credentials.json` / paste an OAuth token in Settings.
- **Cursor:** reads the `WorkosCursorSessionToken` cookie if you are signed in at [cursor.com](https://cursor.com) in this Chrome profile (or you import a JWT).
- **Copilot:** import `apps.json` (or paste the token) in Settings. Imported tokens stay in `chrome.storage.local` for this profile only — never synced, never sent to a Quota Tray server.
- **Antigravity:** Chrome cannot read the Antigravity/gemini-cli keychain either, so import your OAuth credentials (refresh token) in Settings — accepts the full oauth JSON, the nested `{token: {...}}` keychain shape, `gemini-cli`'s `oauth_creds.json`, or a raw `1//...` refresh token. The extension refreshes the access token itself using the same public installed-app OAuth client Antigravity uses, and re-saves the refreshed token to `chrome.storage.local`.

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
4. For Claude, sign in at claude.ai then **Recheck** (or import `%USERPROFILE%\.claude\.credentials.json` / `~/.claude/.credentials.json`).
5. For Cursor, sign in at cursor.com then **Recheck login**.
6. For Copilot, import `%LOCALAPPDATA%\github-copilot\apps.json`.
7. For Antigravity, import your OAuth credentials JSON (or refresh token) in Settings.

The toolbar badge shows combined % used (worst healthy provider). Polling uses Chrome alarms (minimum 1 minute).
