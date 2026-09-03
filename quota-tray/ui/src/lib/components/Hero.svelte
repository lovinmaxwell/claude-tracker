<script lang="ts">
  import { onMount } from "svelte";
  import { formatHeadlinePercent } from "../format";
  import type { ProviderSnapshot } from "../types";

  interface Props {
    fill: number | null;
    staleGlobal: boolean;
    providers?: ProviderSnapshot[];
  }
  let { fill, staleGlobal, providers = [] }: Props = $props();
  let fillReady = $state(false);

  onMount(() => {
    const id = requestAnimationFrame(() => {
      fillReady = true;
    });
    return () => cancelAnimationFrame(id);
  });

  const label = $derived(formatHeadlinePercent(fill));
  const tone = $derived(
    staleGlobal
      ? "warn"
      : fill == null
        ? "wait"
        : fill >= 80
          ? "tight"
          : "ok"
  );
  const status = $derived(
    tone === "warn"
      ? "Stale reading — last good kept"
      : tone === "wait"
        ? "Waiting for first reading"
        : tone === "tight"
          ? "Quota getting tight"
          : "Looking fine"
  );
  const badge = $derived(
    tone === "warn"
      ? "Stale"
      : tone === "wait"
        ? "Waiting"
        : tone === "tight"
          ? "High"
          : "OK"
  );
  const bodyFill = $derived(fill == null ? 0 : Math.min(100, Math.max(0, fill)));

  const chipFor = (id: string) =>
    id === "Claude"
      ? "var(--chip-claude)"
      : id === "Cursor"
        ? "var(--chip-cursor)"
        : "var(--chip-copilot)";

  const ring = $derived.by(() => {
    const live = providers.filter((p) => p.headline_percent != null);
    if (live.length === 0) {
      return `conic-gradient(var(--terracotta) ${bodyFill * 3.6}deg, var(--cream-deep) 0)`;
    }
    const slice = 360 / live.length;
    const parts: string[] = [];
    live.forEach((p, i) => {
      const pct = Math.min(100, Math.max(0, p.headline_percent ?? 0));
      const start = i * slice;
      const filled = start + (pct / 100) * slice;
      const end = (i + 1) * slice;
      const color = chipFor(p.provider);
      parts.push(`${color} ${start}deg ${filled}deg`);
      parts.push(`var(--cream-deep) ${filled}deg ${end}deg`);
    });
    return `conic-gradient(${parts.join(", ")})`;
  });
</script>

<header class="hero">
  <div
    class="mascot-ring"
    class:muted={staleGlobal || fill == null}
    style={`background: ${ring}`}
    aria-hidden="true"
  >
    <div class="mascot">
      <div
        class="mascot-fill"
        class:ready={fillReady}
        style={`--fill: ${bodyFill}%`}
      ></div>
      <div class="mascot-center">
        <span class="ring-pct">{label}</span>
      </div>
    </div>
  </div>
  <div class="copy">
    <div class="topline">
      <p class="brand">Quota Tray</p>
      <span class={`pill tone-${tone}`}>{badge}</span>
    </div>
    <p class="status">{status}</p>
    <p class="meta">Peak used · healthy providers</p>
  </div>
</header>

<style>
  .hero {
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: 0.9rem;
    align-items: center;
    padding: 1rem 1.1rem 0.75rem;
    animation: rise-in 360ms var(--ease) both;
  }
  .mascot-ring {
    width: 92px;
    height: 92px;
    border-radius: 50%;
    padding: 6px;
    box-shadow: var(--shadow-soft);
    transition:
      filter 240ms var(--ease),
      background 650ms var(--ease-fill);
  }
  .mascot-ring.muted {
    filter: saturate(0.7);
  }
  .mascot {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background: var(--cream);
    border: 1px solid var(--cream-line);
    position: relative;
    overflow: hidden;
  }
  .mascot-fill {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: var(--fill, 0%);
    background: var(--terracotta);
    transform-origin: bottom center;
    transform: scaleY(0);
  }
  .mascot-fill.ready {
    transform: scaleY(1);
    animation: fill-y var(--fill-duration) var(--ease-fill) both;
  }
  .mascot-center {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 1;
    background: radial-gradient(circle at 50% 42%, color-mix(in srgb, var(--cream) 55%, transparent), transparent 65%);
  }
  .ring-pct {
    font-size: 1.05rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.03em;
    color: var(--ink);
  }
  .topline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .brand {
    margin: 0;
    font-family: var(--font);
    font-size: 1.2rem;
    font-weight: 700;
    letter-spacing: -0.03em;
  }
  .pill {
    font-size: 0.66rem;
    font-weight: 650;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding: 0.18rem 0.45rem;
    border-radius: 999px;
    border: 1px solid transparent;
  }
  .tone-ok {
    color: var(--ok);
    background: color-mix(in srgb, var(--ok) 12%, var(--sheet));
    border-color: color-mix(in srgb, var(--ok) 28%, var(--cream-line));
  }
  .tone-tight {
    color: var(--terracotta-deep);
    background: color-mix(in srgb, var(--terracotta) 14%, var(--sheet));
    border-color: color-mix(in srgb, var(--terracotta) 35%, var(--cream-line));
  }
  .tone-wait,
  .tone-warn {
    color: var(--ink-muted);
    background: var(--cream-deep);
    border-color: var(--cream-line);
  }
  .tone-warn {
    color: var(--danger);
  }
  .status {
    margin: 0.35rem 0 0;
    color: var(--ink);
    font-size: 0.92rem;
    font-weight: 600;
    line-height: 1.3;
  }
  .meta {
    margin: 0.15rem 0 0;
    color: var(--ink-muted);
    font-size: 0.76rem;
  }
</style>
