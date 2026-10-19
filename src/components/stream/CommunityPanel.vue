<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Check, Incognito, Plus, RefreshCw, Search, Train, Undo2, Users, UserTime, X } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiSegmented from "../ui/UiSegmented.vue";

const { t, locale } = useI18n();
const { app, id, tw, userByLogin } = useTwitch();

interface Person {
  user_id: string;
  user_login?: string;
  user_name: string;
  expires_at?: string;
  reason?: string;
}
interface UnbanRequest {
  id: string;
  user_id: string;
  user_name: string;
  text: string;
  created_at: string;
}
interface HypeTrain {
  current: { level: number; total: number; goal: number; expires_at: string } | null;
  all_time_high: { level: number; total: number } | null;
}
type Tab = "mods" | "vips" | "banned" | "requests" | "watch" | "stats";
const tab = ref<Tab>("mods");
const tabs = computed(() =>
  (["mods", "vips", "banned", "requests", "watch", "stats"] as const).map((v) => ({
    value: v,
    label: v === "requests" && requests.value.length ? `${t(`stream.community.${v}`)} (${requests.value.length})` : t(`stream.community.${v}`),
  })),
);
const mods = ref<Person[]>([]);
const vips = ref<Person[]>([]);
const banned = ref<Person[]>([]);
const requests = ref<UnbanRequest[]>([]);
const editors = ref<Person[]>([]);
const followers = ref<number | null>(null);
const subs = ref<{ total: number; points: number } | null>(null);
const hype = ref<HypeTrain | null>(null);
const login = ref("");
const filter = ref("");
const busy = ref(false);
const watchLogin = ref("");
const watchStatus = ref<"ACTIVE_MONITORING" | "RESTRICTED">("ACTIVE_MONITORING");
const resolution = ref<Record<string, string>>({});

/** Follows Helix pagination (max. `limit` entries) so big channels see everyone. */
async function all<T>(path: string, query: Record<string, string | number>, limit = 1000): Promise<T[]> {
  const out: T[] = [];
  let after: string | undefined;
  do {
    const r = await tw<{ data: T[]; pagination?: { cursor?: string } }>("GET", path, { ...query, first: 100, after }, undefined, true);
    if (!r) break;
    out.push(...(r.data ?? []));
    after = r.pagination?.cursor || undefined;
  } while (after && out.length < limit);
  return out;
}

async function load() {
  const b = { broadcaster_id: id.value };
  const bm = { broadcaster_id: id.value, moderator_id: id.value };
  const [m, v, bn, rq, f, s, e, h] = await Promise.all([
    all<Person>("/moderation/moderators", b),
    all<Person>("/channels/vips", b),
    all<Person>("/moderation/banned", b),
    all<UnbanRequest>("/moderation/unban_requests", { ...bm, status: "pending" }),
    tw<{ total: number }>("GET", "/channels/followers", { broadcaster_id: id.value, first: 1 }, undefined, true),
    tw<{ total: number; points: number }>("GET", "/subscriptions", { broadcaster_id: id.value, first: 1 }, undefined, true),
    tw<{ data: Person[] }>("GET", "/channels/editors", b, undefined, true),
    tw<{ data: HypeTrain[] }>("GET", "/hypetrain/status", b, undefined, true),
  ]);
  mods.value = m;
  vips.value = v;
  banned.value = bn;
  requests.value = rq;
  followers.value = f?.total ?? null;
  subs.value = s ? { total: s.total, points: s.points } : null;
  editors.value = e?.data ?? [];
  hype.value = h?.data?.[0] ?? null;
}

const path = computed(() => (tab.value === "mods" ? "/moderation/moderators" : "/channels/vips"));

async function add() {
  busy.value = true;
  const u = await userByLogin(login.value);
  if (u) {
    const r = await tw("POST", path.value, { broadcaster_id: id.value, user_id: u.id });
    if (r !== null) {
      app.toast(t("stream.community.added", { name: u.display_name }), "success");
      login.value = "";
      await load();
    }
  }
  busy.value = false;
}

async function remove(p: Person) {
  if (!confirm(t("stream.community.removeConfirm", { name: p.user_name }))) return;
  const r = await tw("DELETE", path.value, { broadcaster_id: id.value, user_id: p.user_id });
  if (r !== null) await load();
}

async function unban(p: Person) {
  const r = await tw("DELETE", "/moderation/bans", { broadcaster_id: id.value, moderator_id: id.value, user_id: p.user_id });
  if (r !== null) {
    app.toast(t("stream.community.unbanned", { name: p.user_name }), "success");
    await load();
  }
}

async function resolve(req: UnbanRequest, status: "approved" | "denied") {
  const r = await tw("PATCH", "/moderation/unban_requests", {
    broadcaster_id: id.value,
    moderator_id: id.value,
    unban_request_id: req.id,
    status,
    resolution_text: (resolution.value[req.id] ?? "").trim().slice(0, 500) || undefined,
  });
  if (r !== null) {
    app.toast(t(`stream.community.${status}`, { name: req.user_name }), "success");
    await load();
  }
}

async function setWatch(remove: boolean) {
  busy.value = true;
  const u = await userByLogin(watchLogin.value);
  if (u) {
    const q = { broadcaster_id: id.value, moderator_id: id.value };
    const r = remove
      ? await tw("DELETE", "/moderation/suspicious_users", { ...q, user_id: u.id })
      : await tw("POST", "/moderation/suspicious_users", q, { user_id: u.id, status: watchStatus.value });
    if (r !== null) {
      app.toast(t(remove ? "stream.community.watchRemoved" : "stream.community.watchSet", { name: u.display_name }), "success");
      watchLogin.value = "";
    }
  }
  busy.value = false;
}

const list = computed(() => {
  const l = tab.value === "mods" ? mods.value : tab.value === "vips" ? vips.value : banned.value;
  const f = filter.value.trim().toLowerCase();
  return f ? l.filter((p) => p.user_name.toLowerCase().includes(f) || (p.user_login ?? "").includes(f)) : l;
});
const fmt = (s?: string) => (s ? new Date(s).toLocaleString(locale.value, { dateStyle: "short", timeStyle: "short" }) : "");

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-4">
    <div class="flex flex-wrap items-center gap-2">
      <h2 class="section-title mb-0 flex flex-1 items-center gap-2"><Users :size="17" class="text-brand-500" /> {{ t("stream.community.title") }}</h2>
      <button class="btn" :title="t('stream.reload')" @click="load"><RefreshCw :size="15" /></button>
    </div>
    <UiSegmented v-model="tab" :options="tabs" />

    <!-- Numbers -->
    <div v-if="tab === 'stats'" class="grid grid-cols-2 gap-3 text-sm">
      <div class="rounded-lg bg-panel-2 p-3">
        <div class="text-xs text-muted">{{ t("stream.community.followers") }}</div>
        <div class="text-lg font-semibold tabular-nums">{{ followers?.toLocaleString(locale) ?? "—" }}</div>
      </div>
      <div class="rounded-lg bg-panel-2 p-3">
        <div class="text-xs text-muted">{{ t("stream.community.subs") }}</div>
        <div class="text-lg font-semibold tabular-nums">{{ subs ? `${subs.total} · ${subs.points} P` : "—" }}</div>
      </div>
      <div class="col-span-2 rounded-lg bg-panel-2 p-3">
        <div class="mb-1 flex items-center gap-1.5 text-xs text-muted"><Train :size="13" /> {{ t("stream.community.hypeTrain") }}</div>
        <div v-if="hype?.current">
          {{ t("stream.community.hypeActive", { level: hype.current.level, pct: Math.round((hype.current.total / Math.max(1, hype.current.goal)) * 100) }) }}
        </div>
        <div v-else>{{ t("stream.community.hypeNone") }}</div>
        <div v-if="hype?.all_time_high" class="text-xs text-muted">{{ t("stream.community.hypeRecord", { level: hype.all_time_high.level }) }}</div>
      </div>
      <div class="col-span-2 rounded-lg bg-panel-2 p-3">
        <div class="mb-1 text-xs text-muted">{{ t("stream.community.editors") }}</div>
        <div>{{ editors.map((e) => e.user_name).join(", ") || "—" }}</div>
      </div>
    </div>

    <!-- Unban requests -->
    <template v-else-if="tab === 'requests'">
      <p v-if="!requests.length" class="text-sm text-muted">{{ t("stream.community.noRequests") }}</p>
      <div v-for="r in requests" :key="r.id" class="space-y-2 rounded-lg border border-line bg-panel-2 p-3 text-sm">
        <div class="flex items-center gap-2">
          <UserTime :size="15" class="text-brand-500" />
          <span class="font-semibold">{{ r.user_name }}</span>
          <span class="text-xs text-muted">{{ fmt(r.created_at) }}</span>
        </div>
        <p class="selectable break-words italic">“{{ r.text }}”</p>
        <input v-model="resolution[r.id]" maxlength="500" class="input" :placeholder="t('stream.community.resolutionPh')" />
        <div class="flex gap-2">
          <button class="btn" @click="resolve(r, 'approved')"><Check :size="14" /> {{ t("stream.community.approve") }}</button>
          <button class="btn-ghost" @click="resolve(r, 'denied')"><X :size="14" /> {{ t("stream.community.deny") }}</button>
        </div>
      </div>
    </template>

    <!-- Suspicious users -->
    <template v-else-if="tab === 'watch'">
      <p class="hint mt-0">{{ t("stream.community.watchHint") }}</p>
      <input v-model="watchLogin" class="input" :placeholder="t('stream.loginPh')" />
      <div class="flex flex-wrap items-center gap-2">
        <select v-model="watchStatus" class="input w-auto">
          <option value="ACTIVE_MONITORING">{{ t("stream.community.monitor") }}</option>
          <option value="RESTRICTED">{{ t("stream.community.restrict") }}</option>
        </select>
        <button class="btn" :disabled="!watchLogin.trim() || busy" @click="setWatch(false)"><Incognito :size="15" /> {{ t("stream.community.watchApply") }}</button>
        <button class="btn-ghost" :disabled="!watchLogin.trim() || busy" @click="setWatch(true)"><X :size="14" /> {{ t("stream.community.watchClear") }}</button>
      </div>
    </template>

    <!-- Mods / VIPs / banned -->
    <template v-else>
      <div v-if="tab !== 'banned'" class="flex gap-2">
        <input v-model="login" class="input" :placeholder="t('stream.loginPh')" @keydown.enter="add" />
        <button class="btn shrink-0" :disabled="!login.trim() || busy" @click="add"><Plus :size="15" /> {{ t("common.add") }}</button>
      </div>
      <div v-else-if="banned.length > 8" class="relative">
        <Search :size="14" class="absolute top-2.5 left-2.5 text-muted" />
        <input v-model="filter" class="input pl-8" :placeholder="t('stream.community.filterPh')" />
      </div>
      <p v-if="!list.length" class="text-sm text-muted">{{ t("stream.empty") }}</p>
      <ul v-else class="max-h-72 divide-y divide-line overflow-y-auto rounded-lg border border-line">
        <li v-for="p in list" :key="p.user_id" class="flex items-center gap-2 px-3 py-1.5 text-sm">
          <span class="min-w-0 flex-1 truncate">
            <span class="font-medium">{{ p.user_name }}</span>
            <span v-if="tab === 'banned'" class="text-xs text-muted">
              · {{ p.expires_at ? t("stream.community.until", { d: fmt(p.expires_at) }) : t("stream.community.permanent") }}<template v-if="p.reason"> · {{ p.reason }}</template>
            </span>
          </span>
          <button v-if="tab === 'banned'" class="btn-ghost py-0.5 text-xs" @click="unban(p)"><Undo2 :size="13" /> {{ t("stream.community.unban") }}</button>
          <button v-else class="btn-ghost py-0.5" :title="t('common.remove')" @click="remove(p)"><X :size="14" /></button>
        </li>
      </ul>
      <p v-if="tab === 'banned' && banned.length" class="hint mt-0">{{ t("stream.community.count", { n: banned.length }) }}</p>
    </template>
  </section>
</template>
