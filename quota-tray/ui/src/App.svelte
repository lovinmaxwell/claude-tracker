<script lang="ts">
  import { onMount } from "svelte";
  import Hero from "./lib/components/Hero.svelte";
  import ProviderRow from "./lib/components/ProviderRow.svelte";
  import EmptyState from "./lib/components/EmptyState.svelte";
  import { fetchTrayState, onTrayStateUpdated, refreshNow } from "./lib/api";
  import { emptyProvidersBody, hidePanel, isChromeExtension } from "./lib/platform";
  import { initPalette, applyPalette, cyclePalette, readPalette, type PaletteId } from "./lib/theme";
  import type { TrayState } from "./lib/types";

  let state = $state<TrayState | null>(null);
  let loadError = $state<string | null>(null);
  let currentPalette = $state<PaletteId>("studio");

  async function refresh(forceNetwork = false) {
    try {
      state = forceNetwork ? await refreshNow() : await fetchTrayState();
      loadError = null;
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  }

  function handleCyclePalette() {
    const next = cyclePalette(currentPalette);
    currentPalette = next;
    applyPalette(next);
  }

  onMount(() => {
    currentPalette = initPalette();
    refresh(isChromeExtension());

    let unlisten: (() => void) | undefined;
    onTrayStateUpdated(() => {
      refresh();
    }).then((u) => {
      unlisten = u;
    });

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        void hidePanel();
      }
    };
    window.addEventListener("keydown", onKey);

    const onStorage = (e: StorageEvent) => {
      if (e.key === "quota-tray-palette") {
        currentPalette = readPalette();
        applyPalette(currentPalette);
      }
    };
    window.addEventListener("storage", onStorage);

    return () => {
      unlisten?.();
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("storage", onStorage);
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
      body={emptyProvidersBody()}
    />
  {:else if state && state.providers.length > 0}
    <section class="providers-section" aria-label="Providers">
      <div class="list">
        {#each state.providers as snap, i (snap.provider)}
          <ProviderRow {snap} index={i} />
        {/each}
      </div>
    </section>
  {/if}

  <footer class="hud-footer">
    <div class="footer-left">
      <a class="btn-action" href="settings.html">Settings</a>
      <button type="button" class="btn-action" onclick={handleCyclePalette} title="Cycle theme palette">
        🎨 {currentPalette}
      </button>
    </div>
    <div class="footer-right">
      <button type="button" class="btn-action" onclick={() => refresh(true)}>↻ Refresh</button>
    </div>
  </footer>
</main>

<style>
  .panel {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--sheet);
    border: 0.5px solid var(--popover-stroke);
    border-radius: 12px;
    overflow: hidden;
    margin: 0;
  }

  .providers-section {
    padding: 0.75rem 0.85rem 0.5rem;
    animation: rise-in 360ms var(--ease) both;
  }

  .list {
    display: flex;
    flex-direction: column;
  }

  .hud-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem 0.85rem;
    margin-top: auto;
    border-top: 0.5px solid var(--popover-stroke);
    background: color-mix(in srgb, var(--pale) 40%, transparent);
  }

  .footer-left,
  .footer-right {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .btn-action {
    appearance: none;
    background: transparent;
    border: 0.5px solid var(--popover-stroke);
    border-radius: 5px;
    color: var(--ink);
    padding: 0.28rem 0.55rem;
    font: inherit;
    font-size: 0.75rem;
    font-weight: 550;
    text-decoration: none;
    cursor: pointer;
    text-transform: capitalize;
    transition: background 140ms ease;
  }
  .btn-action:hover {
    background: color-mix(in srgb, var(--card-bg) 90%, transparent);
  }

  .banner {
    margin: 0.6rem 0.85rem 0.3rem;
    padding: 0.45rem 0.65rem;
    color: var(--ink-muted);
    font-size: 0.82rem;
    line-height: 1.3;
    background: var(--pale);
    border: 0.5px solid var(--popover-stroke);
    border-radius: 6px;
  }
  .banner.warn {
    color: var(--danger);
  }
</style>
