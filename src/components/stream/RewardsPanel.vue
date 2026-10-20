<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Plus, RefreshCw, Trash2, Lock, Gift, Save, X, Check, Undo2, Sparkles } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import { onEvent } from "../../lib/events";
import UiToggle from "../ui/UiToggle.vue";
import UiNumber from "../ui/UiNumber.vue";
import type { RewardActionKind, RewardTemplate } from "../../lib/types";

const { t, locale } = useI18n();
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
  should_redemptions_skip_request_queue?: boolean;
}
interface Redemption {
  id: string;
  user_name: string;
  user_input: string;
  redeemed_at: string;
  reward: { id: string; title: string; cost: number };
}

const ACTIONS: RewardActionKind[] = ["", "ask", "roast", "rhyme", "fact", "custom", "emoteOnly"];

// Built-in templates; names and descriptions come from the UI language.
type Builtin = Omit<RewardTemplate, "name" | "prompt"> & { key: string };
const BUILTIN: Builtin[] = [
  { id: "b-ask", key: "ask", cost: 300, input: true, cooldown: 60, color: "#7C3AED", maxPerStream: 0, maxPerUser: 0, action: "ask", instructions: "" },
  { id: "b-roast", key: "roast", cost: 500, input: false, cooldown: 120, color: "#DC2626", maxPerStream: 0, maxPerUser: 1, action: "roast", instructions: "" },
  { id: "b-rhyme", key: "rhyme", cost: 400, input: false, cooldown: 120, color: "#DB2777", maxPerStream: 0, maxPerUser: 0, action: "rhyme", instructions: "" },
  { id: "b-fact", key: "fact", cost: 200, input: false, cooldown: 60, color: "#0EA5E9", maxPerStream: 0, maxPerUser: 0, action: "fact", instructions: "" },
  { id: "b-hydrate", key: "hydrate", cost: 100, input: false, cooldown: 600, color: "#22C55E", maxPerStream: 0, maxPerUser: 0, action: "", instructions: "" },
  { id: "b-game", key: "gameWish", cost: 1000, input: true, cooldown: 0, color: "#F59E0B", maxPerStream: 0, maxPerUser: 1, action: "", instructions: "" },
  { id: "b-emote", key: "emoteOnly", cost: 2000, input: false, cooldown: 900, color: "#8B5CF6", maxPerStream: 0, maxPerUser: 0, action: "emoteOnly", instructions: "" },
];

const rewards = ref<Reward[]>([]);
const manageable = ref(new Set<string>());
const queue = ref<Redemption[]>([]);
const error = ref("");
const blank = () => ({
  title: "",
  cost: 500,
  prompt: "",
  input: false,
  cooldown: 0,
  color: "#7C3AED",
  maxPerStream: 0,
  maxPerUser: 0,
  skipQueue: false,
  action: "" as RewardActionKind,
  instructions: "",
});
const draft = ref(blank());
const busy = ref(false);
const naming = ref(false);
const templateName = ref("");

const channel = computed(() => app.activeChannel);
const userTemplates = computed(() => app.config?.settings.rewardTemplates ?? []);
const actionOf = (rewardId: string) => channel.value?.rewardActions?.find((a) => a.rewardId === rewardId);

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
  await loadQueue();
}

/** Open redemptions of the rewards HighBazar may manage (Twitch only lists those). */
async function loadQueue() {
  const ids = rewards.value.filter((r) => manageable.value.has(r.id) && !r.should_redemptions_skip_request_queue).map((r) => r.id);
  const lists = await Promise.all(
    ids.map((rid) =>
      tw<{ data: Redemption[] }>(
        "GET",
        "/channel_points/custom_rewards/redemptions",
        { broadcaster_id: id.value, reward_id: rid, status: "UNFULFILLED", first: 50 },
        undefined,
        true,
      ),
    ),
  );
  queue.value = lists.flatMap((l) => l?.data ?? []).sort((a, b) => a.redeemed_at.localeCompare(b.redeemed_at));
}

async function resolve(r: Redemption, status: "FULFILLED" | "CANCELED") {
  const res = await tw("PATCH", "/channel_points/custom_rewards/redemptions", { broadcaster_id: id.value, reward_id: r.reward.id, id: r.id }, { status });
  if (res !== null) queue.value = queue.value.filter((x) => x.id !== r.id);
}

// --- templates: built-in ones and your own --------------------------------------
function useTemplate(tpl: Builtin | RewardTemplate) {
  const key = "key" in tpl ? tpl.key : "";
  draft.value = {
    title: key ? t(`stream.rewards.tpl.${key}.name`) : (tpl as RewardTemplate).name,
    prompt: key ? t(`stream.rewards.tpl.${key}.prompt`) : (tpl as RewardTemplate).prompt,
    cost: tpl.cost,
    input: tpl.input,
    cooldown: tpl.cooldown,
    color: tpl.color,
    maxPerStream: tpl.maxPerStream,
    maxPerUser: tpl.maxPerUser,
    skipQueue: false,
    action: tpl.action,
    instructions: tpl.instructions,
  };
}

function saveTemplate() {
  const name = templateName.value.trim();
  if (!name || !app.config) return;
  const d = draft.value;
  const tpl: RewardTemplate = {
    id: crypto.randomUUID().slice(0, 12),
    name,
    cost: d.cost,
    prompt: d.prompt,
    input: d.input,
    cooldown: d.cooldown,
    color: d.color,
    maxPerStream: d.maxPerStream,
    maxPerUser: d.maxPerUser,
    action: d.action,
    instructions: d.instructions,
  };
  const list = app.config.settings.rewardTemplates;
  const i = list.findIndex((x) => x.name.toLowerCase() === name.toLowerCase());
  if (i >= 0) list.splice(i, 1, { ...tpl, id: list[i]!.id });
  else list.push(tpl);
  templateName.value = "";
  naming.value = false;
}

function removeTemplate(tpl: RewardTemplate) {
  const list = app.config!.settings.rewardTemplates;
  list.splice(list.indexOf(tpl), 1);
}

// --- what the bot does with a reward --------------------------------------------------
function setAction(rewardId: string, action: RewardActionKind, instructions = "") {
  const ch = channel.value;
  if (!ch) return;
  ch.rewardActions ??= [];
  const i = ch.rewardActions.findIndex((a) => a.rewardId === rewardId);
  if (!action) {
    if (i >= 0) ch.rewardActions.splice(i, 1);
  } else if (i >= 0) {
    ch.rewardActions[i]!.action = action;
    if (instructions) ch.rewardActions[i]!.instructions = instructions;
  } else {
    ch.rewardActions.push({ rewardId, action, instructions });
  }
}

// "Ask the bot" only works with a question.
const needsInput = computed(() => draft.value.action === "ask");
async function create() {
  busy.value = true;
  const d = draft.value;
  const r = await tw<{ data: Reward[] }>("POST", "/channel_points/custom_rewards", { broadcaster_id: id.value }, {
    title: d.title.trim(),
    cost: d.cost,
    prompt: d.prompt.trim(),
    is_user_input_required: d.input || needsInput.value,
    background_color: d.color,
    is_global_cooldown_enabled: d.cooldown > 0,
    global_cooldown_seconds: d.cooldown > 0 ? d.cooldown : undefined,
    is_max_per_stream_enabled: d.maxPerStream > 0,
    max_per_stream: d.maxPerStream > 0 ? d.maxPerStream : undefined,
    is_max_per_user_per_stream_enabled: d.maxPerUser > 0,
    max_per_user_per_stream: d.maxPerUser > 0 ? d.maxPerUser : undefined,
    // Rewards the bot handles stay in the queue until it is done (or refunds them).
    should_redemptions_skip_request_queue: d.action ? false : d.skipQueue,
  });
  busy.value = false;
  const created = r?.data?.[0];
  if (created) {
    if (d.action) setAction(created.id, d.action, d.instructions.trim());
    app.toast(t("stream.rewards.created"), "success");
    draft.value = blank();
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
  if (r !== null) {
    rewards.value = rewards.value.filter((x) => x.id !== rw.id);
    setAction(rw.id, "");
  }
}

const when = (s: string) => new Date(s).toLocaleTimeString(locale.value, { hour: "2-digit", minute: "2-digit" });

// New and handled redemptions refresh the queue (bundled).
let timer: ReturnType<typeof setTimeout> | undefined;
let unlisten: (() => void) | undefined;
onMounted(async () => {
  unlisten = await onEvent("hb:redemption", () => {
    clearTimeout(timer);
    timer = setTimeout(() => void loadQueue(), 800);
  });
  await load();
});
onUnmounted(() => {
  clearTimeout(timer);
  unlisten?.();
});
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
      <!-- queue -->
      <div v-if="queue.length" class="space-y-1.5">
        <span class="label">{{ t("stream.rewards.queue", { n: queue.length }) }}</span>
        <ul class="max-h-60 divide-y divide-line overflow-y-auto rounded-lg border border-line">
          <li v-for="r in queue" :key="r.id" class="flex items-center gap-2 px-3 py-1.5 text-sm">
            <span class="w-11 shrink-0 text-xs tabular-nums text-muted">{{ when(r.redeemed_at) }}</span>
            <span class="min-w-0 flex-1 truncate">
              <span class="font-medium">{{ r.user_name }}</span>
              <span class="text-muted"> · {{ r.reward.title }}</span>
              <template v-if="r.user_input">: {{ r.user_input }}</template>
            </span>
            <button class="btn-ghost py-0.5 text-green-600" :title="t('stream.rewards.fulfill')" :aria-label="t('stream.rewards.fulfill')" @click="resolve(r, 'FULFILLED')"><Check :size="15" /></button>
            <button class="btn-ghost py-0.5" :title="t('stream.rewards.refund')" :aria-label="t('stream.rewards.refund')" @click="resolve(r, 'CANCELED')"><Undo2 :size="15" /></button>
          </li>
        </ul>
      </div>

      <!-- templates -->
      <div class="space-y-1.5">
        <span class="label">{{ t("stream.rewards.templates") }}</span>
        <div class="flex flex-wrap items-center gap-1.5">
          <button
            v-for="tpl in BUILTIN"
            :key="tpl.id"
            type="button"
            class="chip cursor-pointer gap-1 bg-panel-2 hover:bg-brand-500/15"
            :title="t(`stream.rewards.tpl.${tpl.key}.prompt`)"
            @click="useTemplate(tpl)"
          >
            <Sparkles v-if="tpl.action" :size="11" class="text-brand-500" />
            {{ t(`stream.rewards.tpl.${tpl.key}.name`) }}
          </button>
          <span
            v-for="tpl in userTemplates"
            :key="tpl.id"
            role="button"
            tabindex="0"
            class="chip cursor-pointer gap-1 bg-panel-2 hover:bg-brand-500/15"
            :title="tpl.prompt"
            @click="useTemplate(tpl)"
            @keydown.enter="useTemplate(tpl)"
          >
            <Sparkles v-if="tpl.action" :size="11" class="text-brand-500" />
            {{ tpl.name }}
            <button type="button" class="opacity-60 hover:opacity-100" :title="t('common.delete')" :aria-label="t('common.delete')" @click.stop="removeTemplate(tpl)"><X :size="11" /></button>
          </span>
        </div>
        <p class="hint">{{ t("stream.rewards.templatesHint") }}</p>
      </div>

      <div class="grid gap-3 md:grid-cols-[minmax(0,2fr)_auto_minmax(0,2fr)_auto_auto] md:items-end">
        <label><span class="label">{{ t("stream.rewards.name") }}</span><input v-model="draft.title" maxlength="45" class="input" /></label>
        <UiNumber v-model="draft.cost" :label="t('stream.rewards.cost')" :min="1" :max="10000000" />
        <label><span class="label">{{ t("stream.rewards.prompt") }}</span><input v-model="draft.prompt" maxlength="200" class="input" /></label>
        <UiNumber v-model="draft.cooldown" :label="t('stream.rewards.cooldown')" :min="0" :max="604800" :suffix="t('stream.seconds')" />
        <label><span class="label">{{ t("stream.rewards.color") }}</span><input v-model="draft.color" type="color" class="h-9 w-12 rounded border border-line bg-panel-2" /></label>
      </div>
      <div class="flex flex-wrap items-end gap-4">
        <UiNumber v-model="draft.maxPerStream" :label="t('stream.rewards.maxPerStream')" :hint="t('stream.rewards.zeroNoLimit')" :min="0" :max="100000" />
        <UiNumber v-model="draft.maxPerUser" :label="t('stream.rewards.maxPerUser')" :hint="t('stream.rewards.zeroNoLimit')" :min="0" :max="100000" />
        <label class="min-w-48">
          <span class="label">{{ t("stream.rewards.action") }}</span>
          <select v-model="draft.action" class="input">
            <option v-for="a in ACTIONS" :key="a" :value="a">{{ t(`stream.rewards.actions.${a || "none"}`) }}</option>
          </select>
        </label>
      </div>
      <label v-if="draft.action === 'custom'" class="block">
        <span class="label">{{ t("stream.rewards.instructions") }}</span>
        <input v-model="draft.instructions" maxlength="300" class="input" :placeholder="t('stream.rewards.instructionsPlaceholder')" />
      </label>
      <p v-if="draft.action" class="hint">{{ t(`stream.rewards.actionHints.${draft.action}`) }} {{ t("stream.rewards.actionRefund") }}</p>
      <div class="flex flex-wrap items-center gap-4">
        <UiToggle
          :model-value="draft.input || needsInput"
          :disabled="needsInput"
          :label="t('stream.rewards.input')"
          @update:model-value="(v: boolean) => (draft.input = v)"
        />
        <UiToggle v-if="!draft.action" v-model="draft.skipQueue" :label="t('stream.rewards.skipQueue')" />
        <div class="ml-auto flex items-center gap-2">
          <template v-if="naming">
            <input
              v-model="templateName"
              maxlength="40"
              class="input w-40 py-1 text-xs"
              :placeholder="t('stream.rewards.templateName')"
              @keydown.enter="saveTemplate"
              @keydown.esc="naming = false"
            />
            <button type="button" class="btn py-1 text-xs" :disabled="!templateName.trim()" @click="saveTemplate"><Save :size="13" /> {{ t("common.save") }}</button>
          </template>
          <button v-else type="button" class="btn-ghost text-xs" :disabled="!draft.title.trim()" @click="(naming = true), (templateName = draft.title)">
            <Save :size="13" /> {{ t("stream.rewards.saveTemplate") }}
          </button>
          <button class="btn-primary" :disabled="!draft.title.trim() || (draft.input && !draft.prompt.trim()) || busy" @click="create"><Plus :size="15" /> {{ t("stream.rewards.create") }}</button>
        </div>
      </div>

      <ul class="divide-y divide-line rounded-lg border border-line">
        <li v-for="rw in rewards" :key="rw.id" class="flex flex-wrap items-center gap-3 px-3 py-2 text-sm">
          <span class="h-4 w-4 shrink-0 rounded" :style="{ background: rw.background_color }" />
          <span class="min-w-0 flex-1">
            <span class="font-medium">{{ rw.title }}</span>
            <span class="text-xs text-muted"> · {{ rw.cost }} P<template v-if="rw.prompt"> · {{ rw.prompt }}</template></span>
          </span>
          <template v-if="manageable.has(rw.id)">
            <select
              class="input w-auto py-0.5 text-xs"
              :value="actionOf(rw.id)?.action ?? ''"
              :title="t('stream.rewards.action')"
              :aria-label="t('stream.rewards.action')"
              @change="setAction(rw.id, ($event.target as HTMLSelectElement).value as RewardActionKind)"
            >
              <option v-for="a in ACTIONS" :key="a" :value="a">{{ t(`stream.rewards.actions.${a || "none"}`) }}</option>
            </select>
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
