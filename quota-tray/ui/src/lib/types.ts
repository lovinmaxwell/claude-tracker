export type ProviderId = "Claude" | "Cursor" | "Copilot" | "OpenAI";

export type WindowKind =
  | { Percent: { used: number | null } }
  | { Currency: { used: number | null; limit: number | null; code: string } }
  | { Count: { used: number | null; limit: number | null } };

export interface UsageWindow {
  id: string;
  label: string;
  kind: WindowKind;
  resets_at: string | null;
}

export interface ProviderSnapshot {
  provider: ProviderId;
  fetched_at: string;
  windows: UsageWindow[];
  headline_percent: number | null;
  stale: boolean;
  error: string | null;
}

export interface TrayState {
  providers: ProviderSnapshot[];
  shared_mascot_fill: number | null;
}
