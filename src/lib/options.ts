import { pdesc, pname } from "../i18n";
import type { ComboOption } from "../components/ui/UiCombobox.vue";
import type { AiProfile } from "./types";
import type { useApp } from "../stores/app";

type App = ReturnType<typeof useApp>;
type T = (key: string) => string;

/** Companion or moderation profiles for a searchable select. */
export function profileOptions(items: { id: string; name: string; description: string; builtin: boolean }[], t: T): ComboOption[] {
  return items.map((p) => ({
    value: p.id,
    label: pname(p),
    sub: pdesc(p),
    badge: p.builtin ? t("common.builtin") : undefined,
  }));
}

/** AI profiles with provider and model, so "haiku" or "ollama" finds them too. */
export function aiProfileOptions(app: App, items: AiProfile[], t: T): ComboOption[] {
  return items.map((p) => {
    const provider = app.provider(p.provider)?.name ?? p.provider;
    return {
      value: p.id,
      label: pname(p),
      sub: [provider, p.model].filter(Boolean).join(" · "),
      keywords: `${p.provider} ${pdesc(p)}`,
      badge: app.hasKey(p) ? undefined : t("ai.keyMissing"),
    };
  });
}
