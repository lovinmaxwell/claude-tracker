/** Never treat null/undefined as a live 0%. */
export function formatHeadlinePercent(used: number | null | undefined): string {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return "—";
  }
  return `${Math.round(used)}%`;
}

export function meterWidthPercent(used: number | null | undefined): number | null {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return null;
  }
  return Math.min(100, Math.max(0, used));
}

/**
 * Maps a percentage (0..100) to discrete illuminated segments (0..totalSegments).
 * Returns null if percentage is unknown/null.
 */
export function segmentedActiveCount(
  used: number | null | undefined,
  totalSegments = 12
): number | null {
  if (used === null || used === undefined || Number.isNaN(used)) {
    return null;
  }
  const clamped = Math.min(100, Math.max(0, used));
  return Math.round((clamped / 100) * totalSegments);
}

/**
 * Returns true if a segment index is in the warning/critical zone (top 25% of gauge).
 */
export function isWarningSegment(
  segmentIndex: number,
  totalSegments = 12
): boolean {
  return segmentIndex >= Math.floor(totalSegments * 0.75);
}
