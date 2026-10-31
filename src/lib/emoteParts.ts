// Splits chat text into plain text and emotes. Zero-width emotes (7TV
// "zero-width", BTTV hats) are drawn over the emote before them, like the
// 7TV and BTTV extensions do; on their own they stay normal emotes.

export interface EmoteImage {
  text: string;
  url: string;
}
export interface EmotePart extends EmoteImage {
  /** Zero-width emotes stacked on top of this one. */
  over?: EmoteImage[];
}

/**
 * `local`: emotes of this message (native Twitch emotes, by name);
 * `known`: the channel's emotes; `zeroWidth`: names of overlay emotes.
 */
export function emoteParts(
  text: string,
  local: Record<string, string> | undefined,
  known: Record<string, string>,
  zeroWidth: ReadonlySet<string>,
): EmotePart[] {
  const out: EmotePart[] = [];
  let buf = "";
  for (const token of text.split(/(\s+)/)) {
    // Own keys only: "constructor" or "toString" in chat are words, not emotes.
    const native = local && Object.hasOwn(local, token) ? local[token] : undefined;
    const url = native ?? (Object.hasOwn(known, token) ? known[token] : "");
    if (!url) {
      buf += token;
      continue;
    }
    const last = out[out.length - 1];
    // Only whitespace since the previous emote: stack on it.
    if (!native && zeroWidth.has(token) && last?.url && !buf.trim()) {
      (last.over ??= []).push({ text: token, url });
      buf = "";
      continue;
    }
    if (buf) out.push({ text: buf, url: "" });
    buf = "";
    out.push({ text: token, url });
  }
  if (buf) out.push({ text: buf, url: "" });
  return out;
}
