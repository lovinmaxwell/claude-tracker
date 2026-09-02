# Quota Tray Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Tauri 2 macOS-first menu-bar app that polls Claude Code, Cursor, and Copilot locally and shows combined multi-segment tray + panel usage — no PHP, no our servers.

**Architecture:** Cargo workspace under `quota-tray/`: `crates/core` (types, aggregation, poller), provider crates (Claude → Cursor → Copilot), `src-tauri` (tray, IPC, icon paint), `ui/` (Svelte panel). Providers implement one `Provider` trait and map vendor JSON → `ProviderSnapshot`. `shared_mascot_fill = max(% used)` among healthy snapshots.

**Tech Stack:** Rust 2021, Tauri 2, Svelte 5 (or 4 if template pins), `reqwest`, `serde`, `rusqlite` (Cursor), `image`/`png` for icons, `tokio` for async poll, `wiremock` for HTTP tests.

**Spec:** [`docs/superpowers/specs/2026-09-02-quota-tray-design.md`](../specs/2026-09-02-quota-tray-design.md)  
**Research:** [`docs/superpowers/specs/_research-providers.md`](../specs/_research-providers.md)

## Global Constraints

- `headline_percent` = **percent used** 0–100 (higher = worse / fuller mascot)
- `shared_mascot_fill = max(headline_percent)` among healthy (`!stale`, `error.is_none()`, `headline_percent.is_some()`)
- Empty healthy set → last successful fill muted, or pale empty — **never** live fake `0%`
- Poll: default **60s**, settings clamp **60..=120**
- Panel lists **enabled providers only**
- No telemetry, no Quota Tray backend — HTTPS only to vendor endpoints
- Open source credential trust; document unofficial APIs in README
- Visual: cream + terracotta mascot; provider colors only on chips/segments — no purple SaaS default
- v1 providers: Claude, Cursor, Copilot; OpenAI = **out of this plan** (v1.1 separate)
- Greenfield under `quota-tray/` in this repo (monorepo); do not port Laravel/PHP
- TDD: write failing test → run fail → implement → run pass → commit (per task)
- macOS is release-blocking for tray polish; keep Win/Linux compiling when cheap

## File map

| Path | Responsibility |
| --- | --- |
| `quota-tray/Cargo.toml` | Workspace members |
| `quota-tray/crates/core/` | Types, `Provider` trait, aggregation, poller, `AppConfig` |
| `quota-tray/crates/provider-claude/` | Keychain/file creds + Anthropic oauth usage |
| `quota-tray/crates/provider-cursor/` | `state.vscdb` token + Cursor usage RPC |
| `quota-tray/crates/provider-copilot/` | `apps.json` / gh token + `copilot_internal/user` |
| `quota-tray/src-tauri/` | Tauri app, tray, IPC, icon paint, background poll |
| `quota-tray/ui/` | Svelte panel + settings |
| `quota-tray/README.md` | Build, privacy, unofficial API warnings |

---

## Tasks

Ship order: Tasks 1–6 = Claude MVP core; 7–9 = tray/UI; 10–11 = Cursor + Copilot; 12 = docs. OpenAI v1.1 is **out of scope** for this plan.

### Task 1: Scaffold Cargo workspace + Tauri 2 hello app

**Files:**
- Create: `quota-tray/Cargo.toml`
- Create: `quota-tray/crates/core/Cargo.toml`
- Create: `quota-tray/crates/core/src/lib.rs`
- Create: `quota-tray/crates/provider-claude/Cargo.toml`
- Create: `quota-tray/crates/provider-claude/src/lib.rs`
- Create: `quota-tray/src-tauri/Cargo.toml`
- Create: `quota-tray/src-tauri/tauri.conf.json`
- Create: `quota-tray/src-tauri/capabilities/default.json`
- Create: `quota-tray/src-tauri/src/lib.rs`
- Create: `quota-tray/src-tauri/src/main.rs`
- Create: `quota-tray/src-tauri/build.rs`
- Create: `quota-tray/ui/package.json`
- Create: `quota-tray/ui/index.html`
- Create: `quota-tray/ui/vite.config.ts`
- Create: `quota-tray/ui/src/main.ts`
- Create: `quota-tray/ui/src/App.svelte`
- Create: `quota-tray/README.md`
- Create: `quota-tray/.gitignore`

**Interfaces:**
- Consumes: none (greenfield)
- Produces: Cargo workspace members `quota_tray_core`, `provider_claude`, `quota-tray` (Tauri binary); Svelte UI that loads in the Tauri webview; empty public crates ready for Tasks 2–6

- [ ] **Step 1: Create workspace root `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/provider-claude",
    "src-tauri",
]

[workspace.package]
edition = "2021"
license = "MIT"
version = "0.1.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
time = { version = "0.3", features = ["serde", "formatting", "parsing"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync"] }
```

- [ ] **Step 2: Create `crates/core` stub**

`quota-tray/crates/core/Cargo.toml`:

```toml
[package]
name = "quota_tray_core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
time = { workspace = true }
```

`quota-tray/crates/core/src/lib.rs`:

```rust
//! Shared types, aggregation, and poller for Quota Tray.

pub fn workspace_smoke() -> &'static str {
    "quota_tray_core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_smoke_returns_crate_name() {
        assert_eq!(workspace_smoke(), "quota_tray_core");
    }
}
```

- [ ] **Step 3: Create `crates/provider-claude` stub**

`quota-tray/crates/provider-claude/Cargo.toml`:

```toml
[package]
name = "provider_claude"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
quota_tray_core = { path = "../core" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

`quota-tray/crates/provider-claude/src/lib.rs`:

```rust
//! Claude Code credentials + Anthropic OAuth usage fetch.

pub fn provider_id_str() -> &'static str {
    "claude"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_id_str_is_claude() {
        assert_eq!(provider_id_str(), "claude");
    }
}
```

- [ ] **Step 4: Scaffold Tauri 2 + Svelte UI**

Prefer generating with the official template, then align paths:

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
# If quota-tray already has Cargo.toml from Steps 1–3, keep crates/ and merge:
npm create tauri-app@latest quota-tray-tmp -- --template svelte-ts --manager npm --yes
# Move generated src-tauri + ui into quota-tray/, discard tmp root Cargo if conflicting;
# ensure workspace members include src-tauri as above.
```

If interactive create is unavailable, hand-write the minimal files below.

`quota-tray/src-tauri/Cargo.toml`:

```toml
[package]
name = "quota-tray"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
name = "quota_tray_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
serde = { workspace = true }
serde_json = { workspace = true }
quota_tray_core = { path = "../crates/core" }
provider_claude = { path = "../crates/provider-claude" }

[dev-dependencies]
```

`quota-tray/src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

`quota-tray/src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    quota_tray_lib::run();
}
```

`quota-tray/src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! from {}", quota_tray_core::workspace_smoke())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`quota-tray/src-tauri/tauri.conf.json` (adjust identifier/icons as needed; `frontendDist` must match Vite outDir):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Quota Tray",
  "version": "0.1.0",
  "identifier": "dev.quotatray.app",
  "build": {
    "beforeDevCommand": "npm run dev --prefix ../ui",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build --prefix ../ui",
    "frontendDist": "../ui/dist"
  },
  "app": {
    "windows": [
      {
        "title": "Quota Tray",
        "width": 360,
        "height": 480,
        "resizable": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

`quota-tray/src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": ["core:default", "shell:allow-open"]
}
```

Generate default icons (or copy from `create-tauri-app` output) into `quota-tray/src-tauri/icons/`.

`quota-tray/ui/package.json`:

```json
{
  "name": "quota-tray-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "^5.0.0",
    "@tauri-apps/api": "^2.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0"
  }
}
```

`quota-tray/ui/vite.config.ts`:

```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
```

`quota-tray/ui/index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Quota Tray</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

`quota-tray/ui/src/main.ts`:

```ts
import App from "./App.svelte";
import { mount } from "svelte";

mount(App, { target: document.getElementById("app")! });
```

`quota-tray/ui/src/App.svelte`:

```svelte
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let message = $state("…");

  async function load() {
    message = await invoke<string>("greet", { name: "Quota Tray" });
  }

  $effect(() => {
    void load();
  });
</script>

<main>
  <h1>Quota Tray</h1>
  <p>{message}</p>
</main>
```

`quota-tray/.gitignore`:

```
target/
ui/node_modules/
ui/dist/
src-tauri/gen/
.DS_Store
```

`quota-tray/README.md`:

```markdown
# Quota Tray

Local-only multi-provider AI quota tray (Tauri 2 + Rust + Svelte).

## Develop

```bash
cd quota-tray/ui && npm install
cd ../src-tauri && cargo tauri dev
```

## Test Rust crates

```bash
cd quota-tray && cargo test -p quota_tray_core -p provider_claude
```

Credentials never leave the machine. No Quota Tray servers. Vendor usage APIs may be unofficial.
```

- [ ] **Step 5: Install UI deps and verify Rust workspace tests**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray/ui && npm install`
Expected: `added … packages` with exit code 0

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core -p provider_claude`
Expected: PASS — `workspace_smoke_returns_crate_name` and `provider_id_str_is_claude`

- [ ] **Step 6: Smoke-check Tauri compile (optional GUI)**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray/src-tauri && cargo check`
Expected: finishes without errors (icons + `tauri.conf.json` present)

- [ ] **Step 7: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/
git commit -m "$(cat <<'EOF'
chore(quota-tray): scaffold Cargo workspace and Tauri 2 hello app

EOF
)"
```

---

### Task 2: Core types + `aggregate_mascot_fill` (TDD)

**Files:**
- Create: `quota-tray/crates/core/src/types.rs`
- Create: `quota-tray/crates/core/src/aggregate.rs`
- Create: `quota-tray/crates/core/src/config.rs`
- Create: `quota-tray/crates/core/src/provider.rs`
- Create: `quota-tray/crates/core/tests/aggregate_mascot_fill.rs`
- Modify: `quota-tray/crates/core/src/lib.rs`
- Modify: `quota-tray/crates/core/Cargo.toml`

**Interfaces:**
- Consumes: Task 1 workspace crate `quota_tray_core`
- Produces:
  - `pub enum ProviderId { Claude, Cursor, Copilot, OpenAI }`
  - `pub enum WindowKind { Percent { used: Option<f64> }, Currency { used: Option<f64>, limit: Option<f64>, code: String }, Count { used: Option<u64>, limit: Option<u64> } }`
  - `pub struct UsageWindow { pub id: String, pub label: String, pub kind: WindowKind, pub resets_at: Option<time::OffsetDateTime> }`
  - `pub struct ProviderSnapshot { pub provider: ProviderId, pub fetched_at: time::OffsetDateTime, pub windows: Vec<UsageWindow>, pub headline_percent: Option<f64>, pub stale: bool, pub error: Option<String> }`
  - `pub struct TrayState { pub providers: Vec<ProviderSnapshot>, pub shared_mascot_fill: Option<f64> }`
  - `pub fn aggregate_mascot_fill(providers: &[ProviderSnapshot], last_successful: Option<f64>) -> Option<f64>`
  - `pub fn clamp_poll_interval_secs(secs: u64) -> u64` → clamps to `60..=120`
  - `pub enum ClaudeHeadlineMetric { FiveHour, SevenDay, Highest }` (default `Highest`)
  - `pub struct AppConfig { pub poll_interval_secs: u64, pub enabled: Vec<ProviderId>, pub claude_headline: ClaudeHeadlineMetric }`
  - `pub trait Provider: Send + Sync { fn id(&self) -> ProviderId; fn credentials(&self) -> Result<Credentials, CredentialError>; fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError>; }`
  - `pub struct Credentials { pub raw: secrecy::SecretString }`
  - `pub enum CredentialError` / `pub enum FetchError` (`thiserror`)

- [ ] **Step 1: Add dependencies and write failing integration test**

Append to `quota-tray/crates/core/Cargo.toml`:

```toml
[dependencies]
secrecy = { version = "0.10", features = ["serde"] }

[dev-dependencies]
time = { workspace = true }
```

`quota-tray/crates/core/tests/aggregate_mascot_fill.rs`:

```rust
use quota_tray_core::{
    aggregate_mascot_fill, clamp_poll_interval_secs, ProviderId, ProviderSnapshot,
};
use time::OffsetDateTime;

fn snap(
    provider: ProviderId,
    headline: Option<f64>,
    stale: bool,
    error: Option<&str>,
) -> ProviderSnapshot {
    ProviderSnapshot {
        provider,
        fetched_at: OffsetDateTime::UNIX_EPOCH,
        windows: vec![],
        headline_percent: headline,
        stale,
        error: error.map(str::to_string),
    }
}

#[test]
fn max_among_healthy_some_headlines() {
    let providers = vec![
        snap(ProviderId::Claude, Some(62.0), false, None),
        snap(ProviderId::Cursor, Some(41.0), false, None),
        snap(ProviderId::Copilot, Some(10.0), false, None),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, None), Some(62.0));
}

#[test]
fn excludes_stale_error_and_none_headline() {
    let providers = vec![
        snap(ProviderId::Claude, Some(90.0), true, None), // stale
        snap(ProviderId::Cursor, Some(80.0), false, Some("boom")), // error
        snap(ProviderId::Copilot, None, false, None), // no headline
        snap(ProviderId::OpenAI, Some(55.0), false, None),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, Some(12.0)), Some(55.0));
}

#[test]
fn empty_healthy_reuses_last_successful() {
    let providers = vec![
        snap(ProviderId::Claude, Some(90.0), true, Some("fail")),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, Some(33.0)), Some(33.0));
}

#[test]
fn empty_healthy_and_no_last_returns_none_never_zero() {
    let providers = vec![
        snap(ProviderId::Claude, None, true, Some("waiting")),
    ];
    assert_eq!(aggregate_mascot_fill(&providers, None), None);
}

#[test]
fn clamp_poll_interval_secs_bounds() {
    assert_eq!(clamp_poll_interval_secs(1), 60);
    assert_eq!(clamp_poll_interval_secs(60), 60);
    assert_eq!(clamp_poll_interval_secs(90), 90);
    assert_eq!(clamp_poll_interval_secs(120), 120);
    assert_eq!(clamp_poll_interval_secs(999), 120);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core --test aggregate_mascot_fill`
Expected: FAIL with unresolved import / `aggregate_mascot_fill` not found

- [ ] **Step 3: Implement types, trait, aggregation, config**

`quota-tray/crates/core/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Stable id used in config, tray segments, and UI chips.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    Claude,
    Cursor,
    Copilot,
    OpenAI,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: ProviderId,
    pub display_name: &'static str,
    /// Chip / segment color only (not mascot fill).
    pub chip_color: &'static str,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageWindow {
    pub id: String,
    pub label: String,
    pub kind: WindowKind,
    pub resets_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WindowKind {
    /// 0.0..=100.0 used; None means "vendor did not say"
    Percent { used: Option<f64> },
    Currency {
        used: Option<f64>,
        limit: Option<f64>,
        code: String,
    },
    Count {
        used: Option<u64>,
        limit: Option<u64>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderSnapshot {
    pub provider: ProviderId,
    pub fetched_at: OffsetDateTime,
    pub windows: Vec<UsageWindow>,
    /// Drives tray segment + contribution to shared mascot fill (% used).
    pub headline_percent: Option<f64>,
    pub stale: bool,
    pub error: Option<String>,
}

impl ProviderSnapshot {
    /// Healthy = not stale, no error, and headline_percent is Some.
    pub fn is_healthy(&self) -> bool {
        !self.stale && self.error.is_none() && self.headline_percent.is_some()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrayState {
    /// Enabled only, stable order.
    pub providers: Vec<ProviderSnapshot>,
    /// max % used among healthy; None if none healthy and no last success.
    pub shared_mascot_fill: Option<f64>,
}
```

`quota-tray/crates/core/src/aggregate.rs`:

```rust
use crate::types::ProviderSnapshot;

/// `shared_mascot_fill = max(headline_percent)` among healthy providers.
/// If the healthy set is empty, reuse `last_successful` (muted in UI) or `None`.
/// Never invents a live `0.0`.
pub fn aggregate_mascot_fill(
    providers: &[ProviderSnapshot],
    last_successful: Option<f64>,
) -> Option<f64> {
    let mut max_used: Option<f64> = None;
    for snap in providers {
        if !snap.is_healthy() {
            continue;
        }
        let Some(p) = snap.headline_percent else {
            continue;
        };
        max_used = Some(match max_used {
            Some(m) => m.max(p),
            None => p,
        });
    }
    max_used.or(last_successful)
}
```

`quota-tray/crates/core/src/config.rs`:

```rust
use crate::types::ProviderId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaudeHeadlineMetric {
    FiveHour,
    SevenDay,
    #[default]
    Highest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// v1: default 60; settings UI clamps 60..=120
    pub poll_interval_secs: u64,
    pub enabled: Vec<ProviderId>,
    pub claude_headline: ClaudeHeadlineMetric,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 60,
            enabled: vec![ProviderId::Claude],
            claude_headline: ClaudeHeadlineMetric::Highest,
        }
    }
}

/// Clamp poll interval to the v1 allowed range.
pub fn clamp_poll_interval_secs(secs: u64) -> u64 {
    secs.clamp(60, 120)
}
```

`quota-tray/crates/core/src/provider.rs`:

```rust
use crate::types::{ProviderId, ProviderSnapshot};
use secrecy::SecretString;
use thiserror::Error;

/// Opaque per-provider secret material; never logged.
pub struct Credentials {
    pub raw: SecretString,
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credentials missing: {0}")]
    Missing(String),
    #[error("credentials malformed: {0}")]
    Malformed(String),
    #[error("credentials expired")]
    Expired,
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("network: {0}")]
    Network(String),
    #[error("http {status}: {body}")]
    Http { status: u16, body: String },
    #[error("parse: {0}")]
    Parse(String),
}

pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn credentials(&self) -> Result<Credentials, CredentialError>;
    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError>;
}
```

`quota-tray/crates/core/src/lib.rs`:

```rust
//! Shared types, aggregation, and poller for Quota Tray.

mod aggregate;
mod config;
mod provider;
mod types;

pub use aggregate::aggregate_mascot_fill;
pub use config::{clamp_poll_interval_secs, AppConfig, ClaudeHeadlineMetric};
pub use provider::{CredentialError, Credentials, FetchError, Provider};
pub use types::{
    ProviderId, ProviderInfo, ProviderSnapshot, TrayState, UsageWindow, WindowKind,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core --test aggregate_mascot_fill`
Expected: PASS — all 5 tests green

- [ ] **Step 5: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/crates/core/
git commit -m "$(cat <<'EOF'
feat(core): add snapshot types and aggregate_mascot_fill

EOF
)"
```

---

### Task 3: Claude credential JSON parse (unit tests, no Keychain)

**Files:**
- Create: `quota-tray/crates/provider-claude/src/credentials.rs`
- Create: `quota-tray/crates/provider-claude/tests/parse_credentials.rs`
- Modify: `quota-tray/crates/provider-claude/src/lib.rs`
- Modify: `quota-tray/crates/provider-claude/Cargo.toml`

**Interfaces:**
- Consumes: `CredentialError` from `quota_tray_core`
- Produces:
  - `pub struct ClaudeOAuthCreds { pub access_token: String, pub expires_at_ms: Option<i64> }`
  - `pub fn parse_claude_credentials_json(json: &str) -> Result<ClaudeOAuthCreds, CredentialError>`
  - Contract: reads `claudeAiOauth.accessToken` (required) and `claudeAiOauth.expiresAt` (optional ms epoch); rejects missing/empty token as `CredentialError::Malformed`
  - Keychain service name constant (for later OS glue, not used in unit tests): `pub const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";`
  - File fallback path helper: `pub fn credentials_file_path() -> PathBuf` → `$CLAUDE_CONFIG_DIR/.credentials.json` or `~/.claude/.credentials.json`
  - Unit tests must **not** touch real Keychain; only parse string blobs (sample from Claude Tracker tests)

- [ ] **Step 1: Write failing parse tests**

Update `quota-tray/crates/provider-claude/Cargo.toml`:

```toml
[package]
name = "provider_claude"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
quota_tray_core = { path = "../core" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
dirs = "6"

[dev-dependencies]
```

`quota-tray/crates/provider-claude/tests/parse_credentials.rs`:

```rust
use provider_claude::{parse_claude_credentials_json, KEYCHAIN_SERVICE};
use quota_tray_core::CredentialError;

/// Mirrors claude-tracker `tests/Pest.php` `claudeKeychainBlob` shape
/// (and the locked sample from the Quota Tray plan).
const SAMPLE: &str = r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat-test","expiresAt":9999999999999}}"#;

const FULL_BLOB: &str = r#"{
  "claudeAiOauth": {
    "accessToken": "sk-ant-oat01-testing",
    "refreshToken": "sk-ant-ort01-testing",
    "expiresAt": 9999999999999,
    "subscriptionType": "pro"
  },
  "organizationUuid": "org-testing"
}"#;

#[test]
fn parses_locked_sample_blob() {
    let creds = parse_claude_credentials_json(SAMPLE).expect("parse sample");
    assert_eq!(creds.access_token, "sk-ant-oat-test");
    assert_eq!(creds.expires_at_ms, Some(9999999999999));
}

#[test]
fn parses_full_tracker_style_blob() {
    let creds = parse_claude_credentials_json(FULL_BLOB).expect("parse full");
    assert_eq!(creds.access_token, "sk-ant-oat01-testing");
    assert_eq!(creds.expires_at_ms, Some(9999999999999));
}

#[test]
fn rejects_missing_oauth_object() {
    let err = parse_claude_credentials_json("{}").unwrap_err();
    assert!(matches!(err, CredentialError::Malformed(_)));
}

#[test]
fn rejects_empty_access_token() {
    let json = r#"{"claudeAiOauth":{"accessToken":"","expiresAt":1}}"#;
    let err = parse_claude_credentials_json(json).unwrap_err();
    assert!(matches!(err, CredentialError::Malformed(_)));
}

#[test]
fn keychain_service_name_matches_claude_code() {
    assert_eq!(KEYCHAIN_SERVICE, "Claude Code-credentials");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p provider_claude --test parse_credentials`
Expected: FAIL — `parse_claude_credentials_json` / `KEYCHAIN_SERVICE` not found

- [ ] **Step 3: Implement JSON parse only (no Keychain reads)**

`quota-tray/crates/provider-claude/src/credentials.rs`:

```rust
use dirs::home_dir;
use quota_tray_core::CredentialError;
use serde::Deserialize;
use std::path::PathBuf;

/// macOS Keychain generic-password service name (Claude Code).
pub const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeOAuthCreds {
    pub access_token: String,
    pub expires_at_ms: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct Root {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthBlock>,
}

#[derive(Debug, Deserialize)]
struct OAuthBlock {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<i64>,
}

/// Parse the JSON blob Claude Code stores in Keychain / `.credentials.json`.
/// Does not read Keychain or the filesystem — callers pass the blob.
pub fn parse_claude_credentials_json(json: &str) -> Result<ClaudeOAuthCreds, CredentialError> {
    let root: Root = serde_json::from_str(json)
        .map_err(|e| CredentialError::Malformed(e.to_string()))?;
    let oauth = root
        .claude_ai_oauth
        .ok_or_else(|| CredentialError::Malformed("missing claudeAiOauth".into()))?;
    let token = oauth
        .access_token
        .filter(|t| !t.is_empty())
        .ok_or_else(|| CredentialError::Malformed("missing accessToken".into()))?;
    Ok(ClaudeOAuthCreds {
        access_token: token,
        expires_at_ms: oauth.expires_at,
    })
}

/// Linux/Win (and macOS fallback) path: `$CLAUDE_CONFIG_DIR/.credentials.json`
/// or `~/.claude/.credentials.json`.
pub fn credentials_file_path() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(dir).join(".credentials.json");
    }
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join(".credentials.json")
}
```

`quota-tray/crates/provider-claude/src/lib.rs`:

```rust
//! Claude Code credentials + Anthropic OAuth usage fetch.

mod credentials;

pub use credentials::{
    credentials_file_path, parse_claude_credentials_json, ClaudeOAuthCreds, KEYCHAIN_SERVICE,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p provider_claude --test parse_credentials`
Expected: PASS — all 5 tests green

- [ ] **Step 5: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/crates/provider-claude/
git commit -m "$(cat <<'EOF'
feat(provider-claude): parse Claude Code credential JSON blobs

EOF
)"
```

---

### Task 4: Claude usage HTTP fetch (wiremock) + window mapping

**Files:**
- Create: `quota-tray/crates/provider-claude/src/usage.rs`
- Create: `quota-tray/crates/provider-claude/tests/fetch_usage.rs`
- Modify: `quota-tray/crates/provider-claude/src/lib.rs`
- Modify: `quota-tray/crates/provider-claude/Cargo.toml`

**Interfaces:**
- Consumes: `ClaudeOAuthCreds` (Task 3); `ProviderSnapshot`, `UsageWindow`, `WindowKind`, `ProviderId`, `ClaudeHeadlineMetric`, `FetchError` from `quota_tray_core`
- Produces:
  - `pub const USAGE_PATH: &str = "/api/oauth/usage";`
  - `pub const ANTHROPIC_BETA_OAUTH: &str = "oauth-2025-04-20";`
  - `pub async fn fetch_claude_usage(base_url: &str, access_token: &str, headline: ClaudeHeadlineMetric) -> Result<ProviderSnapshot, FetchError>`
  - HTTP: `GET {base_url}/api/oauth/usage` with `Authorization: Bearer {token}` and `anthropic-beta: oauth-2025-04-20`
  - Maps `five_hour.utilization` / `seven_day.utilization` → `UsageWindow` ids `"five_hour"` / `"seven_day"` (`WindowKind::Percent`)
  - `headline_percent`: `Highest` → `max` of present used %; `FiveHour` / `SevenDay` → that window only; missing windows stay `None` (never fake `0.0`)
  - Production base URL constant: `pub const DEFAULT_API_BASE: &str = "https://api.anthropic.com";`

- [ ] **Step 1: Add HTTP deps and write failing wiremock tests**

Update dependencies in `quota-tray/crates/provider-claude/Cargo.toml`:

```toml
[dependencies]
quota_tray_core = { path = "../core" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
dirs = "6"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
tokio = { workspace = true }
time = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
wiremock = "0.6"
```

`quota-tray/crates/provider-claude/tests/fetch_usage.rs`:

```rust
use provider_claude::{fetch_claude_usage, ANTHROPIC_BETA_OAUTH, USAGE_PATH};
use quota_tray_core::{ClaudeHeadlineMetric, ProviderId, WindowKind};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn fetches_and_maps_five_hour_and_seven_day() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .and(header("Authorization", "Bearer sk-ant-oat-test"))
        .and(header("anthropic-beta", ANTHROPIC_BETA_OAUTH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": {
                "utilization": 42.0,
                "resets_at": "2099-01-01T00:00:00Z"
            },
            "seven_day": {
                "utilization": 13.0,
                "resets_at": "2099-01-07T00:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(
        &server.uri(),
        "sk-ant-oat-test",
        ClaudeHeadlineMetric::Highest,
    )
    .await
    .expect("fetch");

    assert_eq!(snap.provider, ProviderId::Claude);
    assert!(!snap.stale);
    assert!(snap.error.is_none());
    assert_eq!(snap.headline_percent, Some(42.0)); // Highest = max(42, 13)

    let five = snap.windows.iter().find(|w| w.id == "five_hour").unwrap();
    let seven = snap.windows.iter().find(|w| w.id == "seven_day").unwrap();
    match five.kind {
        WindowKind::Percent { used } => assert_eq!(used, Some(42.0)),
        _ => panic!("expected percent"),
    }
    match seven.kind {
        WindowKind::Percent { used } => assert_eq!(used, Some(13.0)),
        _ => panic!("expected percent"),
    }
}

#[tokio::test]
async fn five_hour_headline_metric() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "five_hour": { "utilization": 10.0 },
            "seven_day": { "utilization": 90.0 }
        })))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::FiveHour)
        .await
        .unwrap();
    assert_eq!(snap.headline_percent, Some(10.0));
}

#[tokio::test]
async fn http_error_is_fetch_error_not_zeros() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(401).set_body_string("nope"))
        .mount(&server)
        .await;

    let err = fetch_claude_usage(&server.uri(), "bad", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("401") || msg.contains("http"), "{msg}");
}

#[tokio::test]
async fn missing_windows_yield_none_headline_not_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(USAGE_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&server)
        .await;

    let snap = fetch_claude_usage(&server.uri(), "tok", ClaudeHeadlineMetric::Highest)
        .await
        .unwrap();
    assert!(snap.windows.is_empty());
    assert_eq!(snap.headline_percent, None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p provider_claude --test fetch_usage`
Expected: FAIL — `fetch_claude_usage` not found

- [ ] **Step 3: Implement usage fetch + mapping**

`quota-tray/crates/provider-claude/src/usage.rs`:

```rust
use quota_tray_core::{
    ClaudeHeadlineMetric, FetchError, ProviderId, ProviderSnapshot, UsageWindow, WindowKind,
};
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub const DEFAULT_API_BASE: &str = "https://api.anthropic.com";
pub const USAGE_PATH: &str = "/api/oauth/usage";
pub const ANTHROPIC_BETA_OAUTH: &str = "oauth-2025-04-20";

#[derive(Debug, Deserialize)]
struct UsageResponse {
    five_hour: Option<WindowDto>,
    seven_day: Option<WindowDto>,
}

#[derive(Debug, Deserialize)]
struct WindowDto {
    utilization: Option<f64>,
    resets_at: Option<String>,
}

fn parse_resets(raw: &Option<String>) -> Option<OffsetDateTime> {
    raw.as_ref()
        .and_then(|s| OffsetDateTime::parse(s, &Rfc3339).ok())
}

fn clamp_used(v: f64) -> f64 {
    v.clamp(0.0, 100.0)
}

fn map_window(id: &str, label: &str, dto: WindowDto) -> UsageWindow {
    UsageWindow {
        id: id.to_string(),
        label: label.to_string(),
        kind: WindowKind::Percent {
            used: dto.utilization.map(clamp_used),
        },
        resets_at: parse_resets(&dto.resets_at),
    }
}

fn percent_used(window: &UsageWindow) -> Option<f64> {
    match window.kind {
        WindowKind::Percent { used } => used,
        _ => None,
    }
}

fn headline_from_windows(
    windows: &[UsageWindow],
    metric: ClaudeHeadlineMetric,
) -> Option<f64> {
    let five = windows.iter().find(|w| w.id == "five_hour").and_then(percent_used);
    let seven = windows.iter().find(|w| w.id == "seven_day").and_then(percent_used);
    match metric {
        ClaudeHeadlineMetric::FiveHour => five,
        ClaudeHeadlineMetric::SevenDay => seven,
        ClaudeHeadlineMetric::Highest => match (five, seven) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        },
    }
}

/// GET `{base_url}/api/oauth/usage` with Bearer + anthropic-beta oauth header.
pub async fn fetch_claude_usage(
    base_url: &str,
    access_token: &str,
    headline: ClaudeHeadlineMetric,
) -> Result<ProviderSnapshot, FetchError> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), USAGE_PATH);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("anthropic-beta", ANTHROPIC_BETA_OAUTH)
        .send()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| FetchError::Network(e.to_string()))?;

    if !status.is_success() {
        return Err(FetchError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let parsed: UsageResponse =
        serde_json::from_str(&body).map_err(|e| FetchError::Parse(e.to_string()))?;

    let mut windows = Vec::new();
    if let Some(dto) = parsed.five_hour {
        windows.push(map_window("five_hour", "5-hour", dto));
    }
    if let Some(dto) = parsed.seven_day {
        windows.push(map_window("seven_day", "7-day", dto));
    }

    let headline_percent = headline_from_windows(&windows, headline);

    Ok(ProviderSnapshot {
        provider: ProviderId::Claude,
        fetched_at: OffsetDateTime::now_utc(),
        windows,
        headline_percent,
        stale: false,
        error: None,
    })
}
```

Update `quota-tray/crates/provider-claude/src/lib.rs`:

```rust
//! Claude Code credentials + Anthropic OAuth usage fetch.

mod credentials;
mod usage;

pub use credentials::{
    credentials_file_path, parse_claude_credentials_json, ClaudeOAuthCreds, KEYCHAIN_SERVICE,
};
pub use usage::{
    fetch_claude_usage, ANTHROPIC_BETA_OAUTH, DEFAULT_API_BASE, USAGE_PATH,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p provider_claude --test fetch_usage`
Expected: PASS — all 4 tests green

- [ ] **Step 5: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/crates/provider-claude/
git commit -m "$(cat <<'EOF'
feat(provider-claude): fetch oauth usage and map 5h/7d windows

EOF
)"
```

---

### Task 5: Parallel poller with independent stale-on-error (TDD)

**Files:**
- Create: `quota-tray/crates/core/src/poller.rs`
- Create: `quota-tray/crates/core/tests/poller.rs`
- Modify: `quota-tray/crates/core/src/lib.rs`
- Modify: `quota-tray/crates/core/Cargo.toml`

**Interfaces:**
- Consumes: `Provider` trait, `ProviderSnapshot`, `TrayState`, `aggregate_mascot_fill`, `clamp_poll_interval_secs`, `AppConfig` (Task 2)
- Produces:
  - `pub struct Poller { /* providers, interval, last snapshots, last_mascot */ }`
  - `pub fn Poller::new(providers: Vec<Box<dyn Provider>>, poll_interval_secs: u64) -> Self` — clamps interval via `clamp_poll_interval_secs`
  - `pub fn Poller::interval_secs(&self) -> u64`
  - `pub fn Poller::tick(&mut self) -> TrayState` — calls each provider’s `credentials` + `fetch` **independently**; on `Ok` replaces that provider’s snapshot (`stale=false`); on `Err` keeps last good windows/headline/`fetched_at` for that provider, sets `stale=true` and `error=Some(...)`; never writes fake zeros; updates `shared_mascot_fill` via `aggregate_mascot_fill`
  - Parallelism: use `std::thread::scope` (or rayon) so provider fetches run concurrently on a tick; tests may use fake in-memory providers
  - Fake providers live only in tests

- [ ] **Step 1: Add sync deps if needed; write failing poller tests**

Ensure `quota-tray/crates/core/Cargo.toml` has tokio only if needed; poller v1 can be sync:

```toml
[dependencies]
# ... existing ...
# no extra required for thread::scope
```

`quota-tray/crates/core/tests/poller.rs`:

```rust
use quota_tray_core::{
    CredentialError, Credentials, FetchError, Poller, Provider, ProviderId, ProviderSnapshot,
};
use secrecy::SecretString;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;

struct FakeProvider {
    id: ProviderId,
    /// Scripted results per tick index.
    results: Mutex<Vec<Result<ProviderSnapshot, FetchError>>>,
    calls: Arc<Mutex<usize>>,
}

impl FakeProvider {
    fn new(id: ProviderId, results: Vec<Result<ProviderSnapshot, FetchError>>) -> Self {
        Self {
            id,
            results: Mutex::new(results),
            calls: Arc::new(Mutex::new(0)),
        }
    }

    fn ok_snap(id: ProviderId, pct: f64) -> ProviderSnapshot {
        ProviderSnapshot {
            provider: id,
            fetched_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
            windows: vec![],
            headline_percent: Some(pct),
            stale: false,
            error: None,
        }
    }
}

impl Provider for FakeProvider {
    fn id(&self) -> ProviderId {
        self.id.clone()
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        Ok(Credentials {
            raw: SecretString::from("fake"),
        })
    }

    fn fetch(&self, _creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        let mut q = self.results.lock().unwrap();
        if q.is_empty() {
            return Err(FetchError::Network("empty script".into()));
        }
        q.remove(0)
    }
}

#[test]
fn clamps_interval_on_construct() {
    let p = Poller::new(vec![], 5);
    assert_eq!(p.interval_secs(), 60);
    let p = Poller::new(vec![], 200);
    assert_eq!(p.interval_secs(), 120);
}

#[test]
fn first_success_sets_live_snapshot_and_mascot_max() {
    let claude = FakeProvider::new(
        ProviderId::Claude,
        vec![Ok(FakeProvider::ok_snap(ProviderId::Claude, 62.0))],
    );
    let cursor = FakeProvider::new(
        ProviderId::Cursor,
        vec![Ok(FakeProvider::ok_snap(ProviderId::Cursor, 41.0))],
    );
    let mut poller = Poller::new(
        vec![Box::new(claude), Box::new(cursor)],
        60,
    );
    let state = poller.tick();
    assert_eq!(state.providers.len(), 2);
    assert!(!state.providers[0].stale);
    assert_eq!(state.shared_mascot_fill, Some(62.0));
}

#[test]
fn one_provider_error_marks_stale_keeps_last_good_others_update() {
    let claude = FakeProvider::new(
        ProviderId::Claude,
        vec![
            Ok(FakeProvider::ok_snap(ProviderId::Claude, 50.0)),
            Err(FetchError::Network("down".into())),
        ],
    );
    let cursor = FakeProvider::new(
        ProviderId::Cursor,
        vec![
            Ok(FakeProvider::ok_snap(ProviderId::Cursor, 10.0)),
            Ok(FakeProvider::ok_snap(ProviderId::Cursor, 20.0)),
        ],
    );
    let mut poller = Poller::new(
        vec![Box::new(claude), Box::new(cursor)],
        60,
    );
    let _ = poller.tick();
    let state = poller.tick();

    let claude_snap = state
        .providers
        .iter()
        .find(|p| p.provider == ProviderId::Claude)
        .unwrap();
    assert!(claude_snap.stale);
    assert_eq!(claude_snap.headline_percent, Some(50.0)); // last good, not zeros
    assert!(claude_snap.error.as_deref().unwrap().contains("down"));

    let cursor_snap = state
        .providers
        .iter()
        .find(|p| p.provider == ProviderId::Cursor)
        .unwrap();
    assert!(!cursor_snap.stale);
    assert_eq!(cursor_snap.headline_percent, Some(20.0));

    // Claude stale → excluded from max; healthy Cursor 20 wins
    assert_eq!(state.shared_mascot_fill, Some(20.0));
}

#[test]
fn never_had_success_stays_none_headline_not_fake_zero() {
    let claude = FakeProvider::new(
        ProviderId::Claude,
        vec![Err(FetchError::Network("nope".into()))],
    );
    let mut poller = Poller::new(vec![Box::new(claude)], 60);
    let state = poller.tick();
    let snap = &state.providers[0];
    assert!(snap.stale);
    assert_eq!(snap.headline_percent, None);
    assert_eq!(state.shared_mascot_fill, None);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core --test poller`
Expected: FAIL — `Poller` not found

- [ ] **Step 3: Implement poller**

`quota-tray/crates/core/src/poller.rs`:

```rust
use crate::aggregate::aggregate_mascot_fill;
use crate::config::clamp_poll_interval_secs;
use crate::provider::Provider;
use crate::types::{ProviderId, ProviderSnapshot, TrayState};
use std::collections::HashMap;
use time::OffsetDateTime;

pub struct Poller {
    providers: Vec<Box<dyn Provider>>,
    interval_secs: u64,
    last: HashMap<ProviderId, ProviderSnapshot>,
    last_mascot: Option<f64>,
}

impl Poller {
    pub fn new(providers: Vec<Box<dyn Provider>>, poll_interval_secs: u64) -> Self {
        Self {
            providers,
            interval_secs: clamp_poll_interval_secs(poll_interval_secs),
            last: HashMap::new(),
            last_mascot: None,
        }
    }

    pub fn interval_secs(&self) -> u64 {
        self.interval_secs
    }

    /// One parallel poll cycle. Failures are independent (stale-on-error).
    pub fn tick(&mut self) -> TrayState {
        let results: Vec<(ProviderId, Result<ProviderSnapshot, String>)> =
            std::thread::scope(|scope| {
                let mut handles = Vec::new();
                for provider in &self.providers {
                    handles.push(scope.spawn(move || {
                        let id = provider.id();
                        let outcome = (|| {
                            let creds = provider
                                .credentials()
                                .map_err(|e| e.to_string())?;
                            provider.fetch(&creds).map_err(|e| e.to_string())
                        })();
                        (id, outcome)
                    }));
                }
                handles
                    .into_iter()
                    .map(|h| h.join().expect("provider thread"))
                    .collect()
            });

        for (id, outcome) in results {
            match outcome {
                Ok(mut snap) => {
                    snap.stale = false;
                    snap.error = None;
                    self.last.insert(id, snap);
                }
                Err(err) => {
                    if let Some(prev) = self.last.get_mut(&id) {
                        prev.stale = true;
                        prev.error = Some(err);
                    } else {
                        self.last.insert(
                            id.clone(),
                            ProviderSnapshot {
                                provider: id,
                                fetched_at: OffsetDateTime::now_utc(),
                                windows: vec![],
                                headline_percent: None,
                                stale: true,
                                error: Some(err),
                            },
                        );
                    }
                }
            }
        }

        let providers: Vec<ProviderSnapshot> = self
            .providers
            .iter()
            .filter_map(|p| self.last.get(&p.id()).cloned())
            .collect();

        let shared = aggregate_mascot_fill(&providers, self.last_mascot);
        if providers.iter().any(|p| p.is_healthy()) {
            self.last_mascot = shared;
        }

        TrayState {
            providers,
            shared_mascot_fill: shared,
        }
    }
}
```

Update `quota-tray/crates/core/src/lib.rs` to add:

```rust
mod poller;
pub use poller::Poller;
```

If `ProviderId` is not `Clone`, derive `Clone` on it (already required in Task 2). Ensure `secrecy` is available to integration tests — add to core `[dev-dependencies]` if needed:

```toml
[dev-dependencies]
secrecy = { version = "0.10", features = ["serde"] }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core --test poller`
Expected: PASS — all 4 tests green

- [ ] **Step 5: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/crates/core/
git commit -m "$(cat <<'EOF'
feat(core): parallel poller with independent stale-on-error

EOF
)"
```

---

### Task 6: Mascot / segment icon renderer (pure Rust → PNG)

**Files:**
- Create: `quota-tray/crates/core/src/icon.rs`
- Create: `quota-tray/crates/core/tests/icon_render.rs`
- Modify: `quota-tray/crates/core/src/lib.rs`
- Modify: `quota-tray/crates/core/Cargo.toml`

**Interfaces:**
- Consumes: none beyond std + `image` crate (pure function; no Tauri)
- Produces:
  - `pub const MASCOT_GRID: usize = 16;`
  - `pub const MASCOT_ART: [&str; 16]` — Claude Tracker `B`/`E`/`.` grid (body / eye / empty)
  - `pub struct RgbaColor { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }`
  - `pub const TERRACOTTA: RgbaColor` — spent fill (`#C96442` opaque)
  - `pub const CREAM: RgbaColor` — unspent / pale body
  - `pub fn render_mascot_rgba(percent_used: Option<f64>, pixel_size: u32) -> Vec<u8>` — length `pixel_size * pixel_size * 4`, fill-from-feet; `None` → pale empty body (never paints a confident live 0% differently from “unknown” for v1: both use empty pale); `Some(0.0)` is a real zero and may leave body cream-only; eyes never filled by usage
  - `pub fn render_mascot_png(percent_used: Option<f64>, pixel_size: u32) -> Result<Vec<u8>, IconError>` — PNG-encoded bytes of the RGBA buffer
  - `pub enum IconError { Encode(String) }`

- [ ] **Step 1: Add `image` dependency and write failing tests**

Append to `quota-tray/crates/core/Cargo.toml`:

```toml
[dependencies]
image = { version = "0.25", default-features = false, features = ["png"] }
```

`quota-tray/crates/core/tests/icon_render.rs`:

```rust
use quota_tray_core::{render_mascot_png, render_mascot_rgba, MASCOT_GRID};

#[test]
fn rgba_buffer_size_matches_pixel_size() {
    let size = 32;
    let buf = render_mascot_rgba(Some(50.0), size);
    assert_eq!(buf.len(), (size * size * 4) as usize);
}

#[test]
fn none_percent_is_pale_not_full_terracotta() {
    let size = 16;
    let empty = render_mascot_rgba(None, size);
    let full = render_mascot_rgba(Some(100.0), size);
    assert_ne!(empty, full, "unknown must not render as fully spent");
}

#[test]
fn higher_percent_changes_pixels_vs_zero() {
    let size = 16;
    let zero = render_mascot_rgba(Some(0.0), size);
    let high = render_mascot_rgba(Some(80.0), size);
    assert_ne!(zero, high);
}

#[test]
fn png_has_signature_and_decodes_to_grid_multiple() {
    let png = render_mascot_png(Some(42.0), 32).expect("png");
    assert_eq!(&png[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
    let img = image::load_from_memory(&png).expect("decode").to_rgba8();
    assert_eq!(img.width(), 32);
    assert_eq!(img.height(), 32);
    assert_eq!(MASCOT_GRID, 16);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core --test icon_render`
Expected: FAIL — `render_mascot_rgba` / `render_mascot_png` not found

- [ ] **Step 3: Implement mascot grid + PNG encoder**

`quota-tray/crates/core/src/icon.rs`:

```rust
use image::{ImageBuffer, ImageError, Rgba, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

pub const MASCOT_GRID: usize = 16;

/// B = body, E = eye, . = empty — from Claude Tracker PixelMascot.
pub const MASCOT_ART: [&str; 16] = [
    "................",
    "................",
    "...BBBBBBBBBB...",
    "...BBBBBBBBBB...",
    "...BEEBBBBEEB...",
    "...BEEBBBBEEB...",
    ".BBBBBBBBBBBBBB.",
    ".BBBBBBBBBBBBBB.",
    ".BBBBBBBBBBBBBB.",
    "...BBBBBBBBBB...",
    "...BBBBBBBBBB...",
    "...BB.BB.BB.BB..",
    "...BB.BB.BB.BB..",
    "...BB.BB.BB.BB..",
    "................",
    "................",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn to_rgba(self) -> Rgba<u8> {
        Rgba([self.r, self.g, self.b, self.a])
    }
}

/// Terracotta spent fill (#C96442).
pub const TERRACOTTA: RgbaColor = RgbaColor {
    r: 0xC9,
    g: 0x64,
    b: 0x42,
    a: 255,
};

/// Pale cream unspent body.
pub const CREAM: RgbaColor = RgbaColor {
    r: 0xF5,
    g: 0xED,
    b: 0xE6,
    a: 255,
};

const EYE: RgbaColor = RgbaColor {
    r: 0x2C,
    g: 0x2C,
    b: 0x2C,
    a: 255,
};

const TRANSPARENT: RgbaColor = RgbaColor {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

#[derive(Debug, Error)]
pub enum IconError {
    #[error("png encode: {0}")]
    Encode(String),
}

fn body_row_bounds() -> (usize, usize) {
    let mut min = MASCOT_GRID;
    let mut max = 0;
    for (y, line) in MASCOT_ART.iter().enumerate() {
        if line.chars().any(|c| c == 'B' || c == 'E') {
            min = min.min(y);
            max = max.max(y);
        }
    }
    (min, max)
}

/// Fill-from-feet: `percent_used` 0–100. `None` → pale empty body (honest unknown).
pub fn render_mascot_rgba(percent_used: Option<f64>, pixel_size: u32) -> Vec<u8> {
    assert!(pixel_size > 0);
    assert_eq!(
        pixel_size as usize % MASCOT_GRID,
        0,
        "pixel_size must be a multiple of {MASCOT_GRID}"
    );
    let cell = (pixel_size as usize) / MASCOT_GRID;
    let fill_pct = percent_used.map(|p| p.clamp(0.0, 100.0)).unwrap_or(0.0);
    // Note: None and Some(0.0) both yield cream-only body; UI distinguishes via
    // stale/muted chrome, not by inventing a fake live zero glyph.
    let (body_min, body_max) = body_row_bounds();
    let body_h = (body_max - body_min + 1) as f64;
    let fill_rows = ((fill_pct / 100.0) * body_h).round() as usize;

    let mut img: RgbaImage = ImageBuffer::from_pixel(pixel_size, pixel_size, TRANSPARENT.to_rgba());

    for (y, line) in MASCOT_ART.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            if ch == '.' {
                continue;
            }
            let color = if ch == 'E' {
                EYE
            } else {
                // Fill from feet (bottom of body bounds upward).
                let row_from_feet = body_max.saturating_sub(y);
                if row_from_feet < fill_rows {
                    TERRACOTTA
                } else {
                    CREAM
                }
            };
            for dy in 0..cell {
                for dx in 0..cell {
                    let px = (x * cell + dx) as u32;
                    let py = (y * cell + dy) as u32;
                    img.put_pixel(px, py, color.to_rgba());
                }
            }
        }
    }

    img.into_raw()
}

pub fn render_mascot_png(percent_used: Option<f64>, pixel_size: u32) -> Result<Vec<u8>, IconError> {
    let raw = render_mascot_rgba(percent_used, pixel_size);
    let img: RgbaImage = ImageBuffer::from_raw(pixel_size, pixel_size, raw)
        .ok_or_else(|| IconError::Encode("buffer size mismatch".into()))?;
    let mut cursor = Cursor::new(Vec::new());
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e: ImageError| IconError::Encode(e.to_string()))?;
    Ok(cursor.into_inner())
}
```

Update `quota-tray/crates/core/src/lib.rs`:

```rust
mod icon;
pub use icon::{
    render_mascot_png, render_mascot_rgba, IconError, RgbaColor, CREAM, MASCOT_ART, MASCOT_GRID,
    TERRACOTTA,
};
```

Add `image` to `[dev-dependencies]` of the test (or rely on public re-export) — tests call `image::load_from_memory`, so:

```toml
[dev-dependencies]
image = { version = "0.25", default-features = false, features = ["png"] }
secrecy = { version = "0.10", features = ["serde"] }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core --test icon_render`
Expected: PASS — all 4 tests green

- [ ] **Step 5: Run full core + claude crate suite**

Run: `cd /Users/lovinmaxwell/Developer/claude-tracker/quota-tray && cargo test -p quota_tray_core -p provider_claude`
Expected: PASS — aggregate, poller, icon, credential parse, and usage fetch tests all green

- [ ] **Step 6: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/crates/core/
git commit -m "$(cat <<'EOF'
feat(core): render terracotta mascot PNG from usage percent

EOF
)"
```

### Task 7: Tauri tray shell + IPC + background poller

**Files:**
- Create: `quota-tray/src-tauri/src/state.rs`
- Create: `quota-tray/src-tauri/src/commands.rs`
- Create: `quota-tray/src-tauri/src/tray.rs`
- Create: `quota-tray/src-tauri/src/poll_loop.rs`
- Modify: `quota-tray/src-tauri/src/lib.rs`
- Modify: `quota-tray/src-tauri/src/main.rs`
- Modify: `quota-tray/src-tauri/Cargo.toml`
- Modify: `quota-tray/src-tauri/capabilities/default.json`
- Modify: `quota-tray/src-tauri/tauri.conf.json`
- Test: `quota-tray/src-tauri/src/commands.rs` (unit test for JSON shape via `serde_json`)

**Interfaces:**
- Consumes: `quota_tray_core::{AppConfig, ProviderId, ProviderSnapshot, TrayState, Poller, aggregate_mascot_fill}`; `quota_tray_core::paint` / icon paint from Task 6 as `quota_tray_core::icon::paint_tray_icon(state: &TrayState) -> Vec<u8>`; `provider_claude::ClaudeProvider`; `Provider` trait
- Produces: Tauri command `get_tray_state() -> Result<TrayState, String>`; managed `AppState { tray_state: Arc<RwLock<TrayState>>, config: Arc<RwLock<AppConfig>>, poller: Poller }`; tray left-click toggles panel window `main`; background poll updates tray icon + `TrayState`

- [ ] **Step 1: Add workspace deps to `src-tauri/Cargo.toml`**

```toml
[package]
name = "quota-tray"
version = "0.1.0"
description = "Quota Tray"
authors = ["Quota Tray"]
edition = "2021"

[lib]
name = "quota_tray_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync"] }
parking_lot = "0.12"
quota-tray-core = { path = "../crates/core" }
provider-claude = { path = "../crates/provider-claude" }
time = { version = "0.3", features = ["serde", "formatting"] }
```

- [ ] **Step 2: Write failing test for `TrayState` JSON IPC shape**

Append to `quota-tray/src-tauri/src/commands.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use quota_tray_core::{
        ProviderId, ProviderSnapshot, TrayState, UsageWindow, WindowKind,
    };
    use time::OffsetDateTime;

    #[test]
    fn tray_state_serializes_for_ipc() {
        let state = TrayState {
            providers: vec![ProviderSnapshot {
                provider: ProviderId::Claude,
                fetched_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
                windows: vec![UsageWindow {
                    id: "five_hour".into(),
                    label: "5-hour".into(),
                    kind: WindowKind::Percent { used: Some(42.0) },
                    resets_at: None,
                }],
                headline_percent: Some(42.0),
                stale: false,
                error: None,
            }],
            shared_mascot_fill: Some(42.0),
        };
        let v = serde_json::to_value(&state).expect("serialize");
        assert_eq!(v["shared_mascot_fill"], 42.0);
        assert_eq!(v["providers"][0]["provider"], "Claude");
        assert_eq!(v["providers"][0]["headline_percent"], 42.0);
        assert_eq!(v["providers"][0]["stale"], false);
    }
}
```

`ProviderId` and `TrayState` must `#[derive(Serialize, Deserialize)]` from Task 2. Assert on the JSON shape Task 2 actually emits (PascalCase unit variants unless Task 2 sets `rename_all`).

- [ ] **Step 3: Run test to verify it fails (module not wired)**

Run: `cd quota-tray && cargo test -p quota-tray tray_state_serializes_for_ipc -- --nocapture`

Expected: FAIL (file/module missing or compile error linking `commands`)

- [ ] **Step 4: Implement `AppState`, commands, poll loop, tray**

`quota-tray/src-tauri/src/state.rs`:

```rust
use parking_lot::RwLock;
use quota_tray_core::{AppConfig, Poller, TrayState};
use std::sync::Arc;

pub struct AppState {
    pub tray_state: Arc<RwLock<TrayState>>,
    pub config: Arc<RwLock<AppConfig>>,
    pub poller: Arc<Poller>,
}

impl AppState {
    pub fn new(config: AppConfig, poller: Poller) -> Self {
        Self {
            tray_state: Arc::new(RwLock::new(TrayState {
                providers: Vec::new(),
                shared_mascot_fill: None,
            })),
            config: Arc::new(RwLock::new(config)),
            poller: Arc::new(poller),
        }
    }
}
```

`quota-tray/src-tauri/src/commands.rs`:

```rust
use crate::state::AppState;
use quota_tray_core::TrayState;
use tauri::State;

#[tauri::command]
pub fn get_tray_state(state: State<'_, AppState>) -> Result<TrayState, String> {
    Ok(state.tray_state.read().clone())
}

#[cfg(test)]
mod tests {
    use quota_tray_core::{
        ProviderId, ProviderSnapshot, TrayState, UsageWindow, WindowKind,
    };
    use time::OffsetDateTime;

    #[test]
    fn tray_state_serializes_for_ipc() {
        let state = TrayState {
            providers: vec![ProviderSnapshot {
                provider: ProviderId::Claude,
                fetched_at: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
                windows: vec![UsageWindow {
                    id: "five_hour".into(),
                    label: "5-hour".into(),
                    kind: WindowKind::Percent { used: Some(42.0) },
                    resets_at: None,
                }],
                headline_percent: Some(42.0),
                stale: false,
                error: None,
            }],
            shared_mascot_fill: Some(42.0),
        };
        let v = serde_json::to_value(&state).expect("serialize");
        assert_eq!(v["shared_mascot_fill"], 42.0);
        assert!(v["providers"][0]["headline_percent"].as_f64().unwrap() > 40.0);
        assert_eq!(v["providers"][0]["stale"], false);
    }
}
```

`quota-tray/src-tauri/src/poll_loop.rs`:

```rust
use crate::state::AppState;
use quota_tray_core::icon::paint_tray_icon;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri::tray::TrayIcon;

pub fn spawn_poll_loop(app: AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let interval_secs = {
                let cfg = state.config.read();
                cfg.poll_interval_secs.clamp(60, 120)
            };
            let enabled = state.config.read().enabled.clone();
            let tick = state.poller.poll_once(&enabled);
            {
                let mut guard = state.tray_state.write();
                *guard = tick;
            }
            if let Some(tray) = app.tray_by_id("main") {
                apply_tray_icon(&tray, &state.tray_state.read());
            }
            let _ = app.emit("tray-state-updated", ());
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}

fn apply_tray_icon(tray: &TrayIcon, state: &quota_tray_core::TrayState) {
    let png = paint_tray_icon(state);
    if let Ok(icon) = tauri::image::Image::from_bytes(&png) {
        let _ = tray.set_icon(Some(icon));
    }
    let tooltip = state
        .providers
        .iter()
        .map(|p| {
            let name = match p.provider {
                quota_tray_core::ProviderId::Claude => "Claude",
                quota_tray_core::ProviderId::Cursor => "Cursor",
                quota_tray_core::ProviderId::Copilot => "Copilot",
                quota_tray_core::ProviderId::OpenAI => "OpenAI",
            };
            if p.stale {
                format!("{name} stale")
            } else if let Some(h) = p.headline_percent {
                format!("{name} {h:.0}%")
            } else {
                format!("{name} —")
            }
        })
        .collect::<Vec<_>>()
        .join(" · ");
    let _ = tray.set_tooltip(Some(&tooltip));
}
```

Task 5 must expose:

```rust
impl Poller {
    pub fn new(providers: Vec<Box<dyn Provider>>, config: AppConfig) -> Self;
    pub fn poll_once(&self, enabled: &[ProviderId]) -> TrayState;
}
```

`quota-tray/src-tauri/src/tray.rs`:

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, WebviewWindowBuilder, WebviewUrl,
};

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("Quota Tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_panel(app);
            }
        })
        .build(app)?;

    Ok(())
}

fn toggle_panel<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Quota Tray")
        .inner_size(360.0, 480.0)
        .decorations(false)
        .resizable(false)
        .visible(true)
        .build();
}
```

`quota-tray/src-tauri/src/lib.rs`:

```rust
mod commands;
mod poll_loop;
mod state;
mod tray;

use provider_claude::ClaudeProvider;
use quota_tray_core::{AppConfig, ClaudeHeadlineMetric, Poller, ProviderId};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig {
        poll_interval_secs: 60,
        enabled: vec![ProviderId::Claude],
        claude_headline: ClaudeHeadlineMetric::Highest,
    };
    let poller = Poller::new(
        vec![Box::new(ClaudeProvider::default())],
        config.clone(),
    );
    let app_state = Arc::new(AppState::new(config, poller));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            tray_state: app_state.tray_state.clone(),
            config: app_state.config.clone(),
            poller: app_state.poller.clone(),
        })
        .invoke_handler(tauri::generate_handler![commands::get_tray_state])
        .setup(move |app| {
            tray::setup_tray(app.handle())?;
            poll_loop::spawn_poll_loop(app.handle().clone(), app_state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Quota Tray");
}
```

First poll runs immediately: `spawn_poll_loop` polls at the top of the loop, then sleeps.

`quota-tray/src-tauri/capabilities/default.json` — allow tray + core:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for Quota Tray panel",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:tray:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:event:allow-listen",
    "core:event:allow-emit",
    "shell:allow-open"
  ]
}
```

`tauri.conf.json` — macOS menubar-style: `app.macOSPrivateApi` only if needed; set `"windows": []` or a hidden bootstrap window per Tauri 2 tray apps, and create `main` on demand in `tray.rs`. Prefer:

```json
{
  "productName": "Quota Tray",
  "identifier": "dev.quotatray.app",
  "build": {
    "frontendDist": "../ui/dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev --prefix ../ui",
    "beforeBuildCommand": "npm run build --prefix ../ui"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "label": "main",
        "title": "Quota Tray",
        "width": 360,
        "height": 480,
        "decorations": false,
        "resizable": false,
        "visible": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 5: Run unit test**

Run: `cd quota-tray && cargo test -p quota-tray tray_state_serializes_for_ipc -- --nocapture`

Expected: PASS

- [ ] **Step 6: Smoke-compile the app binary**

Run: `cd quota-tray/src-tauri && cargo check`

Expected: PASS (no errors). Manual: `cd quota-tray && npm run tauri dev` (or workspace script) shows menubar icon; click opens panel; DevTools/`invoke('get_tray_state')` returns JSON.

- [ ] **Step 7: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/src-tauri/src/state.rs \
  quota-tray/src-tauri/src/commands.rs \
  quota-tray/src-tauri/src/tray.rs \
  quota-tray/src-tauri/src/poll_loop.rs \
  quota-tray/src-tauri/src/lib.rs \
  quota-tray/src-tauri/Cargo.toml \
  quota-tray/src-tauri/capabilities/default.json \
  quota-tray/src-tauri/tauri.conf.json
git commit -m "$(cat <<'EOF'
feat(quota-tray): tray shell, get_tray_state IPC, background poll

Wire Tauri tray click to panel and refresh TrayState + icon on the poller interval.
EOF
)"
```

---

### Task 8: Svelte panel UI (hero + provider meters)

**Files:**
- Create: `quota-tray/ui/src/lib/types.ts`
- Create: `quota-tray/ui/src/lib/api.ts`
- Create: `quota-tray/ui/src/lib/components/Hero.svelte`
- Create: `quota-tray/ui/src/lib/components/ProviderRow.svelte`
- Create: `quota-tray/ui/src/lib/components/WindowGauge.svelte`
- Create: `quota-tray/ui/src/App.svelte`
- Create: `quota-tray/ui/src/app.css`
- Modify: `quota-tray/ui/src/main.ts`
- Modify: `quota-tray/ui/index.html`
- Test: `quota-tray/ui/src/lib/format.test.ts` (Vitest) — or if Vitest not scaffolded, add it in this task

**Interfaces:**
- Consumes: IPC `get_tray_state` → `TrayState`; event `tray-state-updated`; `ProviderSnapshot.windows` for Claude dual gauges
- Produces: Panel UI listing **enabled** providers only (whatever backend returns); cream/terracotta CSS vars; stale badge; never render missing `%` as live `0%`

- [ ] **Step 1: Add Vitest (if missing) and write failing format test**

`quota-tray/ui/package.json` scripts (merge):

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest run"
  },
  "devDependencies": {
    "vitest": "^3.0.0",
    "@sveltejs/vite-plugin-svelte": "^5.0.0",
    "svelte": "^5.0.0",
    "typescript": "^5.0.0",
    "vite": "^6.0.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0"
  }
}
```

`quota-tray/ui/src/lib/format.ts`:

```ts
/** Never treat null/undefined as a live 0%. */
export function formatHeadlinePercent(used: number | null | undefined): string {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return "—";
  }
  return `${Math.round(used)}%`;
}

export function meterWidthPercent(used: number | null | undefined): number | null {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return null;
  }
  return Math.min(100, Math.max(0, used));
}
```

`quota-tray/ui/src/lib/format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { formatHeadlinePercent, meterWidthPercent } from "./format";

describe("formatHeadlinePercent", () => {
  it("does not fake zero for missing", () => {
    expect(formatHeadlinePercent(null)).toBe("—");
    expect(formatHeadlinePercent(undefined)).toBe("—");
  });

  it("shows real zero", () => {
    expect(formatHeadlinePercent(0)).toBe("0%");
  });

  it("rounds used percent", () => {
    expect(formatHeadlinePercent(62.4)).toBe("62%");
  });
});

describe("meterWidthPercent", () => {
  it("returns null when unknown so CSS can hide fill", () => {
    expect(meterWidthPercent(null)).toBeNull();
  });
});
```

- [ ] **Step 2: Run test to verify Vitest works / implement passes**

Run: `cd quota-tray/ui && npm install && npm test`

Expected: PASS for format tests (implement `format.ts` in Step 1 before run if writing test-first: first run FAIL on missing module, then add `format.ts`, re-run PASS).

TDD sequence:

Run: `cd quota-tray/ui && npm test`

Expected (before `format.ts`): FAIL — cannot find module `./format`

Then add `format.ts` as above.

Run: `cd quota-tray/ui && npm test`

Expected: PASS

- [ ] **Step 3: Types + API bridge**

`quota-tray/ui/src/lib/types.ts`:

```ts
export type ProviderId = "Claude" | "Cursor" | "Copilot" | "OpenAI";

export type WindowKind =
  | { Percent: { used: number | null } }
  | { Currency: { used: number | null; limit: number | null; code: string } }
  | { Count: { used: number | null; limit: number | null } };

export interface UsageWindow {
  id: string;
  label: string;
  kind: WindowKind;
  resets_at: string | null;
}

export interface ProviderSnapshot {
  provider: ProviderId;
  fetched_at: string;
  windows: UsageWindow[];
  headline_percent: number | null;
  stale: boolean;
  error: string | null;
}

export interface TrayState {
  providers: ProviderSnapshot[];
  shared_mascot_fill: number | null;
}
```

Task 2 must serialize `WindowKind` as externally tagged (serde default), matching the TS union above (`{ "Percent": { "used": 42.0 } }`).

`quota-tray/ui/src/lib/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { TrayState } from "./types";

export async function fetchTrayState(): Promise<TrayState> {
  return invoke<TrayState>("get_tray_state");
}

export function onTrayStateUpdated(cb: () => void): Promise<() => void> {
  return listen("tray-state-updated", () => cb()).then((unlisten) => unlisten);
}
```

- [ ] **Step 4: Cream / terracotta CSS + components**

`quota-tray/ui/src/app.css`:

```css
:root {
  --cream: #f3ebe1;
  --cream-deep: #e8dccf;
  --ink: #2c241c;
  --ink-muted: #6b5d50;
  --terracotta: #c96442;
  --terracotta-soft: #d9896a;
  --pale: #faf6f1;
  --danger: #a33b2a;
  --chip-claude: #c96442;
  --chip-cursor: #5a7a6a;
  --chip-copilot: #3d5a80;
  --radius: 10px;
  --font: "Iowan Old Style", "Palatino Linotype", Palatino, "Book Antiqua", Georgia, serif;
  --font-ui: "Avenir Next", "Segoe UI", sans-serif;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  min-height: 100%;
  background: var(--cream);
  color: var(--ink);
  font-family: var(--font-ui);
}

body {
  background:
    radial-gradient(120% 80% at 0% 0%, #f7f0e8 0%, transparent 55%),
    radial-gradient(90% 70% at 100% 20%, #eadfce 0%, transparent 50%),
    var(--cream);
}
```

`quota-tray/ui/src/lib/components/Hero.svelte`:

```svelte
<script lang="ts">
  import { formatHeadlinePercent } from "../format";

  interface Props {
    fill: number | null;
    staleGlobal: boolean;
  }
  let { fill, staleGlobal }: Props = $props();

  const label = $derived(formatHeadlinePercent(fill));
  const status = $derived(
    staleGlobal
      ? "Something’s stale"
      : fill == null
        ? "Waiting for a reading"
        : fill >= 80
          ? "You’re tight"
          : "You’re fine"
  );
  const bodyFill = $derived(fill == null ? 0 : Math.min(100, Math.max(0, fill)));
</script>

<header class="hero">
  <div class="mascot" class:muted={staleGlobal || fill == null} aria-hidden="true">
    <div class="mascot-body">
      <div class="mascot-fill" style={`height: ${bodyFill}%`}></div>
    </div>
  </div>
  <div class="copy">
    <p class="brand">Quota Tray</p>
    <p class="headline">{label}</p>
    <p class="status">{status}</p>
  </div>
</header>

<style>
  .hero {
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: 1rem;
    align-items: center;
    padding: 1.25rem 1.25rem 0.75rem;
  }
  .brand {
    margin: 0;
    font-family: var(--font);
    font-size: 1.35rem;
    letter-spacing: -0.02em;
  }
  .headline {
    margin: 0.15rem 0;
    font-size: 1.75rem;
    font-weight: 650;
  }
  .status {
    margin: 0;
    color: var(--ink-muted);
    font-size: 0.9rem;
  }
  .mascot-body {
    width: 84px;
    height: 96px;
    border-radius: 42% 42% 38% 38%;
    background: var(--pale);
    border: 2px solid var(--cream-deep);
    position: relative;
    overflow: hidden;
  }
  .mascot-fill {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--terracotta);
    transition: height 240ms ease;
  }
  .mascot.muted .mascot-fill {
    background: var(--terracotta-soft);
    opacity: 0.45;
  }
</style>
```

`quota-tray/ui/src/lib/components/WindowGauge.svelte`:

```svelte
<script lang="ts">
  import type { UsageWindow } from "../types";
  import { formatHeadlinePercent, meterWidthPercent } from "../format";

  interface Props {
    window: UsageWindow;
  }
  let { window: w }: Props = $props();

  const used = $derived(
    "Percent" in w.kind ? w.kind.Percent.used : null
  );
  const width = $derived(meterWidthPercent(used));
</script>

<div class="gauge">
  <div class="row">
    <span>{w.label}</span>
    <span>{formatHeadlinePercent(used)}</span>
  </div>
  <div class="track" aria-hidden="true">
    {#if width != null}
      <div class="fill" style={`width: ${width}%`}></div>
    {:else}
      <div class="unknown"></div>
    {/if}
  </div>
  {#if w.resets_at}
    <p class="reset">Resets {w.resets_at}</p>
  {/if}
</div>

<style>
  .gauge {
    margin-top: 0.4rem;
  }
  .row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--ink-muted);
  }
  .track {
    height: 8px;
    background: var(--cream-deep);
    border-radius: 999px;
    overflow: hidden;
    margin-top: 0.25rem;
  }
  .fill {
    height: 100%;
    background: var(--terracotta);
  }
  .unknown {
    height: 100%;
    width: 100%;
    background: repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 4px,
      rgba(0, 0, 0, 0.06) 4px,
      rgba(0, 0, 0, 0.06) 8px
    );
  }
  .reset {
    margin: 0.2rem 0 0;
    font-size: 0.72rem;
    color: var(--ink-muted);
  }
</style>
```

`quota-tray/ui/src/lib/components/ProviderRow.svelte`:

```svelte
<script lang="ts">
  import type { ProviderSnapshot } from "../types";
  import { formatHeadlinePercent, meterWidthPercent } from "../format";
  import WindowGauge from "./WindowGauge.svelte";

  interface Props {
    snap: ProviderSnapshot;
  }
  let { snap }: Props = $props();

  const name = $derived(
    snap.provider === "Claude"
      ? "Claude"
      : snap.provider === "Cursor"
        ? "Cursor"
        : snap.provider === "Copilot"
          ? "Copilot"
          : "OpenAI"
  );
  const chip = $derived(
    snap.provider === "Claude"
      ? "var(--chip-claude)"
      : snap.provider === "Cursor"
        ? "var(--chip-cursor)"
        : "var(--chip-copilot)"
  );
  const width = $derived(meterWidthPercent(snap.headline_percent));
</script>

<article class="row">
  <div class="top">
    <span class="chip" style={`background: ${chip}`}>{name}</span>
    <span class="pct">{formatHeadlinePercent(snap.headline_percent)}</span>
    {#if snap.stale}
      <span class="badge">Stale</span>
    {/if}
  </div>
  <div class="track">
    {#if width != null && !snap.stale}
      <div class="fill" style={`width: ${width}%`}></div>
    {:else if width != null && snap.stale}
      <div class="fill muted" style={`width: ${width}%`}></div>
    {:else}
      <div class="unknown"></div>
    {/if}
  </div>
  {#if snap.error}
    <p class="err">{snap.error}</p>
  {/if}
  {#if snap.windows.length > 1}
    <div class="windows">
      {#each snap.windows as w (w.id)}
        <WindowGauge window={w} />
      {/each}
    </div>
  {:else if snap.windows.length === 1}
    <WindowGauge window={snap.windows[0]} />
  {/if}
</article>

<style>
  .row {
    padding: 0.85rem 1.25rem;
    border-top: 1px solid var(--cream-deep);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .chip {
    color: #fff;
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    border-radius: 6px;
  }
  .pct {
    font-weight: 650;
    margin-left: auto;
  }
  .badge {
    font-size: 0.7rem;
    color: var(--danger);
    border: 1px solid var(--danger);
    border-radius: 4px;
    padding: 0.05rem 0.35rem;
  }
  .track {
    margin-top: 0.45rem;
    height: 10px;
    background: var(--cream-deep);
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--terracotta);
  }
  .fill.muted {
    opacity: 0.45;
  }
  .unknown {
    height: 100%;
    width: 100%;
    background: repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 4px,
      rgba(0, 0, 0, 0.06) 4px,
      rgba(0, 0, 0, 0.06) 8px
    );
  }
  .err {
    margin: 0.35rem 0 0;
    color: var(--danger);
    font-size: 0.8rem;
  }
  .windows {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }
</style>
```

`quota-tray/ui/src/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import Hero from "./lib/components/Hero.svelte";
  import ProviderRow from "./lib/components/ProviderRow.svelte";
  import { fetchTrayState, onTrayStateUpdated } from "./lib/api";
  import type { TrayState } from "./lib/types";

  let state = $state<TrayState | null>(null);
  let loadError = $state<string | null>(null);

  async function refresh() {
    try {
      state = await fetchTrayState();
      loadError = null;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(() => {
    refresh();
    let unlisten: (() => void) | undefined;
    onTrayStateUpdated(() => {
      refresh();
    }).then((u) => {
      unlisten = u;
    });
    return () => unlisten?.();
  });

  const staleGlobal = $derived(
    !!state &&
      (state.providers.length === 0 ||
        state.providers.every((p) => p.stale) ||
        state.shared_mascot_fill == null)
  );
</script>

<main>
  <Hero fill={state?.shared_mascot_fill ?? null} staleGlobal={staleGlobal} />
  {#if loadError}
    <p class="banner">{loadError}</p>
  {/if}
  {#if state && state.providers.length === 0}
    <p class="banner">Enable a provider in Settings to start polling.</p>
  {/if}
  {#if state}
    {#each state.providers as snap (snap.provider)}
      <ProviderRow {snap} />
    {/each}
  {/if}
  <footer>
    <a href="/settings.html">Settings</a>
    <button type="button" onclick={refresh}>Refresh</button>
  </footer>
</main>

<style>
  footer {
    display: flex;
    justify-content: space-between;
    padding: 0.75rem 1.25rem 1.25rem;
    color: var(--ink-muted);
    font-size: 0.85rem;
  }
  button {
    background: transparent;
    border: 1px solid var(--cream-deep);
    border-radius: 6px;
    color: var(--ink);
    padding: 0.25rem 0.6rem;
    cursor: pointer;
  }
  .banner {
    margin: 0;
    padding: 0.5rem 1.25rem;
    color: var(--ink-muted);
    font-size: 0.9rem;
  }
  a {
    color: var(--terracotta);
  }
</style>
```

`quota-tray/ui/src/main.ts`:

```ts
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

mount(App, { target: document.getElementById("app")! });
```

- [ ] **Step 5: Build UI**

Run: `cd quota-tray/ui && npm run build`

Expected: PASS — `dist/` emitted with no purple theme classes.

- [ ] **Step 6: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/ui
git commit -m "$(cat <<'EOF'
feat(quota-tray): Svelte panel with hero mascot and provider meters

Cream/terracotta panel lists enabled providers, dual Claude windows, and honest stale/missing percents.
EOF
)"
```

---

### Task 9: Settings (providers, poll interval, Claude headline)

**Files:**
- Create: `quota-tray/src-tauri/src/config_store.rs`
- Modify: `quota-tray/src-tauri/src/commands.rs`
- Modify: `quota-tray/src-tauri/src/lib.rs`
- Create: `quota-tray/ui/src/settings.ts`
- Create: `quota-tray/ui/src/SettingsApp.svelte`
- Create: `quota-tray/ui/settings.html`
- Modify: `quota-tray/ui/vite.config.ts` (multi-page)
- Test: `quota-tray/crates/core/src/config.rs` (clamp helpers — if not in Task 2, add here)
- Test: `quota-tray/ui/src/lib/settings_validate.test.ts`

**Interfaces:**
- Consumes: `AppConfig { poll_interval_secs: u64, enabled: Vec<ProviderId>, claude_headline: ClaudeHeadlineMetric }`
- Produces: IPC `get_config() -> AppConfig`, `set_config(AppConfig) -> Result<AppConfig, String>` with clamp `poll_interval_secs` to `60..=120`; settings UI toggles; persistence under app data dir `config.json`

- [ ] **Step 1: Write failing core clamp test (if missing)**

`quota-tray/crates/core/src/config.rs` (extend Task 2 types):

```rust
use crate::{ClaudeHeadlineMetric, ProviderId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub poll_interval_secs: u64,
    pub enabled: Vec<ProviderId>,
    pub claude_headline: ClaudeHeadlineMetric,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 60,
            enabled: vec![ProviderId::Claude],
            claude_headline: ClaudeHeadlineMetric::Highest,
        }
    }
}

impl AppConfig {
    pub fn clamp_poll_interval(&mut self) {
        self.poll_interval_secs = self.poll_interval_secs.clamp(60, 120);
    }

    pub fn sanitized(mut self) -> Self {
        self.clamp_poll_interval();
        self.enabled.sort_by_key(|id| match id {
            ProviderId::Claude => 0,
            ProviderId::Cursor => 1,
            ProviderId::Copilot => 2,
            ProviderId::OpenAI => 3,
        });
        self.enabled.dedup();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_poll_interval_rejects_below_60_and_above_120() {
        let mut low = AppConfig {
            poll_interval_secs: 15,
            ..AppConfig::default()
        };
        low.clamp_poll_interval();
        assert_eq!(low.poll_interval_secs, 60);

        let mut high = AppConfig {
            poll_interval_secs: 999,
            ..AppConfig::default()
        };
        high.clamp_poll_interval();
        assert_eq!(high.poll_interval_secs, 120);
    }
}
```

Export from `quota-tray/crates/core/src/lib.rs`: `pub use config::AppConfig;` when `AppConfig` lives in `config.rs`. If Task 2 already defined `AppConfig` in another module, add `clamp_poll_interval` / `sanitized` there instead of duplicating the struct.

- [ ] **Step 2: Run failing/passing clamp test**

Run: `cd quota-tray && cargo test -p quota-tray-core clamp_poll_interval -- --nocapture`

Expected: FAIL before implementation, PASS after `clamp_poll_interval` exists.

- [ ] **Step 3: Persist + IPC**

`quota-tray/src-tauri/src/config_store.rs`:

```rust
use quota_tray_core::AppConfig;
use std::fs;
use std::path::PathBuf;

pub fn config_path(app_data: PathBuf) -> PathBuf {
    app_data.join("config.json")
}

pub fn load(app_data: PathBuf) -> AppConfig {
    let path = config_path(app_data);
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str::<AppConfig>(&raw)
            .unwrap_or_default()
            .sanitized(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save(app_data: PathBuf, config: &AppConfig) -> Result<(), String> {
    fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    let path = config_path(app_data);
    let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}
```

Append to `commands.rs`:

```rust
use quota_tray_core::AppConfig;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.read().clone())
}

#[tauri::command]
pub fn set_config(
    app: AppHandle,
    state: State<'_, AppState>,
    mut config: AppConfig,
) -> Result<AppConfig, String> {
    config = config.sanitized();
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    crate::config_store::save(data_dir, &config)?;
    *state.config.write() = config.clone();
    Ok(config)
}
```

Register in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::get_tray_state,
    commands::get_config,
    commands::set_config,
])
```

Load config on startup in `run()` via `config_store::load` once `AppHandle` path is available inside `.setup`, then replace managed config.

- [ ] **Step 4: UI validation test + Settings page**

`quota-tray/ui/src/lib/settings_validate.ts`:

```ts
export function clampPollInterval(secs: number): number {
  if (!Number.isFinite(secs)) return 60;
  return Math.min(120, Math.max(60, Math.round(secs)));
}
```

`quota-tray/ui/src/lib/settings_validate.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { clampPollInterval } from "./settings_validate";

describe("clampPollInterval", () => {
  it("clamps to 60..120", () => {
    expect(clampPollInterval(10)).toBe(60);
    expect(clampPollInterval(60)).toBe(60);
    expect(clampPollInterval(90)).toBe(90);
    expect(clampPollInterval(120)).toBe(120);
    expect(clampPollInterval(500)).toBe(120);
  });
});
```

Run: `cd quota-tray/ui && npm test`

Expected: PASS

`quota-tray/ui/src/SettingsApp.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { clampPollInterval } from "./lib/settings_validate";

  type ProviderId = "Claude" | "Cursor" | "Copilot" | "OpenAI";
  type ClaudeHeadline = "FiveHour" | "SevenDay" | "Highest";

  interface AppConfig {
    poll_interval_secs: number;
    enabled: ProviderId[];
    claude_headline: ClaudeHeadline;
  }

  let config = $state<AppConfig>({
    poll_interval_secs: 60,
    enabled: ["Claude"],
    claude_headline: "Highest",
  });
  let saved = $state(false);
  let error = $state<string | null>(null);

  const allProviders: ProviderId[] = ["Claude", "Cursor", "Copilot"];

  onMount(async () => {
    config = await invoke<AppConfig>("get_config");
  });

  function toggle(id: ProviderId, on: boolean) {
    if (on && !config.enabled.includes(id)) {
      config = { ...config, enabled: [...config.enabled, id] };
    } else if (!on) {
      config = { ...config, enabled: config.enabled.filter((x) => x !== id) };
    }
  }

  async function save() {
    try {
      const next = {
        ...config,
        poll_interval_secs: clampPollInterval(config.poll_interval_secs),
      };
      config = await invoke<AppConfig>("set_config", { config: next });
      saved = true;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<main class="settings">
  <h1>Settings</h1>
  <p class="blurb">Credentials stay on this Mac. We only call vendor APIs.</p>

  <section>
    <h2>Providers</h2>
    {#each allProviders as id}
      <label>
        <input
          type="checkbox"
          checked={config.enabled.includes(id)}
          onchange={(e) => toggle(id, e.currentTarget.checked)}
        />
        {id}
      </label>
    {/each}
  </section>

  <section>
    <h2>Poll interval (seconds)</h2>
    <input
      type="number"
      min="60"
      max="120"
      bind:value={config.poll_interval_secs}
    />
    <p class="hint">Allowed range 60–120. Default 60.</p>
  </section>

  <section>
    <h2>Claude headline metric</h2>
    <select bind:value={config.claude_headline}>
      <option value="Highest">Highest (max of 5h / 7d)</option>
      <option value="FiveHour">Five-hour window</option>
      <option value="SevenDay">Seven-day window</option>
    </select>
  </section>

  <button type="button" onclick={save}>Save</button>
  {#if saved}<p class="ok">Saved.</p>{/if}
  {#if error}<p class="err">{error}</p>{/if}
  <p><a href="/index.html">Back to panel</a></p>
</main>

<style>
  .settings {
    padding: 1.25rem;
    font-family: var(--font-ui);
    color: var(--ink);
  }
  h1 {
    font-family: var(--font);
  }
  section {
    margin: 1rem 0;
  }
  label {
    display: block;
    margin: 0.35rem 0;
  }
  .hint,
  .blurb {
    color: var(--ink-muted);
    font-size: 0.85rem;
  }
  .ok {
    color: var(--terracotta);
  }
  .err {
    color: var(--danger);
  }
  button {
    background: var(--terracotta);
    color: #fff;
    border: 0;
    border-radius: 8px;
    padding: 0.45rem 0.9rem;
    cursor: pointer;
  }
</style>
```

`quota-tray/ui/settings.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Quota Tray Settings</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/settings.ts"></script>
  </body>
</html>
```

`quota-tray/ui/src/settings.ts`:

```ts
import "./app.css";
import { mount } from "svelte";
import SettingsApp from "./SettingsApp.svelte";

mount(SettingsApp, { target: document.getElementById("app")! });
```

`quota-tray/ui/vite.config.ts`:

```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        settings: resolve(__dirname, "settings.html"),
      },
    },
  },
});
```

Panel footer links to `settings.html` in the same webview (`<a href="settings.html">`). Settings page links back with `<a href="index.html">`.

- [ ] **Step 5: Run Rust + UI tests**

Run: `cd quota-tray && cargo test -p quota-tray-core clamp_poll_interval -- --nocapture`

Expected: PASS

Run: `cd quota-tray/ui && npm test`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/crates/core quota-tray/src-tauri/src/config_store.rs \
  quota-tray/src-tauri/src/commands.rs quota-tray/src-tauri/src/lib.rs \
  quota-tray/ui
git commit -m "$(cat <<'EOF'
feat(quota-tray): settings for providers, poll interval, Claude headline

Persist AppConfig locally with 60..=120 clamp and enabled-provider toggles.
EOF
)"
```

---

### Task 10: Cursor provider (`state.vscdb` + GetCurrentPeriodUsage)

**Files:**
- Create: `quota-tray/crates/provider-cursor/Cargo.toml`
- Create: `quota-tray/crates/provider-cursor/src/lib.rs`
- Create: `quota-tray/crates/provider-cursor/src/credentials.rs`
- Create: `quota-tray/crates/provider-cursor/src/fetch.rs`
- Create: `quota-tray/crates/provider-cursor/src/headline.rs`
- Modify: `quota-tray/Cargo.toml` (workspace member)
- Modify: `quota-tray/src-tauri/Cargo.toml` + `lib.rs` (register provider)
- Modify: `quota-tray/crates/core/src/registry.rs` (or `built_in_providers`) if that helper exists from Task 5
- Test: `quota-tray/crates/provider-cursor/src/headline.rs`
- Test: `quota-tray/crates/provider-cursor/src/credentials.rs`
- Test: `quota-tray/crates/provider-cursor/src/fetch.rs` (wiremock)

**Interfaces:**
- Consumes: `quota_tray_core::{Provider, ProviderId, Credentials, CredentialError, FetchError, ProviderSnapshot, UsageWindow, WindowKind}`
- Produces: `CursorProvider` implementing `Provider`; `headline_percent = totalPercentUsed ?? max(auto, api) ?? spend/limit cents`; creds from SQLite `ItemTable` key `cursorAuth/accessToken` (readonly)

- [ ] **Step 1: Scaffold crate + failing headline tests**

`quota-tray/crates/provider-cursor/Cargo.toml`:

```toml
[package]
name = "provider-cursor"
version = "0.1.0"
edition = "2021"

[dependencies]
quota-tray-core = { path = "../core" }
rusqlite = { version = "0.32", features = ["bundled"] }
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
secrecy = { version = "0.10", features = ["serde"] }
thiserror = "2"
time = { version = "0.3", features = ["serde", "parsing"] }
dirs = "6"

[dev-dependencies]
tempfile = "3"
wiremock = "0.6"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Add to workspace `quota-tray/Cargo.toml` members: `"crates/provider-cursor"`.

`quota-tray/crates/provider-cursor/src/headline.rs`:

```rust
use serde_json::Value;

/// Spec formula:
/// totalPercentUsed ?? max(autoPercentUsed, apiPercentUsed) ?? (limit>0 ? 100*totalSpend/limit : None)
pub fn cursor_headline_percent(plan_usage: &Value) -> Option<f64> {
    if let Some(v) = plan_usage.get("totalPercentUsed").and_then(|x| x.as_f64()) {
        return Some(v);
    }
    let auto = plan_usage.get("autoPercentUsed").and_then(|x| x.as_f64());
    let api = plan_usage.get("apiPercentUsed").and_then(|x| x.as_f64());
    match (auto, api) {
        (Some(a), Some(b)) => return Some(a.max(b)),
        (Some(a), None) => return Some(a),
        (None, Some(b)) => return Some(b),
        (None, None) => {}
    }
    let total_spend = plan_usage.get("totalSpend").and_then(|x| x.as_f64());
    let limit = plan_usage.get("limit").and_then(|x| x.as_f64());
    match (total_spend, limit) {
        (Some(spend), Some(lim)) if lim > 0.0 => Some(100.0 * spend / lim),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prefers_total_percent_used() {
        let v = json!({
            "totalPercentUsed": 41.0,
            "autoPercentUsed": 10.0,
            "apiPercentUsed": 90.0
        });
        assert_eq!(cursor_headline_percent(&v), Some(41.0));
    }

    #[test]
    fn falls_back_to_max_auto_api() {
        let v = json!({
            "autoPercentUsed": 10.0,
            "apiPercentUsed": 55.0
        });
        assert_eq!(cursor_headline_percent(&v), Some(55.0));
    }

    #[test]
    fn falls_back_to_spend_over_limit_cents() {
        let v = json!({ "totalSpend": 250.0, "limit": 1000.0 });
        assert_eq!(cursor_headline_percent(&v), Some(25.0));
    }

    #[test]
    fn none_when_no_signal() {
        let v = json!({});
        assert_eq!(cursor_headline_percent(&v), None);
    }
}
```

- [ ] **Step 2: Run headline tests (fail then pass)**

Run: `cd quota-tray && cargo test -p provider-cursor cursor_headline -- --nocapture`

Expected: FAIL until `headline.rs` is in `lib.rs` (`mod headline;`), then PASS.

Wire `lib.rs` minimally:

```rust
mod credentials;
mod fetch;
mod headline;

pub use headline::cursor_headline_percent;
```

- [ ] **Step 3: Credential SQLite reader tests**

`quota-tray/crates/provider-cursor/src/credentials.rs`:

```rust
use quota_tray_core::{CredentialError, Credentials};
use rusqlite::Connection;
use secrecy::SecretString;
use std::path::{Path, PathBuf};

pub fn default_state_vscdb_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library/Application Support/Cursor/User/globalStorage/state.vscdb")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config/Cursor/User/globalStorage/state.vscdb")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_default()
            .join("Cursor/User/globalStorage/state.vscdb")
    }
}

pub fn read_access_token_from_db(path: &Path) -> Result<Credentials, CredentialError> {
    // Immutable / read-only to coexist with Cursor's WAL lock.
    let uri = format!("file:{}?mode=ro", path.display());
    let conn = Connection::open_with_flags(
        &uri,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(|e| CredentialError::Unavailable(e.to_string()))?;

    let token: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            ["cursorAuth/accessToken"],
            |row| row.get(0),
        )
        .map_err(|e| CredentialError::Unavailable(e.to_string()))?;

    if token.trim().is_empty() {
        return Err(CredentialError::Unavailable(
            "cursorAuth/accessToken empty".into(),
        ));
    }
    Ok(Credentials {
        raw: SecretString::from(token),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use secrecy::ExposeSecret;

    #[test]
    fn reads_token_from_itemtable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.vscdb");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT);
                 INSERT INTO ItemTable(key, value) VALUES ('cursorAuth/accessToken', 'jwt-test-token');",
            )
            .unwrap();
        }
        let creds = read_access_token_from_db(&path).unwrap();
        assert_eq!(creds.raw.expose_secret(), "jwt-test-token");
    }
}
```

Task 2/3 must define `CredentialError::{Unavailable(String), Invalid(String)}` (or rename call sites here to those exact variants).

Run: `cd quota-tray && cargo test -p provider-cursor reads_token_from_itemtable -- --nocapture`

Expected: PASS

- [ ] **Step 4: Mock HTTP fetch + map snapshot**

`quota-tray/crates/provider-cursor/src/fetch.rs`:

```rust
use crate::headline::cursor_headline_percent;
use quota_tray_core::{
    FetchError, ProviderId, ProviderSnapshot, UsageWindow, WindowKind,
};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use time::OffsetDateTime;

pub struct CursorClient {
    pub base_url: String,
    pub http: reqwest::blocking::Client,
}

impl Default for CursorClient {
    fn default() -> Self {
        Self {
            base_url: "https://api2.cursor.sh".into(),
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl CursorClient {
    pub fn fetch_usage(&self, token: &SecretString) -> Result<ProviderSnapshot, FetchError> {
        let url = format!(
            "{}/aiserver.v1.DashboardService/GetCurrentPeriodUsage",
            self.base_url.trim_end_matches('/')
        );
        let res = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", token.expose_secret()))
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(FetchError::Http {
                status: res.status().as_u16(),
                body: res.text().unwrap_or_default(),
            });
        }

        let body: Value = res
            .json()
            .map_err(|e| FetchError::Parse(e.to_string()))?;
        parse_usage_body(&body)
    }
}

pub fn parse_usage_body(body: &Value) -> Result<ProviderSnapshot, FetchError> {
    let plan = body
        .get("planUsage")
        .cloned()
        .unwrap_or_else(|| body.clone());

    let headline = cursor_headline_percent(&plan);
    let mut windows = Vec::new();

    if let Some(auto) = plan.get("autoPercentUsed").and_then(|v| v.as_f64()) {
        windows.push(UsageWindow {
            id: "auto".into(),
            label: "Auto / Composer".into(),
            kind: WindowKind::Percent { used: Some(auto) },
            resets_at: None,
        });
    }
    if let Some(api) = plan.get("apiPercentUsed").and_then(|v| v.as_f64()) {
        windows.push(UsageWindow {
            id: "api".into(),
            label: "API".into(),
            kind: WindowKind::Percent { used: Some(api) },
            resets_at: None,
        });
    }
    if windows.is_empty() {
        windows.push(UsageWindow {
            id: "period".into(),
            label: "Current period".into(),
            kind: WindowKind::Percent { used: headline },
            resets_at: None,
        });
    }

    // Defensive: empty object with no usable fields is a parse failure, not fake zeros.
    if headline.is_none()
        && plan.get("totalPercentUsed").is_none()
        && plan.get("autoPercentUsed").is_none()
        && plan.get("apiPercentUsed").is_none()
        && plan.get("totalSpend").is_none()
    {
        return Err(FetchError::Parse(
            "GetCurrentPeriodUsage: no planUsage percent/spend fields".into(),
        ));
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::Cursor,
        fetched_at: OffsetDateTime::now_utc(),
        windows,
        headline_percent: headline,
        stale: false,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetch_maps_plan_usage() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/aiserver.v1.DashboardService/GetCurrentPeriodUsage",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "planUsage": {
                    "totalPercentUsed": 41.5,
                    "autoPercentUsed": 20.0,
                    "apiPercentUsed": 41.5
                }
            })))
            .mount(&server)
            .await;

        // blocking client inside async test via spawn_blocking
        let base = server.uri();
        let snap = tokio::task::spawn_blocking(move || {
            let client = CursorClient {
                base_url: base,
                http: reqwest::blocking::Client::new(),
            };
            client.fetch_usage(&SecretString::from("tok"))
        })
        .await
        .unwrap()
        .unwrap();

        assert_eq!(snap.provider, ProviderId::Cursor);
        assert_eq!(snap.headline_percent, Some(41.5));
        assert!(!snap.stale);
        assert!(snap.windows.len() >= 2);
    }
}
```

Task 4 must define `FetchError::{Network(String), Http { status: u16, body: String }, Parse(String)}`.

- [ ] **Step 5: Implement `CursorProvider` trait object**

`quota-tray/crates/provider-cursor/src/lib.rs`:

```rust
mod credentials;
mod fetch;
mod headline;

pub use credentials::{default_state_vscdb_path, read_access_token_from_db};
pub use fetch::{parse_usage_body, CursorClient};
pub use headline::cursor_headline_percent;

use credentials::{default_state_vscdb_path, read_access_token_from_db};
use fetch::CursorClient;
use quota_tray_core::{
    CredentialError, Credentials, FetchError, Provider, ProviderId, ProviderSnapshot,
};
use std::path::PathBuf;

pub struct CursorProvider {
    pub db_path: PathBuf,
    pub client: CursorClient,
}

impl Default for CursorProvider {
    fn default() -> Self {
        Self {
            db_path: default_state_vscdb_path(),
            client: CursorClient::default(),
        }
    }
}

impl Provider for CursorProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Cursor
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        read_access_token_from_db(&self.db_path)
    }

    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        self.client.fetch_usage(&creds.raw)
    }
}
```

Register in Tauri `lib.rs` poller providers:

```rust
use provider_cursor::CursorProvider;

let poller = Poller::new(
    vec![
        Box::new(ClaudeProvider::default()),
        Box::new(CursorProvider::default()),
    ],
    config.clone(),
);
```

Add `provider-cursor` dependency to `src-tauri/Cargo.toml`.

- [ ] **Step 6: Run all Cursor crate tests**

Run: `cd quota-tray && cargo test -p provider-cursor -- --nocapture`

Expected: PASS (headline, sqlite, wiremock)

- [ ] **Step 7: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/Cargo.toml quota-tray/crates/provider-cursor \
  quota-tray/src-tauri/Cargo.toml quota-tray/src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(quota-tray): Cursor provider via state.vscdb and usage RPC

Read access token readonly from SQLite; map GetCurrentPeriodUsage into ProviderSnapshot with spec headline formula.
EOF
)"
```

---

### Task 11: Copilot provider (`apps.json` + copilot_internal/user)

**Files:**
- Create: `quota-tray/crates/provider-copilot/Cargo.toml`
- Create: `quota-tray/crates/provider-copilot/src/lib.rs`
- Create: `quota-tray/crates/provider-copilot/src/credentials.rs`
- Create: `quota-tray/crates/provider-copilot/src/fetch.rs`
- Create: `quota-tray/crates/provider-copilot/src/headline.rs`
- Modify: `quota-tray/Cargo.toml`
- Modify: `quota-tray/src-tauri/Cargo.toml` + `lib.rs`
- Test: `quota-tray/crates/provider-copilot/src/headline.rs`
- Test: `quota-tray/crates/provider-copilot/src/credentials.rs`
- Test: `quota-tray/crates/provider-copilot/src/fetch.rs`

**Interfaces:**
- Consumes: same `Provider` trait
- Produces: `CopilotProvider`; token from `~/.config/github-copilot/apps.json` (Win: `%LOCALAPPDATA%\github-copilot\`); `headline_percent = 100 - premium_interactions.percent_remaining` (skip `unlimited`); free-plan fallback `max(100 - chat, 100 - completions)`

- [ ] **Step 1: Scaffold + failing headline tests**

`quota-tray/crates/provider-copilot/Cargo.toml`:

```toml
[package]
name = "provider-copilot"
version = "0.1.0"
edition = "2021"

[dependencies]
quota-tray-core = { path = "../core" }
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
secrecy = { version = "0.10", features = ["serde"] }
thiserror = "2"
time = { version = "0.3", features = ["serde", "parsing"] }
dirs = "6"

[dev-dependencies]
tempfile = "3"
wiremock = "0.6"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Workspace member: `"crates/provider-copilot"`.

`quota-tray/crates/provider-copilot/src/headline.rs`:

```rust
use serde_json::Value;

fn used_from_bucket(bucket: &Value) -> Option<f64> {
    if bucket.get("unlimited").and_then(|v| v.as_bool()) == Some(true) {
        return None;
    }
    let remaining = bucket.get("percent_remaining")?.as_f64()?;
    Some(100.0 - remaining)
}

/// Prefer premium_interactions; else max(chat, completions); skip unlimited buckets.
pub fn copilot_headline_percent(quota_snapshots: &Value) -> Option<f64> {
    if let Some(premium) = quota_snapshots.get("premium_interactions") {
        if let Some(u) = used_from_bucket(premium) {
            return Some(u);
        }
    }
    let chat = quota_snapshots
        .get("chat")
        .and_then(used_from_bucket);
    let completions = quota_snapshots
        .get("completions")
        .and_then(used_from_bucket);
    match (chat, completions) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn premium_remaining_inverts_to_used() {
        let q = json!({
            "premium_interactions": { "percent_remaining": 40.0, "unlimited": false },
            "chat": { "percent_remaining": 100.0, "unlimited": true }
        });
        assert_eq!(copilot_headline_percent(&q), Some(60.0));
    }

    #[test]
    fn skips_unlimited_premium_and_uses_free_buckets() {
        let q = json!({
            "premium_interactions": { "percent_remaining": 100.0, "unlimited": true },
            "chat": { "percent_remaining": 70.0, "unlimited": false },
            "completions": { "percent_remaining": 50.0, "unlimited": false }
        });
        assert_eq!(copilot_headline_percent(&q), Some(50.0));
    }

    #[test]
    fn none_when_all_unlimited_or_missing() {
        let q = json!({
            "chat": { "unlimited": true },
            "completions": { "unlimited": true }
        });
        assert_eq!(copilot_headline_percent(&q), None);
    }
}
```

- [ ] **Step 2: Run headline tests**

Run: `cd quota-tray && cargo test -p provider-copilot copilot_headline -- --nocapture`

Expected: FAIL then PASS once module is wired.

- [ ] **Step 3: Parse `apps.json` credentials**

`quota-tray/crates/provider-copilot/src/credentials.rs`:

```rust
use quota_tray_core::{CredentialError, Credentials};
use secrecy::SecretString;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub fn default_apps_json_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("github-copilot/apps.json")
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config/github-copilot/apps.json")
    }
}

pub fn parse_apps_json(raw: &str) -> Result<Credentials, CredentialError> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| CredentialError::Invalid(e.to_string()))?;
    // Shape: { "github.com:Iv1....": { "oauth_token": "gho_..." }, ... }
    if let Some(obj) = v.as_object() {
        for (_host, entry) in obj {
            if let Some(token) = entry.get("oauth_token").and_then(|t| t.as_str()) {
                if token.starts_with("gho_") || token.starts_with("ghu_") || !token.is_empty() {
                    return Ok(Credentials {
                        raw: SecretString::from(token.to_string()),
                    });
                }
            }
        }
    }
    Err(CredentialError::Unavailable(
        "no oauth_token in github-copilot apps.json".into(),
    ))
}

pub fn read_apps_json(path: &Path) -> Result<Credentials, CredentialError> {
    let raw = fs::read_to_string(path)
        .map_err(|e| CredentialError::Unavailable(e.to_string()))?;
    parse_apps_json(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    #[test]
    fn parses_oauth_token_from_apps_json() {
        let raw = r#"{
          "github.com:Iv1.b507a08c87ecfe23": {
            "user": "octo",
            "oauth_token": "gho_testtoken123"
          }
        }"#;
        let creds = parse_apps_json(raw).unwrap();
        assert_eq!(creds.raw.expose_secret(), "gho_testtoken123");
    }
}
```

Run: `cd quota-tray && cargo test -p provider-copilot parses_oauth_token -- --nocapture`

Expected: PASS

- [ ] **Step 4: Mock `GET /copilot_internal/user`**

`quota-tray/crates/provider-copilot/src/fetch.rs`:

```rust
use crate::headline::copilot_headline_percent;
use quota_tray_core::{
    FetchError, ProviderId, ProviderSnapshot, UsageWindow, WindowKind,
};
use secrecy::{ExposeSecret, SecretString};
use serde_json::Value;
use time::OffsetDateTime;

pub struct CopilotClient {
    pub base_url: String,
    pub http: reqwest::blocking::Client,
}

impl Default for CopilotClient {
    fn default() -> Self {
        Self {
            base_url: "https://api.github.com".into(),
            http: reqwest::blocking::Client::new(),
        }
    }
}

impl CopilotClient {
    pub fn fetch_user(&self, token: &SecretString) -> Result<ProviderSnapshot, FetchError> {
        let url = format!(
            "{}/copilot_internal/user",
            self.base_url.trim_end_matches('/')
        );
        let res = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", token.expose_secret()))
            .header("Accept", "application/json")
            .header("X-GitHub-Api-Version", "2025-04-01")
            .header("User-Agent", "QuotaTray/0.1")
            .send()
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(FetchError::Http {
                status: res.status().as_u16(),
                body: res.text().unwrap_or_default(),
            });
        }
        let body: Value = res
            .json()
            .map_err(|e| FetchError::Parse(e.to_string()))?;
        parse_copilot_user(&body)
    }
}

pub fn parse_copilot_user(body: &Value) -> Result<ProviderSnapshot, FetchError> {
    let snapshots = body
        .get("quota_snapshots")
        .ok_or_else(|| FetchError::Parse("missing quota_snapshots".into()))?;
    let headline = copilot_headline_percent(snapshots);

    let mut windows = Vec::new();
    for (id, label) in [
        ("premium_interactions", "Premium"),
        ("chat", "Chat"),
        ("completions", "Completions"),
    ] {
        if let Some(bucket) = snapshots.get(id) {
            let unlimited = bucket.get("unlimited").and_then(|v| v.as_bool()) == Some(true);
            let used = if unlimited {
                None
            } else {
                bucket
                    .get("percent_remaining")
                    .and_then(|v| v.as_f64())
                    .map(|r| 100.0 - r)
            };
            windows.push(UsageWindow {
                id: id.into(),
                label: label.into(),
                kind: WindowKind::Percent { used },
                resets_at: None,
            });
        }
    }

    if windows.is_empty() {
        return Err(FetchError::Parse(
            "copilot_internal/user: empty quota_snapshots".into(),
        ));
    }

    Ok(ProviderSnapshot {
        provider: ProviderId::Copilot,
        fetched_at: OffsetDateTime::now_utc(),
        windows,
        headline_percent: headline,
        stale: false,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetch_maps_premium_used() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/copilot_internal/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "copilot_plan": "individual_pro",
                "quota_snapshots": {
                    "premium_interactions": { "percent_remaining": 25.0, "unlimited": false },
                    "chat": { "unlimited": true },
                    "completions": { "unlimited": true }
                }
            })))
            .mount(&server)
            .await;

        let base = server.uri();
        let snap = tokio::task::spawn_blocking(move || {
            let client = CopilotClient {
                base_url: base,
                http: reqwest::blocking::Client::new(),
            };
            client.fetch_user(&SecretString::from("gho_x"))
        })
        .await
        .unwrap()
        .unwrap();

        assert_eq!(snap.provider, ProviderId::Copilot);
        assert_eq!(snap.headline_percent, Some(75.0));
    }
}
```

- [ ] **Step 5: `CopilotProvider` + register**

`quota-tray/crates/provider-copilot/src/lib.rs`:

```rust
mod credentials;
mod fetch;
mod headline;

pub use credentials::{default_apps_json_path, parse_apps_json, read_apps_json};
pub use fetch::{parse_copilot_user, CopilotClient};
pub use headline::copilot_headline_percent;

use credentials::{default_apps_json_path, read_apps_json};
use fetch::CopilotClient;
use quota_tray_core::{
    CredentialError, Credentials, FetchError, Provider, ProviderId, ProviderSnapshot,
};
use std::path::PathBuf;

pub struct CopilotProvider {
    pub apps_json_path: PathBuf,
    pub client: CopilotClient,
}

impl Default for CopilotProvider {
    fn default() -> Self {
        Self {
            apps_json_path: default_apps_json_path(),
            client: CopilotClient::default(),
        }
    }
}

impl Provider for CopilotProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Copilot
    }

    fn credentials(&self) -> Result<Credentials, CredentialError> {
        read_apps_json(&self.apps_json_path)
    }

    fn fetch(&self, creds: &Credentials) -> Result<ProviderSnapshot, FetchError> {
        self.client.fetch_user(&creds.raw)
    }
}
```

Update Tauri poller:

```rust
use provider_copilot::CopilotProvider;

let poller = Poller::new(
    vec![
        Box::new(ClaudeProvider::default()),
        Box::new(CursorProvider::default()),
        Box::new(CopilotProvider::default()),
    ],
    config.clone(),
);
```

Settings UI already lists Copilot (Task 9).

- [ ] **Step 6: Run Copilot tests**

Run: `cd quota-tray && cargo test -p provider-copilot -- --nocapture`

Expected: PASS

- [ ] **Step 7: Commit**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/Cargo.toml quota-tray/crates/provider-copilot \
  quota-tray/src-tauri/Cargo.toml quota-tray/src-tauri/src/lib.rs
git commit -m "$(cat <<'EOF'
feat(quota-tray): Copilot provider via apps.json and internal user API

Parse editor OAuth token, invert percent_remaining to used %, skip unlimited buckets.
EOF
)"
```

---

### Task 12: README privacy + macOS run instructions + unofficial API docs

**Files:**
- Create: `quota-tray/README.md`
- Modify: `docs/superpowers/plans/2026-09-02-quota-tray.md` only if assembling note needed — **do not** rewrite the plan in this task unless merging drafts; this task is app docs
- Test: verify steps below (link existence + `rg` content checks)

**Interfaces:**
- Consumes: credential paths + endpoints from design §6 / `_research-providers.md`
- Produces: `quota-tray/README.md` sections: Privacy, Unofficial APIs, macOS run, build from source

- [ ] **Step 1: Write README with required sections**

Create `quota-tray/README.md`:

```markdown
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
cargo build -p quota-tray-core
cd src-tauri && cargo tauri dev
```

Or from `quota-tray/` if a root script exists:

```bash
npm run tauri dev
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
```

- [ ] **Step 2: Verify links resolve in-repo**

Run: `test -f /Users/lovinmaxwell/Developer/claude-tracker/docs/superpowers/specs/2026-09-02-quota-tray-design.md && test -f /Users/lovinmaxwell/Developer/claude-tracker/docs/superpowers/specs/_research-providers.md && test -f /Users/lovinmaxwell/Developer/claude-tracker/quota-tray/README.md && echo OK`

Expected: `OK`

- [ ] **Step 3: Self-check README contains required phrases**

Run: `rg -n "Privacy|Unofficial|api/oauth/usage|GetCurrentPeriodUsage|copilot_internal|state.vscdb|apps.json|60" quota-tray/README.md`

Expected: matches for privacy section, all three unofficial endpoints, credential files, poll 60.

- [ ] **Step 4: Commit docs**

```bash
cd /Users/lovinmaxwell/Developer/claude-tracker
git add quota-tray/README.md
git commit -m "$(cat <<'EOF'
docs(quota-tray): privacy, unofficial APIs, and macOS run guide

Document local-only credential handling and vendor endpoint risk for open-source trust.
EOF
)"
```

---

## Spec coverage self-review

| Spec requirement | Task(s) |
| --- | --- |
| Tauri 2 + Rust + Svelte; no PHP | 1, 7, 8 |
| Core types + `max` aggregation | 2 |
| Claude credentials + usage | 3, 4 |
| Parallel poll, independent stale | 5 |
| Mascot / segment icon | 6 |
| Combined tray + IPC | 7 |
| Panel list + dual windows + cream/terracotta | 8 |
| Settings enable/poll 60..=120 / Claude headline | 9 |
| Cursor provider | 10 |
| Copilot provider (invert remaining) | 11 |
| Privacy README + unofficial APIs | 12 |
| OpenAI v1.1 | **Deferred** (separate plan) |

**Placeholder scan:** no TBD / TODO / “similar to Task N” left in task steps.

**Type consistency:** `Provider` trait + `TrayState` / `ProviderSnapshot` / `AppConfig` / `ClaudeHeadlineMetric` shared across Tasks 2–11; icon paint consumed by Task 7.

