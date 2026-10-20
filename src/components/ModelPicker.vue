<script lang="ts">
import type { ModelInfo } from "../lib/types";
// Loaded model lists survive switching profiles (key: provider|base URL|key).
const cache = new Map<string, ModelInfo[]>();
</script>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Check, Loader2, RefreshCw, Search } from "../icons";
import { useApp } from "../stores/app";
import { fuzzySearch, type Ranges } from "../lib/fuzzy";
import FuzzyText from "./ui/FuzzyText.vue";
import { api, errorText } from "../lib/api";
import type { AiProfile } from "../lib/types";
import { useCloseOutside } from "../lib/outside";

// Searchable model dropdown: models are loaded from the provider
// automatically; any other model name can still be typed in.
const props = defineProps<{ profile: AiProfile; disabled?: boolean }>();
const model = defineModel<string>({ required: true });
const emit = defineEmits<{ picked: [ModelInfo] }>();
const app = useApp();
const { t, te } = useI18n();

const open = ref(false);
const filter = ref("");
const active = ref(0);
const loading = ref(false);
const error = ref("");
const models = ref<ModelInfo[]>([]);
const root = ref<HTMLElement>();
const input = ref<HTMLInputElement>();
const listEl = ref<HTMLElement>();

const info = computed(() => app.provider(props.profile.provider));
const isCli = computed(() => props.profile.provider === "claudeCli");
// Codex lists its catalog; empty = the default model of the ChatGPT plan.
const isCodex = computed(() => props.profile.provider === "codexCli");
const cacheKey = computed(() => [props.profile.provider, props.profile.baseUrl, props.profile.keyRef].join("|"));
// OpenRouter publishes its model list without a key.
const needsKey = computed(() => !!info.value?.needsKey && props.profile.provider !== "openrouter" && !app.hasKey(props.profile));
const needsUrl = computed(() => props.profile.provider === "openaiCompatible" && !props.profile.baseUrl.trim());

async function load(silent: boolean) {
  if (needsKey.value || needsUrl.value) {
    models.value = [];
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const list = await api.aiListModels(JSON.parse(JSON.stringify(props.profile)));
    cache.set(cacheKey.value, list);
    models.value = list;
  } catch (e) {
    models.value = [];
    error.value = errorText(e);
    if (!silent) app.toast(error.value, "error", 7000);
  } finally {
    loading.value = false;
  }
}
watch(
  cacheKey,
  (k) => {
    error.value = "";
    const hit = cache.get(k);
    if (hit) models.value = hit;
    else void load(true);
  },
  { immediate: true },
);
// A key that is added later makes the list loadable.
watch(needsKey, (v, old) => {
  if (old && !v) void load(true);
});

function label(m: ModelInfo) {
  if (isCodex.value && !m.id) return t("ai.codexDefaultModel");
  if (isCli.value) return te(`ai.cliModels.${m.id || "default"}`) ? t(`ai.cliModels.${m.id || "default"}`) : m.id;
  return m.name || m.id;
}
function hint(m: ModelInfo) {
  if (isCli.value && te(`ai.cliModelHints.${m.id || "default"}`)) return t(`ai.cliModelHints.${m.id || "default"}`);
  return m.name && m.name !== m.id ? m.id : "";
}

const options = computed<ModelInfo[]>(() => (isCli.value || isCodex.value ? [{ id: "", name: "" }, ...models.value] : models.value));
// "son 5" finds "Claude Sonnet 5.5", "gpt6 luna" finds "gpt-6-luna".
const ranked = computed(() => fuzzySearch(options.value, filter.value, (m) => [label(m), hint(m) || m.id]));
const filtered = computed(() => ranked.value.map((r) => r.item));
const marks = computed(() => new Map<string, Ranges[]>(ranked.value.map((r) => [r.item.id, r.ranges])));
const custom = computed(() => {
  const f = filter.value.trim();
  return f && !options.value.some((m) => m.id === f) ? f : "";
});
const rows = computed(() => (custom.value ? [...filtered.value, { id: custom.value, name: "", custom: true }] : filtered.value));

const current = computed(() => options.value.find((m) => m.id === model.value));
const buttonLabel = computed(() => {
  if (current.value) return label(current.value);
  return model.value || t("ai.models.choose");
});

async function toggle() {
  if (props.disabled) return;
  open.value = !open.value;
  if (!open.value) return;
  filter.value = "";
  active.value = Math.max(0, rows.value.findIndex((m) => m.id === model.value));
  await nextTick();
  input.value?.focus();
  scrollActive();
}
function pick(id: string) {
  model.value = id;
  open.value = false;
  const m = models.value.find((x) => x.id === id);
  if (m) emit("picked", m);
}
function scrollActive() {
  void nextTick(() => listEl.value?.querySelector<HTMLElement>(`[data-i="${active.value}"]`)?.scrollIntoView({ block: "nearest" }));
}
function onKey(e: KeyboardEvent) {
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    const n = rows.value.length;
    if (n) active.value = (active.value + (e.key === "ArrowDown" ? 1 : n - 1)) % n;
    scrollActive();
  } else if (e.key === "Enter") {
    e.preventDefault();
    const row = rows.value[active.value];
    if (row) pick(row.id);
  } else if (e.key === "Escape") {
    open.value = false;
  }
}
watch(filter, () => (active.value = 0));

useCloseOutside(root, open);
</script>

<template>
  <div ref="root" class="relative">
    <div class="flex gap-2">
      <button
        type="button"
        class="input flex min-w-0 items-center gap-2 text-left"
        :class="disabled && 'cursor-not-allowed opacity-70'"
        :aria-expanded="open"
        aria-haspopup="listbox"
        @click="toggle"
      >
        <span class="min-w-0 flex-1 truncate" :class="!model && !isCli && 'text-muted'">{{ buttonLabel }}</span>
        <span v-if="model && current && label(current).toLowerCase() !== model.toLowerCase()" class="shrink-0 truncate font-mono text-xs text-muted">{{ model }}</span>
        <svg class="h-3.5 w-3.5 shrink-0 text-muted" viewBox="0 0 12 12" aria-hidden="true"><path d="M3 4.5 6 7.5 9 4.5" fill="none" stroke="currentColor" stroke-width="1.5" /></svg>
      </button>
      <button v-if="!isCli" type="button" class="btn shrink-0" :disabled="loading || disabled" :title="t('ai.loadModels')" @click="load(false)">
        <Loader2 v-if="loading" :size="15" class="animate-spin" />
        <RefreshCw v-else :size="15" />
      </button>
    </div>

    <div v-if="open" class="card absolute z-30 mt-1 w-full overflow-hidden shadow-xl">
      <div class="flex items-center gap-2 border-b border-line px-3 py-2">
        <Search :size="14" class="shrink-0 text-muted" />
        <input
          ref="input"
          v-model="filter"
          class="min-w-0 flex-1 bg-transparent text-sm outline-none"
          :placeholder="t('ai.models.search')"
          spellcheck="false"
          @keydown="onKey"
        />
      </div>
      <ul ref="listEl" role="listbox" class="max-h-72 overflow-y-auto py-1">
        <li
          v-for="(m, i) in rows"
          :key="(('custom' in m) ? 'custom:' : '') + m.id"
          :data-i="i"
          role="option"
          :aria-selected="m.id === model"
          class="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-sm"
          :class="i === active ? 'bg-brand-500/12' : ''"
          @mouseenter="active = i"
          @click="pick(m.id)"
        >
          <Check :size="14" class="shrink-0 text-brand-500" :class="m.id === model ? '' : 'invisible'" />
          <template v-if="'custom' in m">
            <span class="min-w-0 flex-1 truncate">{{ t("ai.models.useCustom") }} <span class="font-mono">{{ m.id }}</span></span>
          </template>
          <template v-else>
            <span class="min-w-0 flex-1 truncate"><FuzzyText :text="label(m)" :ranges="marks.get(m.id)?.[0]" /></span>
            <span v-if="hint(m)" class="max-w-[55%] shrink-0 truncate text-xs text-muted" :class="!isCli && 'font-mono'"><FuzzyText :text="hint(m)" :ranges="marks.get(m.id)?.[1]" /></span>
          </template>
        </li>
        <li v-if="!rows.length" class="px-3 py-3 text-sm text-muted">{{ t("ai.models.typeName") }}</li>
      </ul>
    </div>

    <p class="hint">
      <template v-if="loading">{{ t("ai.models.loading") }}</template>
      <template v-else-if="needsKey">{{ t("ai.models.needKey") }}</template>
      <template v-else-if="needsUrl">{{ t("ai.models.needUrl") }}</template>
      <span v-else-if="error" class="text-red-500">{{ t("ai.models.loadFailed") }}: {{ error }}</span>
      <template v-else-if="isCli">{{ t("ai.models.cliHint") }}</template>
      <template v-else-if="models.length">{{ t("ai.models.count", { n: models.length }) }}</template>
      <template v-else>{{ t("ai.modelHint") }}</template>
    </p>
  </div>
</template>
