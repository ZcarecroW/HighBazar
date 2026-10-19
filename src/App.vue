<script setup lang="ts">
import { computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  Bot,
  Cpu,
  LayoutDashboard,
  Radio,
  Settings,
  ShieldCheck,
  Sparkles,
  Loader2,
  CircleAlert,
  CircleCheck,
  Info,
  MonitorPlay,
} from "./icons";
import { useApp, type View } from "./stores/app";
import { setLocale } from "./i18n";
import { checkForUpdate } from "./lib/updater";
import DashboardView from "./views/DashboardView.vue";
import StreamView from "./views/StreamView.vue";
import ChannelsView from "./views/ChannelsView.vue";
import CompanionView from "./views/CompanionView.vue";
import ModerationView from "./views/ModerationView.vue";
import AiView from "./views/AiView.vue";
import SettingsView from "./views/SettingsView.vue";
import WizardView from "./views/WizardView.vue";
import UpdateDialog from "./components/UpdateDialog.vue";

const app = useApp();
const { t } = useI18n();

const nav: { id: View; icon: any }[] = [
  { id: "dashboard", icon: LayoutDashboard },
  { id: "stream", icon: MonitorPlay },
  { id: "channels", icon: Radio },
  { id: "companion", icon: Bot },
  { id: "moderation", icon: ShieldCheck },
  { id: "ai", icon: Cpu },
  { id: "settings", icon: Settings },
];

const views = {
  dashboard: DashboardView,
  stream: StreamView,
  channels: ChannelsView,
  companion: CompanionView,
  moderation: ModerationView,
  ai: AiView,
  settings: SettingsView,
};
const current = computed(() => views[app.view]);

// Theme handling
const media = window.matchMedia("(prefers-color-scheme: dark)");
function applyTheme() {
  const pref = app.config?.settings.theme ?? "system";
  const dark = pref === "dark" || (pref === "system" && media.matches);
  document.documentElement.classList.toggle("dark", dark);
  document.documentElement.dataset.theme = app.config?.settings.colorTheme || "bazaar";
}
media.addEventListener("change", applyTheme);
applyTheme();

watch(() => [app.config?.settings.theme, app.config?.settings.colorTheme], applyTheme);
watch(() => app.config?.settings.language, (l) => setLocale(l ?? "auto"));

onMounted(async () => {
  await app.listenAll();
  await app.load();
  applyTheme();
  setLocale(app.config?.settings.language ?? "auto");
  if (app.config?.settings.checkUpdatesOnStart) {
    checkForUpdate().catch((e) => console.warn("update check failed", e));
  }
});

const statusDot = computed(() => {
  if (!app.running) return "bg-zinc-400";
  if (app.running.sandbox) return "bg-amber-400";
  return app.status?.connected ? "bg-emerald-500" : "bg-amber-400 animate-pulse";
});
</script>

<template>
  <div v-if="!app.ready" class="flex h-full items-center justify-center">
    <div v-if="app.loadError" class="card-pad max-w-md text-sm text-red-500">{{ app.loadError }}</div>
    <Loader2 v-else :size="28" class="animate-spin text-brand-500" />
  </div>

  <div v-else class="flex h-full">
    <!-- Sidebar -->
    <aside class="flex w-16 shrink-0 flex-col border-r border-line bg-panel xl:w-56">
      <div class="flex h-14 items-center gap-2.5 px-4">
        <img src="/icon.png" alt="" class="h-8 w-8 rounded-lg" />
        <div class="hidden min-w-0 xl:block">
          <div class="text-[15px] leading-tight font-bold">HighBazar</div>
          <div class="text-[11px] text-muted">v{{ app.version }}</div>
        </div>
      </div>
      <nav class="flex flex-1 flex-col gap-1 px-2 py-2">
        <button
          v-for="n in nav"
          :key="n.id"
          class="flex items-center justify-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition xl:justify-start"
          :class="app.view === n.id ? 'bg-brand-500 text-white shadow-sm' : 'text-muted hover:bg-panel-2 hover:text-fg'"
          :title="t(`nav.${n.id}`)"
          @click="app.view = n.id"
        >
          <component :is="n.icon" :size="18" />
          <span class="hidden xl:inline">{{ t(`nav.${n.id}`) }}</span>
          <span v-if="n.id === 'dashboard'" class="ml-auto hidden h-2 w-2 rounded-full xl:block" :class="statusDot" />
        </button>
      </nav>
      <div class="px-2 pb-3">
        <button
          class="flex w-full items-center justify-center gap-3 rounded-lg px-3 py-2.5 text-sm text-muted transition hover:bg-panel-2 hover:text-fg xl:justify-start"
          :title="t('nav.wizard')"
          @click="app.wizardOpen = true"
        >
          <Sparkles :size="18" />
          <span class="hidden xl:inline">{{ t("nav.wizard") }}</span>
        </button>
      </div>
    </aside>

    <!-- Main -->
    <main class="min-w-0 flex-1 overflow-y-auto">
      <component :is="current" />
    </main>

    <WizardView v-if="app.wizardOpen" />
    <UpdateDialog />

    <!-- Toasts -->
    <div class="pointer-events-none fixed right-4 bottom-4 z-[60] flex w-96 max-w-[calc(100vw-2rem)] flex-col gap-2">
      <div
        v-for="toast in app.toasts"
        :key="toast.id"
        class="card pointer-events-auto flex items-start gap-2 px-4 py-3 text-sm shadow-lg"
        :class="{ 'border-red-500/50': toast.kind === 'error', 'border-emerald-500/50': toast.kind === 'success' }"
      >
        <CircleAlert v-if="toast.kind === 'error'" :size="16" class="mt-0.5 shrink-0 text-red-500" />
        <CircleCheck v-else-if="toast.kind === 'success'" :size="16" class="mt-0.5 shrink-0 text-emerald-500" />
        <Info v-else :size="16" class="mt-0.5 shrink-0 text-brand-500" />
        <span class="selectable min-w-0 break-words">{{ toast.text }}</span>
      </div>
    </div>
  </div>
</template>
