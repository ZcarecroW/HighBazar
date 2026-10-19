<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, FolderOpen, Loader2, RefreshCw, Sparkles, Trash2, Settings, Palette, Bolt, Key, Info } from "../icons";
import { useApp } from "../stores/app";
import { api, errorText } from "../lib/api";
import { checkForUpdate } from "../lib/updater";
import UiToggle from "../components/ui/UiToggle.vue";
import UiField from "../components/ui/UiField.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import UiSegmented from "../components/ui/UiSegmented.vue";
import ThemePicker from "../components/ThemePicker.vue";

const app = useApp();
const { t } = useI18n();
const s = computed(() => app.config!.settings);
const checking = ref(false);

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
</script>

<template>
  <div class="mx-auto max-w-4xl space-y-5 p-4 xl:p-6">
    <h1 class="flex items-center gap-2 text-xl font-bold"><Settings :size="22" class="text-brand-500" /> {{ t("settings.title") }}</h1>

    <section class="card-pad space-y-5">
      <h2 class="section-title mb-0 flex items-center gap-2"><Palette :size="17" class="text-brand-500" /> {{ t("settings.appearance") }}</h2>
      <UiSegmented v-model="s.theme" :label="t('settings.theme')" :options="themeOptions" />
      <div>
        <span class="label">{{ t("settings.colorTheme") }}</span>
        <ThemePicker v-model="s.colorTheme" />
      </div>
      <UiSegmented v-model="s.language" :label="t('settings.language')" :options="langOptions" />
      <UiToggle v-model="s.showEmotes" :label="t('settings.showEmotes')" :hint="t('settings.showEmotesHint')" />
    </section>

    <section class="card-pad space-y-4">
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

    <section class="card-pad space-y-4">
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

    <section class="card-pad space-y-4">
      <h2 class="section-title mb-0 flex items-center gap-2"><Key :size="17" class="text-brand-500" /> {{ t("settings.advanced") }}</h2>
      <UiField :label="t('settings.clientId')" :hint="t('settings.clientIdHint')">
        <input v-model="s.twitchClientId" class="input font-mono" spellcheck="false" autocomplete="off" />
      </UiField>
      <UiNumber v-model="s.feedLimit" :label="t('settings.feedLimit')" :min="50" :max="5000" :step="50" />
    </section>

    <section class="card-pad space-y-3">
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
