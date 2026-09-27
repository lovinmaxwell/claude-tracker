import { aggregateMascotFill } from "./aggregate";
import type { AppConfig } from "../lib/config";
import type { ProviderId, ProviderSnapshot, TrayState } from "../lib/types";

export const WAITING_FIRST_READING = "Waiting for the first reading";

export function waitingSnapshot(id: ProviderId): ProviderSnapshot {
  return {
    provider: id,
    fetched_at: new Date(0).toISOString(),
    windows: [],
    headline_percent: null,
    stale: true,
    error: WAITING_FIRST_READING,
  };
}

export class ExtensionPoller {
  last = new Map<ProviderId, ProviderSnapshot>();
  lastMascot: number | null = null;
  enabled: ProviderId[] = [];

  setEnabled(enabled: ProviderId[]): TrayState {
    this.enabled = [...enabled];
    const keep = new Set(enabled);
    for (const id of [...this.last.keys()]) {
      if (!keep.has(id)) {
        this.last.delete(id);
      }
    }
    return this.trayFromCache();
  }

  applyFetchResults(
    results: Array<[ProviderId, ProviderSnapshot | Error]>
  ): TrayState {
    for (const [id, outcome] of results) {
      if (outcome instanceof Error) {
        const prev = this.last.get(id);
        if (prev) {
          this.last.set(id, { ...prev, stale: true, error: outcome.message });
        } else {
          this.last.set(id, {
            provider: id,
            fetched_at: new Date().toISOString(),
            windows: [],
            headline_percent: null,
            stale: true,
            error: outcome.message,
          });
        }
      } else {
        this.last.set(id, { ...outcome, stale: false, error: null });
      }
    }
    const providers = this.snapshotsForEnabled();
    const shared = aggregateMascotFill(providers, this.lastMascot);
    if (providers.some((p) => !p.stale && p.error == null && p.headline_percent != null)) {
      this.lastMascot = shared;
    }
    return { providers, shared_mascot_fill: shared };
  }

  restore(state: TrayState | null, lastMascot: number | null, enabled: ProviderId[]): void {
    this.enabled = [...enabled];
    this.lastMascot = lastMascot;
    this.last.clear();
    for (const snap of state?.providers ?? []) {
      this.last.set(snap.provider, snap);
    }
  }

  trayFromCache(): TrayState {
    const providers = this.snapshotsForEnabled();
    return {
      providers,
      shared_mascot_fill: aggregateMascotFill(providers, this.lastMascot),
    };
  }

  private snapshotsForEnabled(): ProviderSnapshot[] {
    return this.enabled.map(
      (id) => this.last.get(id) ?? waitingSnapshot(id)
    );
  }
}

export function configEnabled(config: AppConfig): ProviderId[] {
  return config.enabled.filter(
    (id): id is ProviderId => id === "Claude" || id === "Cursor" || id === "Copilot"
  );
}
