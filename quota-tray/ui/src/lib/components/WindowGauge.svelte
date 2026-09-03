<script lang="ts">
  import { onMount } from "svelte";
  import type { UsageWindow } from "../types";
  import { formatHeadlinePercent, meterWidthPercent } from "../format";

  interface Props {
    window: UsageWindow;
    accent?: string;
  }
  let { window: w, accent = "var(--terracotta)" }: Props = $props();
  let fillReady = $state(false);

  onMount(() => {
    const id = requestAnimationFrame(() => {
      fillReady = true;
    });
    return () => cancelAnimationFrame(id);
  });

  const used = $derived("Percent" in w.kind ? w.kind.Percent.used : null);
  const width = $derived(meterWidthPercent(used));
  const shown = $derived(fillReady ? (width ?? 0) : 0);
</script>

<div class="gauge">
  <div class="row">
    <span>{w.label}</span>
    <span class="pct">{formatHeadlinePercent(used)}</span>
  </div>
  <div class="track" aria-hidden="true">
    {#if width != null}
      <div
        class="fill"
        class:ready={fillReady}
        style={`--w: ${shown}%; background: ${accent}`}
      ></div>
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
    margin-top: 0.45rem;
    padding: 0.45rem 0.5rem;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--pale) 70%, transparent);
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.78rem;
    color: var(--ink-muted);
  }
  .pct {
    font-weight: 600;
    color: var(--ink);
    font-variant-numeric: tabular-nums;
  }
  .track {
    height: 8px;
    background: var(--cream-deep);
    border-radius: 999px;
    overflow: hidden;
    margin-top: 0.28rem;
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
    animation: fill-x calc(var(--fill-duration) * 0.85) var(--ease-fill) both;
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
  }
  .reset {
    margin: 0.22rem 0 0;
    font-size: 0.7rem;
    color: var(--ink-muted);
  }
</style>
