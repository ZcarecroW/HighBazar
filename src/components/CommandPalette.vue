<script setup lang="ts">
import { computed, nextTick, ref, watch, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { ArrowRight, Bot, Cpu, Heart, Layers, LayoutDashboard, MonitorPlay, Moon, Play, Radio, Search, Settings, ShieldCheck, Sparkles, Square, Sun, Swords, Users } from "../icons";
import { useApp, type View } from "../stores/app";
import { api } from "../lib/api";
import { pdesc, pname } from "../i18n";
import { fuzzySearch } from "../lib/fuzzy";
import FuzzyText from "./ui/FuzzyText.vue";

// Quick search (Ctrl+K): jump to any page, section, setting, profile or channel,
// or run a common action – found by fuzzy search.
const open = defineModel<boolean>({ required: true });
const app = useApp();
const { t } = useI18n();

type Group = "actions" | "pages" | "sections" | "channels" | "profiles";
interface Cmd {
  id: string;
  group: Group;
  label: string;
  sub?: string;
  keywords?: string;
  icon: Component;
  run: () => void;
}

const VIEW_ICONS: Record<View, Component> = {
  dashboard: LayoutDashboard,
  stream: MonitorPlay,
  channels: Radio,
  companion: Bot,
  moderation: ShieldCheck,
  ai: Cpu,
  settings: Settings,
};
// Words on each settings section, so "tray" or "Prioritär" find it.
const SETTINGS: Record<string, string[]> = {
  appearance: ["settings.theme", "settings.colorTheme", "settings.language", "settings.showEmotes"],
  chatDisplay: ["settings.uiZoom", "settings.chatZoom", "settings.smoothScroll", "settings.scrollDuration", "settings.scrollEasing"],
  performance: ["settings.lowPriority", "settings.performanceMode", "settings.startInTray", "settings.toTrayNow"],
  overlay: ["overlay.enabled"],
  startup: ["settings.checkUpdates", "settings.startBotOnLaunch", "settings.openWizard"],
  data: ["settings.openData", "settings.clearResearch", "settings.secrets"],
  advanced: ["settings.clientId", "settings.feedLimit"],
  about: ["settings.version", "settings.github"],
};
const COMPANION = ["personality", "timing", "content", "language", "emotes", "interactions", "commands", "research"];
const MODERATION = ["general", "categories", "escalation", "rules", "exemptions", "messages"];

const commands = computed<Cmd[]>(() => {
  const cfg = app.config!;
  const list: Cmd[] = [];
  const go = (view: View, opts: Parameters<typeof app.goto>[1] = {}) => () => app.goto(view, opts);

  // Actions
  if (app.activeChannel) {
    if (app.isRunning) list.push({ id: "a:stop", group: "actions", label: t("palette.stopBot"), icon: Square, run: () => void app.stopBot() });
    else {
      list.push({ id: "a:start", group: "actions", label: t("palette.startBot"), sub: app.activeChannel.name, icon: Play, run: () => void app.startBot(false) });
      list.push({ id: "a:test", group: "actions", label: t("palette.startTest"), sub: app.activeChannel.name, keywords: "sandbox simulation", icon: Sparkles, run: () => void app.startBot(true) });
    }
  }
  list.push({ id: "a:raid", group: "actions", label: t("stream.actions.raid"), keywords: "raid raiden", icon: Swords, run: go("stream", { section: "raid", focus: true }) });
  list.push({ id: "a:shoutout", group: "actions", label: t("stream.actions.shoutout"), keywords: "shoutout so", icon: Heart, run: go("stream", { section: "shoutout", focus: true }) });
  list.push({ id: "a:title", group: "actions", label: t("palette.editTitle"), keywords: `${t("stream.streamTitle")} ${t("stream.category")} ${t("stream.tags")}`, icon: MonitorPlay, run: go("stream", { section: "info", focus: true }) });
  const dark = document.documentElement.classList.contains("dark");
  list.push({
    id: "a:theme",
    group: "actions",
    label: dark ? t("palette.lightMode") : t("palette.darkMode"),
    keywords: "theme design hell dunkel light dark",
    icon: dark ? Sun : Moon,
    run: () => (cfg.settings.theme = dark ? "light" : "dark"),
  });
  list.push({ id: "a:tray", group: "actions", label: t("settings.toTrayNow"), keywords: "tray minimize performance", icon: ArrowRight, run: () => void api.appToTray() });

  // Pages
  for (const v of Object.keys(VIEW_ICONS) as View[]) list.push({ id: `p:${v}`, group: "pages", label: t(`nav.${v}`), icon: VIEW_ICONS[v], run: go(v) });
  list.push({ id: "p:wizard", group: "pages", label: t("nav.wizard"), icon: Sparkles, run: () => (app.wizardOpen = true) });

  // Sections
  for (const s of COMPANION) list.push({ id: `s:c:${s}`, group: "sections", label: t(`companion.sections.${s}`), sub: t("nav.companion"), icon: Bot, run: go("companion", { section: s }) });
  for (const s of MODERATION) list.push({ id: `s:m:${s}`, group: "sections", label: t(`moderation.sections.${s}`), sub: t("nav.moderation"), icon: ShieldCheck, run: go("moderation", { section: s }) });
  list.push({ id: "s:m:log", group: "sections", label: t("moderation.tabs.log"), sub: t("nav.moderation"), keywords: "log verlauf history bans", icon: ShieldCheck, run: go("moderation", { section: "log" }) });
  for (const s of ["profiles", "keys", "webSearch", "usage"]) list.push({ id: `s:a:${s}`, group: "sections", label: t(`ai.tabs.${s}`), sub: t("nav.ai"), icon: Cpu, run: go("ai", { section: s }) });
  for (const [s, keys] of Object.entries(SETTINGS)) {
    list.push({
      id: `s:s:${s}`,
      group: "sections",
      label: s === "overlay" ? t("overlay.title") : t(`settings.${s}`),
      sub: t("nav.settings"),
      keywords: keys.map((k) => t(k)).join(" "),
      icon: Settings,
      run: go("settings", { section: s }),
    });
  }
  list.push({ id: "s:ch:translation", group: "sections", label: t("translation.title"), sub: t("nav.channels"), icon: Radio, run: go("channels", { section: "translation" }) });

  // Channels and profiles
  for (const c of cfg.channels) list.push({ id: `c:${c.id}`, group: "channels", label: c.name, sub: t("nav.channels"), icon: Radio, run: go("channels", { profile: c.id }) });
  for (const p of cfg.aiProfiles) {
    list.push({ id: `pr:a:${p.id}`, group: "profiles", label: pname(p), sub: `${t("nav.ai")} · ${app.provider(p.provider)?.name ?? p.provider}`, keywords: `${p.model} ${p.provider}`, icon: Cpu, run: go("ai", { profile: p.id }) });
  }
  for (const p of cfg.companionProfiles) list.push({ id: `pr:c:${p.id}`, group: "profiles", label: pname(p), sub: t("nav.companion"), keywords: pdesc(p), icon: Users, run: go("companion", { profile: p.id }) });
  for (const p of cfg.moderationProfiles) list.push({ id: `pr:m:${p.id}`, group: "profiles", label: pname(p), sub: t("nav.moderation"), keywords: pdesc(p), icon: Layers, run: go("moderation", { profile: p.id }) });
  return list;
});

const query = ref("");
const active = ref(0);
const input = ref<HTMLInputElement>();
const listEl = ref<HTMLElement>();
const GROUPS: Group[] = ["actions", "pages", "sections", "channels", "profiles"];

const rows = computed(() => {
  const res = fuzzySearch(commands.value, query.value, (c) => [c.label, c.sub, c.keywords], { weights: [1, 0.6, 0.5] });
  // Without a query: grouped overview; with one: best matches first.
  if (!query.value.trim()) return res.filter((r) => r.item.group !== "profiles").sort((a, b) => GROUPS.indexOf(a.item.group) - GROUPS.indexOf(b.item.group));
  return res.slice(0, 40);
});

watch(open, async (v) => {
  if (!v) return;
  query.value = "";
  active.value = 0;
  await nextTick();
  input.value?.focus();
});
watch(query, () => (active.value = 0));

function run(c: Cmd) {
  open.value = false;
  c.run();
}
function scrollActive() {
  void nextTick(() => listEl.value?.querySelector<HTMLElement>(`[data-i="${active.value}"]`)?.scrollIntoView({ block: "nearest" }));
}
function onKey(e: KeyboardEvent) {
  const n = rows.value.length;
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    if (n) active.value = (active.value + (e.key === "ArrowDown" ? 1 : n - 1)) % n;
    scrollActive();
  } else if (e.key === "Enter") {
    e.preventDefault();
    const r = rows.value[active.value];
    if (r) run(r.item);
  } else if (e.key === "Escape") {
    e.preventDefault();
    open.value = false;
  }
}
</script>

<template>
  <div v-if="open" class="fixed inset-0 z-50 flex items-start justify-center bg-black/40 p-4 pt-[12vh]" @mousedown.self="open = false">
    <div class="card w-full max-w-xl overflow-hidden shadow-2xl" role="dialog" aria-modal="true" :aria-label="t('palette.title')">
      <div class="flex items-center gap-2 border-b border-line px-4 py-3">
        <Search :size="17" class="shrink-0 text-muted" />
        <input
          ref="input"
          v-model="query"
          class="min-w-0 flex-1 bg-transparent text-base outline-none"
          :placeholder="t('palette.placeholder')"
          spellcheck="false"
          role="combobox"
          aria-autocomplete="list"
          aria-controls="palette-list"
          :aria-activedescendant="rows.length ? `palette-${active}` : undefined"
          @keydown="onKey"
        />
        <kbd class="rounded border border-line px-1.5 py-0.5 text-[11px] text-muted">Esc</kbd>
      </div>
      <ul id="palette-list" ref="listEl" role="listbox" class="max-h-[60vh] overflow-y-auto py-1">
        <template v-for="(r, i) in rows" :key="r.item.id">
          <li
            v-if="!query.trim() && (i === 0 || rows[i - 1]!.item.group !== r.item.group)"
            role="presentation"
            class="px-4 pt-2.5 pb-1 text-[11px] font-semibold tracking-wide text-muted uppercase"
          >
            {{ t(`palette.groups.${r.item.group}`) }}
          </li>
          <li
            :id="`palette-${i}`"
            :data-i="i"
            role="option"
            :aria-selected="i === active"
            class="flex cursor-pointer items-center gap-3 px-4 py-2"
            :class="i === active ? 'bg-brand-500/12' : ''"
            @mouseenter="active = i"
            @click="run(r.item)"
          >
            <component :is="r.item.icon" :size="16" class="shrink-0 text-muted" />
            <span class="min-w-0 flex-1 truncate text-sm"><FuzzyText :text="r.item.label" :ranges="r.ranges[0]" /></span>
            <span v-if="r.item.sub" class="max-w-[45%] shrink-0 truncate text-xs text-muted"><FuzzyText :text="r.item.sub" :ranges="r.ranges[1]" /></span>
          </li>
        </template>
        <li v-if="!rows.length" class="px-4 py-6 text-center text-sm text-muted">{{ t("common.noMatches") }}</li>
      </ul>
      <div class="flex gap-4 border-t border-line px-4 py-2 text-[11px] text-muted">
        <span><kbd class="font-sans">↑↓</kbd> {{ t("palette.navigate") }}</span>
        <span><kbd class="font-sans">↵</kbd> {{ t("palette.open") }}</span>
        <span class="ml-auto"><kbd class="font-sans">Ctrl K</kbd></span>
      </div>
    </div>
  </div>
</template>
