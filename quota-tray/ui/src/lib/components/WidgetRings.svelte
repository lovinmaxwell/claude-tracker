<script lang="ts">
  import { onMount } from "svelte";
  import type { WidgetRing } from "../widget";

  interface Props {
    rings: WidgetRing[];
    size?: number;
    stroke?: number;
    /** Gap between concentric rings, in px. */
    gap?: number;
    /** Colour of the glyph in the middle. */
    glyphColor?: string;
  }
  let { rings, size = 104, stroke = 9, gap = 3, glyphColor = "var(--ink-muted)" }: Props = $props();

  let ready = $state(false);

  onMount(() => {
    const id = requestAnimationFrame(() => {
      ready = true;
    });
    return () => cancelAnimationFrame(id);
  });

  const center = $derived(size / 2);
  const tracks = $derived(
    rings.map((ring, i) => {
      const radius = size / 2 - stroke / 2 - i * (stroke + gap);
      const circumference = 2 * Math.PI * radius;
      const fraction = ring.used === null ? 0 : ring.used / 100;
      return {
        ring,
        radius,
        circumference,
        // A sliver stays visible at tiny values so "1%" never looks like "unknown".
        offset: circumference * (1 - (ready ? Math.max(fraction, fraction > 0 ? 0.015 : 0) : 0)),
        color: ring.tone === "hot" ? "var(--seg-warn)" : `var(--seg-active-${ring.accent})`,
      };
    })
  );
</script>

<svg
  class="rings"
  width={size}
  height={size}
  viewBox={`0 0 ${size} ${size}`}
  role="img"
  aria-label={rings
    .map((r) => `${r.label} ${r.used === null ? "unknown" : `${Math.round(r.used)} percent`}`)
    .join(", ")}
>
  {#each tracks as t (t.ring.id)}
    <circle
      class="track"
      class:unknown={t.ring.used === null}
      cx={center}
      cy={center}
      r={t.radius}
      stroke-width={stroke}
    />
    {#if t.ring.used !== null}
      <circle
        class="fill"
        cx={center}
        cy={center}
        r={t.radius}
        stroke-width={stroke}
        stroke={t.color}
        stroke-dasharray={t.circumference}
        stroke-dashoffset={t.offset}
        transform={`rotate(-90 ${center} ${center})`}
      />
    {/if}
  {/each}

  <!-- Spark glyph: eight rays, the centre of the dial. -->
  <g class="glyph" transform={`translate(${center} ${center})`} style={`color: ${glyphColor}`}>
    {#each [0, 45, 90, 135] as angle}
      <line x1="0" y1="-8" x2="0" y2="8" transform={`rotate(${angle})`} />
    {/each}
  </g>
</svg>

<style>
  .rings {
    display: block;
    flex-shrink: 0;
  }
  .track {
    fill: none;
    stroke: var(--seg-inactive);
  }
  .track.unknown {
    stroke-dasharray: 2 5;
    stroke-linecap: round;
    opacity: 0.8;
  }
  .fill {
    fill: none;
    stroke-linecap: round;
    transition:
      stroke-dashoffset 900ms var(--ease-fill),
      stroke 300ms var(--ease);
  }
  .glyph line {
    stroke: currentColor;
    stroke-width: 2.4;
    stroke-linecap: round;
    transition: stroke 300ms var(--ease);
  }
  @media (prefers-reduced-motion: reduce) {
    .fill,
    .glyph line {
      transition: none;
    }
  }
</style>
