<script lang="ts">
  import { formatHeadlinePercent } from "../format";

  interface Props {
    fill: number | null;
    staleGlobal: boolean;
  }
  let { fill, staleGlobal }: Props = $props();

  const label = $derived(formatHeadlinePercent(fill));
  const status = $derived(
    staleGlobal
      ? "Something’s stale"
      : fill == null
        ? "Waiting for a reading"
        : fill >= 80
          ? "You’re tight"
          : "You’re fine"
  );
  const bodyFill = $derived(fill == null ? 0 : Math.min(100, Math.max(0, fill)));
</script>

<header class="hero">
  <div class="mascot" class:muted={staleGlobal || fill == null} aria-hidden="true">
    <div class="mascot-body">
      <div class="mascot-fill" style={`height: ${bodyFill}%`}></div>
    </div>
  </div>
  <div class="copy">
    <p class="brand">Quota Tray</p>
    <p class="headline">{label}</p>
    <p class="status">{status}</p>
  </div>
</header>

<style>
  .hero {
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: 1rem;
    align-items: center;
    padding: 1.25rem 1.25rem 0.75rem;
  }
  .brand {
    margin: 0;
    font-family: var(--font);
    font-size: 1.35rem;
    letter-spacing: -0.02em;
  }
  .headline {
    margin: 0.15rem 0;
    font-size: 1.75rem;
    font-weight: 650;
  }
  .status {
    margin: 0;
    color: var(--ink-muted);
    font-size: 0.9rem;
  }
  .mascot-body {
    width: 84px;
    height: 96px;
    border-radius: 42% 42% 38% 38%;
    background: var(--pale);
    border: 2px solid var(--cream-deep);
    position: relative;
    overflow: hidden;
  }
  .mascot-fill {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--terracotta);
    transition: height 240ms ease;
  }
  .mascot.muted .mascot-fill {
    background: var(--terracotta-soft);
    opacity: 0.45;
  }
</style>
