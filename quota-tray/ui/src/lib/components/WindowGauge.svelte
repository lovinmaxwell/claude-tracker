<script lang="ts">
  import type { UsageWindow } from "../types";
  import { formatHeadlinePercent, meterWidthPercent } from "../format";

  interface Props {
    window: UsageWindow;
  }
  let { window: w }: Props = $props();

  const used = $derived(
    "Percent" in w.kind ? w.kind.Percent.used : null
  );
  const width = $derived(meterWidthPercent(used));
</script>

<div class="gauge">
  <div class="row">
    <span>{w.label}</span>
    <span>{formatHeadlinePercent(used)}</span>
  </div>
  <div class="track" aria-hidden="true">
    {#if width != null}
      <div class="fill" style={`width: ${width}%`}></div>
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
    margin-top: 0.4rem;
  }
  .row {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--ink-muted);
  }
  .track {
    height: 8px;
    background: var(--cream-deep);
    border-radius: 999px;
    overflow: hidden;
    margin-top: 0.25rem;
  }
  .fill {
    height: 100%;
    background: var(--terracotta);
  }
  .unknown {
    height: 100%;
    width: 100%;
    background: repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 4px,
      rgba(0, 0, 0, 0.06) 4px,
      rgba(0, 0, 0, 0.06) 8px
    );
  }
  .reset {
    margin: 0.2rem 0 0;
    font-size: 0.72rem;
    color: var(--ink-muted);
  }
</style>
