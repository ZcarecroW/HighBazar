<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Plus, RefreshCw, X, Undo2 } from "@lucide/vue";
import { useTwitch } from "../../lib/twitch";
import UiSegmented from "../ui/UiSegmented.vue";

const { t } = useI18n();
const { app, id, tw, userByLogin } = useTwitch();

interface Person {
  user_id: string;
  user_login?: string;
  user_name: string;
  expires_at?: string;
  reason?: string;
}
type Tab = "mods" | "vips" | "banned" | "stats";
const tab = ref<Tab>("mods");
const tabs = computed(() => (["mods", "vips", "banned", "stats"] as const).map((v) => ({ value: v, label: t(`stream.community.${v}`) })));
const mods = ref<Person[]>([]);
const vips = ref<Person[]>([]);
const banned = ref<Person[]>([]);
const editors = ref<Person[]>([]);
const followers = ref<number | null>(null);
const subs = ref<{ total: number; points: number } | null>(null);
const login = ref("");
const busy = ref(false);

async function load() {
  const b = { broadcaster_id: id.value, first: 100 };
  const [m, v, bn, f, s, e] = await Promise.all([
    tw<{ data: Person[] }>("GET", "/moderation/moderators", b, undefined, true),
    tw<{ data: Person[] }>("GET", "/channels/vips", b, undefined, true),
    tw<{ data: Person[] }>("GET", "/moderation/banned", b, undefined, true),
    tw<{ total: number }>("GET", "/channels/followers", { broadcaster_id: id.value, first: 1 }, undefined, true),
    tw<{ total: number; points: number }>("GET", "/subscriptions", { broadcaster_id: id.value, first: 1 }, undefined, true),
    tw<{ data: Person[] }>("GET", "/channels/editors", { broadcaster_id: id.value }, undefined, true),
  ]);
  mods.value = m?.data ?? [];
  vips.value = v?.data ?? [];
  banned.value = bn?.data ?? [];
  followers.value = f?.total ?? null;
  subs.value = s ? { total: s.total, points: s.points } : null;
  editors.value = e?.data ?? [];
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

const list = computed(() => (tab.value === "mods" ? mods.value : tab.value === "vips" ? vips.value : banned.value));
const fmt = (s?: string) => (s ? new Date(s).toLocaleString() : "");

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-4">
    <div class="flex flex-wrap items-center gap-2">
      <h2 class="section-title mb-0 flex-1">{{ t("stream.community.title") }}</h2>
      <button class="btn" :title="t('stream.reload')" @click="load"><RefreshCw :size="15" /></button>
    </div>
    <UiSegmented v-model="tab" :options="tabs" />

    <div v-if="tab === 'stats'" class="grid grid-cols-2 gap-3 text-sm">
      <div class="rounded-lg bg-panel-2 p-3">
        <div class="text-xs text-muted">{{ t("stream.community.followers") }}</div>
        <div class="text-lg font-semibold tabular-nums">{{ followers ?? "—" }}</div>
      </div>
      <div class="rounded-lg bg-panel-2 p-3">
        <div class="text-xs text-muted">{{ t("stream.community.subs") }}</div>
        <div class="text-lg font-semibold tabular-nums">{{ subs ? `${subs.total} · ${subs.points} P` : "—" }}</div>
      </div>
      <div class="col-span-2 rounded-lg bg-panel-2 p-3">
        <div class="mb-1 text-xs text-muted">{{ t("stream.community.editors") }}</div>
        <div>{{ editors.map((e) => e.user_name).join(", ") || "—" }}</div>
      </div>
    </div>

    <template v-else>
      <div v-if="tab !== 'banned'" class="flex gap-2">
        <input v-model="login" class="input" :placeholder="t('stream.loginPh')" @keydown.enter="add" />
        <button class="btn shrink-0" :disabled="!login.trim() || busy" @click="add"><Plus :size="15" /> {{ t("common.add") }}</button>
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
    </template>
  </section>
</template>
