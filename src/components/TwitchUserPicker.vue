<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Eye, Loader2, Search } from "../icons";
import { api } from "../lib/api";
import { fuzzySearch, type Ranges } from "../lib/fuzzy";
import { useTwitch } from "../lib/twitch";
import FuzzyText from "./ui/FuzzyText.vue";
import { useCloseOutside } from "../lib/outside";

// Login field with suggestions while typing: live channels you follow, people
// in chat and Twitch's channel search – with avatar, title and category.
export interface PickedUser {
  id: string;
  login: string;
  displayName: string;
}
interface Suggestion extends PickedUser {
  avatar: string;
  live: boolean;
  viewers: number | null;
  title: string;
  game: string;
  group: "follow" | "chat" | "search";
}

const props = withDefaults(
  defineProps<{
    /** raid: live channels only; shoutout: channels; user: people (e.g. chatters for mod/VIP). */
    mode?: "raid" | "shoutout" | "user";
    placeholder?: string;
    disabled?: boolean;
    label?: string;
  }>(),
  { mode: "user", placeholder: "", disabled: false, label: "" },
);
const model = defineModel<string>({ required: true });
const emit = defineEmits<{ pick: [user: PickedUser]; submit: [] }>();
const { t, locale } = useI18n();
const { app, id, tw } = useTwitch();

const open = ref(false);
const active = ref(-1);
const loading = ref(false);
const root = ref<HTMLElement>();
const listEl = ref<HTMLElement>();
const uid = `up-${Math.random().toString(36).slice(2, 8)}`;

// --- sources -------------------------------------------------------------
const followed = ref<Suggestion[]>([]);
const chatters = ref<Suggestion[]>([]);
const searched = ref<Suggestion[]>([]);
let followedAt = 0;
let chattersAt = 0;

interface Stream { user_id: string; user_login: string; user_name: string; game_name: string; title: string; viewer_count: number }
interface Channel { id: string; broadcaster_login: string; display_name: string; thumbnail_url: string; title: string; game_name: string; is_live: boolean }

const avatars = new Map<string, string>();
async function fillAvatars(list: Suggestion[]) {
  const missing = list.filter((s) => s.id && !avatars.has(s.id)).map((s) => s.id).slice(0, 100);
  if (missing.length && id.value) {
    try {
      const r = await api.twitchApi<{ data: { id: string; profile_image_url: string }[] }>(id.value, "GET", "/users", missing.map((x) => ["id", x] as [string, string]));
      for (const u of r.data ?? []) avatars.set(u.id, u.profile_image_url);
    } catch {
      // Avatars are a nicety.
    }
  }
  for (const s of list) s.avatar ||= avatars.get(s.id) ?? "";
}

async function loadFollowed() {
  if (props.mode === "user" || !id.value || Date.now() - followedAt < 60_000) return;
  followedAt = Date.now();
  // Needs user:read:follows; without it, this list simply stays empty.
  const r = await tw<{ data: Stream[] }>("GET", "/streams/followed", { user_id: id.value, first: 100 }, undefined, true);
  const list: Suggestion[] = (r?.data ?? []).map((s) => ({
    id: s.user_id,
    login: s.user_login,
    displayName: s.user_name,
    avatar: "",
    live: true,
    viewers: s.viewer_count,
    title: s.title,
    game: s.game_name,
    group: "follow",
  }));
  await fillAvatars(list);
  followed.value = list;
}

function loadChatters() {
  if (props.mode === "raid" || Date.now() - chattersAt < 30_000) return;
  chattersAt = Date.now();
  // People who wrote recently, newest first …
  const seen = new Set<string>();
  const list: Suggestion[] = [];
  for (let i = app.feed.length - 1; i >= 0 && list.length < 200; i--) {
    const f = app.feed[i]!;
    if (f.kind !== "chat" || !f.login || seen.has(f.login)) continue;
    seen.add(f.login);
    list.push({ id: f.userId, login: f.login, displayName: f.user || f.login, avatar: "", live: false, viewers: null, title: "", game: "", group: "chat" });
  }
  chatters.value = list;
  // … plus everyone currently in chat (moderator:read:chatters).
  const ch = app.activeChannel;
  if (!id.value || !ch) return;
  void tw<{ data: { user_id: string; user_login: string; user_name: string }[] }>(
    "GET",
    "/chat/chatters",
    { broadcaster_id: id.value, moderator_id: id.value, first: 1000 },
    undefined,
    true,
  ).then((r) => {
    for (const u of r?.data ?? []) {
      if (seen.has(u.user_login)) continue;
      seen.add(u.user_login);
      list.push({ id: u.user_id, login: u.user_login, displayName: u.user_name, avatar: "", live: false, viewers: null, title: "", game: "", group: "chat" });
    }
    chatters.value = [...list];
  });
}

let seq = 0;
let timer = 0;
const cache = new Map<string, Suggestion[]>();
function searchLater(q: string) {
  clearTimeout(timer);
  const query = q.trim().replace(/^@/, "");
  if (query.length < 2 || !id.value) {
    searched.value = [];
    loading.value = false;
    return;
  }
  const key = `${props.mode}:${query.toLowerCase()}`;
  if (cache.has(key)) {
    searched.value = cache.get(key)!;
    return;
  }
  loading.value = true;
  timer = window.setTimeout(async () => {
    const mine = ++seq;
    const r = await tw<{ data: Channel[] }>("GET", "/search/channels", { query, first: 12, live_only: props.mode === "raid" }, undefined, true);
    const list: Suggestion[] = (r?.data ?? []).map((c) => ({
      id: c.id,
      login: c.broadcaster_login,
      displayName: c.display_name,
      avatar: c.thumbnail_url,
      live: c.is_live,
      viewers: null,
      title: c.title,
      game: c.game_name,
      group: "search",
    }));
    // Viewer numbers for the live ones.
    const live = list.filter((s) => s.live).map((s) => s.id);
    if (live.length) {
      try {
        const s = await api.twitchApi<{ data: Stream[] }>(id.value, "GET", "/streams", live.map((x) => ["user_id", x] as [string, string]));
        const v = new Map((s.data ?? []).map((x) => [x.user_id, x.viewer_count]));
        for (const x of list) x.viewers = v.get(x.id) ?? x.viewers;
      } catch {
        // Without numbers then.
      }
    }
    if (mine !== seq) return;
    cache.set(key, list);
    searched.value = list;
    loading.value = false;
  }, 250);
}

// --- merged, fuzzy-ranked list ---------------------------------------------
interface Row {
  s: Suggestion;
  ranges: Ranges[];
}
const rows = computed<Row[]>(() => {
  const q = model.value.trim().replace(/^@/, "");
  const keys = (s: Suggestion) => [s.displayName, s.login, s.game, s.title];
  const local = [...followed.value, ...chatters.value];
  const ranked = fuzzySearch(local, q, keys, { weights: [1, 1, 0.5, 0.35] });
  const out: Row[] = [];
  const seen = new Set<string>();
  const add = (s: Suggestion, ranges: Ranges[]) => {
    if (seen.has(s.login)) return;
    seen.add(s.login);
    out.push({ s, ranges });
  };
  const groupOrder = (g: Suggestion["group"]) => (g === "follow" ? 0 : g === "chat" ? 1 : 2);
  // Empty field: live follows by viewers, then recent chatters.
  const localRows = q
    ? ranked
    : [...ranked].sort((a, b) => groupOrder(a.item.group) - groupOrder(b.item.group) || (b.item.viewers ?? 0) - (a.item.viewers ?? 0));
  for (const r of localRows.slice(0, q ? 8 : 10)) add(r.item, r.ranges);
  if (q) {
    for (const r of fuzzySearch(searched.value, q, keys, { weights: [1, 1, 0.5, 0.35] })) add(r.item, r.ranges);
    // Twitch's own ranking may find things fuzzy matching does not (e.g. by tags).
    for (const s of searched.value) add(s, []);
  }
  return out.slice(0, 15);
});

function groupLabel(g: Suggestion["group"]) {
  return t(`picker.groups.${g}`);
}
const nf = computed(() => new Intl.NumberFormat(locale.value, { notation: "compact" }));

// --- interaction -------------------------------------------------------------
function show() {
  if (props.disabled) return;
  open.value = true;
  void loadFollowed();
  loadChatters();
}
function onInput(e: Event) {
  show();
  active.value = -1;
  searchLater((e.target as HTMLInputElement).value);
}
function pick(s: Suggestion) {
  model.value = s.login;
  open.value = false;
  emit("pick", { id: s.id, login: s.login, displayName: s.displayName });
}
function scrollActive() {
  void nextTick(() => listEl.value?.querySelector<HTMLElement>(`[data-i="${active.value}"]`)?.scrollIntoView({ block: "nearest" }));
}
function onKey(e: KeyboardEvent) {
  const n = rows.value.length;
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    if (!open.value) show();
    if (n) active.value = e.key === "ArrowDown" ? (active.value + 1) % n : active.value <= 0 ? n - 1 : active.value - 1;
    scrollActive();
  } else if (e.key === "Enter") {
    const r = active.value >= 0 ? rows.value[active.value] : undefined;
    if (open.value && r) {
      e.preventDefault();
      pick(r.s);
    } else {
      open.value = false;
      emit("submit");
    }
  } else if (e.key === "Escape") {
    open.value = false;
  } else if (e.key === "Tab") {
    open.value = false;
  }
}
useCloseOutside(root, open);
onBeforeUnmount(() => clearTimeout(timer));
</script>

<template>
  <div ref="root" class="relative min-w-0 flex-1">
    <div class="relative">
      <Search :size="14" class="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-muted" />
      <input
        v-model.trim="model"
        class="input pl-8"
        :placeholder="placeholder"
        :disabled="disabled"
        :aria-label="label || placeholder"
        role="combobox"
        aria-autocomplete="list"
        :aria-expanded="open && rows.length > 0"
        :aria-controls="uid"
        :aria-activedescendant="active >= 0 ? `${uid}-${active}` : undefined"
        autocomplete="off"
        spellcheck="false"
        @focus="show"
        @input="onInput"
        @keydown="onKey"
      />
      <Loader2 v-if="loading" :size="14" class="absolute top-1/2 right-2.5 -translate-y-1/2 animate-spin text-muted" />
    </div>

    <ul
      v-if="open && rows.length"
      :id="uid"
      ref="listEl"
      role="listbox"
      class="card absolute z-30 mt-1 max-h-80 w-full min-w-72 overflow-y-auto py-1 shadow-xl"
    >
      <template v-for="(r, i) in rows" :key="r.s.login">
        <li
          v-if="i === 0 || rows[i - 1]!.s.group !== r.s.group"
          role="presentation"
          class="px-3 pt-2 pb-1 text-[11px] font-semibold tracking-wide text-muted uppercase"
        >
          {{ groupLabel(r.s.group) }}
        </li>
        <li
          :id="`${uid}-${i}`"
          :data-i="i"
          role="option"
          :aria-selected="i === active"
          class="flex cursor-pointer items-center gap-2.5 px-3 py-1.5"
          :class="i === active ? 'bg-brand-500/12' : ''"
          @mouseenter="active = i"
          @mousedown.prevent
          @click="pick(r.s)"
        >
          <img v-if="r.s.avatar" :src="r.s.avatar" alt="" class="h-8 w-8 shrink-0 rounded-full bg-panel-2 object-cover" loading="lazy" />
          <span v-else class="grid h-8 w-8 shrink-0 place-items-center rounded-full bg-panel-2 text-xs font-bold text-muted" aria-hidden="true">{{ r.s.displayName.slice(0, 1).toUpperCase() }}</span>
          <span class="min-w-0 flex-1">
            <span class="flex items-center gap-1.5 text-sm">
              <span class="truncate font-semibold"><FuzzyText :text="r.s.displayName" :ranges="r.ranges[0]" /></span>
              <span v-if="r.s.login.toLowerCase() !== r.s.displayName.toLowerCase()" class="truncate text-xs text-muted"><FuzzyText :text="r.s.login" :ranges="r.ranges[1]" /></span>
              <span v-if="r.s.live" class="chip ml-auto shrink-0 bg-red-500/15 text-red-600 dark:text-red-400">
                {{ t("picker.live") }}<template v-if="r.s.viewers !== null"> · <Eye :size="11" class="inline" /> {{ nf.format(r.s.viewers) }}</template>
              </span>
            </span>
            <span v-if="r.s.title || r.s.game" class="block truncate text-xs text-muted">
              <FuzzyText v-if="r.s.game" :text="r.s.game" :ranges="r.ranges[2]" class="font-medium" /><template v-if="r.s.game && r.s.title"> · </template
              ><FuzzyText v-if="r.s.title" :text="r.s.title" :ranges="r.ranges[3]" />
            </span>
          </span>
        </li>
      </template>
    </ul>
  </div>
</template>
