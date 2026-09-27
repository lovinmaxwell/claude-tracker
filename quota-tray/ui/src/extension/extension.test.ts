import { describe, expect, it } from "vitest";
import {
  extractCursorTokenFromCookie,
  parseAntigravitySecret,
  parseClaudeSecret,
  parseCopilotSecret,
} from "../extension/credentials";
import {
  parseModelsFallback,
  parseQuotaSummary,
} from "../extension/providers/antigravity";
import {
  parseClaudeUsageBody,
  headlineFromWindows,
  pickClaudeOrgUuid,
} from "../extension/providers/claude";
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

describe("pickClaudeOrgUuid", () => {
  it("picks first org uuid", () => {
    expect(
      pickClaudeOrgUuid([{ uuid: "org-1", name: "Acme" }, { uuid: "org-2" }])
    ).toBe("org-1");
  });

  it("is null when no orgs", () => {
    expect(pickClaudeOrgUuid([])).toBeNull();
  });

  it("is null when body is not an array", () => {
    expect(pickClaudeOrgUuid({})).toBeNull();
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

describe("antigravity credentials", () => {
  it("parses full oauth JSON (shape a)", () => {
    const out = JSON.parse(
      parseAntigravitySecret(
        '{"access_token":"ya29.test","refresh_token":"1//test-refresh","expiry_date":9999999999999}'
      )
    );
    expect(out.refresh_token).toBe("1//test-refresh");
    expect(out.access_token).toBe("ya29.test");
    expect(out.expiry_date).toBe(9999999999999);
  });

  it("parses nested keychain/jetski shape (shape b)", () => {
    const out = JSON.parse(
      parseAntigravitySecret(
        JSON.stringify({
          auth_method: "oauth",
          token: { access_token: "ya29.nested", refresh_token: "1//nested-refresh" },
        })
      )
    );
    expect(out.refresh_token).toBe("1//nested-refresh");
    expect(out.access_token).toBe("ya29.nested");
  });

  it("parses gemini oauth_creds.json flat shape (shape c)", () => {
    const out = JSON.parse(
      parseAntigravitySecret(
        JSON.stringify({
          access_token: "ya29.gemini",
          refresh_token: "1//gemini-refresh",
          scope: "openid",
          token_type: "Bearer",
          expiry: "2099-01-01T00:00:00.000Z",
        })
      )
    );
    expect(out.refresh_token).toBe("1//gemini-refresh");
    expect(out.token_type).toBe("Bearer");
    expect(out.expiry).toBe("2099-01-01T00:00:00.000Z");
  });

  it("parses a raw refresh token string (shape d)", () => {
    const out = JSON.parse(parseAntigravitySecret("1//raw-refresh-token"));
    expect(out.refresh_token).toBe("1//raw-refresh-token");
  });

  it("rejects JSON with no refresh_token anywhere", () => {
    expect(() => parseAntigravitySecret("{}")).toThrow(/no refresh_token/);
  });

  it("rejects empty input", () => {
    expect(() => parseAntigravitySecret("   ")).toThrow(/empty Antigravity/);
  });
});

describe("antigravity parseQuotaSummary", () => {
  it("converts remainingFraction to used percent and builds windows", () => {
    const { windows, headline } = parseQuotaSummary({
      groups: [
        {
          displayName: "Gemini 3 Pro",
          buckets: [
            {
              bucketId: "gemini-3-pro-daily",
              displayName: "Daily",
              remainingFraction: 0.75,
              resetTime: "2026-09-28T00:00:00Z",
            },
          ],
        },
      ],
    });
    expect(windows).toHaveLength(1);
    expect(windows[0].id).toBe("gemini-3-pro-daily");
    expect(windows[0].kind).toEqual({ Percent: { used: 25 } });
    expect(windows[0].resets_at).toBe("2026-09-28T00:00:00Z");
    expect(headline).toBe(25);
  });

  it("skips disabled buckets", () => {
    const { windows, headline } = parseQuotaSummary({
      groups: [
        {
          displayName: "Gemini",
          buckets: [
            { bucketId: "a", displayName: "A", remainingFraction: 0.5, disabled: true },
            { bucketId: "b", displayName: "B", remainingFraction: 0.9 },
          ],
        },
      ],
    });
    expect(windows).toHaveLength(1);
    expect(windows[0].id).toBe("b");
    expect(headline).toBeCloseTo(10);
  });

  it("headline is the max used percent across non-disabled buckets", () => {
    const { headline } = parseQuotaSummary({
      groups: [
        {
          displayName: "Gemini",
          buckets: [
            { bucketId: "a", displayName: "A", remainingFraction: 0.9 },
            { bucketId: "b", displayName: "B", remainingFraction: 0.2 },
          ],
        },
      ],
    });
    expect(headline).toBe(80);
  });

  it("throws when no usable buckets (no invented numbers)", () => {
    expect(() =>
      parseQuotaSummary({
        groups: [
          {
            displayName: "Gemini",
            buckets: [
              { bucketId: "a", displayName: "A", disabled: true, remainingFraction: 0.5 },
              { bucketId: "b", displayName: "B" },
            ],
          },
        ],
      })
    ).toThrow(/no usable quota buckets/);
  });

  it("throws on missing groups", () => {
    expect(() => parseQuotaSummary({})).toThrow(/missing groups/);
  });
});

describe("antigravity parseModelsFallback", () => {
  it("skips internal and empty-displayName models, aggregates worst remaining", () => {
    const { windows, headline } = parseModelsFallback({
      models: {
        m1: { displayName: "Gemini 3 Pro", isInternal: true, remainingFraction: 0.1 },
        m2: { displayName: "", remainingFraction: 0.1 },
        m3: { displayName: "Gemini 3 Pro", remainingFraction: 0.6 },
        m4: { displayName: "Gemini 3 Pro", remainingFraction: 0.3 },
      },
    });
    expect(windows).toHaveLength(1);
    expect(windows[0].label).toBe("Gemini 3 Pro");
    expect(windows[0].kind).toEqual({ Percent: { used: 70 } });
    expect(headline).toBe(70);
  });

  it("throws when no usable model quota", () => {
    expect(() =>
      parseModelsFallback({ models: { m1: { isInternal: true, remainingFraction: 0.5 } } })
    ).toThrow(/no usable model quota/);
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
