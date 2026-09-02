<script lang="ts">
  import type { ProviderSnapshot } from "../types";
  import { formatHeadlinePercent, meterWidthPercent } from "../format";
  import WindowGauge from "./WindowGauge.svelte";

  interface Props {
    snap: ProviderSnapshot;
  }
  let { snap }: Props = $props();

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
</script>

<article class="row">
  <div class="top">
    <span class="chip" style={`background: ${chip}`}>{name}</span>
    <span class="pct">{formatHeadlinePercent(snap.headline_percent)}</span>
    {#if snap.stale}
      <span class="badge">Stale</span>
    {/if}
  </div>
  <div class="track">
    {#if width != null && !snap.stale}
      <div class="fill" style={`width: ${width}%`}></div>
    {:else if width != null && snap.stale}
      <div class="fill muted" style={`width: ${width}%`}></div>
    {:else}
      <div class="unknown"></div>
    {/if}
  </div>
  {#if snap.error}
    <p class="err">{snap.error}</p>
  {/if}
  {#if snap.windows.length > 1}
    <div class="windows">
      {#each snap.windows as w (w.id)}
        <WindowGauge window={w} />
      {/each}
    </div>
  {:else if snap.windows.length === 1}
    <WindowGauge window={snap.windows[0]} />
  {/if}
</article>

<style>
  .row {
    padding: 0.85rem 1.25rem;
    border-top: 1px solid var(--cream-deep);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .chip {
    color: #fff;
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    border-radius: 6px;
  }
  .pct {
    font-weight: 650;
    margin-left: auto;
  }
  .badge {
    font-size: 0.7rem;
    color: var(--danger);
    border: 1px solid var(--danger);
    border-radius: 4px;
    padding: 0.05rem 0.35rem;
  }
  .track {
    margin-top: 0.45rem;
    height: 10px;
    background: var(--cream-deep);
    border-radius: 999px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--terracotta);
  }
  .fill.muted {
    opacity: 0.45;
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
  .err {
    margin: 0.35rem 0 0;
    color: var(--danger);
    font-size: 0.8rem;
  }
  .windows {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }
</style>
