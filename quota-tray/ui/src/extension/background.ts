import { type AppConfig } from "../lib/config";
import {
  extractCursorTokenFromCookie,
  parseImportedSecret,
} from "./credentials";
import { configEnabled, ExtensionPoller } from "./poller";
import { fetchClaudeUsage } from "./providers/claude";
import { fetchCopilotUsage } from "./providers/copilot";
import { fetchCursorUsage } from "./providers/cursor";
import { updateActionBadge } from "./badge";
import {
  clearSecret,
  loadConfig,
  loadLastMascot,
  loadSecrets,
  loadTrayState,
  saveConfig,
  saveLastMascot,
  saveSecret,
  saveTrayState,
  type ConnectionKind,
  type ConnectionMap,
} from "./storage";
import type { ProviderId, ProviderSnapshot, TrayState } from "../lib/types";

const poller = new ExtensionPoller();
let bootstrapped = false;

type Incoming =
  | { type: "get_tray_state" }
  | { type: "get_config" }
  | { type: "set_config"; config: AppConfig }
  | { type: "refresh_now" }
  | { type: "get_connections" }
  | { type: "import_secret"; provider: ProviderId; text: string }
  | { type: "clear_secret"; provider: ProviderId };

chrome.runtime.onInstalled.addListener(() => {
  void bootstrapAndTick();
});
chrome.runtime.onStartup.addListener(() => {
  void bootstrapAndTick();
});

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === "quota-tray-poll") {
    void tick();
  }
});

chrome.runtime.onMessage.addListener((message: Incoming, _sender, sendResponse) => {
  void handle(message)
    .then(sendResponse)
    .catch((err: unknown) => {
      sendResponse({
        error: err instanceof Error ? err.message : String(err),
      });
    });
  return true;
});

async function handle(message: Incoming): Promise<unknown> {
  await bootstrap();
  switch (message.type) {
    case "get_tray_state":
      return poller.trayFromCache();
    case "get_config":
      return loadConfig();
    case "set_config": {
      const next = await saveConfig(message.config);
      poller.setEnabled(configEnabled(next));
      await scheduleAlarm(next);
      const state = await tick();
      return { config: next, state };
    }
    case "refresh_now":
      return tick();
    case "get_connections":
      return readConnections();
    case "import_secret": {
      const token = parseImportedSecret(message.provider, message.text);
      await saveSecret(message.provider, token);
      await tick();
      return { ok: true, provider: message.provider };
    }
    case "clear_secret":
      await clearSecret(message.provider);
      await tick();
      return { ok: true };
    default:
      throw new Error("unknown message");
  }
}

async function bootstrap(): Promise<void> {
  if (bootstrapped) {
    return;
  }
  const config = await loadConfig();
  const state = await loadTrayState();
  const mascot = await loadLastMascot();
  poller.restore(state, mascot, configEnabled(config));
  await scheduleAlarm(config);
  bootstrapped = true;
}

async function bootstrapAndTick(): Promise<void> {
  await bootstrap();
  await tick();
}

async function scheduleAlarm(config: AppConfig): Promise<void> {
  const minutes = Math.max(1, Math.round(config.poll_interval_secs / 60));
  await chrome.alarms.clear("quota-tray-poll");
  await chrome.alarms.create("quota-tray-poll", { periodInMinutes: minutes });
}

async function tick(): Promise<TrayState> {
  const config = await loadConfig();
  poller.setEnabled(configEnabled(config));
  const enabled = configEnabled(config);
  const results: Array<[ProviderId, ProviderSnapshot | Error]> = [];
  await Promise.all(
    enabled.map(async (id) => {
      try {
        const snap = await fetchProvider(id, config);
        results.push([id, snap]);
      } catch (e) {
        results.push([
          id,
          e instanceof Error ? e : new Error(String(e)),
        ]);
      }
    })
  );
  const state = poller.applyFetchResults(results);
  await saveTrayState(state);
  await saveLastMascot(poller.lastMascot);
  await updateActionBadge(state);
  return state;
}

async function fetchProvider(
  id: ProviderId,
  config: AppConfig
): Promise<ProviderSnapshot> {
  const token = await resolveToken(id);
  if (!token) {
    throw new Error(missingTokenHint(id));
  }
  if (id === "Claude") {
    return fetchClaudeUsage(token, config.claude_headline);
  }
  if (id === "Cursor") {
    return fetchCursorUsage(token);
  }
  if (id === "Copilot") {
    return fetchCopilotUsage(token);
  }
  throw new Error(`unsupported provider ${id}`);
}

async function resolveToken(id: ProviderId): Promise<string | null> {
  const secrets = await loadSecrets();
  const imported = secrets[id]?.trim();
  if (imported) {
    return imported;
  }
  if (id === "Cursor") {
    return readCursorCookieToken();
  }
  return null;
}

async function readCursorCookieToken(): Promise<string | null> {
  const urls = ["https://cursor.com", "https://www.cursor.com"];
  for (const url of urls) {
    const cookie = await chrome.cookies.get({
      url,
      name: "WorkosCursorSessionToken",
    });
    const token = extractCursorTokenFromCookie(cookie?.value ?? "");
    if (token) {
      return token;
    }
  }
  return null;
}

async function readConnections(): Promise<ConnectionMap> {
  const secrets = await loadSecrets();
  const cookie = await readCursorCookieToken();
  const cursor: ConnectionKind = secrets.Cursor
    ? "imported"
    : cookie
      ? "browser"
      : "missing";
  return {
    Claude: secrets.Claude ? "imported" : "missing",
    Cursor: cursor,
    Copilot: secrets.Copilot ? "imported" : "missing",
  };
}

function missingTokenHint(id: ProviderId): string {
  if (id === "Cursor") {
    return "Sign in at cursor.com in this browser, or import a token in Settings";
  }
  if (id === "Claude") {
    return "Import ~/.claude/.credentials.json in Settings";
  }
  return "Import github-copilot apps.json in Settings";
}

void bootstrapAndTick();
