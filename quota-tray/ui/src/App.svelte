<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Hero from "./lib/components/Hero.svelte";
  import ProviderRow from "./lib/components/ProviderRow.svelte";
  import EmptyState from "./lib/components/EmptyState.svelte";
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
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void getCurrentWindow().hide();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      unlisten?.();
      window.removeEventListener("keydown", onKey);
    };
  });

  const hasAnyReading = $derived(
    !!state &&
      state.providers.some(
        (p) => p.headline_percent != null || p.windows.length > 0
      )
  );
  const staleGlobal = $derived(
    !!state && hasAnyReading && state.providers.every((p) => p.stale)
  );
</script>

<main class="panel">
  <Hero
    fill={state?.shared_mascot_fill ?? null}
    staleGlobal={staleGlobal}
    providers={state?.providers ?? []}
  />

  {#if loadError}
    <p class="banner warn">{loadError}</p>
  {/if}

  {#if state && state.providers.length === 0}
    <EmptyState
      title="No providers yet"
      body="Turn on Claude, Cursor, or Copilot in Settings. Tokens stay on this Mac."
    />
  {:else if state && state.providers.length > 0}
    <section class="sheet" aria-label="Providers">
      <div class="sheet-head">
        <h2>Usage by provider</h2>
        <span>{state.providers.length}</span>
      </div>
      {#each state.providers as snap, i (snap.provider)}
        <ProviderRow {snap} index={i} />
      {/each}
    </section>
  {/if}

  <footer>
    <a class="ghost" href="settings.html">Settings</a>
    <button type="button" class="ghost" onclick={refresh}>Refresh</button>
  </footer>
</main>

<style>
  .panel {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--sheet);
    /* Native undecorated window supplies shadow; hairline only. */
    border: 0.5px solid var(--popover-stroke);
    border-radius: 12px;
    overflow: hidden;
    margin: 0;
  }
  .sheet {
    margin: 0 0.85rem;
    background: color-mix(in srgb, var(--pale) 92%, transparent);
    border: 0.5px solid var(--popover-stroke);
    border-radius: 10px;
    overflow: hidden;
    animation: rise-in 420ms var(--ease) both;
    animation-delay: 60ms;
  }
  .sheet-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.7rem 0.9rem 0.3rem;
  }
  .sheet-head h2 {
    margin: 0;
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .sheet-head span {
    font-size: 0.72rem;
    color: var(--ink-muted);
  }
  footer {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.85rem 1rem 1rem;
    margin-top: auto;
    border-top: 0.5px solid var(--popover-stroke);
  }
  .ghost {
    appearance: none;
    background: transparent;
    border: 0.5px solid var(--popover-stroke);
    border-radius: 6px;
    color: var(--ink);
    padding: 0.32rem 0.7rem;
    font: inherit;
    font-size: 0.82rem;
    font-weight: 500;
    text-decoration: none;
    cursor: pointer;
  }
  .ghost:hover {
    background: color-mix(in srgb, var(--cream-deep) 55%, transparent);
  }
  .banner {
    margin: 0 0.85rem 0.55rem;
    padding: 0.55rem 0.7rem;
    color: var(--ink-muted);
    font-size: 0.86rem;
    line-height: 1.35;
    background: var(--pale);
    border: 0.5px solid var(--popover-stroke);
    border-radius: 8px;
  }
  .banner.warn {
    color: var(--danger);
  }
</style>
