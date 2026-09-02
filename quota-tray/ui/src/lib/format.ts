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
