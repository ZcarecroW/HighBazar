<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ArrowLeft, ArrowRight, Bot, Check, CircleCheck, Cpu, ExternalLink, FlaskConical, Loader2, MonitorPlay, Play, Send, ShieldCheck, Smile, Sparkles, TriangleAlert, Trophy, Languages } from "../icons";
import { useApp } from "../stores/app";
import { api, errorText } from "../lib/api";
import { pdesc, pname } from "../i18n";
import TwitchLogin from "../components/TwitchLogin.vue";
import ApiKeyInput from "../components/ApiKeyInput.vue";
import UiToggle from "../components/ui/UiToggle.vue";
import UiSegmented from "../components/ui/UiSegmented.vue";
import ThemePicker from "../components/ThemePicker.vue";
import LanguagePicker from "../components/LanguagePicker.vue";
import UiCombobox from "../components/ui/UiCombobox.vue";
import { aiProfileOptions } from "../lib/options";

const app = useApp();
const { t } = useI18n();
const cfg = computed(() => app.config!);
const ch = computed(() => app.activeChannel!);

const steps = ["welcome", "twitch", "bot", "ai", "companion", "moderation", "done"] as const;
const idx = ref(0);
const step = computed(() => steps[idx.value]!);
const stepIcons = { welcome: Sparkles, twitch: MonitorPlay, bot: Bot, ai: Cpu, companion: Smile, moderation: ShieldCheck, done: Trophy };

function next() {
  if (idx.value < steps.length - 1) idx.value++;
}
function back() {
  if (idx.value > 0) idx.value--;
}
async function finish() {
  cfg.value.settings.wizardCompleted = true;
  await app.saveNow();
  app.wizardOpen = false;
}
async function finishAndStart(sandbox: boolean) {
  await finish();
  app.view = "dashboard";
  await app.startBot(sandbox);
}

const themeOptions = computed(() => (["system", "dark", "light"] as const).map((v) => ({ value: v, label: t(`settings.themes.${v}`) })));
const langOptions = computed(() => (["auto", "en", "de"] as const).map((v) => ({ value: v, label: t(`settings.languages.${v}`) })));

const streamer = computed(() => app.account(ch.value.streamerAccountId));
const bot = computed(() => app.account(ch.value.botAccountId));

// ------------------------------------------------------------- AI
const aiChoices = computed(() => cfg.value.aiProfiles);
const selectedAi = computed(() => app.aiProfile(ch.value.aiProfileId));
const aiInfo = computed(() => (selectedAi.value ? app.provider(selectedAi.value.provider) : undefined));
const testing = ref(false);
const testText = ref("");
const testError = ref("");
async function testAi() {
  if (!selectedAi.value) return;
  testing.value = true;
  testText.value = "";
  testError.value = "";
  try {
    testText.value = (await api.aiTest(JSON.parse(JSON.stringify(selectedAi.value)))).text;
  } catch (e) {
    testError.value = errorText(e);
  } finally {
    testing.value = false;
  }
}
function pickAi(id: string) {
  ch.value.aiProfileId = id;
  testText.value = "";
  testError.value = "";
}
const recommendedAi = "builtin-ai-claude-opus";
const recommendedModAi = "builtin-ai-claude-haiku";
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/55 p-4 backdrop-blur-sm">
      <div class="card flex max-h-full w-full max-w-3xl flex-col overflow-hidden shadow-2xl">
        <!-- Header -->
        <header class="flex items-center gap-3 border-b border-line px-6 py-4">
          <img src="/icon.png" class="h-9 w-9 rounded-lg" alt="" />
          <div class="min-w-0 flex-1">
            <h2 class="text-base font-bold">{{ t("wizard.title") }}</h2>
            <p class="text-xs text-muted">{{ t("wizard.step", { n: idx + 1, total: steps.length }) }} · {{ t(`wizard.steps.${step}`) }}</p>
          </div>
          <button class="btn-ghost text-xs" @click="finish">{{ t("wizard.skipAll") }}</button>
        </header>

        <!-- Progress -->
        <div class="flex items-center gap-1.5 px-6 pt-4">
          <template v-for="(s, i) in steps" :key="s">
            <button
              type="button"
              class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full transition"
              :class="i < idx ? 'bg-brand-500/15 text-brand-600 dark:text-brand-300' : i === idx ? 'bg-brand-500 text-white shadow-sm' : 'bg-panel-2 text-muted'"
              :title="t(`wizard.steps.${s}`)"
              @click="idx = i"
            >
              <component :is="stepIcons[s]" :size="14" />
            </button>
            <div v-if="i < steps.length - 1" class="h-0.5 flex-1 rounded-full" :class="i < idx ? 'bg-brand-500' : 'bg-panel-2'" />
          </template>
        </div>

        <!-- Body -->
        <div class="min-h-[340px] flex-1 overflow-y-auto px-6 py-5">
          <!-- Welcome -->
          <div v-if="step === 'welcome'" class="space-y-6">
            <p class="text-[15px] leading-relaxed">{{ t("wizard.welcomeText") }}</p>
            <UiSegmented v-model="cfg.settings.language" :label="t('settings.language')" :options="langOptions" />
            <UiSegmented v-model="cfg.settings.theme" :label="t('settings.theme')" :options="themeOptions" />
            <div>
              <span class="label">{{ t("settings.colorTheme") }}</span>
              <ThemePicker v-model="cfg.settings.colorTheme" />
            </div>
          </div>

          <!-- Twitch -->
          <div v-else-if="step === 'twitch'" class="space-y-5">
            <p class="text-sm leading-relaxed text-muted">{{ t("wizard.twitchText") }}</p>
            <div v-if="!app.clientIdConfigured" class="flex items-center gap-2 rounded-xl border border-amber-400/40 bg-amber-400/10 px-4 py-3 text-sm">
              <TriangleAlert :size="16" class="text-amber-500" /> {{ t("channels.noClientId") }}
            </div>
            <div class="flex items-center gap-3 rounded-xl bg-panel-2 p-4">
              <template v-if="streamer">
                <img v-if="streamer.profileImageUrl" :src="streamer.profileImageUrl" class="h-11 w-11 rounded-full" alt="" />
                <div class="flex-1">
                  <div class="font-semibold">{{ streamer.displayName }}</div>
                  <div class="flex items-center gap-1 text-xs text-emerald-600"><CircleCheck :size="13" /> {{ t("dashboard.connected") }}</div>
                </div>
                <TwitchLogin kind="streamer" :label="t('common.reconnect')" @connected="(a) => (ch.streamerAccountId = a.id)" />
              </template>
              <template v-else>
                <span class="flex-1 text-sm text-muted">{{ t("channels.notConnected") }}</span>
                <TwitchLogin kind="streamer" primary @connected="(a) => (ch.streamerAccountId = a.id)" />
              </template>
            </div>
          </div>

          <!-- Bot account -->
          <div v-else-if="step === 'bot'" class="space-y-5">
            <p class="text-sm leading-relaxed text-muted">{{ t("wizard.botText") }}</p>
            <div class="flex items-center gap-3 rounded-xl bg-panel-2 p-4">
              <template v-if="bot">
                <img v-if="bot.profileImageUrl" :src="bot.profileImageUrl" class="h-11 w-11 rounded-full" alt="" />
                <div class="flex-1 font-semibold">{{ bot.displayName }}</div>
                <button class="btn" @click="ch.botAccountId = null">{{ t("channels.removeAccount") }}</button>
              </template>
              <template v-else>
                <span class="flex-1 text-sm text-muted">{{ t("channels.notConnected") }} ({{ t("common.optional") }})</span>
                <TwitchLogin kind="bot" @connected="(a) => (ch.botAccountId = a.id)" />
              </template>
            </div>
            <div class="space-y-2">
              <UiToggle v-model="ch.prefixEnabled" :label="t('wizard.usePrefix')" />
              <input v-model="ch.prefix" class="input max-w-xs" :disabled="!ch.prefixEnabled" />
            </div>
          </div>

          <!-- AI -->
          <div v-else-if="step === 'ai'" class="space-y-5">
            <p class="text-sm leading-relaxed text-muted">{{ t("wizard.aiText") }}</p>
            <div class="grid gap-2 sm:grid-cols-2">
              <button
                v-for="a in aiChoices"
                :key="a.id"
                class="rounded-xl border p-3 text-left transition"
                :class="ch.aiProfileId === a.id ? 'border-brand-500 bg-brand-500/10 ring-2 ring-brand-500/25' : 'border-line hover:border-brand-300'"
                @click="pickAi(a.id)"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm font-semibold">{{ a.name }}</span>
                  <span v-if="a.id === recommendedAi" class="chip bg-brand-500 text-white">{{ t("common.recommended") }}</span>
                  <CircleCheck v-if="app.hasKey(a)" :size="14" class="ml-auto text-emerald-500" />
                </div>
                <p class="mt-1 line-clamp-2 text-xs text-muted">{{ a.description }}</p>
              </button>
            </div>
            <div v-if="selectedAi" class="space-y-3 rounded-xl bg-panel-2 p-4">
              <div v-if="selectedAi.provider === 'claudeCli'" class="flex gap-2 text-xs text-amber-600 dark:text-amber-400">
                <TriangleAlert :size="15" class="shrink-0" /> {{ t("ai.cliWarning") }}
              </div>
              <div v-else-if="selectedAi.provider === 'codexCli'" class="text-xs text-muted">{{ t("ai.codexInfo") }}</div>
              <div v-if="aiInfo?.needsKey">
                <div class="mb-1 flex items-center">
                  <span class="label mb-0">{{ t("wizard.aiKey", { provider: aiInfo.name }) }}</span>
                  <button class="btn-ghost ml-auto text-xs" @click="openUrl(aiInfo.keyUrl)"><ExternalLink :size="13" /> {{ t("ai.getKey") }}</button>
                </div>
                <ApiKeyInput :key-ref="app.keyRefOf(selectedAi)" />
              </div>
              <button class="btn" :disabled="testing" @click="testAi">
                <Loader2 v-if="testing" :size="15" class="animate-spin" />
                <Send v-else :size="15" />
                {{ t("wizard.aiTest") }}
              </button>
              <div v-if="testText" class="rounded-lg border border-emerald-500/30 bg-emerald-500/8 p-3 text-sm">
                <span class="font-semibold text-emerald-600">{{ t("wizard.aiOk") }}</span> {{ testText }}
              </div>
              <p v-if="testError" class="selectable text-sm text-red-500">{{ testError }}</p>
            </div>
          </div>

          <!-- Companion -->
          <div v-else-if="step === 'companion'" class="space-y-5">
            <p class="text-sm leading-relaxed text-muted">{{ t("wizard.companionText") }}</p>
            <div class="grid gap-2 sm:grid-cols-2">
              <button
                v-for="c in cfg.companionProfiles"
                :key="c.id"
                class="rounded-xl border p-3 text-left transition"
                :class="ch.companionProfileId === c.id ? 'border-brand-500 bg-brand-500/10 ring-2 ring-brand-500/25' : 'border-line hover:border-brand-300'"
                @click="ch.companionProfileId = c.id"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm font-semibold">{{ pname(c) }}</span>
                  <span v-if="c.id === 'builtin-companion-balanced'" class="chip bg-brand-500 text-white">{{ t("common.recommended") }}</span>
                </div>
                <p class="mt-1 text-xs text-muted">{{ pdesc(c) }}</p>
              </button>
            </div>
            <div>
              <span class="label">{{ t("channels.aboutStream") }}</span>
              <textarea v-model="ch.aboutStream" rows="3" class="input" :placeholder="t('channels.aboutStreamHint')" />
            </div>
          </div>

          <!-- Moderation -->
          <div v-else-if="step === 'moderation'" class="space-y-5">
            <p class="text-sm leading-relaxed text-muted">{{ t("wizard.moderationText") }}</p>
            <UiToggle v-model="ch.moderationEnabled" :label="t('channels.moderationEnabled')" />
            <div class="grid gap-2 sm:grid-cols-2" :class="!ch.moderationEnabled && 'pointer-events-none opacity-50'">
              <button
                v-for="m in cfg.moderationProfiles"
                :key="m.id"
                class="rounded-xl border p-3 text-left transition"
                :class="ch.moderationProfileId === m.id ? 'border-brand-500 bg-brand-500/10 ring-2 ring-brand-500/25' : 'border-line hover:border-brand-300'"
                @click="ch.moderationProfileId = m.id"
              >
                <div class="flex items-center gap-2">
                  <span class="text-sm font-semibold">{{ pname(m) }}</span>
                  <span v-if="m.id === 'builtin-mod-balanced'" class="chip bg-brand-500 text-white">{{ t("common.recommended") }}</span>
                </div>
                <p class="mt-1 text-xs text-muted">{{ pdesc(m) }}</p>
              </button>
            </div>
            <div v-if="ch.moderationEnabled">
              <span class="label">{{ t("channels.moderationAiProfile") }}</span>
              <UiCombobox
                v-model="ch.moderationAiProfileId"
                class="max-w-md"
                :options="[
                  { value: '', label: t('channels.sameAsMain') },
                  ...aiProfileOptions(app, cfg.aiProfiles, t).map((o) => (o.value === recommendedModAi ? { ...o, badge: t('common.recommended') } : o)),
                ]"
                :label="t('channels.moderationAiProfile')"
                stacked
              />
              <p class="hint">{{ t("wizard.modAiHint") }}</p>
            </div>
            <div v-if="ch.translation" class="rounded-xl bg-panel-2 p-4">
              <span class="label flex items-center gap-1.5"><Languages :size="14" class="text-brand-500" /> {{ t("translation.languages") }}</span>
              <p class="hint mb-2.5">{{ t("translation.wizardHint") }}</p>
              <LanguagePicker v-model="ch.translation.languages" />
            </div>
          </div>

          <!-- Done -->
          <div v-else class="flex flex-col items-center gap-5 py-6 text-center">
            <div class="flex h-16 w-16 items-center justify-center rounded-full bg-emerald-500/15"><Check :size="32" class="text-emerald-500" /></div>
            <p class="max-w-md text-[15px]">{{ t("wizard.doneText") }}</p>
            <div class="flex flex-wrap justify-center gap-2">
              <button class="btn" @click="finishAndStart(true)"><FlaskConical :size="16" /> {{ t("wizard.startTest") }}</button>
              <button class="btn-primary" :disabled="!streamer" @click="finishAndStart(false)"><Play :size="16" /> {{ t("wizard.startLive") }}</button>
            </div>
            <button class="btn-ghost text-sm" @click="finish">{{ t("wizard.close") }}</button>
          </div>
        </div>

        <!-- Footer -->
        <footer v-if="step !== 'done'" class="flex items-center gap-2 border-t border-line px-6 py-3">
          <button class="btn" :disabled="idx === 0" @click="back"><ArrowLeft :size="15" /> {{ t("common.back") }}</button>
          <span class="flex-1" />
          <button v-if="['twitch', 'bot', 'ai'].includes(step)" class="btn-ghost" @click="next">{{ t("wizard.skipStep") }}</button>
          <button class="btn-primary" @click="next">{{ t("common.next") }} <ArrowRight :size="15" /></button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>
