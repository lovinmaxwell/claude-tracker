export function clampPollInterval(secs: number): number {
  if (!Number.isFinite(secs)) return 60;
  return Math.min(120, Math.max(60, Math.round(secs)));
}
