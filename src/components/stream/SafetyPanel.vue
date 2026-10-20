<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Plus, RefreshCw, Save, X, Shield, Info, ShieldAlert } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiToggle from "../ui/UiToggle.vue";
import UiNumber from "../ui/UiNumber.vue";
import type { AutomodPreset } from "../../lib/types";

const { t } = useI18n();
const { app, id, tw } = useTwitch();

const CATS = [
  "disability",
  "aggression",
  "sexuality_sex_or_gender",
  "misogyny",
  "bullying",
  "swearing",
  "race_ethnicity_or_religion",
  "sex_based_terms",
] as const;
type Cat = (typeof CATS)[number];
type Automod = Record<Cat, number> & { overall_level: number | null };

// Built-in AutoMod presets (levels in the order of CATS).
const BUILTIN: { id: string; levels: number[] }[] = [
  { id: "off", levels: [0, 0, 0, 0, 0, 0, 0, 0] },
  { id: "hateOnly", levels: [3, 0, 3, 3, 0, 0, 3, 0] },
  { id: "gaming", levels: [2, 1, 2, 2, 1, 0, 2, 1] },
  { id: "balanced", levels: [3, 2, 3, 3, 2, 0, 3, 2] },
  { id: "strict", levels: [4, 3, 4, 4, 3, 1, 4, 3] },
  { id: "family", levels: [4, 4, 4, 4, 4, 4, 4, 4] },
];

const automod = ref<Automod | null>(null);
const individual = ref(false);
const shield = ref<boolean | null>(null);
const terms = ref<{ id: string; text: string }[]>([]);
const newTerm = ref("");
const busy = ref(false);
const presetName = ref("");
const naming = ref(false);

const channel = computed(() => app.activeChannel);
const userPresets = computed<AutomodPreset[]>(() => app.config?.settings.automodPresets ?? []);

const q = () => ({ broadcaster_id: id.value, moderator_id: id.value });

async function load() {
  const [a, s, b] = await Promise.all([
    tw<{ data: Automod[] }>("GET", "/moderation/automod/settings", q(), undefined, true),
    tw<{ data: { is_active: boolean }[] }>("GET", "/moderation/shield_mode", q(), undefined, true),
    tw<{ data: { id: string; text: string }[] }>("GET", "/moderation/blocked_terms", { ...q(), first: 100 }, undefined, true),
  ]);
  const am = a?.data?.[0];
  if (am) {
    automod.value = am;
    individual.value = am.overall_level === null;
    if (am.overall_level === null) am.overall_level = 0;
  }
  shield.value = s?.data?.[0]?.is_active ?? null;
  terms.value = b?.data ?? [];
}

async function saveAutomod() {
  const a = automod.value;
  if (!a) return;
  busy.value = true;
  const body = individual.value ? Object.fromEntries(CATS.map((c) => [c, a[c]])) : { overall_level: a.overall_level };
  const r = await tw("PUT", "/moderation/automod/settings", q(), body);
  busy.value = false;
  if (r !== null) {
    app.toast(t("stream.saved"), "success");
    await load();
  }
}

// --- presets: built-in ones and your own --------------------------------------
const current = computed(() => (automod.value ? CATS.map((c) => automod.value![c]) : []));
const same = (levels: number[]) => individual.value && levels.every((v, i) => v === current.value[i]);
const userLevels = (p: AutomodPreset) => CATS.map((c) => p.levels[c] ?? 0);

function apply(levels: number[]) {
  const a = automod.value;
  if (!a) return;
  individual.value = true;
  CATS.forEach((c, i) => (a[c] = levels[i] ?? 0));
}

function savePreset() {
  const name = presetName.value.trim();
  const a = automod.value;
  if (!name || !a || !app.config) return;
  const levels = Object.fromEntries(CATS.map((c) => [c, a[c]]));
  const existing = app.config.settings.automodPresets.find((p) => p.name.toLowerCase() === name.toLowerCase());
  if (existing) existing.levels = levels;
  else app.config.settings.automodPresets.push({ id: crypto.randomUUID().slice(0, 12), name, levels });
  presetName.value = "";
  naming.value = false;
}

function removePreset(p: AutomodPreset) {
  const list = app.config!.settings.automodPresets;
  list.splice(list.indexOf(p), 1);
}

async function setShield(on: boolean) {
  const r = await tw<{ data: { is_active: boolean }[] }>("PUT", "/moderation/shield_mode", q(), { is_active: on });
  if (r) shield.value = r.data?.[0]?.is_active ?? on;
}

async function addTerm() {
  const text = newTerm.value.trim();
  if (text.length < 2) return;
  const r = await tw("POST", "/moderation/blocked_terms", q(), { text });
  if (r !== null) {
    newTerm.value = "";
    await load();
  }
}

async function removeTerm(term: { id: string }) {
  const r = await tw("DELETE", "/moderation/blocked_terms", { ...q(), id: term.id });
  if (r !== null) terms.value = terms.value.filter((x) => x.id !== term.id);
}

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-5">
    <div class="flex items-center gap-2">
      <h2 class="section-title mb-0 flex items-center gap-2 flex-1"><Shield :size="17" class="text-brand-500" /> {{ t("stream.safety.title") }}</h2>
      <button class="btn" :title="t('stream.reload')" @click="load"><RefreshCw :size="15" /></button>
    </div>

    <div v-if="shield !== null" class="space-y-3 rounded-lg border border-line bg-panel-2 p-3">
      <UiToggle :model-value="shield" :label="t('stream.safety.shield')" :hint="t('stream.safety.shieldHint')" @update:model-value="setShield" />
      <template v-if="channel?.autoShield">
        <UiToggle v-model="channel.autoShield.enabled" :label="t('stream.safety.autoShield')" :hint="t('stream.safety.autoShieldHint')" />
        <div v-if="channel.autoShield.enabled" class="flex flex-wrap gap-4">
          <UiNumber v-model="channel.autoShield.attackers" :label="t('stream.safety.attackers')" :min="2" :max="50" />
          <UiNumber v-model="channel.autoShield.windowSecs" :label="t('stream.safety.window')" :min="30" :max="900" :suffix="t('stream.seconds')" />
          <UiNumber v-model="channel.autoShield.calmMins" :label="t('stream.safety.calm')" :min="1" :max="120" :suffix="t('stream.minutes')" />
        </div>
        <p v-if="channel.autoShield.enabled" class="flex items-start gap-1.5 text-xs text-muted">
          <ShieldAlert :size="13" class="mt-0.5 shrink-0" />
          {{ t("stream.safety.autoShieldHow", { n: channel.autoShield.attackers, s: channel.autoShield.windowSecs, m: channel.autoShield.calmMins }) }}
        </p>
      </template>
    </div>

    <div v-if="automod" class="space-y-3">
      <span class="label">{{ t("stream.safety.automod") }}</span>
      <div class="flex flex-wrap items-center gap-1.5">
        <button
          v-for="p in BUILTIN"
          :key="p.id"
          type="button"
          class="chip cursor-pointer"
          :class="same(p.levels) ? 'bg-brand-500 text-white' : 'bg-panel-2 hover:bg-brand-500/15'"
          :title="t(`stream.safety.presets.${p.id}Hint`)"
          @click="apply(p.levels)"
        >
          {{ t(`stream.safety.presets.${p.id}`) }}
        </button>
        <span
          v-for="p in userPresets"
          :key="p.id"
          role="button"
          tabindex="0"
          class="chip cursor-pointer gap-1"
          :class="same(userLevels(p)) ? 'bg-brand-500 text-white' : 'bg-panel-2 hover:bg-brand-500/15'"
          @click="apply(userLevels(p))"
          @keydown.enter="apply(userLevels(p))"
        >
          {{ p.name }}
          <button type="button" class="opacity-60 hover:opacity-100" :title="t('common.delete')" :aria-label="t('common.delete')" @click.stop="removePreset(p)"><X :size="11" /></button>
        </span>
        <template v-if="naming">
          <input v-model="presetName" maxlength="40" class="input w-40 py-1 text-xs" :placeholder="t('stream.safety.presetName')" @keydown.enter="savePreset" @keydown.esc="naming = false" />
          <button type="button" class="btn py-1 text-xs" :disabled="!presetName.trim()" @click="savePreset"><Save :size="13" /> {{ t("common.save") }}</button>
        </template>
        <button v-else type="button" class="btn-ghost py-1 text-xs" :title="t('stream.safety.savePresetHint')" @click="naming = true"><Plus :size="13" /> {{ t("stream.safety.savePreset") }}</button>
      </div>
      <UiToggle v-model="individual" :label="t('stream.safety.individual')" />
      <div v-if="!individual">
        <input v-model.number="automod.overall_level" type="range" min="0" max="4" step="1" class="w-full accent-brand-500" :aria-label="t('stream.safety.automod')" />
        <p class="hint">{{ t(`stream.safety.levels.${automod.overall_level}`) }}</p>
      </div>
      <div v-else class="grid gap-x-4 gap-y-2 sm:grid-cols-2">
        <label v-for="c in CATS" :key="c" class="text-xs" :title="t(`stream.safety.catHints.${c}`)">
          <span class="flex items-center justify-between gap-1">
            <span class="flex items-center gap-1">
              {{ t(`stream.safety.cats.${c}`) }}
              <Info :size="12" class="text-muted" />
            </span>
            <span class="tabular-nums text-muted">{{ automod[c] }} · {{ t(`stream.safety.catLevels.${automod[c]}`) }}</span>
          </span>
          <input v-model.number="automod[c]" type="range" min="0" max="4" step="1" class="w-full accent-brand-500" :aria-label="t(`stream.safety.cats.${c}`)" />
        </label>
      </div>
      <p v-if="individual" class="hint">{{ t("stream.safety.levelsHint") }}</p>
      <button class="btn-primary" :disabled="busy" @click="saveAutomod"><Save :size="15" /> {{ t("common.save") }}</button>
    </div>

    <div>
      <span class="label">{{ t("stream.safety.blockedTerms") }}</span>
      <p class="hint mb-2">{{ t("stream.safety.blockedTermsHint") }}</p>
      <div class="mb-2 flex gap-2">
        <input v-model="newTerm" maxlength="500" class="input" @keydown.enter="addTerm" />
        <button class="btn shrink-0" :disabled="newTerm.trim().length < 2" @click="addTerm"><Plus :size="15" /> {{ t("common.add") }}</button>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <span v-for="term in terms" :key="term.id" class="chip bg-panel-2">
          {{ term.text }}
          <button class="opacity-60 hover:opacity-100" @click="removeTerm(term)"><X :size="12" /></button>
        </span>
        <span v-if="!terms.length" class="text-sm text-muted">{{ t("stream.empty") }}</span>
      </div>
    </div>
  </section>
</template>
