<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { clampPollInterval } from "./lib/settings_validate";
  import {
    applyThemeMode,
    initTheme,
    type ThemeMode,
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
  let saved = $state(false);
  let error = $state<string | null>(null);

  const allProviders: { id: ProviderId; blurb: string }[] = [
    { id: "Claude", blurb: "Keychain / ~/.claude credentials → Anthropic usage" },
    { id: "Cursor", blurb: "Local state.vscdb token → Cursor period usage" },
    { id: "Copilot", blurb: "apps.json / hosts.json → GitHub Copilot user API" },
  ];

  const themeOptions: { id: ThemeMode; label: string; hint: string }[] = [
    { id: "system", label: "System", hint: "Match macOS" },
    { id: "light", label: "Light", hint: "Cream" },
    { id: "dark", label: "Dark", hint: "Charcoal" },
  ];

  onMount(async () => {
    theme = initTheme();
    try {
      config = await invoke<AppConfig>("get_config");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  });

  function setTheme(mode: ThemeMode) {
    theme = mode;
    applyThemeMode(mode);
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
      config = await invoke<AppConfig>("set_config", { config: next });
      saved = true;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
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
      <p class="blurb">Credentials stay on this Mac. We only call vendor APIs.</p>
    </div>
  </header>

  <section class="card">
    <h2>Appearance</h2>
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
    <h2>Providers</h2>
    {#each allProviders as p}
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

  <section class="card">
    <div class="card-top">
      <h2>Poll interval</h2>
      <span class="value">{config.poll_interval_secs}s</span>
    </div>
    <input
      class="range"
      type="range"
      min="60"
      max="120"
      step="5"
      value={config.poll_interval_secs}
      oninput={onPollInput}
    />
    <div class="range-labels">
      <span>60s</span>
      <span>120s</span>
    </div>
    <p class="hint">Allowed range 60–120. Default 60.</p>
  </section>

  <section class="card">
    <h2>Claude headline metric</h2>
    <select
      bind:value={config.claude_headline}
      onchange={() => {
        saved = false;
      }}
    >
      <option value="Highest">Highest (max of 5h / 7d)</option>
      <option value="FiveHour">Five-hour window</option>
      <option value="SevenDay">Seven-day window</option>
    </select>
  </section>

  <div class="actions">
    <button type="button" class="save" onclick={save}>Save changes</button>
    {#if saved}<p class="ok">Saved.</p>{/if}
    {#if error}<p class="err">{error}</p>{/if}
  </div>
</main>

<style>
  .settings {
    min-height: 100vh;
    padding: 1.1rem 1.15rem 1.5rem;
    animation: rise-in 420ms var(--ease) both;
  }
  .head {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.75rem;
    align-items: start;
    margin-bottom: 1rem;
  }
  .back {
    width: 2rem;
    height: 2rem;
    display: grid;
    place-items: center;
    border-radius: 999px;
    border: 1px solid var(--cream-line);
    background: var(--pale);
    color: var(--ink);
    text-decoration: none;
    font-weight: 700;
  }
  h1 {
    margin: 0;
    font-family: var(--font);
    font-size: 1.55rem;
    letter-spacing: -0.03em;
  }
  h2 {
    margin: 0 0 0.55rem;
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .blurb,
  .hint {
    margin: 0.25rem 0 0;
    color: var(--ink-muted);
    font-size: 0.82rem;
    line-height: 1.4;
  }
  .card {
    margin: 0.75rem 0;
    padding: 0.85rem 0.9rem;
    background: var(--sheet);
    border: 1px solid var(--cream-line);
    border-radius: var(--radius);
    box-shadow: var(--shadow-soft);
  }
  .theme-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.55rem;
  }
  .theme-card {
    appearance: none;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    padding: 0.65rem 0.55rem;
    border-radius: var(--radius-sm);
    border: 1.5px solid var(--cream-line);
    background: var(--pale);
    color: var(--ink);
    cursor: pointer;
    text-align: left;
    font: inherit;
    transition:
      border-color 160ms var(--ease),
      box-shadow 160ms var(--ease),
      transform 160ms var(--ease);
  }
  .theme-card.active {
    border-color: var(--terracotta);
    box-shadow: 0 0 0 1px var(--terracotta);
  }
  .theme-card:active {
    transform: translateY(1px);
  }
  .swatch {
    width: 100%;
    height: 28px;
    border-radius: 8px;
    margin-bottom: 0.25rem;
    border: 1px solid var(--cream-line);
  }
  .swatch-system {
    background: linear-gradient(90deg, #f5ede6 50%, #1c1612 50%);
  }
  .swatch-light {
    background: linear-gradient(135deg, #fbf7f2, #c96442 120%);
  }
  .swatch-dark {
    background: linear-gradient(135deg, #1c1612, #c96442 140%);
  }
  .theme-label {
    font-weight: 700;
    font-size: 0.85rem;
  }
  .theme-hint {
    font-size: 0.7rem;
    color: var(--ink-muted);
  }
  .card-top {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  .card-top h2 {
    margin: 0;
  }
  .value {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--terracotta-deep);
  }
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0;
    border-top: 1px solid var(--cream-line);
    cursor: pointer;
  }
  .toggle-row:first-of-type {
    border-top: 0;
    padding-top: 0.15rem;
  }
  .copy {
    display: flex;
    flex-direction: column;
    gap: 0.12rem;
    min-width: 0;
  }
  .title {
    font-weight: 650;
  }
  .desc {
    font-size: 0.75rem;
    color: var(--ink-muted);
    line-height: 1.3;
  }
  .switch {
    appearance: none;
    width: 44px;
    height: 26px;
    margin-left: auto;
    flex-shrink: 0;
    border-radius: 999px;
    background: var(--cream-deep);
    border: 1px solid var(--cream-line);
    position: relative;
    cursor: pointer;
    transition: background 180ms var(--ease);
  }
  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
    transition: transform 180ms var(--ease);
  }
  .switch:checked {
    background: var(--terracotta);
    border-color: var(--terracotta-deep);
  }
  .switch:checked::after {
    transform: translateX(18px);
  }
  .range {
    width: 100%;
    accent-color: var(--terracotta);
    margin-top: 0.55rem;
  }
  .range-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.72rem;
    color: var(--ink-muted);
  }
  select {
    width: 100%;
    font: inherit;
    padding: 0.55rem 0.65rem;
    border: 1px solid var(--cream-line);
    border-radius: var(--radius-sm);
    background: var(--pale);
    color: var(--ink);
  }
  .actions {
    margin-top: 1rem;
  }
  .save {
    width: 100%;
    background: linear-gradient(180deg, var(--terracotta-soft), var(--terracotta));
    color: #fff;
    border: 0;
    border-radius: 999px;
    padding: 0.75rem 1rem;
    font: inherit;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 8px 18px color-mix(in srgb, var(--terracotta) 28%, transparent);
  }
  .save:hover {
    filter: brightness(1.03);
  }
  .ok {
    margin: 0.55rem 0 0;
    color: var(--ok);
    font-weight: 600;
  }
  .err {
    margin: 0.55rem 0 0;
    color: var(--danger);
  }
</style>
