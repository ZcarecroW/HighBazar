<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Clapperboard, Plus, RefreshCw, Trash2, AlarmClockOff, Calendar, ExternalLink } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiToggle from "../ui/UiToggle.vue";
import UiNumber from "../ui/UiNumber.vue";

defineProps<{ live: boolean }>();
const { t, locale } = useI18n();
const { app, id, tw } = useTwitch();

interface Ads {
  next_ad_at: string;
  last_ad_at: string;
  duration: number;
  preroll_free_time: number;
  snooze_count: number;
  snooze_refresh_at: string;
}
interface Segment {
  id: string;
  start_time: string;
  end_time: string;
  title: string;
  is_recurring: boolean;
  category: { name: string } | null;
}

const adLength = ref(60);
const channel = computed(() => app.activeChannel);
const ADS_MANAGER = "https://dashboard.twitch.tv/monetization/ads";
const ads = ref<Ads | null>(null);
const segments = ref<Segment[]>([]);
const seg = ref({ start: "", duration: 180, title: "", recurring: true });
const busy = ref(false);

const when = (s?: string | number) => {
  if (!s || s === "0") return "—";
  const d = typeof s === "number" || /^\d+$/.test(String(s)) ? new Date(Number(s) * 1000) : new Date(s);
  return Number.isNaN(d.getTime()) ? "—" : d.toLocaleString(locale.value, { dateStyle: "short", timeStyle: "short" });
};

async function load() {
  const [a, s] = await Promise.all([
    tw<{ data: Ads[] }>("GET", "/channels/ads", { broadcaster_id: id.value }, undefined, true),
    tw<{ data: { segments: Segment[] | null } }>("GET", "/schedule", { broadcaster_id: id.value, first: 10 }, undefined, true),
  ]);
  ads.value = a?.data?.[0] ?? null;
  segments.value = s?.data?.segments ?? [];
}

async function runAd() {
  if (!confirm(t("stream.ads.confirm", { s: adLength.value }))) return;
  busy.value = true;
  const r = await tw<{ data: { message: string; retry_after: number }[] }>("POST", "/channels/commercial", {}, { broadcaster_id: id.value, length: adLength.value });
  busy.value = false;
  if (r?.data?.[0]) app.toast(t("stream.ads.started", { s: adLength.value }), "success");
}

async function snooze() {
  const r = await tw("POST", "/channels/ads/schedule/snooze", { broadcaster_id: id.value });
  if (r !== null) await load();
}

const canAddSegment = computed(() => seg.value.start && seg.value.duration >= 30);
async function addSegment() {
  busy.value = true;
  const r = await tw("POST", "/schedule/segment", { broadcaster_id: id.value }, {
    start_time: new Date(seg.value.start).toISOString(),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
    duration: String(seg.value.duration),
    is_recurring: seg.value.recurring,
    title: seg.value.title.trim() || undefined,
  });
  busy.value = false;
  if (r !== null) {
    seg.value = { start: "", duration: 180, title: "", recurring: true };
    await load();
  }
}

async function removeSegment(s: Segment) {
  if (!confirm(t("stream.schedule.deleteConfirm"))) return;
  const r = await tw("DELETE", "/schedule/segment", { broadcaster_id: id.value, id: s.id });
  if (r !== null) await load();
}

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-5">
    <div class="flex items-center gap-2">
      <h2 class="section-title mb-0 flex items-center gap-2 flex-1"><Calendar :size="17" class="text-brand-500" /> {{ t("stream.ads.title") }}</h2>
      <button class="btn" :title="t('stream.reload')" @click="load"><RefreshCw :size="15" /></button>
    </div>

    <div class="space-y-2">
      <div class="flex flex-wrap items-center gap-2">
        <select v-model.number="adLength" class="input w-28">
          <option v-for="l in [30, 60, 90, 120, 150, 180]" :key="l" :value="l">{{ l }} s</option>
        </select>
        <button class="btn" :disabled="!live || busy" @click="runAd"><Clapperboard :size="15" /> {{ t("stream.ads.run") }}</button>
        <button v-if="ads && ads.snooze_count > 0" class="btn-ghost" @click="snooze"><AlarmClockOff :size="15" /> {{ t("stream.ads.snooze", { n: ads.snooze_count }) }}</button>
      </div>
      <p v-if="ads" class="text-xs text-muted">
        {{ t("stream.ads.next") }}: {{ when(ads.next_ad_at) }} · {{ t("stream.ads.last") }}: {{ when(ads.last_ad_at) }} · {{ t("stream.ads.prerollFree", { n: Math.round(ads.preroll_free_time / 60) }) }}
      </p>
      <p v-if="!live" class="hint">{{ t("stream.onlyLive") }}</p>
    </div>

    <!-- automation (Twitch's Ads Manager settings are not reachable for apps) -->
    <div v-if="channel?.ads" class="space-y-3 rounded-lg border border-line bg-panel-2 p-3">
      <div class="flex flex-wrap items-start gap-2">
        <UiToggle v-model="channel.ads.auto" class="flex-1" :label="t('stream.ads.auto')" :hint="t('stream.ads.autoHint')" />
        <button type="button" class="btn shrink-0" :title="t('stream.ads.openManagerHint')" @click="openUrl(ADS_MANAGER)"><ExternalLink :size="15" /> {{ t("stream.ads.openManager") }}</button>
      </div>
      <template v-if="channel.ads.auto">
        <UiToggle v-model="channel.ads.prerollGuard" :label="t('stream.ads.prerollGuard')" :hint="t('stream.ads.prerollGuardHint')" />
        <label v-if="channel.ads.prerollGuard" class="block w-40">
          <span class="label">{{ t("stream.ads.length") }}</span>
          <select v-model.number="channel.ads.length" class="input">
            <option v-for="l in [30, 60, 90, 120, 150, 180]" :key="l" :value="l">{{ l }} s</option>
          </select>
        </label>
        <UiToggle v-model="channel.ads.announce" :label="t('stream.ads.announce')" :hint="t('stream.ads.announceHint')" />
        <div v-if="channel.ads.announce" class="grid gap-3 sm:grid-cols-[auto_1fr]">
          <UiNumber v-model="channel.ads.announceSecs" :label="t('stream.ads.announceSecs')" :min="15" :max="300" :suffix="t('stream.seconds')" />
          <label>
            <span class="label">{{ t("stream.ads.announceText") }}</span>
            <input v-model="channel.ads.announceText" maxlength="300" class="input" />
            <span class="hint">{{ t("stream.ads.announceTextHint") }}</span>
          </label>
        </div>
        <UiToggle v-model="channel.ads.snoozeOnRaid" :label="t('stream.ads.snoozeOnRaid')" :hint="t('stream.ads.snoozeOnRaidHint')" />
        <UiToggle v-model="channel.ads.quietDuringAds" :label="t('stream.ads.quietDuringAds')" :hint="t('stream.ads.quietDuringAdsHint')" />
      </template>
    </div>

    <hr class="border-line" />

    <div class="space-y-3">
      <span class="label">{{ t("stream.schedule.title") }}</span>
      <ul class="divide-y divide-line rounded-lg border border-line">
        <li v-for="s in segments" :key="s.id" class="flex items-center gap-2 px-3 py-1.5 text-sm">
          <span class="min-w-0 flex-1">
            <span class="font-medium">{{ when(s.start_time) }}</span>
            <span class="text-xs text-muted"> · {{ s.title || s.category?.name || "—" }}<template v-if="s.is_recurring"> · {{ t("stream.schedule.weekly") }}</template></span>
          </span>
          <button class="btn-ghost py-0.5" :title="t('common.delete')" @click="removeSegment(s)"><Trash2 :size="14" /></button>
        </li>
        <li v-if="!segments.length" class="px-3 py-1.5 text-sm text-muted">{{ t("stream.empty") }}</li>
      </ul>
      <div class="grid gap-3 sm:grid-cols-2">
        <label><span class="label">{{ t("stream.schedule.start") }}</span><input v-model="seg.start" type="datetime-local" class="input" /></label>
        <UiNumber v-model="seg.duration" :label="t('stream.schedule.duration')" :min="30" :max="1380" :suffix="t('stream.minutes')" />
        <label class="sm:col-span-2"><span class="label">{{ t("stream.schedule.segmentTitle") }}</span><input v-model="seg.title" maxlength="140" class="input" /></label>
      </div>
      <div class="flex items-center gap-3">
        <UiToggle v-model="seg.recurring" :label="t('stream.schedule.weekly')" />
        <button class="btn ml-auto" :disabled="!canAddSegment || busy" @click="addSegment"><Plus :size="15" /> {{ t("common.add") }}</button>
      </div>
    </div>
  </section>
</template>
