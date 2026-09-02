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
    <a href="settings.html">Settings</a>
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
