<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Plus, Trash2, RefreshCw } from "@lucide/vue";
import { useApp } from "../stores/app";
import { pdesc, pname } from "../i18n";
import { api, errorText } from "../lib/api";
import type { ModLogEntry, ModerationProfile } from "../lib/types";
import ProfileList from "../components/ProfileList.vue";
import BuiltinBanner from "../components/BuiltinBanner.vue";
import ActionPicker from "../components/ActionPicker.vue";
import UiToggle from "../components/ui/UiToggle.vue";
import UiField from "../components/ui/UiField.vue";
import UiSlider from "../components/ui/UiSlider.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import UiTags from "../components/ui/UiTags.vue";
import UiSegmented from "../components/ui/UiSegmented.vue";

const app = useApp();
const { t } = useI18n();
const cfg = computed(() => app.config!);

const tab = ref<"profile" | "log">("profile");
const selectedId = ref(app.activeChannel?.moderationProfileId ?? cfg.value.moderationProfiles[0]!.id);
watch(
  () => cfg.value.moderationProfiles.map((p) => p.id),
  (ids) => {
    if (!ids.includes(selectedId.value)) selectedId.value = ids[0]!;
  },
);
const p = computed<ModerationProfile>(() => cfg.value.moderationProfiles.find((x) => x.id === selectedId.value)!);
const ro = computed(() => p.value.builtin);
const isUsed = computed(() => app.activeChannel?.moderationProfileId === p.value.id);

const sections = ["general", "categories", "escalation", "rules", "exemptions", "messages"] as const;
const section = ref<(typeof sections)[number]>("general");

function duplicate(id = selectedId.value) {
  const src = cfg.value.moderationProfiles.find((x) => x.id === id);
  if (src) selectedId.value = app.duplicateModeration(src, t("common.copySuffix")).id;
}
function create() {
  duplicate(cfg.value.moderationProfiles[0]!.id);
}
function remove(id: string) {
  const prof = cfg.value.moderationProfiles.find((x) => x.id === id);
  if (!prof || prof.builtin || !confirm(t("common.confirmDelete", { name: prof.name }))) return;
  cfg.value.moderationProfiles = cfg.value.moderationProfiles.filter((x) => x.id !== id);
}
function useIt() {
  if (app.activeChannel) app.activeChannel.moderationProfileId = p.value.id;
}

const modeOptions = computed(() => (["off", "log", "suggest", "auto"] as const).map((m) => ({ value: m, label: t(`moderation.modes.${m}`) })));
const aiOptions = computed(() => (["off", "suspicious", "all"] as const).map((m) => ({ value: m, label: t(`moderation.aiChecks.${m}`) })));

function addStep() {
  p.value.escalation.steps.push({ kind: "timeout", durationSecs: 3600 });
}

// ------------------------------------------------------------- log
const log = ref<ModLogEntry[]>([]);
const logFilter = ref<"all" | "actions" | "suggested">("all");
const loadingLog = ref(false);
async function loadLog() {
  if (!app.activeChannel) return;
  loadingLog.value = true;
  try {
    log.value = await api.modlog(app.activeChannel.id);
  } catch (e) {
    app.toast(errorText(e), "error");
  } finally {
    loadingLog.value = false;
  }
}
watch(tab, (v) => v === "log" && loadLog());
// Merge live updates of this session.
const merged = computed(() => {
  const map = new Map(log.value.map((e) => [e.id, e]));
  for (const e of app.modEntries.values()) map.set(e.id, e);
  let list = [...map.values()].sort((a, b) => b.ts - a.ts);
  if (logFilter.value === "actions") list = list.filter((e) => ["executed", "simulated", "undone", "failed"].includes(e.status));
  if (logFilter.value === "suggested") list = list.filter((e) => e.status === "suggested");
  return list;
});
function fmt(ts: number) {
  return new Date(ts).toLocaleString([], { dateStyle: "short", timeStyle: "short" });
}
</script>

<template>
  <div class="mx-auto max-w-[1500px] space-y-5 p-4 xl:p-6">
    <header class="flex flex-wrap items-end gap-4">
      <div class="min-w-0 flex-1">
        <h1 class="text-xl font-bold">{{ t("moderation.title") }}</h1>
        <p class="mt-1 text-sm text-muted">{{ t("moderation.intro") }}</p>
      </div>
      <UiSegmented
        v-model="tab"
        :options="[
          { value: 'profile', label: t('moderation.tabs.profile') },
          { value: 'log', label: t('moderation.tabs.log') },
        ]"
      />
    </header>

    <!-- PROFILE -->
    <div v-if="tab === 'profile'" class="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]">
      <ProfileList
        v-model="selectedId"
        :items="cfg.moderationProfiles"
        :active-id="app.activeChannel?.moderationProfileId"
        @new="create"
        @duplicate="duplicate"
        @delete="remove"
      />

      <div v-if="p" class="min-w-0 space-y-4">
        <BuiltinBanner v-if="ro" @duplicate="duplicate()" />
        <div class="card-pad flex flex-wrap items-start gap-4">
          <UiField :label="t('common.name')" class="min-w-60 flex-1">
            <input v-if="!ro" v-model="p.name" class="input" />
            <input v-else :value="pname(p)" class="input" disabled />
          </UiField>
          <UiField :label="t('common.description')" class="min-w-72 flex-[2]">
            <input v-if="!ro" v-model="p.description" class="input" />
            <input v-else :value="pdesc(p)" class="input" disabled />
          </UiField>
          <div class="self-end">
            <span v-if="isUsed" class="chip bg-emerald-500/15 py-1 text-emerald-600">{{ t("common.active") }}</span>
            <button v-else class="btn" @click="useIt">{{ t("common.setActive") }}</button>
          </div>
        </div>

        <div class="flex flex-wrap gap-1 rounded-xl border border-line bg-panel p-1">
          <button
            v-for="s in sections"
            :key="s"
            class="rounded-lg px-3 py-1.5 text-sm font-medium transition"
            :class="section === s ? 'bg-brand-500 text-white' : 'text-muted hover:bg-panel-2 hover:text-fg'"
            @click="section = s"
          >
            {{ t(`moderation.sections.${s}`) }}
          </button>
        </div>

        <fieldset :disabled="ro" class="card-pad min-w-0">
          <!-- General -->
          <div v-if="section === 'general'" class="space-y-5">
            <UiSegmented v-model="p.mode" :label="t('moderation.mode')" :options="modeOptions" :disabled="ro" />
            <UiSegmented v-model="p.aiCheck" :label="t('moderation.aiCheck')" :options="aiOptions" :disabled="ro" />
            <div class="grid gap-5 md:grid-cols-2">
              <UiSlider v-model="p.strictness" :label="t('moderation.strictness')" :hint="t('moderation.strictnessHint')" :disabled="ro" />
              <UiSlider
                v-model="p.minConfidence"
                :label="t('moderation.minConfidence')"
                :min="0.5"
                :max="1"
                :step="0.05"
                :display="Math.round(p.minConfidence * 100) + '%'"
                :disabled="ro"
              />
              <UiNumber v-model="p.maxAiCallsPerMin" :label="t('moderation.maxCalls')" :min="1" :max="120" :disabled="ro" />
            </div>
            <UiField :label="t('moderation.channelRules')" :hint="t('moderation.channelRulesHint')">
              <textarea v-model="p.channelRules" rows="3" class="input" />
            </UiField>
          </div>

          <!-- Categories -->
          <div v-else-if="section === 'categories'" class="overflow-x-auto">
            <table class="w-full min-w-[620px] text-sm">
              <thead>
                <tr class="text-left text-xs text-muted uppercase">
                  <th class="py-2 pr-3 font-semibold"></th>
                  <th class="py-2 pr-3 font-semibold">{{ t("moderation.category") }}</th>
                  <th class="py-2 pr-3 font-semibold">{{ t("moderation.clear") }}</th>
                  <th class="py-2 font-semibold">{{ t("moderation.borderline") }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="r in p.categories" :key="r.category" class="border-t border-line">
                  <td class="py-2 pr-3"><UiToggle v-model="r.enabled" :disabled="ro" /></td>
                  <td class="py-2 pr-3 font-medium">{{ t(`moderation.categories.${r.category}`) }}</td>
                  <td class="py-2 pr-3"><ActionPicker v-model="r.clearAction" :disabled="ro || !r.enabled" /></td>
                  <td class="py-2"><ActionPicker v-model="r.borderlineAction" :disabled="ro || !r.enabled" /></td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Escalation -->
          <div v-else-if="section === 'escalation'" class="space-y-5">
            <UiToggle v-model="p.escalation.enabled" :label="t('moderation.escalationEnabled')" :disabled="ro" />
            <UiNumber v-model="p.escalation.windowMins" :label="t('moderation.window')" :suffix="t('common.minutes')" :min="1" :max="10080" :disabled="ro" />
            <div class="space-y-2">
              <div v-for="(step, i) in p.escalation.steps" :key="i" class="flex items-center gap-3">
                <span class="w-36 text-sm text-muted">{{ t("moderation.step", { n: i + 1 }) }}</span>
                <ActionPicker v-model="p.escalation.steps[i]!" :disabled="ro" />
                <button v-if="!ro" class="btn-ghost" @click="p.escalation.steps.splice(i, 1)"><Trash2 :size="14" /></button>
                <span class="sr-only">{{ step.kind }}</span>
              </div>
              <button v-if="!ro" class="btn" @click="addStep"><Plus :size="14" /> {{ t("moderation.addStep") }}</button>
            </div>
          </div>

          <!-- Local rules -->
          <div v-else-if="section === 'rules'" class="space-y-6">
            <div class="grid gap-4 md:grid-cols-[minmax(0,1fr)_auto] md:items-end">
              <UiTags v-model="p.rules.blockedTerms" :label="t('moderation.blockedTerms')" :hint="t('moderation.blockedTermsHint')" :disabled="ro" />
              <UiField :label="t('moderation.action')"><ActionPicker v-model="p.rules.blockedTermsAction" :disabled="ro" /></UiField>
            </div>
            <hr class="border-line" />
            <div class="grid gap-4 md:grid-cols-3">
              <UiField :label="t('moderation.links')">
                <select v-model="p.rules.links" class="input">
                  <option value="allow">{{ t("moderation.linkModes.allow") }}</option>
                  <option value="block">{{ t("moderation.linkModes.block") }}</option>
                </select>
              </UiField>
              <UiTags v-model="p.rules.allowedDomains" :label="t('moderation.allowedDomains')" :disabled="ro || p.rules.links === 'allow'" />
              <UiField :label="t('moderation.action')"><ActionPicker v-model="p.rules.linksAction" :disabled="ro || p.rules.links === 'allow'" /></UiField>
            </div>
            <hr class="border-line" />
            <div class="grid gap-4 md:grid-cols-4 md:items-end">
              <UiToggle v-model="p.rules.capsEnabled" :label="t('moderation.caps')" :disabled="ro" />
              <UiNumber v-model="p.rules.capsMinLength" :label="t('moderation.capsMin')" :min="5" :max="500" :disabled="ro || !p.rules.capsEnabled" />
              <UiSlider v-model="p.rules.capsRatio" :label="t('moderation.capsRatio')" :min="0.5" :max="1" :step="0.05" :display="Math.round(p.rules.capsRatio * 100) + '%'" :disabled="ro || !p.rules.capsEnabled" />
              <UiField :label="t('moderation.action')"><ActionPicker v-model="p.rules.capsAction" :disabled="ro || !p.rules.capsEnabled" /></UiField>
            </div>
            <div class="grid gap-4 md:grid-cols-4 md:items-end">
              <UiToggle v-model="p.rules.repeatEnabled" :label="t('moderation.repeat')" :disabled="ro" />
              <UiNumber v-model="p.rules.repeatCount" :label="t('moderation.repeatCount')" :min="2" :max="20" :disabled="ro || !p.rules.repeatEnabled" />
              <span />
              <UiField :label="t('moderation.action')"><ActionPicker v-model="p.rules.repeatAction" :disabled="ro || !p.rules.repeatEnabled" /></UiField>
            </div>
            <div class="grid gap-4 md:grid-cols-4 md:items-end">
              <UiToggle v-model="p.rules.emoteSpamEnabled" :label="t('moderation.emoteSpam')" :disabled="ro" />
              <UiNumber v-model="p.rules.maxEmotes" :label="t('moderation.maxEmotes')" :min="1" :max="200" :disabled="ro || !p.rules.emoteSpamEnabled" />
              <span />
              <UiField :label="t('moderation.action')"><ActionPicker v-model="p.rules.emoteSpamAction" :disabled="ro || !p.rules.emoteSpamEnabled" /></UiField>
            </div>
            <div class="grid gap-4 md:grid-cols-4 md:items-end">
              <UiNumber v-model="p.rules.maxLength" :label="t('moderation.maxLength')" :min="0" :max="500" :disabled="ro" />
              <span />
              <span />
              <UiField :label="t('moderation.action')"><ActionPicker v-model="p.rules.lengthAction" :disabled="ro || !p.rules.maxLength" /></UiField>
            </div>
          </div>

          <!-- Exemptions -->
          <div v-else-if="section === 'exemptions'" class="space-y-5">
            <p class="hint mt-0">{{ t("moderation.exemptHint") }}</p>
            <div class="grid gap-4 sm:grid-cols-3">
              <UiToggle v-model="p.exempt.moderators" :label="t('moderation.exemptMods')" :disabled="ro" />
              <UiToggle v-model="p.exempt.vips" :label="t('moderation.exemptVips')" :disabled="ro" />
              <UiToggle v-model="p.exempt.subscribers" :label="t('moderation.exemptSubs')" :disabled="ro" />
            </div>
            <UiTags v-model="p.exempt.users" :label="t('moderation.exemptUsers')" :disabled="ro" />
          </div>

          <!-- Messages -->
          <div v-else-if="section === 'messages'" class="space-y-5">
            <UiField :label="t('moderation.reasonTemplate')" hint="{reason}">
              <input v-model="p.reasonTemplate" class="input" />
            </UiField>
            <UiToggle v-model="p.announceActions" :label="t('moderation.announce')" :disabled="ro" />
          </div>
        </fieldset>
      </div>
    </div>

    <!-- LOG -->
    <div v-else class="card overflow-hidden">
      <div class="flex flex-wrap items-center gap-3 border-b border-line px-4 py-2.5">
        <UiSegmented
          v-model="logFilter"
          :options="[
            { value: 'all', label: t('moderation.log.all') },
            { value: 'actions', label: t('moderation.log.onlyActions') },
            { value: 'suggested', label: t('moderation.log.onlySuggested') },
          ]"
        />
        <span v-if="!app.isRunning" class="text-xs text-muted">{{ t("moderation.log.notRunning") }}</span>
        <button class="btn ml-auto" :disabled="loadingLog" @click="loadLog"><RefreshCw :size="14" :class="loadingLog && 'animate-spin'" /></button>
      </div>
      <div v-if="!merged.length" class="p-10 text-center text-sm text-muted">{{ t("moderation.log.empty") }}</div>
      <div v-else class="selectable max-h-[calc(100vh-14rem)] overflow-y-auto">
        <table class="w-full text-sm">
          <tbody>
            <tr v-for="e in merged" :key="e.id" class="border-b border-line align-top">
              <td class="px-4 py-2.5 text-xs whitespace-nowrap text-muted">{{ fmt(e.ts) }}</td>
              <td class="px-2 py-2.5">
                <div class="font-semibold">{{ e.displayName }}</div>
                <div class="text-xs text-muted">{{ e.source }}</div>
              </td>
              <td class="w-full px-2 py-2.5">
                <div class="break-words italic">“{{ e.text }}”</div>
                <div class="mt-0.5 text-xs text-muted">
                  {{ t(`moderation.categories.${e.category}`) }} · {{ e.verdict }}<template v-if="e.source === 'ai'"> · {{ Math.round(e.confidence * 100) }}%</template>
                  <template v-if="e.reason"> — {{ e.reason }}</template>
                </div>
                <div v-if="e.error" class="text-xs text-red-500">{{ e.error }}</div>
              </td>
              <td class="px-2 py-2.5 whitespace-nowrap">
                <span class="chip bg-panel-2">
                  {{ t(`moderation.actions.${e.action.kind}`) }}<template v-if="e.action.kind === 'timeout'"> {{ Math.round(e.action.durationSecs / 60) }}m</template>
                </span>
                <div class="mt-1 text-xs text-muted">{{ t(`moderation.log.status.${e.status}`) }}</div>
              </td>
              <td class="px-4 py-2.5 whitespace-nowrap">
                <template v-if="app.isRunning && e.status === 'suggested'">
                  <button class="btn py-0.5 text-xs" @click="app.control('modApply', { id: e.id })">{{ t("common.apply") }}</button>
                  <button class="btn-ghost py-0.5 text-xs" @click="app.control('modDismiss', { id: e.id })">{{ t("common.dismiss") }}</button>
                </template>
                <button
                  v-else-if="app.isRunning && e.status === 'executed' && ['timeout', 'ban'].includes(e.action.kind)"
                  class="btn-ghost py-0.5 text-xs"
                  @click="app.control('modUndo', { id: e.id })"
                >
                  {{ t("common.undo") }}
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
