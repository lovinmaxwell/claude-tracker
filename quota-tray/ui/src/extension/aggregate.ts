import type { ProviderSnapshot } from "../lib/types";

export function isHealthy(snap: ProviderSnapshot): boolean {
  return !snap.stale && snap.error == null && snap.headline_percent != null;
}

/** max(headline_percent) among healthy providers; else lastSuccessful; never a fake 0. */
export function aggregateMascotFill(
  providers: ProviderSnapshot[],
  lastSuccessful: number | null
): number | null {
  let maxUsed: number | null = null;
  for (const snap of providers) {
    if (!isHealthy(snap)) {
      continue;
    }
    if (snap.headline_percent == null) {
      continue;
    }
    maxUsed =
      maxUsed == null
        ? snap.headline_percent
        : Math.max(maxUsed, snap.headline_percent);
  }
  return maxUsed ?? lastSuccessful;
}
