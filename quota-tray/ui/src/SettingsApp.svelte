<script lang="ts">
  import { onMount } from "svelte";
  import { clampPollInterval } from "./lib/settings_validate";
  import {
    fetchConfig,
    saveConfig as persistConfig,
    fetchConnections,
    importSecret,
    clearSecret,
    type ConnectionKind,
    type ConnectionMap,
  } from "./lib/config_api";
  import { isChromeExtension } from "./lib/platform";
  import {
    applyThemeMode,
    initTheme,
    readPalette,
    applyPalette,
    initPalette,
    PALETTE_OPTIONS,
    type ThemeMode,
    type PaletteId,
  } from "./lib/theme";

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
  let theme = $state<ThemeMode>("system");
  let palette = $state<PaletteId>("studio");
  let saved = $state(false);
  let error = $state<string | null>(null);
  let chromeExt = $state(false);
  let connections = $state<ConnectionMap>({
    Claude: "missing",
    Cursor: "missing",
    Copilot: "missing",
  });
  let importBusy = $state<string | null>(null);

  const desktopBlurbs: Record<Exclude<ProviderId, "OpenAI">, string> = {
    Claude: "Keychain / ~/.claude credentials → Anthropic usage",
    Cursor: "Local state.vscdb token → Cursor period usage",
    Copilot: "apps.json / hosts.json → GitHub Copilot user API",
  };
  const chromeBlurbs: Record<Exclude<ProviderId, "OpenAI">, string> = {
    Claude: "Import ~/.claude/.credentials.json (browser cannot read Keychain)",
    Cursor: "Uses your cursor.com login cookie, or paste a JWT",
    Copilot: "Import github-copilot apps.json / hosts.json",
  };

  const providerIds: Exclude<ProviderId, "OpenAI">[] = ["Claude", "Cursor", "Copilot"];
  const providerRows = $derived(
    providerIds.map((id) => ({
      id,
      blurb: chromeExt ? chromeBlurbs[id] : desktopBlurbs[id],
    }))
  );

  const themeOptions: { id: ThemeMode; label: string; hint: string }[] = [
    { id: "system", label: "System", hint: "Match OS" },
    { id: "light", label: "Light", hint: "Cream" },
    { id: "dark", label: "Dark", hint: "Charcoal" },
  ];

  onMount(async () => {
    theme = initTheme();
    palette = initPalette();
    chromeExt = isChromeExtension();
    try {
      config = await fetchConfig();
      if (chromeExt) {
        connections = await fetchConnections();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  function setTheme(mode: ThemeMode) {
    theme = mode;
    applyThemeMode(mode);
  }

  function setPalette(id: PaletteId) {
    palette = id;
    applyPalette(id);
  }

  function toggle(id: ProviderId, on: boolean) {
    if (on && !config.enabled.includes(id)) {
      config = { ...config, enabled: [...config.enabled, id] };
    } else if (!on) {
      config = { ...config, enabled: config.enabled.filter((x) => x !== id) };
    }
    saved = false;
  }

  async function save() {
    try {
      const next = {
        ...config,
        poll_interval_secs: clampPollInterval(config.poll_interval_secs),
      };
      config = await persistConfig(next);
      saved = true;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function connectionLabel(kind: ConnectionKind): string {
    if (kind === "browser") {
      return "Signed in on cursor.com";
    }
    if (kind === "imported") {
      return "Token imported";
    }
    return "Not connected";
  }

  async function onImportFile(provider: Exclude<ProviderId, "OpenAI">, e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) {
      return;
    }
    importBusy = provider;
    try {
      const text = await file.text();
      await importSecret(provider, text);
      connections = await fetchConnections();
      error = null;
      saved = true;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      importBusy = null;
    }
  }

  async function onPasteToken(provider: Exclude<ProviderId, "OpenAI">) {
    const text = window.prompt(`Paste ${provider} token or credentials JSON`);
    if (!text) {
      return;
    }
    importBusy = provider;
    try {
      await importSecret(provider, text);
      connections = await fetchConnections();
      error = null;
      saved = true;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      importBusy = null;
    }
  }

  async function onClearToken(provider: Exclude<ProviderId, "OpenAI">) {
    importBusy = provider;
    try {
      await clearSecret(provider);
      connections = await fetchConnections();
      error = null;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      importBusy = null;
    }
  }

  async function onRecheckCursor() {
    importBusy = "Cursor";
    try {
      connections = await fetchConnections();
      await persistConfig(config);
      error = null;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      importBusy = null;
    }
  }

  function onPollInput(e: Event) {
    const v = Number((e.currentTarget as HTMLInputElement).value);
    config = { ...config, poll_interval_secs: v };
    saved = false;
  }
</script>

<main class="settings">
  <header class="head">
    <a class="back" href="index.html" aria-label="Back to panel">←</a>
    <div>
      <h1>Settings</h1>
      <p class="blurb">
        {chromeExt
          ? "Credentials stay in this Chrome profile. We only call vendor APIs."
          : "Credentials stay on this Mac. We only call vendor APIs."}
      </p>
    </div>
  </header>

  <section class="card">
    <h2>Appearance Mode</h2>
    <div class="theme-grid" role="radiogroup" aria-label="Theme">
      {#each themeOptions as opt}
        <button
          type="button"
          class="theme-card"
          class:active={theme === opt.id}
          role="radio"
          aria-checked={theme === opt.id}
          onclick={() => setTheme(opt.id)}
        >
          <span class={`swatch swatch-${opt.id}`}></span>
          <span class="theme-label">{opt.label}</span>
          <span class="theme-hint">{opt.hint}</span>
        </button>
      {/each}
    </div>
  </section>

  <section class="card">
    <h2>Color Palette</h2>
    <div class="palette-grid" role="radiogroup" aria-label="Color Palette">
      {#each PALETTE_OPTIONS as p}
        <button
          type="button"
          class="palette-card"
          class:active={palette === p.id}
          role="radio"
          aria-checked={palette === p.id}
          onclick={() => setPalette(p.id)}
        >
          <div class="palette-header">
            <span class="palette-title">{p.label}</span>
            <div class="swatches-row">
              {#each p.swatches as s}
                <span class="swatch-dot" style={`background: ${s}`}></span>
              {/each}
            </div>
          </div>
          <p class="palette-desc">{p.description}</p>
        </button>
      {/each}
    </div>
  </section>

  <section class="card">
    <h2>Providers</h2>
    {#each providerRows as p}
      <label class="toggle-row">
        <div class="copy">
          <span class="title">{p.id}</span>
          <span class="desc">{p.blurb}</span>
        </div>
        <input
          class="switch"
          type="checkbox"
          checked={config.enabled.includes(p.id)}
          onchange={(e) => toggle(p.id, e.currentTarget.checked)}
        />
      </label>
    {/each}
  </section>

  {#if chromeExt}
    <section class="card">
      <h2>Connect in this browser</h2>
      <p class="desc connect-lead">
        Chrome cannot read Keychain or Cursor’s state.vscdb. Cursor can use your
        cursor.com session. Claude and Copilot need a local credentials file import.
      </p>
      {#each providerRows as p}
        <div class="connect-row">
          <div class="copy">
            <span class="title">{p.id}</span>
            <span class="desc">{connectionLabel(connections[p.id])}</span>
          </div>
          <div class="connect-actions">
            {#if p.id === "Cursor"}
              <button type="button" class="btn-action" onclick={onRecheckCursor}>
                Recheck login
              </button>
            {/if}
            <label class="btn-action file-btn">
              Import file
              <input
                type="file"
                accept=".json,application/json,text/plain"
                hidden
                onchange={(e) => onImportFile(p.id, e)}
              />
            </label>
            <button type="button" class="btn-action" onclick={() => onPasteToken(p.id)}>
              Paste
            </button>
            {#if connections[p.id] === "imported"}
              <button type="button" class="btn-action" onclick={() => onClearToken(p.id)}>
                Clear
              </button>
            {/if}
          </div>
        </div>
      {/each}
      {#if importBusy}
        <p class="desc">Updating {importBusy}…</p>
      {/if}
    </section>
  {/if}

  <section class="card">
    <div class="card-top">
      <h2>Poll interval</h2>
      <span class="value">{config.poll_interval_secs}s</span>
    </div>
    <input
      class="range"
      type="range"
      min="60"
      max="1800"
      step="60"
      value={config.poll_interval_secs}
      oninput={onPollInput}
    />
  </section>

  <div class="actions">
    <button type="button" class="btn primary" onclick={save}>Save</button>
    {#if saved}
      <span class="saved">Saved</span>
    {/if}
    {#if error}
      <span class="err">{error}</span>
    {/if}
  </div>
</main>

<style>
  .settings {
    max-width: 480px;
    margin: 0 auto;
    padding: 1rem 1.25rem 2rem;
    font-family: var(--font-ui);
    color: var(--ink);
    background: var(--sheet);
  }

  .head {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    margin-bottom: 1.2rem;
  }

  .back {
    display: inline-grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 0.5px solid var(--popover-stroke);
    color: var(--ink);
    text-decoration: none;
    font-size: 1.1rem;
    background: var(--card-bg);
  }

  h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
  }

  .blurb {
    margin: 0.2rem 0 0;
    font-size: 0.76rem;
    color: var(--ink-muted);
  }

  .card {
    background: var(--card-bg);
    border: 0.5px solid var(--card-border);
    border-radius: var(--radius);
    padding: 0.9rem 1rem;
    margin-bottom: 1rem;
  }

  .card h2 {
    margin: 0 0 0.75rem;
    font-size: 0.85rem;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .theme-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.6rem;
  }

  .theme-card {
    appearance: none;
    border: 1px solid var(--card-border);
    border-radius: 8px;
    background: var(--pale);
    padding: 0.65rem 0.5rem;
    text-align: center;
    cursor: pointer;
    color: var(--ink);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
  }
  .theme-card.active {
    border-color: var(--chip-claude);
    box-shadow: 0 0 0 1.5px var(--chip-claude);
  }

  .swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 1px solid var(--popover-stroke);
  }
  .swatch-system {
    background: linear-gradient(135deg, #ffffff 50%, #2c2c2c 50%);
  }
  .swatch-light {
    background: #fbf7f2;
  }
  .swatch-dark {
    background: #231c18;
  }

  .theme-label {
    font-size: 0.78rem;
    font-weight: 600;
  }
  .theme-hint {
    font-size: 0.68rem;
    color: var(--ink-muted);
  }

  /* Palette Grid */
  .palette-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.6rem;
  }

  .palette-card {
    appearance: none;
    border: 1px solid var(--card-border);
    border-radius: 8px;
    background: var(--pale);
    padding: 0.65rem 0.75rem;
    text-align: left;
    cursor: pointer;
    color: var(--ink);
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    transition: border-color 150ms ease;
  }
  .palette-card.active {
    border-color: var(--chip-claude);
    box-shadow: 0 0 0 1.5px var(--chip-claude);
  }

  .palette-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .palette-title {
    font-size: 0.8rem;
    font-weight: 650;
  }

  .swatches-row {
    display: flex;
    gap: 4px;
  }

  .swatch-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .palette-desc {
    margin: 0;
    font-size: 0.68rem;
    color: var(--ink-muted);
    line-height: 1.3;
  }

  /* Providers */
  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.55rem 0;
    border-top: 0.5px solid var(--popover-stroke);
    cursor: pointer;
  }
  .toggle-row:first-of-type {
    border-top: none;
    padding-top: 0;
  }

  .copy {
    display: flex;
    flex-direction: column;
  }
  .title {
    font-weight: 600;
    font-size: 0.86rem;
  }
  .desc {
    font-size: 0.72rem;
    color: var(--ink-muted);
  }

  .switch {
    width: 40px;
    height: 22px;
    cursor: pointer;
  }

  .card-top {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .value {
    font-size: 0.82rem;
    font-weight: 700;
    font-family: var(--font-mono);
  }

  .range {
    width: 100%;
    margin-top: 0.4rem;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 1.25rem;
  }

  .btn.primary {
    appearance: none;
    background: var(--chip-claude);
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.45rem 1.2rem;
    font-weight: 650;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .saved {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--ok);
  }
  .err {
    font-size: 0.82rem;
    color: var(--danger);
  }

  .connect-lead {
    margin: 0 0 0.75rem;
    display: block;
  }

  .connect-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    padding: 0.55rem 0;
    border-top: 0.5px solid var(--popover-stroke);
    flex-wrap: wrap;
  }

  .connect-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .btn-action {
    appearance: none;
    background: transparent;
    border: 0.5px solid var(--popover-stroke);
    border-radius: 5px;
    color: var(--ink);
    padding: 0.28rem 0.55rem;
    font: inherit;
    font-size: 0.72rem;
    font-weight: 550;
    cursor: pointer;
  }

  .file-btn {
    display: inline-flex;
    align-items: center;
  }
</style>
