# Appendix: Multi-provider credential & usage API research

Research date: 2026-09-02. Prefer verified paths/endpoints from this repo, OpenUsage docs, quotas crate, and live community reverse-engineering. Mark uncertain claims.

**Convention:** `headline_percent` = **percent used** (0–100), matching this app’s Claude tray (higher = fuller / more alarming). Convert remaining→used where APIs invert.

---

## Claude Code (Anthropic subscription)

### 1. Local credentials

| OS | Location |
|---|---|
| **macOS** | Keychain generic password, service **`Claude Code-credentials`** (account ≈ `$USER`). Read: `security find-generic-password -s "Claude Code-credentials" -w`. Payload JSON includes `claudeAiOauth.accessToken` / `refreshToken` / `expiresAt`. |
| **Linux / WSL** | Plaintext **`~/.claude/.credentials.json`** (mode typically `0600`). Also probe **`~/.config/claude/`** and `$CLAUDE_CONFIG_DIR`. No libsecret for OAuth yet (open feature request). |
| **Windows** | Plaintext **`%USERPROFILE%\.claude\.credentials.json`** (NTFS ACLs). |

**Also seen (secondary):** orphaned `~/.claude/.credentials.json` on macOS can diverge from Keychain (Keychain is source of truth when present). Env `CLAUDE_CODE_OAUTH_TOKEN` / `claude setup-token` often **inference-only** — may lack scopes needed for usage (`user:profile` / session scopes). Claude Desktop uses separate Keychain item **`Claude Safe Storage`** (cookies/token decrypt) — not Claude Code’s store.

This repo: `config/claude.php` → `keychain_service` = `Claude Code-credentials`.

### 2. Usage HTTP endpoints

| Endpoint | Official? | Auth |
|---|---|---|
| **`GET https://api.anthropic.com/api/oauth/usage`** | **Unofficial** (powers Claude Code `/usage`) | `Authorization: Bearer <oauth accessToken>`; header `anthropic-beta: oauth-2025-04-20`. Some clients also send `User-Agent: claude-code/<ver>` (reported to avoid aggressive 429s — **uncertain if still required**). |
| `GET https://claude.ai/api/organizations/{org_uuid}/usage` | Unofficial (web/admin cookie surface) | Browser/desktop session cookies — optional org rollup. |
| Token refresh | Semi-documented OAuth | `platform.claude.com/v1/oauth/token` (OpenUsage). |

**Not usable:** workspace / `ANTHROPIC_API_KEY` → 401 on oauth usage.

### 3. Units returned

Response buckets (any may be `null`):

- `five_hour.utilization` — **% used**, 0–100; `resets_at`
- `seven_day.utilization` — **% used**; `resets_at`
- Optional: `seven_day_opus` / `seven_day_sonnet` / Fable-style model windows — **% used**
- Optional: `extra_usage` — monthly extra credits (`used_credits`, `monthly_limit`, `utilization`)

Dual windows: **5h session + 7d weekly** (plus optional per-model weeklies).

### 4. Reliability

**Unofficial / may break.** Undocumented; fields optional; throttling reported. Prefer degrade-to-last-known (this app’s pattern). No Anthropic SLA.

### 5. Suggested `headline_percent`

```
headline_percent = max(five_hour.utilization, seven_day.utilization)
# optional: also max with seven_day_opus / seven_day_sonnet if present
# tray primary label: five_hour (matches config claude.menu_bar_metric default)
```

---

## Cursor

### 1. Local credentials

| OS | Path |
|---|---|
| **macOS** | `~/Library/Application Support/Cursor/User/globalStorage/state.vscdb` |
| **Linux** | `~/.config/Cursor/User/globalStorage/state.vscdb` |
| **Windows** | `%APPDATA%\Cursor\User\globalStorage\state.vscdb` |

SQLite `ItemTable` key **`cursorAuth/accessToken`** (JWT). Open read-only / immutable (IDE holds WAL lock).

**Also:** cursor-agent `~/.config/cursor/auth.json` (`accessToken`); tracking DB `~/.cursor/ai-tracking/ai-code-tracking.db` (local telemetry, not billing).

Auth to APIs: Bearer JWT and/or cookie **`WorkosCursorSessionToken`** = `{userId}%3A%3A{accessToken}` (URL-encoded `::`).

### 2. Usage HTTP endpoints

| Endpoint | Official? | Notes |
|---|---|---|
| **`POST https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage`** | **Unofficial** (IDE/dashboard RPC) | Primary for personal Pro/Ultra; returns `planUsage`, billing cycle. |
| Same host: `GetPlanInfo`, `GetHardLimit`, `GetAggregatedUsageEvents`, `GetUsageLimitPolicyStatus`, `GetTeamMembers` | Unofficial | OpenUsage set. |
| `GET https://cursor.com/api/usage-summary` | Unofficial REST | Good for enterprise/team when RPC `planUsage` empty. |
| `GET https://cursor.com/api/usage?user=` | Unofficial **legacy** | Often null caps / zeros on token-based plans — avoid as primary. |
| **`POST https://api.cursor.com/teams/daily-usage-data`** (+ `filtered-usage-events`, spend, members) | **Official Admin API** | Enterprise **team** admin key (Basic auth). Not personal quota. |

### 3. Units returned

From `GetCurrentPeriodUsage` / usage-summary (shapes vary by plan):

- `planUsage.totalPercentUsed` / `autoPercentUsed` / `apiPercentUsed` — **% used** of included pools
- Spend fields in **cents** (`totalSpend`, `includedSpend`, `bonusSpend`, `limit`) → USD
- Billing cycle: `billingCycleStart` / `billingCycleEnd`
- Team: pooled/on-demand spend limits in cents; request counts on some enterprise contracts

Single billing-cycle window (not Claude-style 5h/7d). Dual bars often = **Auto/Composer vs API/named models**.

### 4. Reliability

**Unofficial / may break** for personal dashboard RPC & REST. Schema drifts with pricing (request → $ → %). Official Admin API is **stable but Enterprise-team-only**.

### 5. Suggested `headline_percent`

```
headline_percent = planUsage.totalPercentUsed
  ?? max(autoPercentUsed, apiPercentUsed)
  ?? (limit_cents > 0 ? 100 * totalSpend / limit : null)
```

---

## GitHub Copilot

### 1. Local credentials

Checked in order (OpenUsage / community):

1. Editor plugin token:
   - macOS/Linux: **`~/.config/github-copilot/apps.json`** (newer) or **`hosts.json`** (older)
   - Windows: **`%LOCALAPPDATA%\github-copilot\apps.json`** / `hosts.json`
   - Map keys like `github.com:Iv1.…` → `oauth_token` (`gho_*`)
2. GitHub CLI: `~/.config/gh/hosts.yml` (`oauth_token`) or Keychain service **`gh:github.com`**
3. Env overrides used by some tools: `COPILOT_TOKEN`, `GITHUB_COPILOT_*` (PATs often **fail** on internal endpoint)

### 2. Usage HTTP endpoints

| Endpoint | Official? | Auth / headers |
|---|---|---|
| **`GET https://api.github.com/copilot_internal/user`** | **Unofficial** (VS Code Copilot clients) | `Authorization: Bearer` or `token <oauth>`; Copilot client headers; `X-GitHub-Api-Version: 2025-04-01` (common). |
| `GET /users/{user}/settings/billing/premium_request/usage` | **Official** billing | Historical spend lines — **not** remaining entitlement. |
| Org: `GET /user/orgs` + `GET /orgs/{org}/settings/billing/usage/summary` | Official billing | Org-wide AI credits; needs owner/billing manager. |

### 3. Units returned

`quota_snapshots` categories: `chat`, `completions`, `premium_interactions` (AI credits / premium):

- `percent_remaining` — **% remaining** (invert for used)
- `entitlement`, `remaining` / `quota_remaining`, `unlimited`, `overage_*`
- `quota_reset_date` / `quota_reset_date_utc`
- `copilot_plan` (e.g. free / individual_pro / business)

Paid plans: chat/completions often `unlimited: true`; headline = **premium_interactions / credits**. Free: fixed chat/completions counts. Org seats: may lack personal %; org billing is org-wide only.

### 4. Reliability

**Unofficial / may break** (`copilot_internal`). Requires Copilot-issued OAuth, not arbitrary PAT. Official billing APIs are stable but wrong shape for a live “% left” tray.

### 5. Suggested `headline_percent`

```
# prefer premium / credits bucket when not unlimited
used = 100 - premium_interactions.percent_remaining
# free plan fallback:
used = max(100 - chat.percent_remaining, 100 - completions.percent_remaining)
# skip buckets with unlimited: true
headline_percent = used
```

---

## OpenAI (API platform)

### 1. Local credentials

No single IDE quota store. Common:

| Source | Path / name |
|---|---|
| Env | **`OPENAI_API_KEY`** (`sk-…`); admin **`OPENAI_ADMIN_KEY`** / `sk-admin-…` for org usage/costs |
| Shell / tool configs | `.env`, Aider, OpenCode `auth.json`, etc. |
| Cursor DB | May hold user-pasted OpenAI keys in `state.vscdb` (not an official OpenAI store) |

ChatGPT / Codex subscription auth is separate (`~/.codex/auth.json` → chatgpt.com WHAM) — **out of scope** unless treating “OpenAI” as Codex.

### 2. Usage HTTP endpoints

| Endpoint | Official? | What you get |
|---|---|---|
| Any API call or probe **`GET https://api.openai.com/v1/models/{model}`** | **Official** | Response headers: `x-ratelimit-limit/remaining/reset-requests` & `-tokens` (RPM/TPM). |
| **`GET https://api.openai.com/v1/organization/costs`** | **Official** (Admin key) | Daily **$ spend buckets** — not remaining balance. |
| **`GET https://api.openai.com/v1/organization/usage`…** (Usage API) | **Official** (Admin key) | Historical tokens by time/model/key — not “% of monthly cap remaining”. |
| Legacy `dashboard/billing/*`, cookie session usage pages | Unofficial / blocked for API keys | Session-cookie only; do not rely on. |

**Gap:** No API-key endpoint returns prepaid credit balance or ChatGPT Plus “% used”.

### 3. Units returned

- Rate limits: remaining **requests** and **tokens** per minute (model-scoped), plus reset durations
- Costs API: **USD** per day (and groupings)
- Usage API: token counts over time

No dual 5h/7d subscription windows on the API platform.

### 4. Reliability

**Stable** for official rate-limit headers + Admin Usage/Costs APIs. **Poor fit** for subscription-style tray % without Codex/ChatGPT WHAM or cookie scraping (fragile).

### 5. Suggested `headline_percent`

```
# model-scoped RPM/TPM probe (OpenUsage-style):
rpm_used = 100 * (1 - remaining_requests / limit_requests)
tpm_used = 100 * (1 - remaining_tokens / limit_tokens)
headline_percent = max(rpm_used, tpm_used)
# optional alternate: monthly spend vs user-configured soft_cap_usd
# headline_percent = min(100, 100 * month_spend_usd / soft_cap_usd)
```

---

## Open-source references (brief)

| Project | Relevance |
|---|---|
| **[OpenUsage](https://openusage.sh)** ([robinebers/openusage](https://github.com/robinebers/openusage)) | Mature multi-provider TUI; docs for Claude, Cursor, Copilot, OpenAI rate-limit probe; credential discovery order worth copying. |
| **[quotas](https://crates.io/crates/quotas)** ([clankercode/quotas](https://github.com/clankercode/quotas)) | Rust CLI/TUI; Claude oauth/usage, Copilot `copilot_internal/user`, Cursor auth helpers; statusline JSON. |
| **cursor-stats** (Dwtexe) / **cursor-usage** (lixen.cursor-usage) | VS Code/Cursor extensions; read `state.vscdb`, call dashboard usage; historically brittle as Cursor billing changed. |
| **usagenometer** / **ai-usagebar** / **grove-rs** / **headroom** | Smaller CLIs/libs repeating same credential + unofficial endpoint patterns. |
| **This repo (`claude-tracker`)** | Production path for Claude: Keychain + `GET …/api/oauth/usage` + 5h/7d tray. |

---

## Cross-provider tray mapping (summary)

| Provider | `headline_percent` source | Risk tag |
|---|---|---|
| Claude Code | `max(5h, 7d)` % **used** | unofficial |
| Cursor | `totalPercentUsed` (else max auto/api, else $ spend/limit) | unofficial (Admin API = official team-only) |
| Copilot | `100 - premium_interactions.percent_remaining` | unofficial |
| OpenAI API | `max(RPM, TPM)` % used from headers; or soft $ cap | stable headers; weak “quota left” semantics |

**Segment icon suggestion:** one segment per enabled provider; fill = that provider’s `headline_percent`; combined alarm = `max(segments)` or first-to-critical.
