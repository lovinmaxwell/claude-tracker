<script lang="ts">
  import { formatHeadlinePercent } from "../format";
  import type { ProviderSnapshot } from "../types";

  interface Props {
    fill: number | null;
    staleGlobal: boolean;
    providers?: ProviderSnapshot[];
  }
  let { fill, staleGlobal, providers = [] }: Props = $props();

  const activeCount = $derived(
    providers.filter((p) => p.headline_percent != null).length
  );
  const hasErrors = $derived(providers.some((p) => p.error != null));

  const pulseTone = $derived(
    hasErrors
      ? "error"
      : staleGlobal
        ? "warn"
        : fill == null
          ? "wait"
          : "ok"
  );
  const peakText = $derived(
    fill != null ? `PEAK ${formatHeadlinePercent(fill)}` : "WAITING"
  );
</script>

<header class="hud-topbar">
  <div class="brand-group">
    <span class={`status-dot dot-${pulseTone}`} aria-label={`Status: ${pulseTone}`}></span>
    <h1 class="hud-title">Quota Tray</h1>
    <span class="active-badge">{activeCount} ACTIVE</span>
  </div>

  <div class="peak-badge" class:warn={fill != null && fill >= 75}>
    {peakText}
  </div>
</header>

<style>
  .hud-topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem 0.65rem;
    border-bottom: 0.5px solid var(--popover-stroke);
    animation: rise-in 320ms var(--ease) both;
  }

  .brand-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.08);
  }
  .dot-ok {
    background: var(--ok);
    animation: pulse-heartbeat 2.4s ease-in-out infinite;
  }
  .dot-warn {
    background: var(--danger);
  }
  .dot-error {
    background: #e07060;
    animation: pulse-heartbeat 1.2s ease-in-out infinite;
  }
  .dot-wait {
    background: var(--ink-muted);
  }

  .hud-title {
    margin: 0;
    font-size: 0.92rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--ink);
  }

  .active-badge {
    font-size: 0.62rem;
    font-family: var(--font-mono);
    font-weight: 650;
    letter-spacing: 0.04em;
    color: var(--ink-muted);
    background: color-mix(in srgb, var(--card-bg) 80%, transparent);
    border: 0.5px solid var(--card-border);
    padding: 1px 5px;
    border-radius: 4px;
  }

  .peak-badge {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    font-weight: 750;
    padding: 2px 7px;
    border-radius: 5px;
    color: var(--ink);
    background: color-mix(in srgb, var(--pale) 85%, transparent);
    border: 0.5px solid var(--card-border);
  }
  .peak-badge.warn {
    color: var(--seg-warn);
    background: color-mix(in srgb, var(--seg-warn) 12%, transparent);
    border-color: color-mix(in srgb, var(--seg-warn) 35%, transparent);
  }
</style>
