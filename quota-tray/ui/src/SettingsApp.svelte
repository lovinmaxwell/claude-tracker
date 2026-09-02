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
    try {
      config = await invoke<AppConfig>("get_config");
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
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
  <p><a href="index.html">Back to panel</a></p>
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
  a {
    color: var(--terracotta);
  }
  input[type="number"],
  select {
    font: inherit;
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--cream-deep);
    border-radius: 6px;
    background: var(--pale);
    color: var(--ink);
  }
</style>
