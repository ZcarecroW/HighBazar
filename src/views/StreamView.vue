<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Loader2, RefreshCw, Save, Search, ShieldAlert, Radio, MonitorPlay } from "../icons";
import { useTwitch, scopesFor } from "../lib/twitch";
import type { ChannelLevel } from "../lib/types";
import TwitchLogin from "../components/TwitchLogin.vue";
import UiField from "../components/ui/UiField.vue";
import UiSegmented from "../components/ui/UiSegmented.vue";
import UiTags from "../components/ui/UiTags.vue";
import UiToggle from "../components/ui/UiToggle.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import ChatSettingsPanel from "../components/stream/ChatSettingsPanel.vue";
import ActionsPanel from "../components/stream/ActionsPanel.vue";
import PollsPanel from "../components/stream/PollsPanel.vue";
import RewardsPanel from "../components/stream/RewardsPanel.vue";
import CommunityPanel from "../components/stream/CommunityPanel.vue";
import SafetyPanel from "../components/stream/SafetyPanel.vue";
import AdsSchedulePanel from "../components/stream/AdsSchedulePanel.vue";
import ProfilePanel from "../components/stream/ProfilePanel.vue";

const { t, locale } = useI18n();
const { app, account, id, tw } = useTwitch();

const level = computed({
  get: () => (app.config?.settings.channelManagerLevel ?? "simple") as ChannelLevel,
  set: (v: ChannelLevel) => {
    if (app.config) app.config.settings.channelManagerLevel = v;
  },
});
const levelOptions = computed(() => (["simple", "advanced", "expert"] as const).map((v) => ({ value: v, label: t(`stream.levels.${v}`) })));
const missing = computed(() => {
  const have = new Set(account.value?.scopes ?? []);
  return scopesFor(level.value).filter((s) => !have.has(s));
});
const can = (l: ChannelLevel) => level.value === "expert" || (l === "advanced" ? level.value === "advanced" : l === "simple");

// ---- Stream info --------------------------------------------------------------
interface Category {
  id: string;
  name: string;
  box_art_url?: string;
}
interface Ccl {
  id: string;
  name: string;
  description: string;
}
const loading = ref(false);
const saving = ref(false);
const live = ref<{ viewer_count: number; started_at: string } | null>(null);
const title = ref("");
const category = ref<Category | null>(null);
const language = ref("de");
const tags = ref<string[]>([]);
const ccls = ref<string[]>([]);
const branded = ref(false);
const delay = ref(0);
const partner = ref(false);
const cclList = ref<Ccl[]>([]);
const original = ref("");

const LANGS = ["de", "en", "es", "fr", "it", "pt", "nl", "pl", "ru", "tr", "sv", "ja", "ko", "zh", "ar", "other"];
function langName(code: string) {
  if (code === "other") return t("stream.otherLanguage");
  try {
    return new Intl.DisplayNames([locale.value], { type: "language" }).of(code) ?? code;
  } catch {
    return code;
  }
}
const snapshot = () => JSON.stringify([title.value, category.value?.id, language.value, tags.value, ccls.value, branded.value, delay.value]);
const dirty = computed(() => original.value !== "" && snapshot() !== original.value);

async function load() {
  if (!id.value) return;
  loading.value = true;
  const [ch, st, us] = await Promise.all([
    tw<{ data: any[] }>("GET", "/channels", { broadcaster_id: id.value }),
    tw<{ data: any[] }>("GET", "/streams", { user_id: id.value }),
    tw<{ data: { broadcaster_type: string }[] }>("GET", "/users", { id: id.value }, undefined, true),
  ]);
  partner.value = us?.data?.[0]?.broadcaster_type === "partner";
  const c = ch?.data?.[0];
  if (c) {
    title.value = c.title ?? "";
    category.value = c.game_id ? { id: c.game_id, name: c.game_name } : null;
    language.value = c.broadcaster_language || "de";
    tags.value = [...(c.tags ?? [])];
    ccls.value = [...(c.content_classification_labels ?? [])];
    branded.value = !!c.is_branded_content;
    delay.value = c.delay ?? 0;
    original.value = snapshot();
  }
  live.value = st?.data?.[0] ?? null;
  if (level.value !== "simple" && !cclList.value.length) {
    const l = await tw<{ data: Ccl[] }>("GET", "/content_classification_labels", { locale: locale.value === "de" ? "de-DE" : "en-US" }, undefined, true);
    cclList.value = (l?.data ?? []).filter((x) => x.id !== "MatureGame");
  }
  loading.value = false;
}

// Category search
const query = ref("");
const results = ref<Category[]>([]);
let searchTimer: ReturnType<typeof setTimeout> | undefined;
watch(query, (q) => {
  clearTimeout(searchTimer);
  if (q.trim().length < 2) {
    results.value = [];
    return;
  }
  searchTimer = setTimeout(async () => {
    const r = await tw<{ data: Category[] }>("GET", "/search/categories", { query: q.trim(), first: 8 }, undefined, true);
    results.value = r?.data ?? [];
  }, 300);
});
function pick(c: Category) {
  category.value = c;
  query.value = "";
  results.value = [];
}
const boxArt = (url?: string) => (url ?? "").replace("{width}", "52").replace("{height}", "72").replace(/-\d+x\d+(\.\w+)$/, "-52x72$1");

const tagError = computed(() => tags.value.find((tg) => tg.length > 25 || !/^[\p{L}\p{N}]+$/u.test(tg)));

async function save() {
  if (tagError.value || tags.value.length > 10) return;
  saving.value = true;
  const body: Record<string, unknown> = {
    title: title.value.trim(),
    broadcaster_language: language.value,
    tags: tags.value,
    game_id: category.value?.id ?? "0",
  };
  if (level.value !== "simple") {
    body.content_classification_labels = cclList.value.map((c) => ({ id: c.id, is_enabled: ccls.value.includes(c.id) }));
    body.is_branded_content = branded.value;
    if (partner.value) body.delay = delay.value;
  }
  const r = await tw("PATCH", "/channels", { broadcaster_id: id.value }, body);
  saving.value = false;
  if (r !== null) {
    original.value = snapshot();
    app.toast(t("stream.saved"), "success");
    if (app.isRunning) app.control("refresh");
  }
}

function toggleCcl(cid: string, on: boolean) {
  ccls.value = on ? [...ccls.value, cid] : ccls.value.filter((x) => x !== cid);
}

watch(id, load);
watch(level, (l) => {
  if (l !== "simple" && !cclList.value.length) load();
});
onMounted(load);
</script>

<template>
  <div class="mx-auto max-w-6xl space-y-5 p-4 xl:p-6">
    <header class="flex flex-wrap items-end gap-4">
      <div class="min-w-0 flex-1">
        <h1 class="flex items-center gap-2 text-xl font-bold"><MonitorPlay :size="22" class="text-brand-500" /> {{ t("stream.title") }}</h1>
        <p class="text-sm text-muted">{{ t("stream.intro") }}</p>
      </div>
      <UiSegmented v-model="level" :label="t('stream.level')" :options="levelOptions" />
    </header>
    <p class="-mt-2 text-xs text-muted">{{ t(`stream.levelHints.${level}`) }}</p>

    <div v-if="!account" class="card-pad flex flex-wrap items-center gap-3 text-sm">
      <ShieldAlert :size="18" class="text-amber-500" />
      <span class="flex-1">{{ t("stream.noAccount") }}</span>
      <TwitchLogin kind="streamer" :level="level" primary />
    </div>

    <template v-else>
      <div v-if="missing.length" class="flex flex-wrap items-center gap-3 rounded-xl border border-amber-400/40 bg-amber-400/10 px-4 py-3 text-sm">
        <ShieldAlert :size="16" class="shrink-0 text-amber-500" />
        <span class="min-w-0 flex-1">{{ t("stream.missingScopes", { n: missing.length }, missing.length) }}</span>
        <TwitchLogin kind="streamer" :level="level" :label="t('stream.reconnect')" primary />
      </div>

      <!-- Stream info (all levels) -->
      <section class="card-pad space-y-5">
        <div class="flex flex-wrap items-center gap-3">
          <h2 class="section-title mb-0 flex flex-1 items-center gap-2"><MonitorPlay :size="17" class="text-brand-500" /> {{ t("stream.info") }}</h2>
          <span v-if="live" class="chip bg-red-500/15 text-red-600"><Radio :size="12" /> {{ t("stream.live", { n: live.viewer_count }) }}</span>
          <span v-else class="chip bg-panel-2 text-muted">{{ t("stream.offline") }}</span>
          <button class="btn" :disabled="loading" :title="t('stream.reload')" @click="load">
            <RefreshCw :size="15" :class="loading && 'animate-spin'" />
          </button>
        </div>

        <UiField :label="t('stream.streamTitle')" :hint="`${title.length}/140`">
          <input v-model="title" maxlength="140" class="input" />
        </UiField>

        <div class="grid gap-5 md:grid-cols-2">
          <UiField :label="t('stream.category')">
            <div class="relative">
              <div class="mb-2 flex items-center gap-2 text-sm">
                <span class="font-semibold">{{ category?.name || t("stream.noCategory") }}</span>
                <button v-if="category" class="btn-ghost py-0 text-xs" @click="category = null">{{ t("common.remove") }}</button>
              </div>
              <div class="relative">
                <Search :size="14" class="absolute top-2.5 left-2.5 text-muted" />
                <input v-model="query" class="input pl-8" :placeholder="t('stream.searchCategory')" />
              </div>
              <div v-if="results.length" class="card absolute z-20 mt-1 max-h-80 w-full overflow-y-auto p-1 shadow-lg">
                <button v-for="r in results" :key="r.id" class="flex w-full items-center gap-2 rounded-md p-1.5 text-left text-sm hover:bg-panel-2" @click="pick(r)">
                  <img v-if="r.box_art_url" :src="boxArt(r.box_art_url)" class="h-9 w-6.5 rounded object-cover" alt="" />
                  {{ r.name }}
                </button>
              </div>
            </div>
          </UiField>
          <UiField :label="t('stream.language')">
            <select v-model="language" class="input">
              <option v-for="l in LANGS" :key="l" :value="l">{{ langName(l) }}</option>
            </select>
          </UiField>
        </div>

        <div>
          <UiTags v-model="tags" :label="t('stream.tags')" :hint="t('stream.tagsHint')" :placeholder="t('stream.addTag')" />
          <p v-if="tagError || tags.length > 10" class="mt-1 text-xs text-red-500">{{ t("stream.tagInvalid") }}</p>
        </div>

        <template v-if="level !== 'simple'">
          <hr class="border-line" />
          <div>
            <span class="label">{{ t("stream.ccl") }}</span>
            <p class="hint mb-3">{{ t("stream.cclHint") }}</p>
            <div class="grid gap-3 md:grid-cols-2">
              <UiToggle v-for="c in cclList" :key="c.id" :model-value="ccls.includes(c.id)" :label="c.name" :hint="c.description" @update:model-value="(v: boolean) => toggleCcl(c.id, v)" />
            </div>
          </div>
          <UiToggle v-model="branded" :label="t('stream.branded')" :hint="t('stream.brandedHint')" />
          <UiNumber v-if="partner" v-model="delay" :label="t('stream.streamDelay')" :hint="t('stream.streamDelayHint')" :min="0" :max="900" :suffix="t('stream.seconds')" />
        </template>

        <div class="flex items-center gap-3">
          <button class="btn-primary" :disabled="saving || !dirty || !!tagError || !title.trim()" @click="save">
            <Loader2 v-if="saving" :size="15" class="animate-spin" />
            <Save v-else :size="15" />
            {{ t("common.save") }}
          </button>
          <span v-if="dirty" class="text-xs text-muted">{{ t("stream.unsaved") }}</span>
        </div>
      </section>

      <!-- keyed by account: panels reload when the active channel changes -->
      <div v-if="can('advanced')" :key="'adv-' + id" class="space-y-5">
        <div class="grid gap-5 lg:grid-cols-2">
          <ChatSettingsPanel />
          <ActionsPanel :live="!!live" />
        </div>
      </div>

      <div v-if="can('expert')" :key="'exp-' + id" class="space-y-5">
        <PollsPanel />
        <div class="grid gap-5 lg:grid-cols-2">
          <CommunityPanel />
          <SafetyPanel />
        </div>
        <RewardsPanel />
        <div class="grid gap-5 lg:grid-cols-2">
          <AdsSchedulePanel :live="!!live" />
          <ProfilePanel />
        </div>
      </div>
    </template>
  </div>
</template>
