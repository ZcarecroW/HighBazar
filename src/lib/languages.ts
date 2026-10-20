import type { ComboOption } from "../components/ui/UiCombobox.vue";

const displayNames = new Map<string, Intl.DisplayNames | null>();
/** Name of a language code in the given UI language ("de" → "Deutsch"/"German"). */
export function languageName(code: string, inLocale: string): string {
  if (!displayNames.has(inLocale)) {
    try {
      displayNames.set(inLocale, new Intl.DisplayNames([inLocale], { type: "language" }));
    } catch {
      displayNames.set(inLocale, null);
    }
  }
  try {
    return displayNames.get(inLocale)?.of(code) ?? code;
  } catch {
    return code;
  }
}

/**
 * Options for a language combobox: shown in the UI language with the native
 * name next to it, and findable by code, English and native name
 * ("german", "deutsch" and "de" all find German).
 */
export function languageOptions(codes: readonly string[], locale: string, other?: string): ComboOption[] {
  return codes
    .map((code) => {
      if (code === "other") return { value: code, label: other ?? code, keywords: "other andere" };
      const label = languageName(code, locale);
      const native = languageName(code, code);
      return {
        value: code,
        label,
        sub: native.toLowerCase() !== label.toLowerCase() ? native : undefined,
        keywords: `${code} ${languageName(code, "en")}`,
      };
    })
    .sort((a, b) => (a.value === "other" ? 1 : b.value === "other" ? -1 : a.label.localeCompare(b.label, locale)));
}
