<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { BarChart3, Plus, RefreshCw, Trophy, X, Lock, Survey } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiNumber from "../ui/UiNumber.vue";
import UiToggle from "../ui/UiToggle.vue";

const { t } = useI18n();
const { id, tw } = useTwitch();

interface Choice {
  id: string;
  title: string;
  votes?: number;
  users?: number;
  channel_points?: number;
}
interface Poll {
  id: string;
  title: string;
  status: string;
  choices: Choice[];
}
interface Prediction {
  id: string;
  title: string;
  status: string;
  outcomes: Choice[];
  winning_outcome_id: string | null;
}

const polls = ref<Poll[]>([]);
const preds = ref<Prediction[]>([]);
const activePoll = computed(() => polls.value.find((p) => p.status === "ACTIVE"));
const openPred = computed(() => preds.value.find((p) => p.status === "ACTIVE" || p.status === "LOCKED"));

const pollTitle = ref("");
const pollChoices = ref(["", ""]);
const pollDuration = ref(120);
const pollPoints = ref(false);
const pollPointsPerVote = ref(100);
const predTitle = ref("");
const predOutcomes = ref(["", ""]);
const predWindow = ref(120);
const busy = ref(false);

async function load() {
  const [p, r] = await Promise.all([
    tw<{ data: Poll[] }>("GET", "/polls", { broadcaster_id: id.value, first: 5 }, undefined, true),
    tw<{ data: Prediction[] }>("GET", "/predictions", { broadcaster_id: id.value, first: 5 }, undefined, true),
  ]);
  polls.value = p?.data ?? [];
  preds.value = r?.data ?? [];
}

async function act(fn: () => Promise<unknown>) {
  busy.value = true;
  const r = await fn();
  busy.value = false;
  if (r !== null) await load();
}

const createPoll = () =>
  act(() =>
    tw("POST", "/polls", {}, {
      broadcaster_id: id.value,
      title: pollTitle.value.trim(),
      choices: pollChoices.value.filter((c) => c.trim()).map((c) => ({ title: c.trim() })),
      duration: pollDuration.value,
      channel_points_voting_enabled: pollPoints.value,
      channel_points_per_vote: pollPoints.value ? pollPointsPerVote.value : undefined,
    }).then((r) => {
      if (r) {
        pollTitle.value = "";
        pollChoices.value = ["", ""];
      }
      return r;
    }),
  );
const endPoll = (p: Poll) => act(() => tw("PATCH", "/polls", {}, { broadcaster_id: id.value, id: p.id, status: "TERMINATED" }));

const createPred = () =>
  act(() =>
    tw("POST", "/predictions", {}, {
      broadcaster_id: id.value,
      title: predTitle.value.trim(),
      outcomes: predOutcomes.value.filter((c) => c.trim()).map((c) => ({ title: c.trim() })),
      prediction_window: predWindow.value,
    }).then((r) => {
      if (r) {
        predTitle.value = "";
        predOutcomes.value = ["", ""];
      }
      return r;
    }),
  );
const setPred = (p: Prediction, status: string, winner?: string) =>
  act(() => tw("PATCH", "/predictions", {}, { broadcaster_id: id.value, id: p.id, status, winning_outcome_id: winner }));
const cancelPred = (p: Prediction) => {
  if (confirm(t("stream.polls.cancelConfirm"))) setPred(p, "CANCELED");
};

const total = (c: Choice[], k: "votes" | "channel_points") => c.reduce((a, x) => a + (x[k] ?? 0), 0) || 1;
const validPoll = computed(() => pollTitle.value.trim() && pollChoices.value.filter((c) => c.trim()).length >= 2);
const validPred = computed(() => predTitle.value.trim() && predOutcomes.value.filter((c) => c.trim()).length >= 2);

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-4">
    <div class="flex items-center gap-2">
      <h2 class="section-title mb-0 flex items-center gap-2 flex-1"><Survey :size="17" class="text-brand-500" /> {{ t("stream.polls.title") }}</h2>
      <button class="btn" :title="t('stream.reload')" @click="load"><RefreshCw :size="15" /></button>
    </div>
    <p class="hint -mt-2">{{ t("stream.polls.hint") }}</p>
    <div class="grid gap-6 lg:grid-cols-2">
      <!-- Poll -->
      <div class="space-y-3">
        <h3 class="flex items-center gap-1.5 text-sm font-semibold"><BarChart3 :size="15" /> {{ t("stream.polls.poll") }}</h3>
        <div v-if="activePoll" class="rounded-lg border border-line bg-panel-2 p-3">
          <div class="mb-2 font-semibold">{{ activePoll.title }}</div>
          <div v-for="c in activePoll.choices" :key="c.id" class="mb-1.5">
            <div class="flex justify-between text-xs"><span>{{ c.title }}</span><span class="tabular-nums">{{ c.votes ?? 0 }}</span></div>
            <div class="h-1.5 overflow-hidden rounded-full bg-panel"><div class="h-full bg-brand-500" :style="{ width: ((c.votes ?? 0) / total(activePoll.choices, 'votes')) * 100 + '%' }" /></div>
          </div>
          <button class="btn mt-2 text-xs" :disabled="busy" @click="endPoll(activePoll)"><X :size="13" /> {{ t("stream.polls.end") }}</button>
        </div>
        <template v-else>
          <input v-model="pollTitle" maxlength="60" class="input" :placeholder="t('stream.polls.question')" />
          <div v-for="(_, i) in pollChoices" :key="i" class="flex gap-2">
            <input v-model="pollChoices[i]" maxlength="25" class="input" :placeholder="t('stream.polls.choice', { n: i + 1 })" />
            <button v-if="pollChoices.length > 2" class="btn-ghost" @click="pollChoices.splice(i, 1)"><X :size="14" /></button>
          </div>
          <div class="flex flex-wrap items-end gap-3">
            <button v-if="pollChoices.length < 5" class="btn-ghost text-xs" @click="pollChoices.push('')"><Plus :size="13" /> {{ t("stream.polls.addChoice") }}</button>
            <UiNumber v-model="pollDuration" :label="t('stream.polls.duration')" :min="15" :max="1800" :suffix="t('stream.seconds')" class="ml-auto" />
          </div>
          <div class="flex flex-wrap items-end gap-3">
            <UiToggle v-model="pollPoints" :label="t('stream.polls.pointsVoting')" :hint="t('stream.polls.pointsVotingHint')" class="flex-1" />
            <UiNumber v-if="pollPoints" v-model="pollPointsPerVote" :min="1" :max="1000000" :suffix="t('stream.polls.pointsPerVote')" />
          </div>
          <button class="btn-primary" :disabled="!validPoll || busy" @click="createPoll">{{ t("stream.polls.start") }}</button>
        </template>
      </div>

      <!-- Prediction -->
      <div class="space-y-3">
        <h3 class="flex items-center gap-1.5 text-sm font-semibold"><Trophy :size="15" /> {{ t("stream.polls.prediction") }}</h3>
        <div v-if="openPred" class="rounded-lg border border-line bg-panel-2 p-3">
          <div class="mb-1 font-semibold">{{ openPred.title }}</div>
          <div class="mb-2 text-xs text-muted">{{ t(`stream.polls.status.${openPred.status}`) }}</div>
          <div v-for="o in openPred.outcomes" :key="o.id" class="mb-1.5 flex items-center gap-2">
            <div class="min-w-0 flex-1">
              <div class="flex justify-between text-xs"><span>{{ o.title }}</span><span class="tabular-nums">{{ o.channel_points ?? 0 }} · {{ o.users ?? 0 }}👤</span></div>
              <div class="h-1.5 overflow-hidden rounded-full bg-panel"><div class="h-full bg-brand-500" :style="{ width: ((o.channel_points ?? 0) / total(openPred.outcomes, 'channel_points')) * 100 + '%' }" /></div>
            </div>
            <button class="btn py-0.5 text-xs" :disabled="busy" :title="t('stream.polls.winner')" @click="setPred(openPred, 'RESOLVED', o.id)"><Trophy :size="12" /></button>
          </div>
          <div class="mt-2 flex gap-2">
            <button v-if="openPred.status === 'ACTIVE'" class="btn text-xs" :disabled="busy" @click="setPred(openPred, 'LOCKED')"><Lock :size="13" /> {{ t("stream.polls.lock") }}</button>
            <button class="btn-ghost text-xs" :disabled="busy" @click="cancelPred(openPred)"><X :size="13" /> {{ t("stream.polls.cancel") }}</button>
          </div>
        </div>
        <template v-else>
          <input v-model="predTitle" maxlength="45" class="input" :placeholder="t('stream.polls.question')" />
          <div v-for="(_, i) in predOutcomes" :key="i" class="flex gap-2">
            <input v-model="predOutcomes[i]" maxlength="25" class="input" :placeholder="t('stream.polls.outcome', { n: i + 1 })" />
            <button v-if="predOutcomes.length > 2" class="btn-ghost" @click="predOutcomes.splice(i, 1)"><X :size="14" /></button>
          </div>
          <div class="flex flex-wrap items-end gap-3">
            <button v-if="predOutcomes.length < 10" class="btn-ghost text-xs" @click="predOutcomes.push('')"><Plus :size="13" /> {{ t("stream.polls.addOutcome") }}</button>
            <UiNumber v-model="predWindow" :label="t('stream.polls.window')" :min="30" :max="1800" :suffix="t('stream.seconds')" class="ml-auto" />
          </div>
          <button class="btn-primary" :disabled="!validPred || busy" @click="createPred">{{ t("stream.polls.startPrediction") }}</button>
        </template>
      </div>
    </div>
  </section>
</template>
