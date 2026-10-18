<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { CircleCheck, CircleAlert, ExternalLink, Loader2, Plus, Send, Trash2, TriangleAlert, Download, Search } from "@lucide/vue";
import { useApp } from "../stores/app";
import { api, errorText, newId } from "../lib/api";
import type { AiProfile, AiTestResult, UsageStats } from "../lib/types";
import ProfileList from "../components/ProfileList.vue";
import BuiltinBanner from "../components/BuiltinBanner.vue";
import UiToggle from "../components/ui/UiToggle.vue";
import UiField from "../components/ui/UiField.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import UiSegmented from "../components/ui/UiSegmented.vue";
import ApiKeyInput from "../components/ApiKeyInput.vue";

const app = useApp();
const { t } = useI18n();
const cfg = computed(() => app.config!);

const tab = ref<"profiles" | "keys" | "usage">("profiles");
const selectedId = ref(app.activeChannel?.aiProfileId || cfg.value.aiProfiles[0]!.id);
watch(
  () => cfg.value.aiProfiles.map((p) => p.id),
  (ids) => {
    if (!ids.includes(selectedId.value)) selectedId.value = ids[0]!;
  },
);
const p = computed<AiProfile>(() => cfg.value.aiProfiles.find((x) => x.id === selectedId.value)!);
const ro = computed(() => p.value.builtin);
const info = computed(() => app.provider(p.value.provider));
const usedBy = computed(() =>
  cfg.value.channels.filter((c) => c.aiProfileId === p.value.id || c.moderationAiProfileId === p.value.id).map((c) => c.name),
);

function duplicate(id = selectedId.value) {
  const src = cfg.value.aiProfiles.find((x) => x.id === id);
  if (src) selectedId.value = app.duplicateAi(src, t("common.copySuffix")).id;
}
function create() {
  const prof: AiProfile = {
    ...JSON.parse(JSON.stringify(cfg.value.aiProfiles[0])),
    id: newId(),
    name: t("ai.newProfile"),
    description: "",
    builtin: false,
    model: "",
    priceInputPerMtok: 0,
    priceOutputPerMtok: 0,
  };
  cfg.value.aiProfiles.push(prof);
  selectedId.value = prof.id;
}
function remove(id: string) {
  const prof = cfg.value.aiProfiles.find((x) => x.id === id);
  if (!prof || prof.builtin || !confirm(t("common.confirmDelete", { name: prof.name }))) return;
  cfg.value.aiProfiles = cfg.value.aiProfiles.filter((x) => x.id !== id);
}

// Key mode: shared provider key vs. own key
const ownKey = computed({
  get: () => !!p.value.keyRef && p.value.keyRef !== p.value.provider,
  set: (v) => (p.value.keyRef = v ? `profile-${p.value.id}` : ""),
});

// Models
const models = ref<string[]>([]);
const loadingModels = ref(false);
async function loadModels() {
  loadingModels.value = true;
  try {
    models.value = await api.aiListModels(JSON.parse(JSON.stringify(p.value)));
    if (!models.value.length) app.toast("0 models", "info");
  } catch (e) {
    app.toast(errorText(e), "error", 7000);
  } finally {
    loadingModels.value = false;
  }
}
watch(selectedId, () => {
  models.value = [];
  testResult.value = null;
  testError.value = "";
});

// Temperature toggle
const customTemp = computed({
  get: () => p.value.temperature !== null,
  set: (v) => (p.value.temperature = v ? 0.9 : null),
});

// CLI detection
const cliFound = ref<string | null | undefined>(undefined);
async function detectCli() {
  cliFound.value = await api.detectClaudeCli(p.value.cliPath || undefined);
}

// Test
const testPrompt = ref("");
const testing = ref(false);
const testResult = ref<AiTestResult | null>(null);
const testError = ref("");
async function runTest() {
  testing.value = true;
  testResult.value = null;
  testError.value = "";
  try {
    testResult.value = await api.aiTest(JSON.parse(JSON.stringify(p.value)), testPrompt.value || undefined);
  } catch (e) {
    testError.value = errorText(e);
  } finally {
    testing.value = false;
  }
}

function addHeader() {
  p.value.extraHeaders.push({ name: "", value: "" });
}

// ------------------------------------------------------------- usage
const usage = ref<Record<string, Record<string, UsageStats>>>({});
watch(tab, async (v) => {
  if (v === "usage") usage.value = await api.usage();
  if (v === "keys") app.keyStatus = await api.keyStatus();
});
const channelName = (id: string) => cfg.value.channels.find((c) => c.id === id)?.name ?? id;
const usageRows = computed(() =>
  Object.entries(usage.value).flatMap(([date, byChannel]) => Object.entries(byChannel).map(([ch, s]) => ({ date, ch, ...s }))),
);

const keyProviders = computed(() => app.providers.filter((x) => x.needsKey || x.id === "openaiCompatible"));
const ownKeyProfiles = computed(() => cfg.value.aiProfiles.filter((x) => x.keyRef && x.keyRef !== x.provider));
</script>

<template>
  <div class="mx-auto max-w-[1500px] space-y-5 p-4 xl:p-6">
    <header class="flex flex-wrap items-end gap-4">
      <div class="min-w-0 flex-1">
        <h1 class="text-xl font-bold">{{ t("ai.title") }}</h1>
        <p class="mt-1 text-sm text-muted">{{ t("ai.intro") }}</p>
      </div>
      <UiSegmented
        v-model="tab"
        :options="[
          { value: 'profiles', label: t('ai.tabs.profiles') },
          { value: 'keys', label: t('ai.tabs.keys') },
          { value: 'usage', label: t('ai.tabs.usage') },
        ]"
      />
    </header>

    <!-- PROFILES -->
    <div v-if="tab === 'profiles'" class="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]">
      <ProfileList
        v-model="selectedId"
        :items="cfg.aiProfiles"
        :active-id="app.activeChannel?.aiProfileId"
        @new="create"
        @duplicate="duplicate"
        @delete="remove"
      />

      <div v-if="p" class="min-w-0 space-y-4">
        <BuiltinBanner v-if="ro" @duplicate="duplicate()" />

        <fieldset :disabled="ro" class="card-pad min-w-0 space-y-5">
          <div class="grid gap-5 md:grid-cols-2">
            <UiField :label="t('common.name')"><input v-model="p.name" class="input" /></UiField>
            <UiField :label="t('ai.provider')">
              <select v-model="p.provider" class="input">
                <option v-for="pr in app.providers" :key="pr.id" :value="pr.id">{{ pr.name }}</option>
              </select>
            </UiField>
          </div>
          <UiField :label="t('common.description')"><input v-model="p.description" class="input" /></UiField>
          <p v-if="usedBy.length" class="text-xs text-muted">{{ t("ai.usedBy") }}: {{ usedBy.join(", ") }}</p>

          <div v-if="p.provider === 'claudeCli'" class="flex gap-2 rounded-xl border border-amber-400/40 bg-amber-400/10 p-3 text-sm">
            <TriangleAlert :size="16" class="mt-0.5 shrink-0 text-amber-500" />
            <span>{{ t("ai.cliWarning") }}</span>
          </div>

          <div class="grid gap-5 md:grid-cols-2">
            <UiField :label="t('ai.model')" :hint="t('ai.modelHint')">
              <div class="flex gap-2">
                <input v-model="p.model" class="input font-mono" list="hb-models" spellcheck="false" />
                <datalist id="hb-models">
                  <option v-for="m in models" :key="m" :value="m" />
                </datalist>
                <button type="button" class="btn shrink-0" :disabled="loadingModels" :title="t('ai.loadModels')" @click="loadModels">
                  <Loader2 v-if="loadingModels" :size="15" class="animate-spin" />
                  <Download v-else :size="15" />
                </button>
              </div>
            </UiField>
            <UiField v-if="p.provider !== 'claudeCli'" :label="t('ai.baseUrl')" :hint="t('ai.baseUrlHint')">
              <input v-model="p.baseUrl" class="input font-mono" :placeholder="info?.baseUrl" spellcheck="false" />
            </UiField>
            <UiField v-else :label="t('ai.cliPath')">
              <div class="flex gap-2">
                <input v-model="p.cliPath" class="input font-mono" placeholder="claude" spellcheck="false" />
                <button type="button" class="btn shrink-0" @click="detectCli"><Search :size="15" /> {{ t("ai.cliDetect") }}</button>
              </div>
              <p v-if="cliFound" class="hint text-emerald-600">{{ t("ai.cliFound", { path: cliFound }) }}</p>
              <p v-else-if="cliFound === null" class="hint text-red-500">{{ t("ai.cliNotFound") }}</p>
            </UiField>
          </div>

          <div class="grid gap-5 md:grid-cols-2 xl:grid-cols-4">
            <UiField v-if="(info?.reasoningOptions.length ?? 0) > 1" :label="t('ai.reasoning')">
              <select v-model="p.reasoning" class="input">
                <option v-for="r in info!.reasoningOptions" :key="r" :value="r">{{ r || t("ai.reasoningDefault") }}</option>
              </select>
            </UiField>
            <UiNumber v-model="p.maxOutputTokens" :label="t('ai.maxTokens')" :min="64" :max="64000" :step="64" :disabled="ro" />
            <UiNumber v-model="p.timeoutSecs" :label="t('ai.timeout')" :min="10" :max="600" :suffix="t('common.seconds')" :disabled="ro" />
            <div>
              <UiToggle v-model="customTemp" :label="t('ai.temperature')" :disabled="ro" />
              <input v-if="p.temperature !== null" v-model.number="p.temperature" type="number" step="0.1" min="0" max="2" class="input mt-2 w-24" />
              <p v-else class="hint">{{ t("ai.temperatureDefault") }}</p>
            </div>
          </div>

          <div class="grid gap-4 md:grid-cols-2">
            <UiToggle v-if="info?.webSearch" v-model="p.webSearch" :label="t('ai.webSearch')" :disabled="ro" />
            <UiToggle v-if="p.provider === 'openai'" v-model="p.openaiResponses" :label="t('ai.openaiResponses')" :disabled="ro" />
            <UiToggle v-if="p.provider === 'gemini'" v-model="p.geminiInteractions" :label="t('ai.geminiInteractions')" :disabled="ro" />
          </div>

          <div>
            <span class="label">{{ t("ai.prices") }}</span>
            <div class="flex flex-wrap gap-5">
              <UiNumber v-model="p.priceInputPerMtok" :label="t('ai.priceIn')" :min="0" :max="1000" :step="0.01" suffix="$" :disabled="ro" />
              <UiNumber v-model="p.priceOutputPerMtok" :label="t('ai.priceOut')" :min="0" :max="1000" :step="0.01" suffix="$" :disabled="ro" />
            </div>
            <p class="hint">{{ t("ai.pricesHint") }}</p>
          </div>

          <div v-if="!info?.local || p.provider === 'openaiCompatible'">
            <span class="label">{{ t("ai.headers") }}</span>
            <div v-for="(h, i) in p.extraHeaders" :key="i" class="mb-2 flex gap-2">
              <input v-model="h.name" class="input font-mono" placeholder="Header-Name" />
              <input v-model="h.value" class="input font-mono" placeholder="value" />
              <button type="button" class="btn-ghost" @click="p.extraHeaders.splice(i, 1)"><Trash2 :size="14" /></button>
            </div>
            <button v-if="!ro" type="button" class="btn text-xs" @click="addHeader"><Plus :size="13" /> {{ t("common.addItem") }}</button>
          </div>
        </fieldset>

          <!-- Key -->
        <div v-if="info?.needsKey || p.provider === 'openaiCompatible'" class="card-pad">
          <div class="mb-2 flex flex-wrap items-center gap-3">
            <span class="label mb-0">{{ t("ai.keyMode") }}</span>
            <span v-if="app.hasKey(p)" class="chip bg-emerald-500/15 text-emerald-600"><CircleCheck :size="12" /> {{ t("ai.keySet") }}</span>
            <span v-else class="chip bg-amber-500/15 text-amber-600"><CircleAlert :size="12" /> {{ t("ai.keyMissing") }}</span>
            <button v-if="info?.keyUrl" type="button" class="btn-ghost ml-auto text-xs" @click="openUrl(info.keyUrl)"><ExternalLink :size="13" /> {{ t("ai.getKey") }}</button>
          </div>
          <div v-if="!ro" class="mb-2"><UiToggle v-model="ownKey" :label="t('ai.ownKey')" /></div>
          <ApiKeyInput :key-ref="app.keyRefOf(p)" />
          <p v-if="!ownKey" class="hint">{{ t("ai.sharedKey", { provider: info?.name ?? p.provider }) }}</p>
        </div>
        <p v-else class="card-pad text-xs text-muted">{{ t("ai.keyNotNeeded") }}</p>

        <!-- Test (works for built-in profiles too) -->
        <section class="card-pad space-y-3">
          <h2 class="section-title mb-0">{{ t("common.test") }}</h2>
          <div class="flex gap-2">
            <input v-model="testPrompt" class="input" :placeholder="t('ai.testPrompt')" @keydown.enter="runTest" />
            <button class="btn-primary shrink-0" :disabled="testing" @click="runTest">
              <Loader2 v-if="testing" :size="15" class="animate-spin" />
              <Send v-else :size="15" />
              {{ t("ai.testRun") }}
            </button>
          </div>
          <div v-if="testResult" class="rounded-lg border border-emerald-500/30 bg-emerald-500/8 p-3">
            <p class="selectable text-sm">{{ testResult.text }}</p>
            <p class="mt-2 text-xs text-muted">
              {{ t("ai.testResult", { ms: testResult.millis, inT: testResult.inputTokens, outT: testResult.outputTokens, cost: testResult.costUsd.toFixed(5) }) }}
              <template v-if="testResult.model"> · {{ testResult.model }}</template>
            </p>
          </div>
          <p v-if="testError" class="selectable rounded-lg border border-red-500/30 bg-red-500/8 p-3 text-sm text-red-600 dark:text-red-400">{{ testError }}</p>
        </section>
      </div>
    </div>

    <!-- KEYS -->
    <div v-else-if="tab === 'keys'" class="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
      <div v-for="pr in keyProviders" :key="pr.id" class="card-pad space-y-2">
        <div class="flex items-center gap-2">
          <span class="font-semibold">{{ pr.name }}</span>
          <span v-if="app.keyStatus[pr.id]" class="chip bg-emerald-500/15 text-emerald-600"><CircleCheck :size="12" /> {{ t("ai.keySet") }}</span>
          <button v-if="pr.keyUrl" class="btn-ghost ml-auto text-xs" @click="openUrl(pr.keyUrl)"><ExternalLink :size="13" /> {{ t("ai.getKey") }}</button>
        </div>
        <ApiKeyInput :key-ref="pr.id" />
      </div>
      <div v-for="prof in ownKeyProfiles" :key="prof.id" class="card-pad space-y-2">
        <div class="flex items-center gap-2">
          <span class="font-semibold">{{ prof.name }}</span>
          <span v-if="app.keyStatus[prof.keyRef]" class="chip bg-emerald-500/15 text-emerald-600"><CircleCheck :size="12" /> {{ t("ai.keySet") }}</span>
        </div>
        <ApiKeyInput :key-ref="prof.keyRef" />
      </div>
    </div>

    <!-- USAGE -->
    <div v-else class="card overflow-x-auto">
      <div v-if="!usageRows.length" class="p-10 text-center text-sm text-muted">{{ t("ai.usageEmpty") }}</div>
      <table v-else class="w-full min-w-[640px] text-sm">
        <thead>
          <tr class="border-b border-line text-left text-xs text-muted uppercase">
            <th class="px-4 py-2.5">{{ t("ai.usageDate") }}</th>
            <th class="px-2 py-2.5">{{ t("nav.channels") }}</th>
            <th class="px-2 py-2.5 text-right">{{ t("ai.usageCalls") }}</th>
            <th class="px-2 py-2.5 text-right">{{ t("ai.usageTokens") }}</th>
            <th class="px-2 py-2.5 text-right">{{ t("ai.usageMessages") }}</th>
            <th class="px-2 py-2.5 text-right">{{ t("ai.usageMod") }}</th>
            <th class="px-4 py-2.5 text-right">{{ t("ai.usageCost") }}</th>
          </tr>
        </thead>
        <tbody class="tabular-nums">
          <tr v-for="r in usageRows" :key="r.date + r.ch" class="border-b border-line">
            <td class="px-4 py-2">{{ r.date }}</td>
            <td class="px-2 py-2">{{ channelName(r.ch) }}</td>
            <td class="px-2 py-2 text-right">{{ r.calls }}</td>
            <td class="px-2 py-2 text-right">{{ (r.inputTokens + r.outputTokens).toLocaleString() }}</td>
            <td class="px-2 py-2 text-right">{{ r.messagesSent }}</td>
            <td class="px-2 py-2 text-right">{{ r.modActions }}</td>
            <td class="px-4 py-2 text-right font-semibold">${{ r.costUsd.toFixed(3) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
