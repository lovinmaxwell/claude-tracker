# Antigravity OAuth findings (sanitized)

Updated: 2026-09-27 (token refresh HTTP 400 root-cause)

## Root cause of "Antigravity token refresh HTTP 400/401"

| File | Minted by OAuth client | Refresh with Antigravity client | Quota `retrieveUserQuotaSummary` |
|------|------------------------|----------------------------------|----------------------------------|
| `~/.gemini/oauth_creds.json` | **gemini-cli** (`681255809395-oo8ft2…`) | **Fails** (`unauthorized_client`) | N/A (refresh fails); even after gemini-cli refresh → **403** |
| `~/.gemini/jetski-standalone-oauth-token` | **Antigravity** (`1071006060591-tmhssin2…`) | **OK** | **OK** |
| `~/.config/gcloud/application_default_credentials.json` | gcloud ADC (`764086051850-6qr4p6…`) | Fails | N/A |

Quota Tray must import the **jetski** file (Antigravity client), not gemini-cli `oauth_creds.json`.

## Preferred import

1. **`~/.gemini/jetski-standalone-oauth-token`** (nested `{token:{…}, auth_method}`) — Antigravity consumer OAuth
2. Raw Antigravity refresh token string (`1//…`) minted by the Antigravity app

## Do not use for Antigravity quota

- `~/.gemini/oauth_creds.json` — gemini-cli client; wrong for Antigravity refresh/quota
- gcloud ADC JSON — different client again

## Ready-to-import copies (mode 600; contain secrets — do not commit)

- Desktop: `~/Desktop/antigravity-jetski-oauth-IMPORT.json` ← **use this**
- Desktop: `~/Desktop/antigravity-oauth_creds-IMPORT.json` ← gemini-cli only; will not work for Antigravity quota
- Repo copies under `quota-tray/import-ready/` (gitignored / untracked)

## Public client IDs (not secrets)

- Antigravity: `1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com`
- gemini-cli: `681255809395-oo8ft2oprdrnp9e3aqf6av3hmdib135j.apps.googleusercontent.com`
