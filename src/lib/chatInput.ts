// Text logic of the chat input, shared by the app (ChatInput.vue). The overlay
// page has the same rules in plain JavaScript (src-tauri/overlay/overlay.js).

export interface Suggestion {
  insert: string;
  label: string;
  kind: "emote" | "emoji" | "user";
  provider: "twitch" | "bttv" | "ffz" | "7tv" | "emoji" | "user";
  url: string;
  scope: "channel" | "sub" | "global" | "";
  favorite: boolean;
}
export type SuggestMode = "colon" | "mention" | "tab" | "browse";

/**
 * The ":" or "@" word the caret is in, if any. ":" only counts at the start
 * of a word ("Hinweis: …" does not open suggestions); ":7 pog" keeps the
 * provider shortcut (7, b, f, t, e) together with its query.
 */
export function activeToken(text: string, caret: number): { mode: SuggestMode; query: string; start: number } | null {
  const before = text.slice(0, caret);
  // Only letters, digits, "_" and "-" after the colon: ":)" and ":D" stay smileys.
  const colon = before.match(/(^|\s):((?:[7bfteBFTE] )?[\p{L}\p{N}_-]*)$/u);
  if (colon) return { mode: "colon", query: colon[2]!, start: caret - colon[2]!.length - 1 };
  const at = before.match(/(^|\s)@([\p{L}\p{N}_]*)$/u);
  if (at) return { mode: "mention", query: at[2]!, start: caret - at[2]!.length - 1 };
  return null;
}

/**
 * Whether Enter takes the highlighted suggestion (instead of sending): not for
 * a single letter after ":" (":D", ":P" are smileys) unless one was picked
 * with the arrow keys or the mouse.
 */
export function enterPicks(tok: { mode: SuggestMode; query: string }, chosen: boolean): boolean {
  return chosen || tok.mode !== "colon" || tok.query.length >= 2;
}

/**
 * Replaces the token (and the rest of its word after the caret) with the
 * suggestion plus one space – emotes only work as separate words.
 */
export function applySuggestion(text: string, start: number, caret: number, s: Suggestion): { text: string; caret: number } {
  const after = text.slice(caret).replace(/^\S*/, "");
  const ins = after.startsWith(" ") ? s.insert : `${s.insert} `;
  const next = text.slice(0, start) + ins + after;
  const pos = start + ins.length + (after.startsWith(" ") ? 1 : 0);
  return { text: next, caret: pos };
}

export function providerLabel(s: Suggestion, t: (k: string) => string): string {
  if (s.kind === "emoji") return "Emoji";
  if (s.scope === "sub") return t("chatInput.sub");
  return { twitch: "Twitch", bttv: "BTTV", ffz: "FFZ", "7tv": "7TV", emoji: "Emoji", user: "" }[s.provider];
}
