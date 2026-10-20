import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { api, errorText } from "./api";
import { useApp } from "../stores/app";
import type { ChannelLevel } from "./types";

// Mirrors the scope tiers in src-tauri/src/twitch/auth.rs.
export const LEVEL_SCOPES: Record<ChannelLevel, string[]> = {
  simple: ["channel:manage:broadcast", "user:read:emotes"],
  advanced: [
    "moderator:manage:chat_settings",
    "moderator:manage:announcements",
    "moderator:manage:shoutouts",
    "channel:manage:raids",
    "clips:edit",
    "user:read:follows",
  ],
  expert: [
    "channel:manage:polls",
    "channel:manage:predictions",
    "channel:manage:redemptions",
    "channel:manage:moderators",
    "channel:manage:vips",
    "moderator:manage:automod_settings",
    "moderator:manage:blocked_terms",
    "moderator:manage:shield_mode",
    "moderation:read",
    "channel:edit:commercial",
    "channel:read:ads",
    "channel:manage:ads",
    "channel:manage:schedule",
    "channel:read:subscriptions",
    "channel:read:editors",
    "user:edit",
    "user:manage:chat_color",
    "moderator:manage:unban_requests",
    "moderator:manage:suspicious_users",
    "channel:read:hype_train",
  ],
};

export function scopesFor(level: ChannelLevel): string[] {
  const s = [...LEVEL_SCOPES.simple];
  if (level !== "simple") s.push(...LEVEL_SCOPES.advanced);
  if (level === "expert") s.push(...LEVEL_SCOPES.expert);
  return s;
}

/** Helix access for the active channel's streamer account. */
export function useTwitch() {
  const app = useApp();
  const { t } = useI18n();
  const account = computed(() => app.account(app.activeChannel?.streamerAccountId));
  const id = computed(() => account.value?.id ?? "");

  /** Calls Helix; shows a toast and returns null on errors (unless `quiet`). */
  async function tw<T = any>(
    method: string,
    path: string,
    query: Record<string, string | number | boolean | undefined> = {},
    body?: unknown,
    quiet = false,
  ): Promise<T | null> {
    if (!id.value) return null;
    const q: [string, string][] = Object.entries(query)
      .filter(([, v]) => v !== undefined && v !== "")
      .map(([k, v]) => [k, String(v)]);
    try {
      return await api.twitchApi<T>(id.value, method, path, q, body);
    } catch (e) {
      if (!quiet) app.toast(explain(errorText(e)), "error", 8000);
      return null;
    }
  }

  /** Turns "Twitch API error 403: …" into a localized hint plus Twitch's own message. */
  function explain(raw: string): string {
    const m = raw.match(/Twitch API error (\d{3}): ([\s\S]*)/);
    if (!m) return t("stream.apiError", { e: raw });
    const status = m[1]!;
    const msg = m[2]!.trim();
    const known = ["400", "401", "403", "404", "409", "422", "425", "429"];
    const hint = known.includes(status) ? t(`stream.errors.${status}`) : "";
    return hint ? `${hint} (Twitch: ${msg})` : t("stream.apiError", { e: `${status} ${msg}` });
  }

  async function userByLogin(login: string) {
    const r = await tw<{ data: { id: string; login: string; display_name: string }[] }>("GET", "/users", {
      login: login.trim().replace(/^@/, "").toLowerCase(),
    });
    const u = r?.data?.[0];
    if (!u && r) app.toast(t("stream.userNotFound", { login }), "error");
    return u ?? null;
  }

  return { app, account, id, tw, userByLogin };
}
