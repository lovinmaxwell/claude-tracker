<script lang="ts">
  import { onMount } from "svelte";
  import type { ProviderSnapshot } from "../types";
  import { formatHeadlinePercent, meterWidthPercent } from "../format";
  import WindowGauge from "./WindowGauge.svelte";

  interface Props {
    snap: ProviderSnapshot;
    index?: number;
  }
  let { snap, index = 0 }: Props = $props();
  let fillReady = $state(false);

  onMount(() => {
    const id = requestAnimationFrame(() => {
      fillReady = true;
    });
    return () => cancelAnimationFrame(id);
  });

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
  const shownWidth = $derived(fillReady ? (width ?? 0) : 0);
  const remaining = $derived(
    snap.headline_percent == null
      ? null
      : Math.max(0, Math.round((100 - snap.headline_percent) * 10) / 10)
  );
</script>

<article class="row" style={`animation-delay: ${120 + index * 70}ms`}>
  <div class="top">
    <span class="chip" style={`background: ${chip}`}>{name}</span>
    {#if snap.stale}
      <span class="badge">Stale</span>
    {/if}
    <span class="pct" style={`color: ${chip}`}>{formatHeadlinePercent(snap.headline_percent)}</span>
  </div>
  <div class="track" aria-hidden="true">
    {#if width != null}
      <div
        class="fill"
        class:muted={snap.stale}
        class:ready={fillReady}
        style={`--w: ${shownWidth}%; background: ${chip}`}
      ></div>
    {:else}
      <div class="unknown"></div>
    {/if}
  </div>
  <div class="sub">
    {#if remaining != null}
      <span>{remaining}% left</span>
    {:else}
      <span>No reading yet</span>
    {/if}
    {#if snap.fetched_at}
      <span class="when">Updated {snap.fetched_at}</span>
    {/if}
  </div>
  {#if snap.error}
    <p class="err">{snap.error}</p>
  {/if}
  {#if snap.windows.length > 1}
    <div class="windows">
      {#each snap.windows as w (w.id)}
        <WindowGauge window={w} accent={chip} />
      {/each}
    </div>
  {:else if snap.windows.length === 1}
    <WindowGauge window={snap.windows[0]} accent={chip} />
  {/if}
</article>

<style>
  .row {
    padding: 0.95rem 1rem;
    border-top: 1px solid var(--cream-line);
    animation: rise-in 480ms var(--ease) both;
  }
  .row:first-child {
    border-top: 0;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .chip {
    color: #fff;
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    padding: 0.22rem 0.58rem;
    border-radius: 999px;
  }
  .pct {
    font-weight: 700;
    margin-left: auto;
    font-variant-numeric: tabular-nums;
    font-size: 1.05rem;
  }
  .badge {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 40%, transparent);
    border-radius: 999px;
    padding: 0.12rem 0.45rem;
  }
  .track {
    margin-top: 0.55rem;
    height: 13px;
    background: var(--cream-deep);
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    width: var(--w, 0%);
    border-radius: inherit;
    transform-origin: left center;
    transform: scaleX(0);
  }
  .fill.ready {
    transform: scaleX(1);
    animation: fill-x var(--fill-duration) var(--ease-fill) both;
    animation-delay: calc(80ms + var(--i, 0) * 1ms);
  }
  .fill.muted {
    opacity: 0.42;
  }
  .unknown {
    height: 100%;
    width: 100%;
    background: repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 4px,
      color-mix(in srgb, var(--ink) 8%, transparent) 4px,
      color-mix(in srgb, var(--ink) 8%, transparent) 8px
    );
    animation: pulse-soft 1.6s ease-in-out infinite;
  }
  .sub {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 0.35rem;
    font-size: 0.72rem;
    color: var(--ink-muted);
  }
  .when {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 55%;
  }
  .err {
    margin: 0.4rem 0 0;
    color: var(--danger);
    font-size: 0.8rem;
    line-height: 1.35;
  }
  .windows {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.55rem;
    margin-top: 0.15rem;
  }
</style>
