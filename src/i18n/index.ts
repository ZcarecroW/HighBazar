import { createI18n } from "vue-i18n";
import en from "./en";
import de from "./de";

export type Locale = "en" | "de";

export function detectLocale(setting: string): Locale {
  if (setting === "en" || setting === "de") return setting;
  return navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

export const i18n = createI18n({
  legacy: false,
  locale: detectLocale("auto"),
  fallbackLocale: "en",
  messages: { en, de },
});

export function setLocale(setting: string) {
  const l = detectLocale(setting);
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
  return p.builtin && i18n.global.te(k, "en") ? i18n.global.t(k) : p.name;
}

export function pdesc(p: Named): string {
  const k = `presets.${p.id}.description`;
  return p.builtin && i18n.global.te(k, "en") ? i18n.global.t(k) : (p.description ?? "");
}