<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, FolderOpen, Loader2, RefreshCw, Sparkles, Trash2, Settings, Palette, Bolt, Key, Info, Chat, Undo2, Activity, Download } from "../icons";
import { useApp } from "../stores/app";
import { api, errorText } from "../lib/api";
import { checkForUpdate } from "../lib/updater";
import UiToggle from "../components/ui/UiToggle.vue";
import UiField from "../components/ui/UiField.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import UiSegmented from "../components/ui/UiSegmented.vue";
import ThemePicker from "../components/ThemePicker.vue";
import UiSlider from "../components/ui/UiSlider.vue";
import ChatMotionPreview from "../components/ChatMotionPreview.vue";
import OverlaySettings from "../components/OverlaySettings.vue";
import { glide } from "../lib/smoothFollow";
import { clampZoom, CHAT_ZOOM, UI_ZOOM } from "../lib/zoom";
import type { ScrollEasing } from "../lib/types";
import { useJump } from "../lib/jump";

const app = useApp();
const { t } = useI18n();
const s = computed(() => app.config!.settings);
const checking = ref(false);
useJump("settings");

const themeOptions = computed(() => (["system", "dark", "light"] as const).map((v) => ({ value: v, label: t(`settings.themes.${v}`) })));
const langOptions = computed(() => (["auto", "en", "de"] as const).map((v) => ({ value: v, label: t(`settings.languages.${v}`) })));

async function checkNow() {
  checking.value = true;
  try {
    const u = await checkForUpdate();
    if (!u) app.toast(t("update.upToDate"), "success");
  } catch (e) {
    app.toast(t("update.error", { e: errorText(e) }), "error", 7000);
  } finally {
    checking.value = false;
  }
}

async function clearResearch() {
  await api.clearResearchCache();
  app.toast(t("settings.cleared"), "success");
}

const repo = "https://github.com/ZcarecroW/HighBazar";

// ------------------------------------------------------------- chat display
const uiZoomPct = computed({
  get: () => Math.round((s.value.uiZoom || 1) * 100),
  set: (v) => (s.value.uiZoom = clampZoom(v / 100, UI_ZOOM)),
});
const chatZoomPct = computed({
  get: () => Math.round((s.value.chatZoom || 1) * 100),
  set: (v) => (s.value.chatZoom = clampZoom(v / 100, CHAT_ZOOM)),
});
const easings: ScrollEasing[] = ["easeInOut", "easeOut", "linear", "spring"];
const durationWord = computed(() => {
  const d = s.value.chatScroll.durationMs;
  return t(`settings.durationWords.${d < 300 ? "fast" : d <= 650 ? "normal" : "calm"}`);
});
/** Path of the glide (distance over time) for the little curve icons. */
function curve(e: ScrollEasing) {
  const pts: string[] = [];
  for (let i = 0; i <= 24; i++) {
    const tt = i / 24;
    const y = 1 - glide(e, 100, 1, 0, tt * 100);
    pts.push(`${(2 + tt * 44).toFixed(1)} ${(30 - y * 28).toFixed(1)}`);
  }
  return "M" + pts.join(" L");
}
// Performance mode: save first, so the backend knows the window may close.
async function toTray() {
  await app.saveNow();
  await api.appToTray();
}
function resetChatDisplay() {
  s.value.uiZoom = 1;
  s.value.chatZoom = 1;
  s.value.chatScroll = { smooth: true, durationMs: 450, easing: "easeInOut" };
}
</script>

<template>
  <div class="mx-auto max-w-5xl space-y-5 p-4 xl:p-6">
    <h1 class="flex items-center gap-2 text-xl font-bold"><Settings :size="22" class="text-brand-500" /> {{ t("settings.title") }}</h1>

    <section id="jump-appearance" class="card-pad scroll-mt-4 space-y-5">
      <h2 class="section-title mb-0 flex items-center gap-2"><Palette :size="17" class="text-brand-500" /> {{ t("settings.appearance") }}</h2>
      <UiSegmented v-model="s.theme" :label="t('settings.theme')" :options="themeOptions" />
      <div>
        <span class="label">{{ t("settings.colorTheme") }}</span>
        <ThemePicker v-model="s.colorTheme" />
      </div>
      <UiSegmented v-model="s.language" :label="t('settings.language')" :options="langOptions" />
      <UiToggle v-model="s.showEmotes" :label="t('settings.showEmotes')" :hint="t('settings.showEmotesHint')" />
    </section>

    <section id="jump-chatDisplay" class="card-pad scroll-mt-4 space-y-5">
      <div class="flex items-center gap-2">
        <h2 class="section-title mb-0 flex flex-1 items-center gap-2"><Chat :size="17" class="text-brand-500" /> {{ t("settings.chatDisplay") }}</h2>
        <button class="btn-ghost text-xs" @click="resetChatDisplay"><Undo2 :size="14" /> {{ t("settings.resetDefaults") }}</button>
      </div>
      <div class="grid gap-5 md:grid-cols-2">
        <UiSlider v-model="uiZoomPct" :label="t('settings.uiZoom')" :hint="t('settings.uiZoomHint')" :min="70" :max="160" :step="5" suffix=" %" />
        <UiSlider v-model="chatZoomPct" :label="t('settings.chatZoom')" :hint="t('settings.chatZoomHint')" :min="60" :max="250" :step="10" suffix=" %" />
      </div>
      <UiToggle v-model="s.chatScroll.smooth" :label="t('settings.smoothScroll')" :hint="t('settings.smoothScrollHint')" />
      <div class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_300px]">
        <div class="space-y-5" :class="!s.chatScroll.smooth && 'pointer-events-none opacity-50'">
          <UiSlider
            v-model="s.chatScroll.durationMs"
            :label="t('settings.scrollDuration')"
            :hint="t('settings.scrollDurationHint')"
            :min="150"
            :max="1200"
            :step="50"
            :display="`${s.chatScroll.durationMs} ms · ${durationWord}`"
          />
          <div>
            <span class="label">{{ t("settings.scrollEasing") }}</span>
            <div class="grid gap-2 sm:grid-cols-2">
              <button
                v-for="e in easings"
                :key="e"
                type="button"
                class="flex gap-3 rounded-xl border p-3 text-left transition"
                :class="s.chatScroll.easing === e ? 'border-brand-500 bg-brand-500/10 ring-2 ring-brand-500/25' : 'border-line hover:border-brand-300'"
                @click="s.chatScroll.easing = e"
              >
                <svg viewBox="0 0 48 32" class="h-8 w-12 shrink-0 text-brand-500" aria-hidden="true">
                  <path d="M2 30 H46 M2 30 V2" fill="none" stroke="var(--line)" stroke-width="1" />
                  <path :d="curve(e)" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                </svg>
                <span class="min-w-0">
                  <span class="flex flex-wrap items-center gap-1.5 text-sm font-semibold">
                    {{ t(`settings.easings.${e}.name`) }}
                    <span v-if="e === 'easeInOut'" class="chip bg-brand-500 text-white">{{ t("common.recommended") }}</span>
                  </span>
                  <span class="mt-0.5 block text-xs text-muted">{{ t(`settings.easings.${e}.desc`) }}</span>
                </span>
              </button>
            </div>
          </div>
        </div>
        <div>
          <ChatMotionPreview :scroll="s.chatScroll" :chat-zoom="s.chatZoom" :ui-zoom="s.uiZoom" />
          <p class="hint">{{ t("settings.preview.hint") }}</p>
        </div>
      </div>
    </section>

    <section id="jump-performance" class="card-pad scroll-mt-4 space-y-5">
      <h2 class="section-title mb-0 flex items-center gap-2"><Activity :size="17" class="text-brand-500" /> {{ t("settings.performance") }}</h2>
      <p class="max-w-3xl text-sm text-muted">{{ t("settings.performanceIntro") }}</p>
      <UiToggle v-model="s.lowPriority" :label="t('settings.lowPriority')" :hint="t('settings.lowPriorityHint')" />
      <UiToggle v-model="s.performanceMode" :label="t('settings.performanceMode')" :hint="t('settings.performanceModeHint')" />
      <div class="space-y-4 pl-12" :class="!s.performanceMode && 'pointer-events-none opacity-50'">
        <UiToggle v-model="s.startInTray" :label="t('settings.startInTray')" :hint="t('settings.startInTrayHint')" />
        <button class="btn" @click="toTray"><Download :size="15" /> {{ t("settings.toTrayNow") }}</button>
      </div>
    </section>

    <OverlaySettings id="jump-overlay" class="scroll-mt-4" />

    <section id="jump-startup" class="card-pad scroll-mt-4 space-y-4">
      <h2 class="section-title mb-0 flex items-center gap-2"><Bolt :size="17" class="text-brand-500" /> {{ t("settings.startup") }}</h2>
      <div class="flex flex-wrap items-center gap-4">
        <UiToggle v-model="s.checkUpdatesOnStart" :label="t('settings.checkUpdates')" class="flex-1" />
        <button class="btn" :disabled="checking" @click="checkNow">
          <Loader2 v-if="checking" :size="15" class="animate-spin" />
          <RefreshCw v-else :size="15" />
          {{ t("settings.checkNow") }}
        </button>
      </div>
      <UiToggle v-model="s.startBotOnLaunch" :label="t('settings.startBotOnLaunch')" />
      <div class="flex flex-wrap items-center gap-3 pt-1">
        <button class="btn" @click="app.wizardOpen = true"><Sparkles :size="15" /> {{ t("settings.openWizard") }}</button>
      </div>
    </section>

    <section id="jump-data" class="card-pad scroll-mt-4 space-y-4">
      <h2 class="section-title mb-0 flex items-center gap-2"><FolderOpen :size="17" class="text-brand-500" /> {{ t("settings.data") }}</h2>
      <div class="flex flex-wrap gap-2">
        <button class="btn" @click="api.openDataDir()"><FolderOpen :size="15" /> {{ t("settings.openData") }}</button>
        <button class="btn" @click="clearResearch"><Trash2 :size="15" /> {{ t("settings.clearResearch") }}</button>
      </div>
      <p class="selectable text-xs text-muted">{{ app.dataDir }}</p>
      <p class="text-sm">
        <span class="label">{{ t("settings.secrets") }}</span>
        {{ app.secretsBackend === "os" ? t("settings.secretsOs") : t("settings.secretsFile") }}
      </p>
    </section>

    <section id="jump-advanced" class="card-pad scroll-mt-4 space-y-4">
      <h2 class="section-title mb-0 flex items-center gap-2"><Key :size="17" class="text-brand-500" /> {{ t("settings.advanced") }}</h2>
      <UiField :label="t('settings.clientId')" :hint="t('settings.clientIdHint')">
        <input v-model="s.twitchClientId" class="input font-mono" spellcheck="false" autocomplete="off" />
      </UiField>
      <UiNumber v-model="s.feedLimit" :label="t('settings.feedLimit')" :min="50" :max="5000" :step="50" />
    </section>

    <section id="jump-about" class="card-pad scroll-mt-4 space-y-3">
      <h2 class="section-title mb-0 flex items-center gap-2"><Info :size="17" class="text-brand-500" /> {{ t("settings.about") }}</h2>
      <div class="flex items-center gap-3">
        <img src="/icon.png" class="h-12 w-12 rounded-xl" alt="" />
        <div>
          <div class="font-bold">HighBazar</div>
          <div class="text-sm text-muted">{{ t("settings.version", { v: app.version }) }} · MIT</div>
        </div>
      </div>
      <div class="flex flex-wrap gap-2">
        <button class="btn" @click="openUrl(repo)"><ExternalLink :size="15" /> {{ t("settings.github") }}</button>
        <button class="btn" @click="openUrl(repo + '/issues')"><ExternalLink :size="15" /> {{ t("settings.issues") }}</button>
      </div>
    </section>
  </div>
</template>
