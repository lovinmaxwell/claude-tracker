<script lang="ts">
  import { onMount } from "svelte";
  import WidgetRings from "./lib/components/WidgetRings.svelte";
  import { fetchTrayState, onTrayStateUpdated } from "./lib/api";
  import { formatHeadlinePercent } from "./lib/format";
  import type { ProviderSnapshot, TrayState } from "./lib/types";
  import {
    MAX_WIDGET_RINGS,
    WIDGET_REST_OPACITY,
    defaultProvider,
    formatAge,
    formatCountdown,
    formatResetClock,
    peakRing,
    ringsFor,
    summarize,
  } from "./lib/widget";
  import { hideWidget, isTauri, onWidgetHover } from "./lib/widget_api";
  import { mockTrayState } from "./lib/widget_mock";

  const PROVIDER_KEY = "quota-tray-widget-provider";
  const inTauri = isTauri();

  let state = $state<TrayState | null>(null);
  let expanded = $state(false);
  let now = $state(new Date());
  let chosen = $state<string | null>(readChosenProvider());

  function readChosenProvider(): string | null {
    try {
      return localStorage.getItem(PROVIDER_KEY);
    } catch {
      return null;
    }
  }

  function choose(provider: string) {
    chosen = provider;
    try {
      localStorage.setItem(PROVIDER_KEY, provider);
    } catch {
      /* ignore */
    }
  }

  async function refresh() {
    try {
      state = inTauri ? await fetchTrayState() : mockTrayState(new Date());
    } catch {
      // Keep the last good state on screen; a failed read must not blank the widget.
    }
  }

  const snap = $derived<ProviderSnapshot | null>(
    state?.providers.find((p) => p.provider === chosen) ?? defaultProvider(state)
  );
  const allRings = $derived(snap ? ringsFor(snap) : []);
  const rings = $derived(allRings.slice(0, MAX_WIDGET_RINGS));
  const hiddenCount = $derived(allRings.length - rings.length);
  const peak = $derived(peakRing(allRings));
  const status = $derived(summarize(snap, allRings, now));
  const fetchedAt = $derived(snap ? new Date(snap.fetched_at) : null);
  const age = $derived(formatAge(fetchedAt, now));
  const switchable = $derived((state?.providers ?? []).filter((p) => ringsFor(p).length > 0));
  const peakTone = $derived(peak?.tone ?? "wait");

  onMount(() => {
    // Browser preview only: the native shell drives hover inside Tauri.
    const params = new URLSearchParams(location.search);
    if (!inTauri && params.get("expanded") === "1") {
      expanded = true;
    }

    void refresh();
    const tick = setInterval(() => {
      now = new Date();
    }, 15_000);

    let unlistenState: (() => void) | undefined;
    let unlistenHover: (() => void) | undefined;
    if (inTauri) {
      onTrayStateUpdated(() => void refresh()).then((u) => (unlistenState = u));
      onWidgetHover((hovered) => (expanded = hovered)).then((u) => (unlistenHover = u));
    }

    return () => {
      clearInterval(tick);
      unlistenState?.();
      unlistenHover?.();
    };
  });

  function domHover(hovered: boolean) {
    if (!inTauri) {
      expanded = hovered;
    }
  }
</script>

<svelte:head>
  <style>
    html,
    body,
    #app {
      min-width: 0;
      overflow: hidden;
      background: transparent;
    }
  </style>
</svelte:head>

<main
  class="widget"
  class:expanded
  style={`--rest-opacity: ${WIDGET_REST_OPACITY}`}
  onmouseenter={() => domHover(true)}
  onmouseleave={() => domHover(false)}
  data-tauri-drag-region
>
  <!-- Resting pill: one ring, one number. -->
  <section class="pill" aria-hidden={expanded} data-tauri-drag-region>
    <WidgetRings
      rings={peak ? [peak] : []}
      size={40}
      stroke={5}
      glyphColor={peakTone === "hot" ? "var(--seg-warn)" : "var(--ink-muted)"}
    />
    <div class="pill-text">
      <span class="caption">
        {#if peak}
          {snap?.provider ?? ""} · {peak.label}
        {:else}
          Quota Tray
        {/if}
      </span>
      <span class="big" class:hot={peakTone === "hot"}>
        {formatHeadlinePercent(peak?.used)}
        {#if formatCountdown(peak?.resetsAt ?? null, now)}
          <span class="small">· {formatCountdown(peak?.resetsAt ?? null, now)}</span>
        {/if}
      </span>
    </div>
  </section>

  <!-- Hovered card: the full picture. -->
  <section class="card" aria-hidden={!expanded} data-tauri-drag-region>
    <header class="top" data-tauri-drag-region>
      <span class={`chip tone-${status.tone}`} role="status" aria-live="polite">
        <span class="dot"></span>{status.headline}
      </span>
      <button class="close" type="button" title="Hide widget" aria-label="Hide widget" onclick={() => void hideWidget()}>
        <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
          <path d="M1.5 1.5l7 7M8.5 1.5l-7 7" />
        </svg>
      </button>
    </header>
    <p class="detail" data-tauri-drag-region>{status.detail ?? " "}</p>

    <div class="body" data-tauri-drag-region>
      <WidgetRings
        {rings}
        glyphColor={peakTone === "hot" ? "var(--seg-warn)" : "var(--ink-muted)"}
      />
      <ul class="rows" data-tauri-drag-region>
        {#each rings as ring (ring.id)}
          <li data-tauri-drag-region>
            <span class="row-head">
              <span class="swatch" style={`background: ${ring.tone === "hot" ? "var(--seg-warn)" : `var(--seg-active-${ring.accent})`}`}></span>
              <span class="row-label">{ring.label}</span>
              <span class="row-pct" class:hot={ring.tone === "hot"}>{formatHeadlinePercent(ring.used)}</span>
            </span>
            <span class="row-sub">
              {#if ring.resetsAt && formatCountdown(ring.resetsAt, now)}
                Resets in {formatCountdown(ring.resetsAt, now)} · {formatResetClock(ring.resetsAt, now)}
              {:else}
                No reset time reported
              {/if}
            </span>
          </li>
        {:else}
          <li class="empty">Turn on a provider in Settings.</li>
        {/each}
        {#if hiddenCount > 0}
          <li class="more">+{hiddenCount} more in the panel</li>
        {/if}
      </ul>
    </div>

    <footer class="foot" data-tauri-drag-region>
      <div class="providers" role="tablist" aria-label="Provider">
        {#if switchable.length > 1}
          {#each switchable as p (p.provider)}
            <button
              type="button"
              role="tab"
              class="tab"
              class:active={snap?.provider === p.provider}
              aria-selected={snap?.provider === p.provider}
              onclick={() => choose(p.provider)}
            >
              {p.provider}
            </button>
          {/each}
        {:else}
          <span class="solo" data-tauri-drag-region>{snap?.provider ?? ""}</span>
        {/if}
      </div>
      <span class="age" data-tauri-drag-region>{age ? `Updated ${age}` : ""}</span>
    </footer>
  </section>
</main>

<style>
  .widget {
    position: relative;
    width: 100vw;
    height: 100vh;
    opacity: var(--rest-opacity);
    transition: opacity 240ms var(--ease);
    color: var(--ink);
    font-family: var(--font-ui);
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
  }
  .widget.expanded {
    opacity: 1;
  }

  /* Both layers fill the window; the native side sizes the window to match. */
  .pill,
  .card {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    border: 1px solid var(--popover-stroke);
    background: color-mix(in srgb, var(--sheet) 78%, transparent);
    transition: opacity 200ms var(--ease);
  }
  /* Pointer events land on the drag region, not on the text inside it. */
  .pill > *,
  .card > :not(footer):not(header),
  .card .body > *,
  .card .rows > * {
    pointer-events: none;
  }

  .pill {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 18px 0 12px;
    border-radius: 32px;
  }
  .card {
    display: flex;
    flex-direction: column;
    padding: 14px 16px 12px;
    border-radius: 22px;
  }
  .widget:not(.expanded) .card,
  .widget.expanded .pill {
    opacity: 0;
    visibility: hidden;
    transition:
      opacity 200ms var(--ease),
      visibility 0s linear 200ms;
  }
  .widget.expanded .card,
  .widget:not(.expanded) .pill {
    opacity: 1;
    visibility: visible;
  }

  /* — Pill — */
  .pill-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.15;
  }
  .caption {
    font-size: 11px;
    font-weight: 500;
    color: var(--ink-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .big {
    font-size: 19px;
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .big.hot {
    color: var(--seg-warn);
  }
  .small {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0;
    color: var(--ink-muted);
  }

  /* — Card — */
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 0 10px 0 8px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    pointer-events: none;
    background: color-mix(in srgb, var(--tone) 14%, transparent);
    color: color-mix(in srgb, var(--tone) 78%, var(--ink));
    --tone: var(--ink-muted);
  }
  .chip .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--tone);
  }
  .chip.tone-ok {
    --tone: var(--ok);
  }
  .chip.tone-watch {
    --tone: #d9a13b;
  }
  .chip.tone-hot {
    --tone: var(--seg-warn);
  }
  .chip.tone-stale,
  .chip.tone-wait {
    --tone: var(--ink-muted);
  }

  .close {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--ink-muted);
    cursor: pointer;
    pointer-events: auto;
    transition: background 150ms var(--ease);
  }
  .close:hover {
    background: color-mix(in srgb, var(--ink) 10%, transparent);
  }
  .close:focus-visible,
  .tab:focus-visible {
    outline: 2px solid var(--seg-active-claude);
    outline-offset: 1px;
  }
  .close path {
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    fill: none;
  }

  .detail {
    margin: 4px 0 0;
    min-height: 18px;
    font-size: 12px;
    color: var(--ink-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .body {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-top: 8px;
    flex: 1;
    min-height: 0;
  }
  .rows {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .rows li {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .row-head {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .swatch {
    width: 8px;
    height: 8px;
    border-radius: 3px;
    flex-shrink: 0;
  }
  .row-label {
    flex: 1;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--ink-muted);
  }
  .row-pct {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
    font-variant-numeric: tabular-nums;
  }
  .row-pct.hot {
    color: var(--seg-warn);
  }
  .row-sub {
    padding-left: 15px;
    font-size: 11px;
    color: var(--ink-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .empty,
  .more {
    font-size: 11.5px;
    color: var(--ink-muted);
  }

  .foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    height: 24px;
    margin-top: 6px;
  }
  .providers {
    display: flex;
    gap: 2px;
    padding: 2px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--ink) 6%, transparent);
  }
  .tab {
    height: 20px;
    padding: 0 9px;
    border: 0;
    border-radius: 999px;
    background: transparent;
    font: inherit;
    font-size: 11px;
    font-weight: 600;
    color: var(--ink-muted);
    cursor: pointer;
    transition:
      background 150ms var(--ease),
      color 150ms var(--ease);
  }
  .tab.active {
    background: var(--pale);
    color: var(--ink);
    box-shadow: 0 0.5px 1.5px rgba(0, 0, 0, 0.18);
  }
  .solo {
    padding: 0 8px;
    font-size: 11px;
    font-weight: 600;
    color: var(--ink-muted);
    pointer-events: none;
  }
  .age {
    font-size: 11px;
    color: var(--ink-muted);
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .widget,
    .pill,
    .card {
      transition: none !important;
    }
  }
</style>
