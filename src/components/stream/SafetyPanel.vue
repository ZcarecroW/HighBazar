<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Plus, RefreshCw, Save, X, Shield } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiToggle from "../ui/UiToggle.vue";

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
type Automod = Record<(typeof CATS)[number], number> & { overall_level: number | null };

const automod = ref<Automod | null>(null);
const individual = ref(false);
const shield = ref<boolean | null>(null);
const terms = ref<{ id: string; text: string }[]>([]);
const newTerm = ref("");
const busy = ref(false);

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

    <div v-if="shield !== null" class="rounded-lg border border-line bg-panel-2 p-3">
      <UiToggle :model-value="shield" :label="t('stream.safety.shield')" :hint="t('stream.safety.shieldHint')" @update:model-value="setShield" />
    </div>

    <div v-if="automod" class="space-y-3">
      <span class="label">{{ t("stream.safety.automod") }}</span>
      <UiToggle v-model="individual" :label="t('stream.safety.individual')" />
      <div v-if="!individual">
        <input v-model.number="automod.overall_level" type="range" min="0" max="4" step="1" class="w-full accent-brand-500" />
        <p class="hint">{{ t(`stream.safety.levels.${automod.overall_level}`) }}</p>
      </div>
      <div v-else class="grid gap-x-4 gap-y-2 sm:grid-cols-2">
        <label v-for="c in CATS" :key="c" class="text-xs">
          <span class="flex justify-between"><span>{{ t(`stream.safety.cats.${c}`) }}</span><span class="tabular-nums">{{ automod[c] }}</span></span>
          <input v-model.number="automod[c]" type="range" min="0" max="4" step="1" class="w-full accent-brand-500" />
        </label>
      </div>
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
