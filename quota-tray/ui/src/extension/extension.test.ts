import { describe, expect, it } from "vitest";
import {
  extractCursorTokenFromCookie,
  parseClaudeSecret,
  parseCopilotSecret,
} from "../extension/credentials";
import { parseClaudeUsageBody, headlineFromWindows } from "../extension/providers/claude";
import { parseCopilotUser, copilotHeadlinePercent } from "../extension/providers/copilot";
import { parseCursorUsageBody, cursorHeadlinePercent } from "../extension/providers/cursor";
import { ExtensionPoller } from "../extension/poller";
import { sanitizeConfig } from "../lib/config";
import type { ProviderSnapshot } from "../lib/types";

describe("cursorHeadlinePercent", () => {
  it("prefers totalPercentUsed", () => {
    expect(
      cursorHeadlinePercent({
        totalPercentUsed: 41,
        autoPercentUsed: 10,
        apiPercentUsed: 90,
      })
    ).toBe(41);
  });

  it("falls back to max auto/api", () => {
    expect(
      cursorHeadlinePercent({ autoPercentUsed: 10, apiPercentUsed: 55 })
    ).toBe(55);
  });

  it("falls back to spend over limit", () => {
    expect(cursorHeadlinePercent({ totalSpend: 250, limit: 1000 })).toBe(25);
  });

  it("is null when no signal", () => {
    expect(cursorHeadlinePercent({})).toBeNull();
  });
});

describe("parseCursorUsageBody", () => {
  it("rejects empty plan", () => {
    expect(() => parseCursorUsageBody({})).toThrow(/no planUsage/);
  });

  it("maps spend-only to period window", () => {
    const snap = parseCursorUsageBody({
      planUsage: { totalSpend: 250.0, limit: 1000.0 },
    });
    expect(snap.headline_percent).toBe(25);
    expect(snap.windows).toHaveLength(1);
    expect(snap.windows[0].id).toBe("period");
  });
});

describe("claude credentials", () => {
  it("parses locked sample blob", () => {
    const token = parseClaudeSecret(
      '{"claudeAiOauth":{"accessToken":"sk-ant-oat-test","expiresAt":9999999999999}}'
    );
    expect(token).toBe("sk-ant-oat-test");
  });

  it("rejects missing oauth object", () => {
    expect(() => parseClaudeSecret("{}")).toThrow(/missing claudeAiOauth/);
  });
});

describe("claude usage parse", () => {
  it("highest headline is max of both windows", () => {
    const snap = parseClaudeUsageBody(
      {
        five_hour: { utilization: 40 },
        seven_day: { utilization: 70 },
      },
      "Highest"
    );
    expect(snap.headline_percent).toBe(70);
    expect(snap.windows).toHaveLength(2);
  });

  it("missing windows yield none headline not zero", () => {
    const snap = parseClaudeUsageBody({}, "Highest");
    expect(snap.headline_percent).toBeNull();
    expect(snap.windows).toHaveLength(0);
  });

  it("headlineFromWindows five-hour only", () => {
    expect(
      headlineFromWindows(
        [
          {
            id: "five_hour",
            label: "5-hour",
            kind: { Percent: { used: 12 } },
            resets_at: null,
          },
        ],
        "FiveHour"
      )
    ).toBe(12);
  });
});

describe("copilot parse", () => {
  it("parses oauth token from apps.json", () => {
    const token = parseCopilotSecret(
      JSON.stringify({
        "github.com:Iv1.b507a08c87ecfe23": {
          user: "octo",
          oauth_token: "gho_testtoken123",
        },
      })
    );
    expect(token).toBe("gho_testtoken123");
  });

  it("premium remaining inverts to used", () => {
    expect(
      copilotHeadlinePercent({
        premium_interactions: { percent_remaining: 40.0, unlimited: false },
        chat: { percent_remaining: 100.0, unlimited: true },
      })
    ).toBe(60);
  });

  it("maps premium used", () => {
    const snap = parseCopilotUser({
      copilot_plan: "individual_pro",
      quota_snapshots: {
        premium_interactions: { percent_remaining: 25.0, unlimited: false },
        chat: { unlimited: true },
        completions: { unlimited: true },
      },
    });
    expect(snap.headline_percent).toBe(75);
  });
});

describe("cursor cookie token", () => {
  it("splits userId::jwt", () => {
    expect(extractCursorTokenFromCookie("user123::abc.def.ghi")).toBe(
      "abc.def.ghi"
    );
  });

  it("decodes %3A%3A", () => {
    expect(extractCursorTokenFromCookie("user123%3A%3Ajwt-token")).toBe(
      "jwt-token"
    );
  });
});

describe("sanitizeConfig", () => {
  it("clamps interval and orders enabled providers", () => {
    const next = sanitizeConfig({
      poll_interval_secs: 15,
      enabled: ["Copilot", "Claude", "Claude", "Cursor"],
      claude_headline: "FiveHour",
    });
    expect(next.poll_interval_secs).toBe(60);
    expect(next.enabled).toEqual(["Claude", "Cursor", "Copilot"]);
  });
});

describe("extension poller stale-on-error", () => {
  it("keeps last good windows when a later fetch fails", () => {
    const poller = new ExtensionPoller();
    poller.setEnabled(["Cursor"]);
    const live: ProviderSnapshot = {
      provider: "Cursor",
      fetched_at: new Date().toISOString(),
      windows: [
        {
          id: "period",
          label: "Current period",
          kind: { Percent: { used: 41 } },
          resets_at: null,
        },
      ],
      headline_percent: 41,
      stale: false,
      error: null,
    };
    poller.applyFetchResults([["Cursor", live]]);
    const after = poller.applyFetchResults([
      ["Cursor", new Error("network down")],
    ]);
    expect(after.providers[0].stale).toBe(true);
    expect(after.providers[0].headline_percent).toBe(41);
    expect(after.providers[0].error).toBe("network down");
    expect(after.shared_mascot_fill).toBe(41);
  });

  it("never writes a live fake zero on first failure", () => {
    const poller = new ExtensionPoller();
    poller.setEnabled(["Claude"]);
    const after = poller.applyFetchResults([
      ["Claude", new Error("missing token")],
    ]);
    expect(after.providers[0].headline_percent).toBeNull();
    expect(after.shared_mascot_fill).toBeNull();
  });
});
