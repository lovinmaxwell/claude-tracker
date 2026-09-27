import type { ProviderId } from "./types";
import { isChromeExtension } from "./platform";
import { clampPollInterval } from "./settings_validate";

export type ClaudeHeadline = "FiveHour" | "SevenDay" | "Highest";

export interface AppConfig {
  poll_interval_secs: number;
  enabled: ProviderId[];
  claude_headline: ClaudeHeadline;
}

export function defaultConfig(): AppConfig {
  return {
    poll_interval_secs: 60,
    enabled: isChromeExtension() ? ["Cursor"] : ["Claude"],
    claude_headline: "Highest",
  };
}

export function sanitizeConfig(config: AppConfig): AppConfig {
  const order: ProviderId[] = ["Claude", "Cursor", "Copilot", "OpenAI"];
  const enabled = order.filter((id) => config.enabled.includes(id));
  return {
    poll_interval_secs: clampPollInterval(config.poll_interval_secs),
    enabled,
    claude_headline: config.claude_headline ?? "Highest",
  };
}
