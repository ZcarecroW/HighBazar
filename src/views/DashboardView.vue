<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { modReason } from "../i18n";
import EmoteText from "../components/EmoteText.vue";
import { Play, Square, FlaskConical, MessageSquarePlus, RefreshCw, Wifi, WifiOff, Gamepad2, Users, Clock, Coins, Send, Pencil, X, Trash2, TimerReset, Ban, ShieldAlert, Sparkles, Bot, Heart, Gift, Swords, Star, Brain, Smile, Activity, CircleAlert, Info, Bolt, Pin, Languages, EyeOff } from "../icons";
import ModNote from "../components/ModNote.vue";
import { SmoothFollow } from "../lib/smoothFollow";
import { api } from "../lib/api";
import { chatScale, clampZoom, CHAT_ZOOM } from "../lib/zoom";
import { useTwitch } from "../lib/twitch";
import { useApp } from "../stores/app";
import type { FeedItem, PendingMessage } from "../lib/types";
import UiToggle from "../components/ui/UiToggle.vue";

const app = useApp();
const { t, locale } = useI18n();

const now = ref(Date.now());
let timer: ReturnType<typeof setInterval>;
onMounted(() => (timer = setInterval(() => (now.value = Date.now()), 1000)));
onBeforeUnmount(() => clearInterval(timer));

const channel = computed(() => app.activeChannel);
const s = computed(() => app.status);

// ------------------------------------------------------------ feed
const show = ref({ chat: true, bot: true, mod: true, system: true });
const autoscroll = ref(true);
const feedEl = ref<HTMLElement | null>(null);
const feedInner = ref<HTMLElement | null>(null);
const settings = computed(() => app.config!.settings);
let follower: SmoothFollow | null = null;
// Entries newer than this fade in; older ones (re-opened view) are just there.
const mountedAt = Date.now();
onMounted(() => {
  if (!feedEl.value || !feedInner.value) return;
  follower = new SmoothFollow(feedEl.value, feedInner.value, () => settings.value.chatScroll);
  follower.onFollowChange = (v) => (autoscroll.value = v);
  if (import.meta.env.DEV) (window as unknown as { __follower: SmoothFollow }).__follower = follower;
  follower.follow();
});
onBeforeUnmount(() => follower?.destroy());
watch(autoscroll, (v) => {
  if (!follower || follower.following === v) return;
  if (v) follower.follow();
  else follower.stop();
});
async function toTray() {
  await app.saveNow();
  await api.appToTray();
}
const chatZoomStyle = computed(() => ({ zoom: chatScale(settings.value.chatZoom ?? 1, settings.value.uiZoom ?? 1) }));
const fadeMs = computed(() => (settings.value.chatScroll.smooth ? Math.min(400, settings.value.chatScroll.durationMs) : 0));
function stepChatZoom(dir: number) {
  const s = settings.value;
  s.chatZoom = dir === 0 ? 1 : clampZoom((s.chatZoom || 1) + dir * CHAT_ZOOM.step, CHAT_ZOOM);
}
const visible = computed(() =>
  app.feed.filter((f) => {
    if (f.kind === "chat") return show.value.chat;
    if (f.kind === "bot") return show.value.bot;
    if (f.kind === "mod") return show.value.mod;
    return show.value.system;
  }),
);

// One formatter for all rows (toLocaleTimeString builds a new one per call).
const timeFmt = new Intl.DateTimeFormat([], { hour: "2-digit", minute: "2-digit", hour12: false });
function time(ts: number) {
  return timeFmt.format(ts);
}
// The moderation buttons are rendered only for the row under the mouse
// (pre-rendering them in every row tripled the size of the feed).
const hoverId = ref("");
// A row only re-renders when something it shows changed, not on every status tick.
function rowDeps(item: FeedItem) {
  return [
    item,
    hoverId.value === item.id,
    isDark.value,
    locale.value,
    settings.value.showEmotes,
    app.emoteUrls,
    app.isRunning,
    app.translations.get(item.messageId),
    item.kind === "mod" ? app.modEntries.get(item.meta?.id) : null,
    channel.value?.translation?.noteOnDelete,
  ];
}
function msg(item: FeedItem) {
  return item.meta?.key ? t(`feed.${item.meta.key}`, item.meta.params ?? {}) : item.text;
}
// Twitch name colors are picked by viewers and can be unreadable (e.g. #0000FF on dark):
// lift or lower their lightness depending on the current background, like Twitch does.
const isDark = ref(document.documentElement.classList.contains("dark"));
const themeObserver = new MutationObserver(() => (isDark.value = document.documentElement.classList.contains("dark")));
onMounted(() => themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] }));
onBeforeUnmount(() => themeObserver.disconnect());
function luminance(hex: string) {
  const m = hex.match(/^#?([0-9a-f]{6})$/i);
  if (!m) return null;
  const n = parseInt(m[1]!, 16);
  const ch = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => {
    const c = v / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * ch[0]! + 0.7152 * ch[1]! + 0.0722 * ch[2]!;
}
function nameColor(item: FeedItem) {
  const c = item.color || "";
  const l = luminance(c);
  if (l === null) return isDark.value ? "oklch(0.8 calc(var(--c) * 0.8) var(--h))" : "oklch(0.5 calc(var(--c) * 0.95) var(--h))";
  if (isDark.value && l < 0.18) return `color-mix(in oklab, ${c} ${Math.round(45 + l * 200)}%, white)`;
  if (!isDark.value && l > 0.45) return `color-mix(in oklab, ${c} 55%, black)`;
  return c;
}

// ------------------------------------------------------------ pin a chat message
const { tw, id: accountId } = useTwitch();
const canPin = (item: FeedItem) => !!item.messageId && !app.running?.sandbox && !item.messageId.startsWith("sim-");
async function pin(item: FeedItem) {
  const r = await tw("PUT", "/chat/pins", { broadcaster_id: accountId.value, moderator_id: accountId.value, message_id: item.messageId });
  if (r !== null) app.toast(t("dashboard.pinned"), "success");
}

const badgeLabel: Record<string, string> = {
  broadcaster: "🎥",
  moderator: "🗡️",
  lead_moderator: "🗡️",
  vip: "💎",
  subscriber: "⭐",
  founder: "⭐",
};

// ------------------------------------------------------------ status
const nextIn = computed(() => {
  const n = s.value?.nextMessageAt;
  if (!n) return null;
  const secs = Math.max(0, Math.round((n - now.value) / 1000));
  return secs >= 60 ? `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, "0")}` : `${secs}s`;
});
const uptime = computed(() => {
  const st = s.value?.startedAt;
  if (!st || !s.value?.live) return "";
  const mins = Math.floor((now.value - new Date(st).getTime()) / 60000);
  return mins >= 60 ? `${Math.floor(mins / 60)}h ${mins % 60}m` : `${mins}m`;
});
const budgetPct = computed(() => {
  if (!s.value || !s.value.budget) return 0;
  return Math.min(100, (s.value.costToday / s.value.budget) * 100);
});

const canStart = computed(() => !!channel.value);
// Going live needs a connected streamer account; test mode works without one.
const canStartLive = computed(() => !!channel.value?.streamerAccountId && !!app.account(channel.value.streamerAccountId));
const hasAi = computed(() => !!channel.value?.aiProfileId);

// ------------------------------------------------------------ pending
const editing = ref<string | null>(null);
const editText = ref("");
function startEdit(p: PendingMessage) {
  editing.value = p.id;
  editText.value = p.text;
}
function saveEdit(p: PendingMessage) {
  app.control("edit", { id: p.id, text: editText.value });
  editing.value = null;
}
function pendingIn(p: PendingMessage) {
  return p.sendAt ? Math.max(0, Math.round((p.sendAt - now.value) / 1000)) : null;
}

// ------------------------------------------------------------ simulator
const sim = ref({ user: "TestViewer", role: "", text: "" });
function simulate() {
  if (!sim.value.text.trim()) return;
  app.control("simulate", { user: sim.value.user, text: sim.value.text, role: sim.value.role });
  sim.value.text = "";
}

// ------------------------------------------------------------ manual moderation
function manual(item: FeedItem, kind: string, duration = 0) {
  if (kind === "ban" && !confirm(t("dashboard.confirmBan", { name: item.user }))) return;
  app.control("modManual", {
    userId: item.userId,
    login: item.login,
    displayName: item.user,
    messageId: item.messageId,
    text: item.text,
    kind,
    durationSecs: duration,
  });
}

const modStatusClass: Record<string, string> = {
  executed: "bg-red-500/15 text-red-600 dark:text-red-300",
  simulated: "bg-amber-500/15 text-amber-600 dark:text-amber-300",
  suggested: "bg-amber-500/15 text-amber-600 dark:text-amber-300",
  logged: "bg-zinc-500/15 text-muted",
  failed: "bg-red-500/15 text-red-600",
  dismissed: "bg-zinc-500/15 text-muted",
  undone: "bg-emerald-500/15 text-emerald-600",
  pending: "bg-zinc-500/15 text-muted",
};

function modEntry(item: FeedItem) {
  return app.modEntries.get(item.meta?.id) ?? item.meta;
}
</script>

<template>
  <div class="flex h-full flex-col gap-4 p-4 xl:p-5">
    <!-- Header -->
    <header class="card flex flex-wrap items-center gap-3 px-4 py-3">
      <div class="flex min-w-0 items-center gap-3">
        <img
          v-if="app.account(channel?.streamerAccountId)?.profileImageUrl"
          :src="app.account(channel?.streamerAccountId)!.profileImageUrl"
          class="h-10 w-10 rounded-full"
          alt=""
        />
        <div class="min-w-0">
          <select
            v-if="app.config"
            v-model="app.config.settings.activeChannelId"
            class="max-w-64 truncate bg-transparent text-base font-semibold outline-none"
            :disabled="app.isRunning"
          >
            <option v-for="c in app.config.channels" :key="c.id" :value="c.id">{{ c.name }}</option>
          </select>
          <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted">
            <template v-if="s">
              <span class="flex items-center gap-1">
                <Wifi v-if="s.connected" :size="13" class="text-emerald-500" />
                <WifiOff v-else :size="13" class="text-amber-500" />
                {{ s.connected ? t("dashboard.connected") : t("dashboard.connecting") }}
              </span>
              <span v-if="s.live" class="chip bg-red-600 text-white">{{ t("dashboard.live") }} {{ uptime }}</span>
              <span v-else class="chip bg-panel-2">{{ t("dashboard.offline") }}</span>
              <span v-if="s.game" class="flex items-center gap-1"><Gamepad2 :size="13" />{{ s.game }}</span>
              <span v-if="s.live" class="flex items-center gap-1"><Users :size="13" />{{ t("dashboard.viewers", { n: s.viewers }) }}</span>
            </template>
            <span v-else>{{ t("dashboard.notRunning") }}</span>
          </div>
        </div>
      </div>

      <div class="ml-auto flex flex-wrap items-center gap-2">
        <button v-if="settings.performanceMode" class="btn" :title="t('dashboard.toTrayHint')" @click="toTray"><EyeOff :size="16" /> {{ t("dashboard.toTray") }}</button>
        <template v-if="!app.isRunning">
          <button class="btn" :disabled="!canStart" @click="app.startBot(true)"><FlaskConical :size="16" /> {{ t("dashboard.startTest") }}</button>
          <button class="btn-primary" :disabled="!canStartLive" :title="canStartLive ? undefined : t('dashboard.needAccount')" @click="app.startBot(false)"><Play :size="16" /> {{ t("dashboard.start") }}</button>
        </template>
        <template v-else>
          <button class="btn" :title="t('dashboard.refreshInfo')" @click="app.control('refresh')"><RefreshCw :size="16" /></button>
          <button class="btn" @click="app.control('sayNow')"><MessageSquarePlus :size="16" /> {{ t("dashboard.sayNow") }}</button>
          <button class="btn-danger" @click="app.stopBot()"><Square :size="16" /> {{ t("dashboard.stop") }}</button>
        </template>
      </div>
    </header>

    <div v-if="app.running?.sandbox" class="flex items-center gap-2 rounded-xl border border-amber-400/40 bg-amber-400/10 px-4 py-2 text-sm">
      <FlaskConical :size="16" class="text-amber-500" /> {{ t("dashboard.testModeBanner") }}
    </div>
    <div v-if="channel && !hasAi" class="flex items-center gap-2 rounded-xl border border-amber-400/40 bg-amber-400/10 px-4 py-2 text-sm">
      <CircleAlert :size="16" class="text-amber-500" /> {{ t("dashboard.noAi") }}
      <button class="btn ml-auto" @click="app.view = 'channels'">{{ t("nav.channels") }}</button>
    </div>
    <div v-if="channel?.translation?.enabled && !channel.translation.languages.length && !app.isRunning" class="flex items-center gap-2 rounded-xl border border-line bg-panel-2 px-4 py-2 text-sm">
      <Languages :size="16" class="text-brand-500" /> {{ t("translation.dashboardHint") }}
      <button class="btn ml-auto" @click="app.view = 'channels'">{{ t("nav.channels") }}</button>
    </div>

    <!-- Body -->
    <div class="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1fr)_340px] 2xl:grid-cols-[minmax(0,1fr)_400px]">
      <!-- Feed -->
      <section class="card flex min-h-[420px] flex-col overflow-hidden">
        <div class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-line px-4 py-2.5">
          <h2 class="text-sm font-semibold">{{ t("dashboard.feed") }}</h2>
          <div class="flex flex-wrap items-center gap-1 text-xs">
            <button
              v-for="k in (['chat', 'bot', 'mod', 'system'] as const)"
              :key="k"
              class="chip transition"
              :class="show[k] ? 'bg-brand-500/15 text-brand-700 dark:text-brand-200' : 'bg-panel-2 text-muted line-through'"
              @click="show[k] = !show[k]"
            >
              {{ t(`dashboard.filter${k === "mod" ? "Mod" : k[0]!.toUpperCase() + k.slice(1)}`) }}
            </button>
          </div>
          <div class="ml-auto flex items-center gap-0.5 text-xs text-muted" :title="t('dashboard.chatZoomHint')">
            <button class="rounded px-1.5 py-0.5 font-semibold hover:bg-panel-2 hover:text-fg" :aria-label="t('dashboard.chatZoomOut')" @click="stepChatZoom(-1)">A−</button>
            <button class="w-11 rounded py-0.5 tabular-nums hover:bg-panel-2 hover:text-fg" :aria-label="t('dashboard.chatZoomReset')" @click="stepChatZoom(0)">{{ Math.round((settings.chatZoom || 1) * 100) }} %</button>
            <button class="rounded px-1.5 py-0.5 text-sm font-semibold hover:bg-panel-2 hover:text-fg" :aria-label="t('dashboard.chatZoomIn')" @click="stepChatZoom(1)">A+</button>
          </div>
          <label class="flex items-center gap-1.5 text-xs text-muted">
            <input v-model="autoscroll" type="checkbox" class="accent-brand-500" /> {{ t("dashboard.autoscroll") }}
          </label>
        </div>

        <div ref="feedEl" class="selectable min-h-0 flex-1 overflow-y-auto px-2 py-2" data-chat-zoom>
          <div ref="feedInner" class="flex min-h-full flex-col" :style="{ ...chatZoomStyle, '--feed-in-ms': fadeMs + 'ms' }">
          <div v-if="!visible.length" class="flex flex-1 flex-col items-center justify-center gap-2 p-8 text-center text-sm text-muted">
            <Sparkles :size="28" class="text-brand-400" />
            <p class="max-w-sm">{{ app.isRunning ? t("dashboard.feedEmpty") : t("dashboard.notRunningHint") }}</p>
          </div>

          <div
            v-for="item in visible"
            :key="item.id"
            v-memo="rowDeps(item)"
            class="group relative flex gap-2 rounded-lg px-2 py-1 text-[13.5px] leading-snug hover:bg-panel-2"
            @mouseenter="hoverId = item.id"
            @mouseleave="hoverId = ''"
            :class="{
              'feed-in': fadeMs > 0 && item.ts >= mountedAt,
              'bg-brand-500/8': item.kind === 'bot',
              'bg-red-500/6': item.kind === 'mod' || item.kind === 'error',
              'bg-amber-500/8': item.kind === 'event',
            }"
          >
            <span class="w-10 shrink-0 pt-px text-[11px] text-muted tabular-nums">{{ time(item.ts) }}</span>

            <!-- chat -->
            <div v-if="item.kind === 'chat'" class="min-w-0 flex-1 break-words">
              <span v-for="b in item.badges" :key="b" class="mr-0.5 text-[11px]">{{ badgeLabel[b] ?? "" }}</span>
              <span class="font-semibold" :style="{ color: nameColor(item) }">{{ item.user }}</span><span class="text-muted">: </span>
              <span><EmoteText :text="item.text" :emotes="item.meta?.emotes" /></span>
              <span v-if="item.meta?.firstMsg" class="chip ml-1 bg-emerald-500/15 text-emerald-600">{{ t("dashboard.newChatter") }}</span>
              <span v-if="item.meta?.shared" class="chip ml-1 bg-sky-500/15 text-sky-600 dark:text-sky-300">{{ t("dashboard.sharedChat") }}</span>
              <span v-if="item.meta?.bits" class="chip ml-1 bg-purple-500/15 text-purple-600">{{ item.meta.bits }} bits</span>
              <div v-if="app.translations.get(item.messageId)" class="mt-0.5 flex items-start gap-1 text-xs text-muted">
                <Languages :size="12" class="mt-px shrink-0 text-brand-500" />
                <span><span class="font-medium">{{ app.translations.get(item.messageId)!.lang }}:</span> <EmoteText :text="app.translations.get(item.messageId)!.text" :emotes="item.meta?.emotes" /></span>
              </div>
              <span v-if="hoverId === item.id && app.isRunning && item.userId" class="absolute top-0.5 right-1 flex gap-0.5 rounded-md border border-line bg-panel p-0.5 shadow-sm">
                <button v-if="canPin(item)" class="rounded p-1 text-muted hover:text-brand-500" :title="t('dashboard.pin')" @click="pin(item)"><Pin :size="13" /></button>
                <button class="rounded p-1 text-muted hover:text-fg" :title="t('dashboard.deleteMsg')" @click="manual(item, 'delete')"><Trash2 :size="13" /></button>
                <button class="rounded p-1 text-muted hover:text-amber-500" :title="t('dashboard.timeout10')" @click="manual(item, 'timeout', 600)"><TimerReset :size="13" /></button>
                <button class="rounded p-1 text-muted hover:text-red-500" :title="t('dashboard.ban')" @click="manual(item, 'ban')"><Ban :size="13" /></button>
              </span>
            </div>

            <!-- bot -->
            <div v-else-if="item.kind === 'bot'" class="min-w-0 flex-1 break-words">
              <Bot :size="13" class="mr-1 inline text-brand-500" />
              <span class="font-semibold text-brand-600 dark:text-brand-300">{{ item.user || "HighBazar" }}</span><span class="text-muted">: </span>
              <span><EmoteText :text="item.text" /></span>
              <span v-if="item.meta?.reason" class="chip ml-1 bg-brand-500/12 text-brand-600 dark:text-brand-300">{{ t(`dashboard.reasons.${item.meta.reason}`) }}</span>
            </div>

            <!-- moderation -->
            <div v-else-if="item.kind === 'mod'" class="min-w-0 flex-1 break-words">
              <ShieldAlert :size="13" class="mr-1 inline text-red-500" />
              <template v-if="modEntry(item)">
                <span class="font-semibold">{{ modEntry(item).displayName }}</span>
                <span class="chip mx-1" :class="modStatusClass[modEntry(item).status] ?? ''">
                  {{ t(`moderation.actions.${modEntry(item).action.kind}`) }}
                  <template v-if="modEntry(item).action.kind === 'timeout'"> {{ Math.round(modEntry(item).action.durationSecs / 60) }}m</template>
                  · {{ t(`moderation.log.status.${modEntry(item).status}`) }}
                </span>
                <span class="text-muted">{{ t(`moderation.categories.${modEntry(item).category}`) }}</span>
                <span v-if="modEntry(item).reason" class="text-muted"> — {{ modReason(modEntry(item).reason) }}</span>
                <div class="mt-0.5 truncate text-xs text-muted italic">“{{ modEntry(item).text }}”</div>
                <ModNote :entry="modEntry(item)" :note-on-delete="channel?.translation?.noteOnDelete ?? true" />
                <div v-if="modEntry(item).status === 'suggested' || ['executed'].includes(modEntry(item).status)" class="mt-1 flex gap-1">
                  <template v-if="modEntry(item).status === 'suggested'">
                    <button class="btn py-0.5 text-xs" @click="app.control('modApply', { id: modEntry(item).id })">{{ t("common.apply") }}</button>
                    <button class="btn-ghost py-0.5 text-xs" @click="app.control('modDismiss', { id: modEntry(item).id })">{{ t("common.dismiss") }}</button>
                  </template>
                  <button
                    v-else-if="['timeout', 'ban'].includes(modEntry(item).action.kind)"
                    class="btn-ghost py-0.5 text-xs"
                    @click="app.control('modUndo', { id: modEntry(item).id })"
                  >
                    {{ t("common.undo") }}
                  </button>
                </div>
              </template>
              <span v-else>{{ item.text }}</span>
            </div>

            <!-- event -->
            <div v-else-if="item.kind === 'event'" class="min-w-0 flex-1 break-words text-amber-700 dark:text-amber-300">
              <Star :size="13" class="mr-1 inline" />{{ msg(item) }}
            </div>

            <!-- error -->
            <div v-else-if="item.kind === 'error'" class="min-w-0 flex-1 break-words text-red-600 dark:text-red-400">
              <CircleAlert :size="13" class="mr-1 inline" />{{ msg(item) }}
            </div>

            <!-- system -->
            <div v-else class="min-w-0 flex-1 break-words text-muted">
              <Info :size="13" class="mr-1 inline" />{{ msg(item) }}
            </div>
          </div>
          </div>
        </div>
      </section>

      <!-- Side -->
      <aside class="flex min-h-0 flex-col gap-4 overflow-y-auto">
        <!-- Stats -->
        <div v-if="s" class="card-pad grid grid-cols-2 gap-3">
          <div class="col-span-2">
            <div class="flex items-center justify-between text-xs text-muted">
              <span class="flex items-center gap-1"><Clock :size="13" /> {{ t("dashboard.nextMessage") }}</span>
              <span class="font-semibold text-fg tabular-nums">
                <template v-if="s.pausedReason">{{ t(`dashboard.paused.${s.pausedReason}`) }}</template>
                <template v-else-if="nextIn">{{ nextIn }}</template>
                <template v-else>—</template>
              </span>
            </div>
          </div>
          <div class="col-span-2">
            <div class="flex items-center justify-between text-xs text-muted">
              <span class="flex items-center gap-1"><Coins :size="13" /> {{ t("dashboard.costToday") }}</span>
              <span class="font-semibold text-fg tabular-nums">${{ s.costToday.toFixed(3) }}<template v-if="s.budget"> / ${{ s.budget.toFixed(2) }}</template></span>
            </div>
            <div v-if="s.budget" class="mt-1.5 h-1.5 overflow-hidden rounded-full bg-panel-2">
              <div class="h-full transition-all" :class="budgetPct > 90 ? 'bg-red-500' : 'bg-brand-500'" :style="{ width: budgetPct + '%' }" />
            </div>
          </div>
          <div class="rounded-lg bg-panel-2 p-2.5">
            <div class="flex items-center gap-1 text-[11px] text-muted"><Send :size="12" /> {{ t("dashboard.messagesToday") }}</div>
            <div class="text-lg font-semibold tabular-nums">{{ s.messagesToday }}</div>
          </div>
          <div class="rounded-lg bg-panel-2 p-2.5">
            <div class="flex items-center gap-1 text-[11px] text-muted"><Activity :size="12" /> {{ t("dashboard.chatRate") }}</div>
            <div class="text-lg font-semibold tabular-nums">{{ s.chatRate }}</div>
          </div>
          <div class="rounded-lg bg-panel-2 p-2.5">
            <div class="flex items-center gap-1 text-[11px] text-muted"><Smile :size="12" /> {{ t("dashboard.emotes") }}</div>
            <div class="text-lg font-semibold tabular-nums">{{ s.emoteCount }}</div>
          </div>
          <div class="rounded-lg bg-panel-2 p-2.5">
            <div class="flex items-center gap-1 text-[11px] text-muted"><Brain :size="12" /> {{ t("dashboard.knowledge") }}</div>
            <div class="text-lg font-semibold tabular-nums">{{ s.researching ? "…" : s.knowledgeItems }}</div>
          </div>
        </div>

        <!-- Quick controls -->
        <div v-if="channel" class="card-pad space-y-3">
          <h3 class="section-title mb-1 flex items-center gap-2"><Bolt :size="16" class="text-brand-500" /> {{ t("dashboard.quick") }}</h3>
          <UiToggle v-model="channel.companionEnabled" :label="t('dashboard.companionOn')" />
          <UiToggle v-model="channel.moderationEnabled" :label="t('dashboard.moderationOn')" />
          <div>
            <span class="label">{{ t("channels.sendMode") }}</span>
            <select v-model="channel.sendMode" class="input">
              <option value="auto">{{ t("channels.sendModes.auto") }}</option>
              <option value="preview">{{ t("channels.sendModes.preview") }}</option>
              <option value="approve">{{ t("channels.sendModes.approve") }}</option>
            </select>
          </div>
        </div>

        <!-- Pending -->
        <div v-if="app.pending.length" class="card-pad space-y-2">
          <h3 class="section-title mb-1 flex items-center gap-2"><Send :size="16" class="text-brand-500" /> {{ t("dashboard.pending") }} ({{ app.pending.length }})</h3>
          <div v-for="p in app.pending" :key="p.id" class="rounded-lg border border-line bg-panel-2 p-2.5">
            <textarea v-if="editing === p.id" v-model="editText" rows="3" class="input mb-2" />
            <p v-else class="selectable mb-2 text-sm"><EmoteText :text="p.text" /></p>
            <div class="flex flex-wrap items-center gap-1.5">
              <span class="chip bg-brand-500/12 text-brand-600 dark:text-brand-300">{{ t(`dashboard.reasons.${p.reason}`) }}</span>
              <span class="text-xs text-muted">{{ pendingIn(p) !== null ? t("dashboard.sendsIn", { s: pendingIn(p) }) : t("dashboard.needsApproval") }}</span>
              <span class="ml-auto flex gap-1">
                <template v-if="editing === p.id">
                  <button class="btn py-0.5 text-xs" @click="saveEdit(p)">{{ t("common.save") }}</button>
                </template>
                <template v-else>
                  <button class="btn-ghost p-1" :title="t('common.edit')" @click="startEdit(p)"><Pencil :size="14" /></button>
                  <button class="btn-ghost p-1" :title="t('common.discard')" @click="app.control('reject', { id: p.id })"><X :size="14" /></button>
                  <button class="btn-primary py-0.5 text-xs" @click="app.control('approve', { id: p.id })"><Send :size="13" /> {{ t("common.send") }}</button>
                </template>
              </span>
            </div>
          </div>
        </div>

        <!-- Simulator -->
        <div v-if="app.isRunning" class="card-pad space-y-2.5">
          <h3 class="section-title mb-0 flex items-center gap-2"><FlaskConical :size="16" class="text-brand-500" /> {{ t("dashboard.simulator") }}</h3>
          <p class="hint mt-0">{{ t("dashboard.simulatorHint") }}</p>
          <div class="grid grid-cols-2 gap-2">
            <input v-model="sim.user" class="input" :placeholder="t('dashboard.simUser')" />
            <select v-model="sim.role" class="input">
              <option value="">{{ t("roles.viewer") }}</option>
              <option value="sub">{{ t("roles.sub") }}</option>
              <option value="vip">VIP</option>
              <option value="mod">{{ t("roles.mod") }}</option>
              <option value="broadcaster">Streamer</option>
            </select>
          </div>
          <div class="flex gap-2">
            <input v-model="sim.text" class="input" :placeholder="t('dashboard.simText')" @keydown.enter="simulate" />
            <button class="btn-primary" :disabled="!sim.text.trim()" @click="simulate"><Send :size="15" /></button>
          </div>
          <div>
            <span class="label">{{ t("dashboard.simEvents") }}</span>
            <div class="flex flex-wrap gap-1.5">
              <button class="btn py-1 text-xs" @click="app.control('simulateEvent', { kind: 'follow', user: sim.user })"><Heart :size="13" /> {{ t("dashboard.follow") }}</button>
              <button class="btn py-1 text-xs" @click="app.control('simulateEvent', { kind: 'sub', user: sim.user })"><Star :size="13" /> {{ t("dashboard.sub") }}</button>
              <button class="btn py-1 text-xs" @click="app.control('simulateEvent', { kind: 'resub', user: sim.user })"><Star :size="13" /> {{ t("dashboard.resub") }}</button>
              <button class="btn py-1 text-xs" @click="app.control('simulateEvent', { kind: 'gift', user: sim.user })"><Gift :size="13" /> {{ t("dashboard.gift") }}</button>
              <button class="btn py-1 text-xs" @click="app.control('simulateEvent', { kind: 'raid', user: sim.user })"><Swords :size="13" /> {{ t("dashboard.raid") }}</button>
            </div>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>
