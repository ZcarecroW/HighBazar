<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { CircleCheck, CircleAlert, ExternalLink, Globe, Loader2, Search } from "../icons";
import { useApp } from "../stores/app";
import { api, errorText } from "../lib/api";
import type { SearchHit, WebSearchSource } from "../lib/types";
import ApiKeyInput from "./ApiKeyInput.vue";
import UiField from "./ui/UiField.vue";
import UiNumber from "./ui/UiNumber.vue";

// HighBazar's own web search, for AI providers without a built-in one.
const app = useApp();
const { t } = useI18n();
const w = computed(() => app.config!.settings.webSearch);

const SOURCES: { id: WebSearchSource; key: boolean; url: string }[] = [
  { id: "youcom", key: false, url: "" },
  { id: "wikipedia", key: false, url: "" },
  { id: "exa", key: true, url: "https://dashboard.exa.ai/api-keys" },
  { id: "serper", key: true, url: "https://serper.dev/api-key" },
  { id: "tavily", key: true, url: "https://app.tavily.com/home" },
  { id: "brave", key: true, url: "https://api-dashboard.search.brave.com/app/keys" },
  { id: "ollama", key: true, url: "https://ollama.com/settings/keys" },
  { id: "searxng", key: false, url: "https://docs.searxng.org/admin/installation.html" },
];
const current = computed(() => SOURCES.find((s) => s.id === w.value.source) ?? SOURCES[0]!);
const keyRef = computed(() => `websearch-${w.value.source}`);
const hasKey = computed(() => !!app.keyStatus[keyRef.value]);
const usedBy = computed(() =>
  app.config!.aiProfiles.filter((p) => p.webSearch && !app.nativeSearch(p)).map((p) => p.name),
);

const query = ref("");
const testing = ref(false);
const hits = ref<SearchHit[] | null>(null);
const error = ref("");
async function test() {
  const q = query.value.trim() || "Elden Ring Nightreign";
  testing.value = true;
  hits.value = null;
  error.value = "";
  try {
    hits.value = await api.webSearchTest(JSON.parse(JSON.stringify(w.value)), q);
  } catch (e) {
    error.value = errorText(e);
  } finally {
    testing.value = false;
  }
}
</script>

<template>
  <div class="space-y-4">
    <section class="card-pad space-y-4">
      <h2 class="section-title mb-0 flex items-center gap-2"><Globe :size="17" class="text-brand-500" /> {{ t("webSearch.title") }}</h2>
      <p class="text-sm text-muted">{{ t("webSearch.intro") }}</p>
      <p v-if="usedBy.length" class="text-xs text-muted">{{ t("webSearch.usedBy", { names: usedBy.join(", ") }) }}</p>

      <div role="radiogroup" :aria-label="t('webSearch.source')" class="grid gap-2 md:grid-cols-2 xl:grid-cols-3">
        <button
          v-for="s in SOURCES"
          :key="s.id"
          type="button"
          role="radio"
          :aria-checked="w.source === s.id"
          class="rounded-xl border p-3 text-left transition"
          :class="w.source === s.id ? 'border-brand-500 bg-brand-500/10 ring-2 ring-brand-500/25' : 'border-line hover:border-brand-300'"
          @click="w.source = s.id"
        >
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold">{{ t(`webSearch.sources.${s.id}.name`) }}</span>
            <span v-if="!s.key" class="chip bg-emerald-500/15 text-emerald-600">{{ t("webSearch.noKey") }}</span>
            <CircleCheck v-else-if="app.keyStatus[`websearch-${s.id}`]" :size="14" class="ml-auto text-emerald-500" />
          </div>
          <p class="mt-1 text-xs text-muted">{{ t(`webSearch.sources.${s.id}.desc`) }}</p>
        </button>
      </div>

      <div v-if="current.key" class="space-y-2">
        <div class="flex flex-wrap items-center gap-3">
          <span class="label mb-0">{{ t("webSearch.key", { source: t(`webSearch.sources.${current.id}.name`) }) }}</span>
          <span v-if="hasKey" class="chip bg-emerald-500/15 text-emerald-600"><CircleCheck :size="12" /> {{ t("ai.keySet") }}</span>
          <span v-else class="chip bg-amber-500/15 text-amber-600"><CircleAlert :size="12" /> {{ t("ai.keyMissing") }}</span>
          <button type="button" class="btn-ghost ml-auto text-xs" @click="openUrl(current.url)"><ExternalLink :size="13" /> {{ t("ai.getKey") }}</button>
        </div>
        <ApiKeyInput :key-ref="keyRef" />
      </div>

      <UiField v-if="w.source === 'searxng'" :label="t('webSearch.searxngUrl')" :hint="t('webSearch.searxngHint')">
        <div class="flex gap-2">
          <input v-model.trim="w.searxngUrl" class="input font-mono" placeholder="http://localhost:8080" spellcheck="false" />
          <button type="button" class="btn-ghost shrink-0 text-xs" @click="openUrl(current.url)"><ExternalLink :size="13" /> {{ t("webSearch.searxngDocs") }}</button>
        </div>
      </UiField>

      <UiNumber v-model="w.maxResults" :label="t('webSearch.maxResults')" :hint="t('webSearch.maxResultsHint')" :min="1" :max="10" />
    </section>

    <section class="card-pad space-y-3">
      <h2 class="section-title mb-0 flex items-center gap-2"><Search :size="17" class="text-brand-500" /> {{ t("webSearch.test") }}</h2>
      <div class="flex gap-2">
        <input v-model="query" class="input" placeholder="Elden Ring Nightreign" :aria-label="t('webSearch.testQuery')" @keydown.enter="test" />
        <button class="btn-primary shrink-0" :disabled="testing" @click="test">
          <Loader2 v-if="testing" :size="15" class="animate-spin" />
          <Search v-else :size="15" />
          {{ t("webSearch.testRun") }}
        </button>
      </div>
      <p v-if="error" class="selectable rounded-lg border border-red-500/30 bg-red-500/8 p-3 text-sm text-red-600 dark:text-red-400">{{ error }}</p>
      <p v-else-if="hits && !hits.length" class="text-sm text-muted">{{ t("webSearch.noResults") }}</p>
      <ol v-else-if="hits" class="space-y-2">
        <li v-for="(h, i) in hits" :key="i" class="rounded-lg border border-line p-3">
          <p class="text-sm font-semibold">{{ h.title }}</p>
          <p class="selectable mt-0.5 line-clamp-3 text-xs text-muted">{{ h.snippet }}</p>
          <button v-if="h.url" type="button" class="mt-1 max-w-full truncate text-xs text-brand-600 hover:underline dark:text-brand-400" @click="openUrl(h.url)">{{ h.url }}</button>
        </li>
      </ol>
    </section>
  </div>
</template>
