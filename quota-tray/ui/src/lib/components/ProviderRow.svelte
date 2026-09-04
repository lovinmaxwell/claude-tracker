<script lang="ts">
  import { onMount } from "svelte";
  import type { ProviderSnapshot, UsageWindow } from "../types";
  import {
    formatHeadlinePercent,
    segmentedActiveCount,
    isWarningSegment,
  } from "../format";

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
  const vendor = $derived(
    snap.provider === "Claude"
      ? "Anthropic"
      : snap.provider === "Cursor"
        ? "Anysphere"
        : snap.provider === "Copilot"
          ? "GitHub"
          : "OpenAI"
  );
  const providerClass = $derived(
    snap.provider === "Claude"
      ? "claude"
      : snap.provider === "Cursor"
        ? "cursor"
        : "copilot"
  );

  const totalSegments = 12;
  const activeSegments = $derived(
    segmentedActiveCount(snap.headline_percent, totalSegments)
  );
  const isHighUsage = $derived((snap.headline_percent ?? 0) >= 75);

  const remaining = $derived(
    snap.headline_percent == null
      ? null
      : Math.max(0, Math.round(100 - snap.headline_percent))
  );

  function windowUsed(w: UsageWindow): number | null {
    return "Percent" in w.kind ? w.kind.Percent.used : null;
  }

  function windowTokenDetails(w: UsageWindow): string | null {
    if ("TokenCount" in w.kind) {
      const u = w.kind.TokenCount.used;
      const l = w.kind.TokenCount.limit;
      return l != null ? `${u.toLocaleString()} / ${l.toLocaleString()}` : `${u.toLocaleString()} tokens`;
    }
    return null;
  }
</script>

<article class="provider-card" style={`animation-delay: ${60 + index * 50}ms`}>
  <div class="card-head">
    <div class="identity">
      <span class={`vendor-tag tag-${providerClass}`}>{vendor}</span>
      <span class="name">{name}</span>
      {#if snap.stale}
        <span class="stale-badge">Stale</span>
      {/if}
    </div>
    <div class="figures">
      <span class={`pct pct-${providerClass}`} class:warn={isHighUsage}>
        {formatHeadlinePercent(snap.headline_percent)}
      </span>
      {#if remaining != null}
        <span class="left">{remaining}% left</span>
      {:else}
        <span class="left">Waiting</span>
      {/if}
    </div>
  </div>

  <!-- 12-Segment Precision Bar -->
  <div class="segmented-track" class:stale={snap.stale} aria-label={`${name} usage: ${formatHeadlinePercent(snap.headline_percent)}`}>
    {#each Array(totalSegments) as _, i}
      {@const isActive = activeSegments != null && fillReady && i < activeSegments}
      {@const isWarn = isActive && isWarningSegment(i, totalSegments)}
      <div
        class="seg-block"
        class:active={isActive}
        class:warn={isWarn}
        class:inactive={!isActive && activeSegments != null}
        class:scanning={activeSegments == null}
        class:claude={isActive && !isWarn && snap.provider === "Claude"}
        class:cursor={isActive && !isWarn && snap.provider === "Cursor"}
        class:copilot={isActive && !isWarn && snap.provider !== "Claude" && snap.provider !== "Cursor"}
        style={`transition-delay: ${i * 24}ms; animation-delay: ${i * 60}ms`}
      ></div>
    {/each}
  </div>

  {#if snap.error}
    <p class="err">{snap.error}</p>
  {/if}

  {#if snap.windows.length > 0}
    <ul class="subwindows">
      {#each snap.windows as w (w.id)}
        <li class="subwindow-item">
          <span class="w-label">
            {w.label}
            {#if w.resets_at}
              <span class="w-reset">· Resets {w.resets_at}</span>
            {/if}
          </span>
          {#if windowTokenDetails(w)}
            <span class="w-val">{windowTokenDetails(w)}</span>
          {:else}
            <span class="w-val">{formatHeadlinePercent(windowUsed(w))}</span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .provider-card {
    background: var(--card-bg);
    border: 1px solid var(--card-border);
    border-radius: var(--radius-sm);
    padding: 0.75rem 0.85rem;
    margin-bottom: 0.6rem;
    animation: rise-in 380ms var(--ease) both;
    transition: background 180ms ease, border-color 180ms ease;
  }
  .provider-card:hover {
    background: var(--card-hover);
  }
  .provider-card:last-child {
    margin-bottom: 0;
  }

  .card-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 0.5rem;
  }

  .identity {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .vendor-tag {
    font-size: 0.62rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 1px 5px;
    border-radius: 4px;
    color: #ffffff;
  }
  .tag-claude {
    background: var(--chip-claude);
  }
  .tag-cursor {
    background: var(--chip-cursor);
  }
  .tag-copilot {
    background: var(--chip-copilot);
  }

  .name {
    font-weight: 650;
    font-size: 0.88rem;
    letter-spacing: -0.01em;
  }

  .stale-badge {
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
    border-radius: 3px;
    padding: 0.05rem 0.3rem;
  }

  .figures {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    font-family: var(--font-mono);
  }

  .pct {
    font-size: 0.95rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--ink);
  }
  .pct-claude {
    color: var(--chip-claude);
  }
  .pct-cursor {
    color: var(--chip-cursor);
  }
  .pct-copilot {
    color: var(--chip-copilot);
  }
  .pct.warn {
    color: var(--seg-warn);
  }

  .left {
    font-size: 0.68rem;
    color: var(--ink-muted);
  }

  /* 12-segment precision track */
  .segmented-track {
    display: flex;
    gap: 3px;
    height: 8px;
    margin-top: 0.5rem;
  }
  .segmented-track.stale {
    opacity: 0.45;
  }

  .seg-block {
    flex: 1;
    border-radius: 2px;
    background: var(--seg-inactive);
    transition: background 300ms var(--ease), box-shadow 300ms var(--ease);
  }

  .seg-block.claude {
    background: var(--seg-active-claude);
    box-shadow: var(--seg-glow-claude);
  }
  .seg-block.cursor {
    background: var(--seg-active-cursor);
    box-shadow: var(--seg-glow-cursor);
  }
  .seg-block.copilot {
    background: var(--seg-active-copilot);
    box-shadow: var(--seg-glow-copilot);
  }
  .seg-block.warn {
    background: var(--seg-warn);
    box-shadow: var(--seg-glow-warn);
  }

  .seg-block.scanning {
    animation: seg-scan 1.2s ease-in-out infinite;
  }

  .err {
    margin: 0.4rem 0 0;
    color: var(--danger);
    font-size: 0.72rem;
    line-height: 1.3;
  }

  /* Subwindows */
  .subwindows {
    list-style: none;
    margin: 0.5rem 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .subwindow-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.7rem;
    padding: 0.25rem 0.45rem;
    border-radius: 5px;
    background: color-mix(in srgb, var(--pale) 60%, transparent);
  }

  .w-label {
    color: var(--ink-muted);
  }
  .w-reset {
    opacity: 0.75;
  }

  .w-val {
    font-weight: 650;
    font-family: var(--font-mono);
    color: var(--ink);
  }
</style>
