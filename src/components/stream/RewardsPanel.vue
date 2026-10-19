<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Plus, RefreshCw, Trash2, Lock, Gift } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiToggle from "../ui/UiToggle.vue";
import UiNumber from "../ui/UiNumber.vue";

const { t } = useI18n();
const { app, id, tw } = useTwitch();

interface Reward {
  id: string;
  title: string;
  cost: number;
  prompt: string;
  is_enabled: boolean;
  is_paused: boolean;
  is_user_input_required: boolean;
  background_color: string;
  global_cooldown_setting?: { is_enabled: boolean; global_cooldown_seconds: number };
}

const rewards = ref<Reward[]>([]);
const manageable = ref(new Set<string>());
const error = ref("");
const draft = ref({ title: "", cost: 500, prompt: "", input: false, cooldown: 0, color: "#7C3AED" });
const busy = ref(false);

async function load() {
  error.value = "";
  const all = await tw<{ data: Reward[] }>("GET", "/channel_points/custom_rewards", { broadcaster_id: id.value }, undefined, true);
  if (!all) {
    error.value = t("stream.rewards.unavailable");
    rewards.value = [];
    return;
  }
  const mine = await tw<{ data: Reward[] }>("GET", "/channel_points/custom_rewards", { broadcaster_id: id.value, only_manageable_rewards: true }, undefined, true);
  rewards.value = all.data ?? [];
  manageable.value = new Set((mine?.data ?? []).map((r) => r.id));
}

async function create() {
  busy.value = true;
  const d = draft.value;
  const r = await tw("POST", "/channel_points/custom_rewards", { broadcaster_id: id.value }, {
    title: d.title.trim(),
    cost: d.cost,
    prompt: d.prompt.trim(),
    is_user_input_required: d.input,
    background_color: d.color,
    is_global_cooldown_enabled: d.cooldown > 0,
    global_cooldown_seconds: d.cooldown > 0 ? d.cooldown : undefined,
  });
  busy.value = false;
  if (r !== null) {
    app.toast(t("stream.rewards.created"), "success");
    draft.value = { title: "", cost: 500, prompt: "", input: false, cooldown: 0, color: "#7C3AED" };
    await load();
  }
}

async function patch(rw: Reward, body: Record<string, unknown>) {
  const r = await tw<{ data: Reward[] }>("PATCH", "/channel_points/custom_rewards", { broadcaster_id: id.value, id: rw.id }, body);
  const updated = r?.data?.[0];
  if (updated) Object.assign(rw, updated);
}

async function remove(rw: Reward) {
  if (!confirm(t("stream.rewards.deleteConfirm", { name: rw.title }))) return;
  const r = await tw("DELETE", "/channel_points/custom_rewards", { broadcaster_id: id.value, id: rw.id });
  if (r !== null) rewards.value = rewards.value.filter((x) => x.id !== rw.id);
}

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-4">
    <div class="flex items-center gap-2">
      <h2 class="section-title mb-0 flex items-center gap-2 flex-1"><Gift :size="17" class="text-brand-500" /> {{ t("stream.rewards.title") }}</h2>
      <button class="btn" :title="t('stream.reload')" @click="load"><RefreshCw :size="15" /></button>
    </div>
    <p class="hint -mt-2">{{ t("stream.rewards.hint") }}</p>
    <p v-if="error" class="text-sm text-amber-600">{{ error }}</p>

    <template v-else>
      <div class="grid gap-3 md:grid-cols-[minmax(0,2fr)_auto_minmax(0,2fr)_auto_auto] md:items-end">
        <label><span class="label">{{ t("stream.rewards.name") }}</span><input v-model="draft.title" maxlength="45" class="input" /></label>
        <UiNumber v-model="draft.cost" :label="t('stream.rewards.cost')" :min="1" :max="10000000" />
        <label><span class="label">{{ t("stream.rewards.prompt") }}</span><input v-model="draft.prompt" maxlength="200" class="input" /></label>
        <UiNumber v-model="draft.cooldown" :label="t('stream.rewards.cooldown')" :min="0" :max="604800" :suffix="t('stream.seconds')" />
        <label><span class="label">{{ t("stream.rewards.color") }}</span><input v-model="draft.color" type="color" class="h-9 w-12 rounded border border-line bg-panel-2" /></label>
      </div>
      <div class="flex flex-wrap items-center gap-4">
        <UiToggle v-model="draft.input" :label="t('stream.rewards.input')" />
        <button class="btn-primary ml-auto" :disabled="!draft.title.trim() || busy" @click="create"><Plus :size="15" /> {{ t("stream.rewards.create") }}</button>
      </div>

      <ul class="divide-y divide-line rounded-lg border border-line">
        <li v-for="rw in rewards" :key="rw.id" class="flex flex-wrap items-center gap-3 px-3 py-2 text-sm">
          <span class="h-4 w-4 shrink-0 rounded" :style="{ background: rw.background_color }" />
          <span class="min-w-0 flex-1">
            <span class="font-medium">{{ rw.title }}</span>
            <span class="text-xs text-muted"> · {{ rw.cost }} P<template v-if="rw.prompt"> · {{ rw.prompt }}</template></span>
          </span>
          <template v-if="manageable.has(rw.id)">
            <UiToggle :model-value="rw.is_enabled" :label="t('stream.rewards.enabled')" @update:model-value="(v: boolean) => patch(rw, { is_enabled: v })" />
            <UiToggle :model-value="rw.is_paused" :label="t('stream.rewards.paused')" @update:model-value="(v: boolean) => patch(rw, { is_paused: v })" />
            <button class="btn-ghost py-0.5" :title="t('common.delete')" @click="remove(rw)"><Trash2 :size="14" /></button>
          </template>
          <span v-else class="flex items-center gap-1 text-xs text-muted" :title="t('stream.rewards.notManageable')"><Lock :size="12" /> {{ t("stream.rewards.readOnly") }}</span>
        </li>
        <li v-if="!rewards.length" class="px-3 py-2 text-sm text-muted">{{ t("stream.empty") }}</li>
      </ul>
    </template>
  </section>
</template>
