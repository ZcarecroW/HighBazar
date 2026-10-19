import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";

// In a plain browser (`npm run dev`) the backend is mocked for UI previews.
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (import.meta.env.DEV && !isTauri()) {
    const m = await import("./mock");
    return m.mockInvoke(cmd, args ?? {});
  }
  return tauriInvoke<T>(cmd, args);
}
import type {
  AiProfile,
  AiTestResult,
  ModelInfo,
  Bootstrap,
  Config,
  LoginStart,
  ModLogEntry,
  UsageStats,
} from "./types";

export const api = {
  bootstrap: () => invoke<Bootstrap>("get_bootstrap"),
  saveConfig: (config: Config) => invoke<Config>("save_config", { config }),
  keyStatus: () => invoke<Record<string, boolean>>("get_key_status"),
  setAiKey: (keyRef: string, key: string) => invoke<void>("set_ai_key", { keyRef, key }),
  aiTest: (profile: AiProfile, prompt?: string) => invoke<AiTestResult>("ai_test", { profile, prompt }),
  aiListModels: (profile: AiProfile) => invoke<ModelInfo[]>("ai_list_models", { profile }),
  detectClaudeCli: (path?: string) => invoke<string | null>("detect_claude_cli", { path }),
  twitchLoginStart: (kind: "streamer" | "bot", level?: string) => invoke<LoginStart>("twitch_login_start", { kind, level }),
  twitchLoginCancel: (loginId: string) => invoke<void>("twitch_login_cancel", { loginId }),
  twitchRemoveAccount: (id: string) => invoke<Config>("twitch_remove_account", { id }),
  botStart: (channelId: string, sandbox: boolean) => invoke<void>("bot_start", { channelId, sandbox }),
  botStop: () => invoke<void>("bot_stop"),
  botRunning: () => invoke<{ channelId: string; sandbox: boolean } | null>("bot_running"),
  botControl: (action: string, payload?: unknown) => invoke<void>("bot_control", { action, payload }),
  modlog: (channelId: string, limit = 500) => invoke<ModLogEntry[]>("get_modlog", { channelId, limit }),
  usage: () => invoke<Record<string, Record<string, UsageStats>>>("get_usage"),
  clearResearchCache: () => invoke<void>("clear_research_cache"),
  openDataDir: () => invoke<void>("open_data_dir"),
  emoteUrls: () => invoke<Record<string, string>>("get_emote_urls"),
  twitchApi: <T = any>(accountId: string, method: string, path: string, query?: [string, string][], body?: unknown) =>
    invoke<T>("twitch_api", { accountId, method, path, query, body }),
};

export function errorText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

export function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v));
}

export function newId(): string {
  return crypto.randomUUID().replace(/-/g, "").slice(0, 12);
}
