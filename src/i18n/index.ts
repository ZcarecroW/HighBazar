import { createI18n } from "vue-i18n";
import type en from "./en";

export type Locale = "en" | "de";
type Messages = typeof en;

export function detectLocale(setting: string): Locale {
  if (setting === "en" || setting === "de") return setting;
  return navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

// Only the language in use is loaded (each one is about 100 KB of text);
// the other follows when the language is switched. Both have the same keys
// (de.ts is typed against en.ts), so nothing needs a fallback.
const loaders: Record<Locale, () => Promise<{ default: Messages }>> = {
  en: () => import("./en"),
  de: () => import("./de"),
};

export const i18n = createI18n({
  legacy: false,
  locale: "en",
  fallbackLocale: "en",
  messages: {} as Record<Locale, Messages>,
});

export async function loadLocale(l: Locale) {
  if (!i18n.global.availableLocales.includes(l)) i18n.global.setLocaleMessage(l, (await loaders[l]()).default);
}

export async function setLocale(setting: string) {
  const l = detectLocale(setting);
  await loadLocale(l);
  i18n.global.locale.value = l;
  document.documentElement.lang = l;
}

interface Named {
  id: string;
  name: string;
  description?: string;
  builtin?: boolean;
}

/** Localized name of a built-in profile (user profiles keep their own name). */
export function pname(p: Named): string {
  const k = `presets.${p.id}.name`;
  return p.builtin && i18n.global.te(k) ? i18n.global.t(k) : p.name;
}

// Reasons produced by the instant rules in the backend (English) → localized.
const ruleReasons: [RegExp, string, string[]][] = [
  [/^same message (\d+)× within a minute$/, "repeat", ["n"]],
  [/^link not allowed \((.*)\)$/s, "link", ["url"]],
  [/^blocked term "(.*)"$/s, "blockedTerm", ["term"]],
  [/^known spam phrase "(.*)"$/s, "scam", ["term"]],
  [/^excessive caps$/, "caps", []],
  [/^(\d+) emotes$/, "emotes", ["n"]],
  [/^message too long$/, "length", []],
  [/^manual action$/, "manual", []],
];

export function modReason(reason: string): string {
  for (const [re, key, names] of ruleReasons) {
    const m = reason.match(re);
    if (m) return i18n.global.t(`ruleReasons.${key}`, Object.fromEntries(names.map((n, i) => [n, m[i + 1]])));
  }
  return reason;
}

export function pdesc(p: Named): string {
  const k = `presets.${p.id}.description`;
  return p.builtin && i18n.global.te(k) ? i18n.global.t(k) : (p.description ?? "");
}