<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { CircleCheck, LogOut, TriangleAlert, Radio, Users, Bolt, Layers, Send, Languages } from "../icons";
import { useApp } from "../stores/app";
import { api, errorText, newId } from "../lib/api";
import { pname } from "../i18n";
import type { ChannelProfile } from "../lib/types";
import ProfileList from "../components/ProfileList.vue";
import TwitchLogin from "../components/TwitchLogin.vue";
import UiToggle from "../components/ui/UiToggle.vue";
import UiField from "../components/ui/UiField.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import LanguagePicker from "../components/LanguagePicker.vue";

const app = useApp();
const { t } = useI18n();
const cfg = computed(() => app.config!);

const selectedId = ref(app.activeChannel?.id ?? cfg.value.channels[0]?.id ?? "");
watch(
  () => cfg.value.channels.map((c) => c.id),
  (ids) => {
    if (!ids.includes(selectedId.value)) selectedId.value = ids[0] ?? "";
  },
);
const ch = computed<ChannelProfile | undefined>(() => cfg.value.channels.find((c) => c.id === selectedId.value));
const isActive = computed(() => ch.value?.id === cfg.value.settings.activeChannelId);
const runningHere = computed(() => app.running?.channelId === ch.value?.id);

function create() {
  const base = cfg.value.channels[0];
  const p: ChannelProfile = base
    ? { ...JSON.parse(JSON.stringify(base)), id: newId(), name: t("common.new"), streamerAccountId: null, botAccountId: null, aboutStream: "" }
    : ({ id: newId(), name: t("common.new") } as ChannelProfile);
  cfg.value.channels.push(p);
  selectedId.value = p.id;
}
function duplicate(id: string) {
  const src = cfg.value.channels.find((c) => c.id === id);
  if (src) selectedId.value = app.duplicateChannel(src, t("common.copySuffix")).id;
}
function remove(id: string) {
  if (cfg.value.channels.length <= 1) return app.toast(t("channels.deleteLast"), "error");
  const c = cfg.value.channels.find((x) => x.id === id);
  if (!c || !confirm(t("common.confirmDelete", { name: c.name }))) return;
  cfg.value.channels = cfg.value.channels.filter((x) => x.id !== id);
  if (cfg.value.settings.activeChannelId === id) cfg.value.settings.activeChannelId = cfg.value.channels[0]!.id;
}

async function disconnect(accountId: string | null) {
  if (!accountId) return;
  try {
    await app.saveNow();
    app.replaceConfig(await api.twitchRemoveAccount(accountId));
    await app.refresh();
  } catch (e) {
    app.toast(errorText(e), "error");
  }
}

function onConnected(kind: "streamer" | "bot", id: string) {
  // The backend assigns new accounts to the active channel; make sure the
  // channel being edited gets it too.
  if (!ch.value) return;
  if (kind === "bot") ch.value.botAccountId = id;
  else ch.value.streamerAccountId = id;
}

const streamer = computed(() => app.account(ch.value?.streamerAccountId));
const bot = computed(() => app.account(ch.value?.botAccountId));
</script>

<template>
  <div class="mx-auto max-w-[1500px] space-y-5 p-4 xl:p-6">
    <header>
      <h1 class="flex items-center gap-2 text-xl font-bold"><Radio :size="22" class="text-brand-500" /> {{ t("channels.title") }}</h1>
      <p class="mt-1 max-w-3xl text-sm text-muted">{{ t("channels.intro") }}</p>
    </header>

    <div v-if="!app.clientIdConfigured" class="flex items-center gap-2 rounded-xl border border-amber-400/40 bg-amber-400/10 px-4 py-3 text-sm">
      <TriangleAlert :size="16" class="text-amber-500" /> {{ t("channels.noClientId") }}
    </div>

    <div class="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]">
      <ProfileList
        v-model="selectedId"
        :items="cfg.channels"
        :active-id="cfg.settings.activeChannelId ?? undefined"
        @new="create"
        @duplicate="duplicate"
        @delete="remove"
      />

      <div v-if="ch" class="space-y-5">
        <div class="card-pad flex flex-wrap items-end gap-4">
          <UiField :label="t('common.name')" class="min-w-60 flex-1">
            <input v-model="ch.name" class="input" />
          </UiField>
          <span v-if="isActive" class="chip bg-emerald-500/15 py-1 text-emerald-600"><CircleCheck :size="14" /> {{ t("common.active") }}</span>
          <button v-else class="btn" :disabled="app.isRunning" @click="cfg.settings.activeChannelId = ch.id">{{ t("common.setActive") }}</button>
        </div>

        <!-- Accounts -->
        <section class="card-pad">
          <h2 class="section-title flex items-center gap-2"><Users :size="17" class="text-brand-500" /> {{ t("channels.accounts") }}</h2>
          <div class="grid gap-5 xl:grid-cols-2">
            <div>
              <span class="label">{{ t("channels.streamer") }}</span>
              <div class="flex flex-wrap items-center gap-3 rounded-lg bg-panel-2 p-3">
                <template v-if="streamer">
                  <img v-if="streamer.profileImageUrl" :src="streamer.profileImageUrl" class="h-9 w-9 rounded-full" alt="" />
                  <div class="min-w-0 flex-1">
                    <div class="font-semibold">{{ streamer.displayName }}</div>
                    <div v-if="!app.accountStatus[streamer.id]" class="text-xs text-amber-500">{{ t("channels.needsReconnect") }}</div>
                    <div v-else class="text-xs text-muted">twitch.tv/{{ streamer.login }}</div>
                  </div>
                  <TwitchLogin kind="streamer" :label="t('common.reconnect')" @connected="(a) => onConnected('streamer', a.id)" />
                  <button class="btn-ghost" :title="t('channels.removeAccount')" :disabled="runningHere" @click="disconnect(ch.streamerAccountId)"><LogOut :size="16" /></button>
                </template>
                <template v-else>
                  <span class="flex-1 text-sm text-muted">{{ t("channels.notConnected") }}</span>
                  <TwitchLogin kind="streamer" primary @connected="(a) => onConnected('streamer', a.id)" />
                </template>
              </div>
              <p class="hint">{{ t("channels.streamerHint") }}</p>
            </div>
            <div>
              <span class="label">{{ t("channels.botAccount") }} <span class="normal-case">({{ t("common.optional") }})</span></span>
              <div class="flex flex-wrap items-center gap-3 rounded-lg bg-panel-2 p-3">
                <template v-if="bot">
                  <img v-if="bot.profileImageUrl" :src="bot.profileImageUrl" class="h-9 w-9 rounded-full" alt="" />
                  <div class="min-w-0 flex-1">
                    <div class="font-semibold">{{ bot.displayName }}</div>
                    <div class="text-xs text-muted">twitch.tv/{{ bot.login }}</div>
                  </div>
                  <button class="btn-ghost" :title="t('channels.removeAccount')" :disabled="runningHere" @click="ch.botAccountId = null"><LogOut :size="16" /></button>
                </template>
                <template v-else>
                  <span class="flex-1 text-sm text-muted">{{ t("channels.notConnected") }}</span>
                  <TwitchLogin kind="bot" @connected="(a) => onConnected('bot', a.id)" />
                </template>
              </div>
              <p class="hint">{{ t("channels.botAccountHint") }}</p>
            </div>
          </div>
        </section>

        <!-- Identity & features -->
        <section class="card-pad">
          <h2 class="section-title flex items-center gap-2"><Bolt :size="17" class="text-brand-500" /> {{ t("channels.features") }}</h2>
          <div class="grid gap-5 md:grid-cols-2 2xl:grid-cols-3">
            <UiField :label="t('channels.botName')" :hint="t('channels.botNameHint')">
              <input v-model="ch.botName" class="input" />
            </UiField>
            <div>
              <UiToggle v-model="ch.prefixEnabled" :label="t('channels.prefix')" />
              <input v-model="ch.prefix" class="input mt-2" :disabled="!ch.prefixEnabled" />
              <p class="hint">{{ t("channels.prefixHint") }}</p>
            </div>
            <div class="space-y-3">
              <UiToggle v-model="ch.companionEnabled" :label="t('channels.companionEnabled')" />
              <UiToggle v-model="ch.moderationEnabled" :label="t('channels.moderationEnabled')" />
            </div>
          </div>
        </section>

        <!-- Profiles -->
        <section class="card-pad">
          <h2 class="section-title flex items-center gap-2"><Layers :size="17" class="text-brand-500" /> {{ t("channels.profilesSection") }}</h2>
          <div class="grid gap-5 md:grid-cols-2">
            <UiField :label="t('channels.companionProfile')">
              <div class="flex gap-2">
                <select v-model="ch.companionProfileId" class="input">
                  <option v-for="p in cfg.companionProfiles" :key="p.id" :value="p.id">{{ p.builtin ? "🔒 " : "" }}{{ pname(p) }}</option>
                </select>
                <button class="btn" @click="app.view = 'companion'">{{ t("common.edit") }}</button>
              </div>
            </UiField>
            <UiField :label="t('channels.moderationProfile')">
              <div class="flex gap-2">
                <select v-model="ch.moderationProfileId" class="input">
                  <option v-for="p in cfg.moderationProfiles" :key="p.id" :value="p.id">{{ p.builtin ? "🔒 " : "" }}{{ pname(p) }}</option>
                </select>
                <button class="btn" @click="app.view = 'moderation'">{{ t("common.edit") }}</button>
              </div>
            </UiField>
            <UiField :label="t('channels.aiProfile')">
              <select v-model="ch.aiProfileId" class="input">
                <option value="">— {{ t("common.none") }} —</option>
                <option v-for="p in cfg.aiProfiles" :key="p.id" :value="p.id">
                  {{ pname(p) }}{{ app.hasKey(p) ? "" : " ⚠" }}
                </option>
              </select>
            </UiField>
            <UiField :label="t('channels.moderationAiProfile')">
              <select v-model="ch.moderationAiProfileId" class="input">
                <option value="">{{ t("channels.sameAsMain") }}</option>
                <option v-for="p in cfg.aiProfiles" :key="p.id" :value="p.id">
                  {{ pname(p) }}{{ app.hasKey(p) ? "" : " ⚠" }}
                </option>
              </select>
            </UiField>
          </div>
        </section>

        <!-- Languages & translation -->
        <section v-if="ch.translation" class="card-pad">
          <h2 class="section-title flex items-center gap-2"><Languages :size="17" class="text-brand-500" /> {{ t("translation.title") }}</h2>
          <p class="mb-4 max-w-3xl text-sm text-muted">{{ t("translation.intro") }}</p>
          <UiField :label="t('translation.languages')" :hint="t('translation.languagesHint')">
            <LanguagePicker v-model="ch.translation.languages" />
          </UiField>
          <div v-if="ch.translation.enabled && !ch.translation.languages.length" class="mt-3 flex items-center gap-2 rounded-lg border border-amber-400/40 bg-amber-400/10 px-3 py-2 text-sm">
            <TriangleAlert :size="15" class="shrink-0 text-amber-500" /> {{ t("translation.notSetUp") }}
          </div>
          <div class="mt-5 grid gap-4 md:grid-cols-2">
            <UiToggle v-model="ch.translation.enabled" :label="t('translation.enabled')" :hint="t('translation.enabledHint')" />
            <UiToggle v-model="ch.translation.noteOnDelete" :disabled="!ch.translation.enabled" :label="t('translation.noteOnDelete')" :hint="t('translation.noteOnDeleteHint')" />
          </div>
        </section>

        <!-- Behavior -->
        <section class="card-pad">
          <h2 class="section-title flex items-center gap-2"><Send :size="17" class="text-brand-500" /> {{ t("channels.behavior") }}</h2>
          <div class="grid gap-5 md:grid-cols-2 2xl:grid-cols-3">
            <UiField :label="t('channels.sendMode')">
              <select v-model="ch.sendMode" class="input">
                <option value="auto">{{ t("channels.sendModes.auto") }}</option>
                <option value="preview">{{ t("channels.sendModes.preview") }}</option>
                <option value="approve">{{ t("channels.sendModes.approve") }}</option>
              </select>
            </UiField>
            <UiNumber v-if="ch.sendMode === 'preview'" v-model="ch.previewDelaySecs" :label="t('channels.previewDelay')" :min="1" :max="300" :suffix="t('common.seconds')" />
            <UiNumber v-model="ch.dailyBudgetUsd" :label="t('channels.budget')" :hint="t('channels.budgetHint')" :min="0" :max="1000" :step="0.5" suffix="USD" />
          </div>
          <UiField :label="t('channels.aboutStream')" :hint="t('channels.aboutStreamHint')" class="mt-5">
            <textarea v-model="ch.aboutStream" rows="4" class="input" />
          </UiField>
        </section>
      </div>
    </div>
  </div>
</template>
