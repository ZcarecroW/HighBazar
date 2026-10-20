<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Loader2, Send, Smile, Star } from "../icons";
import { useApp } from "../stores/app";
import { api, errorText } from "../lib/api";
import { activeToken, applySuggestion, enterPicks, providerLabel, type Suggestion } from "../lib/chatInput";
import { useCloseOutside } from "../lib/outside";

// Writing in chat yourself: ":" suggests emotes of every provider and emojis,
// "@" people in chat, Tab completes the current word, ↑ brings back what you sent.
// The ranking (favorites, your own use, chat's use) happens in the Rust core.
const app = useApp();
const { t, locale } = useI18n();

const text = ref("");
const input = ref<HTMLInputElement>();
const sending = ref(false);
const list = ref<Suggestion[]>([]);
const active = ref(0);
// The token the list belongs to; whether a suggestion was chosen with the arrow keys.
const token = ref<ReturnType<typeof activeToken>>(null);
let chosen = false;
// Why suggestions may be missing (subscription emotes need a newer Twitch login).
const status = ref<{ subEmotesMissing: boolean; emotes: number; people: number } | null>(null);
const history: string[] = [];
let historyPos = -1;
let tabCycle: { start: number; end: number; items: Suggestion[]; i: number } | null = null;

const running = computed(() => app.isRunning);
const sandbox = computed(() => !!app.running?.sandbox);
const german = computed(() => locale.value.startsWith("de"));
const tooLong = computed(() => [...text.value].length > 500);

// --- suggestions while typing ------------------------------------------------
let seq = 0;
async function refresh() {
  const el = input.value;
  if (!el || !running.value) return;
  const tok = activeToken(text.value, el.selectionStart ?? text.value.length);
  token.value = tok;
  if (!tok) {
    list.value = [];
    return;
  }
  const mine = ++seq;
  const [r, st] = await Promise.all([
    api.chatSuggest(tok.query, tok.mode, german.value, 8).catch(() => []),
    status.value ? Promise.resolve(status.value) : api.chatStatus().catch(() => null),
  ]);
  if (mine !== seq) return;
  status.value = st;
  list.value = r;
  active.value = 0;
  chosen = false;
}
// Shown instead of the list when nothing matches, or under it.
const emptyText = computed(() => {
  const tok = token.value;
  if (!tok || list.value.length) return "";
  if (tok.mode === "mention") return tok.query ? t("common.noMatches") : t("chatInput.nobodyYet");
  return tok.query ? t("common.noMatches") : "";
});
const showSubHint = computed(() => token.value?.mode === "colon" && !!status.value?.subEmotesMissing && (!!emptyText.value || /^[tT]( |$)/.test(token.value.query)));
function onInput() {
  tabCycle = null;
  historyPos = -1;
  void refresh();
}

function pick(s: Suggestion) {
  const el = input.value!;
  const caret = el.selectionStart ?? text.value.length;
  const tok = activeToken(text.value, caret);
  if (!tok) return;
  const r = applySuggestion(text.value, tok.start, caret, s);
  text.value = r.text;
  list.value = [];
  token.value = null;
  void nextTick(() => {
    el.focus();
    el.setSelectionRange(r.caret, r.caret);
  });
}

/** Tab on a plain word: completes emotes and names that start with it; Tab again = next one. */
async function tabComplete(back: boolean) {
  const el = input.value!;
  if (!tabCycle) {
    const caret = el.selectionStart ?? text.value.length;
    const before = text.value.slice(0, caret);
    const m = before.match(/(\S+)$/);
    if (!m) return;
    const word = m[1]!;
    const items = await api.chatSuggest(word.replace(/^@/, ""), "tab", german.value, 20).catch(() => []);
    if (!items.length) return;
    tabCycle = { start: caret - word.length, end: caret, items, i: back ? items.length - 1 : 0 };
  } else {
    const n = tabCycle.items.length;
    tabCycle.i = (tabCycle.i + (back ? n - 1 : 1)) % n;
  }
  const s = tabCycle.items[tabCycle.i]!;
  const ins = `${s.insert} `;
  text.value = text.value.slice(0, tabCycle.start) + ins + text.value.slice(tabCycle.end);
  tabCycle.end = tabCycle.start + ins.length;
  const caret = tabCycle.end;
  void nextTick(() => el.setSelectionRange(caret, caret));
}

function onKey(e: KeyboardEvent) {
  const open = list.value.length > 0;
  if (open && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
    e.preventDefault();
    const n = list.value.length;
    active.value = (active.value + (e.key === "ArrowDown" ? 1 : n - 1)) % n;
    chosen = true;
  } else if (open && (e.key === "Tab" || (e.key === "Enter" && token.value && enterPicks(token.value, chosen)))) {
    e.preventDefault();
    pick(list.value[active.value]!);
  } else if ((open || token.value) && e.key === "Escape") {
    e.preventDefault();
    list.value = [];
    token.value = null;
  } else if (e.key === "Tab") {
    e.preventDefault();
    void tabComplete(e.shiftKey);
  } else if (e.key === "Enter") {
    e.preventDefault();
    void send();
  } else if (e.key === "ArrowUp" && history.length && (!text.value || historyPos >= 0)) {
    e.preventDefault();
    historyPos = Math.min(historyPos + 1, history.length - 1);
    text.value = history[history.length - 1 - historyPos]!;
  } else if (e.key === "ArrowDown" && historyPos >= 0) {
    e.preventDefault();
    historyPos--;
    text.value = historyPos >= 0 ? history[history.length - 1 - historyPos]! : "";
  } else if (e.key !== "Shift") {
    tabCycle = null;
  }
}

async function send() {
  const msg = text.value.trim();
  if (!msg || sending.value || tooLong.value || !running.value) return;
  sending.value = true;
  try {
    await api.chatSend(msg);
    history.push(msg);
    if (history.length > 30) history.shift();
    text.value = "";
    list.value = [];
    token.value = null;
  } catch (e) {
    app.toast(errorText(e), "error", 6000);
  } finally {
    sending.value = false;
    historyPos = -1;
    void nextTick(() => input.value?.focus());
  }
}

// --- favorites -----------------------------------------------------------------
function toggleFavorite(s: Suggestion) {
  const favs = app.config!.settings.chatInput.favorites;
  const i = favs.indexOf(s.insert);
  if (i >= 0) favs.splice(i, 1);
  else favs.unshift(s.insert);
  s.favorite = i < 0;
}

// --- emote picker ----------------------------------------------------------------
const pickerOpen = ref(false);
const pickerTab = ref<"" | "7" | "b" | "f" | "t" | "e">("");
const pickerQuery = ref("");
const pickerItems = ref<Suggestion[]>([]);
const picker = ref<HTMLElement>();
useCloseOutside(picker, pickerOpen);
const TABS = computed(() => [
  { id: "", label: t("chatInput.tabs.mine") },
  { id: "7", label: "7TV" },
  { id: "b", label: "BTTV" },
  { id: "f", label: "FFZ" },
  { id: "t", label: "Twitch" },
  { id: "e", label: "Emoji" },
] as const);
async function loadPicker() {
  const q = pickerTab.value ? `${pickerTab.value} ${pickerQuery.value}` : pickerQuery.value;
  pickerItems.value = await api.chatSuggest(q.trim() || pickerTab.value, "browse", german.value, 200).catch(() => []);
}
watch([pickerTab, pickerQuery, pickerOpen], () => pickerOpen.value && void loadPicker());
function insertFromPicker(s: Suggestion) {
  const el = input.value!;
  const caret = el.selectionStart ?? text.value.length;
  const before = text.value.slice(0, caret);
  const pad = before && !before.endsWith(" ") ? " " : "";
  const ins = `${pad}${s.insert} `;
  text.value = before + ins + text.value.slice(caret);
  void nextTick(() => {
    el.focus();
    el.setSelectionRange(caret + ins.length, caret + ins.length);
  });
}
watch(running, (v) => {
  if (!v) {
    list.value = [];
    token.value = null;
    pickerOpen.value = false;
  } else {
    status.value = null;
  }
});
function reconnect() {
  token.value = null;
  app.goto("channels");
}
</script>

<template>
  <div class="relative border-t border-line p-2">
    <!-- suggestions -->
    <ul
      v-if="list.length"
      id="chat-suggest"
      role="listbox"
      class="card absolute right-2 bottom-full left-2 z-20 mb-1 max-h-72 overflow-y-auto py-1 shadow-xl"
    >
      <li
        v-for="(s, i) in list"
        :id="`chat-suggest-${i}`"
        :key="s.provider + s.insert"
        role="option"
        :aria-selected="i === active"
        class="flex cursor-pointer items-center gap-2.5 px-3 py-1"
        :class="i === active ? 'bg-brand-500/12' : ''"
        @mouseenter="active = i"
        @mousedown.prevent="pick(s)"
      >
        <img v-if="s.url" :src="s.url" alt="" class="h-7 w-7 shrink-0 object-contain" loading="lazy" />
        <span v-else-if="s.kind === 'emoji'" class="w-7 shrink-0 text-center text-xl leading-7">{{ s.insert }}</span>
        <span v-else class="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-panel-2 text-xs font-bold text-muted">@</span>
        <span class="min-w-0 flex-1 truncate text-sm">{{ s.kind === "emoji" ? s.label : s.insert }}</span>
        <span v-if="s.kind !== 'user'" class="chip shrink-0 bg-panel-2 text-[10px] uppercase">{{ providerLabel(s, t) }}</span>
        <button
          v-if="s.kind !== 'user'"
          type="button"
          class="shrink-0 rounded p-0.5"
          :class="s.favorite ? 'text-amber-500' : 'text-muted opacity-40 hover:opacity-100'"
          :title="s.favorite ? t('chatInput.unfavorite') : t('chatInput.favorite')"
          :aria-label="s.favorite ? t('chatInput.unfavorite') : t('chatInput.favorite')"
          @mousedown.prevent.stop="toggleFavorite(s)"
        >
          <Star :size="13" />
        </button>
      </li>
      <li v-if="showSubHint" class="border-t border-line px-3 py-1 text-xs text-amber-600" role="presentation">
        {{ t("chatInput.subsMissing") }} <button type="button" class="underline" @mousedown.prevent="reconnect">{{ t("chatInput.reconnect") }}</button>
      </li>
      <li class="border-t border-line px-3 pt-1 text-[10px] text-muted" role="presentation">{{ t("chatInput.suggestHint") }}</li>
    </ul>
    <div v-else-if="emptyText || showSubHint" class="card absolute right-2 bottom-full left-2 z-20 mb-1 space-y-0.5 px-3 py-1.5 text-xs shadow-xl" role="status">
      <p v-if="emptyText" class="text-muted">{{ emptyText }}</p>
      <p v-if="showSubHint" class="text-amber-600">
        {{ t("chatInput.subsMissing") }} <button type="button" class="underline" @mousedown.prevent="reconnect">{{ t("chatInput.reconnect") }}</button>
      </p>
    </div>

    <!-- emote picker -->
    <div v-if="pickerOpen" ref="picker" class="card absolute right-2 bottom-full z-20 mb-1 flex h-80 w-[min(26rem,calc(100%-1rem))] flex-col shadow-xl">
      <div class="flex gap-1 overflow-x-auto border-b border-line p-1.5">
        <button
          v-for="tb in TABS"
          :key="tb.id"
          type="button"
          class="rounded-md px-2 py-1 text-xs whitespace-nowrap"
          :class="pickerTab === tb.id ? 'bg-brand-500 text-white' : 'hover:bg-panel-2'"
          @click="pickerTab = tb.id"
        >
          {{ tb.label }}
        </button>
      </div>
      <input v-model="pickerQuery" class="mx-1.5 mt-1.5 rounded-md border border-line bg-panel-2 px-2 py-1 text-sm outline-none" :placeholder="t('common.search')" :aria-label="t('common.search')" spellcheck="false" />
      <div class="grid min-h-0 flex-1 grid-cols-[repeat(auto-fill,minmax(2.5rem,1fr))] content-start gap-1 overflow-y-auto p-1.5">
        <button
          v-for="s in pickerItems"
          :key="s.provider + s.insert"
          type="button"
          class="relative grid h-10 place-items-center rounded-md hover:bg-panel-2"
          :title="`${s.kind === 'emoji' ? s.label : s.insert} · ${providerLabel(s, t)}${s.favorite ? ' ★' : ''}\n${t('chatInput.rightClickFavorite')}`"
          @click="insertFromPicker(s)"
          @contextmenu.prevent="toggleFavorite(s)"
        >
          <img v-if="s.url" :src="s.url" :alt="s.insert" class="max-h-8 max-w-9 object-contain" loading="lazy" />
          <span v-else class="text-2xl leading-none">{{ s.insert }}</span>
          <Star v-if="s.favorite" :size="9" class="absolute top-0.5 right-0.5 text-amber-500" />
        </button>
        <p v-if="!pickerItems.length" class="col-span-full p-3 text-center text-xs text-muted">{{ pickerTab || pickerQuery ? t("common.noMatches") : t("chatInput.pickerEmpty") }}</p>
      </div>
    </div>

    <div class="flex items-center gap-1.5">
      <input
        ref="input"
        v-model="text"
        class="input min-w-0 flex-1 py-1.5"
        :class="tooLong && 'border-red-500'"
        :placeholder="!running ? t('chatInput.notRunning') : sandbox ? t('chatInput.placeholderTest') : t('chatInput.placeholder')"
        :disabled="!running"
        :aria-label="t('chatInput.label')"
        role="combobox"
        aria-autocomplete="list"
        aria-controls="chat-suggest"
        :aria-expanded="list.length > 0"
        :aria-activedescendant="list.length ? `chat-suggest-${active}` : undefined"
        maxlength="600"
        spellcheck="true"
        autocomplete="off"
        @input="onInput"
        @click="refresh"
        @keydown="onKey"
        @blur="list = []; token = null"
      />
      <span v-if="text.length > 400" class="shrink-0 text-xs tabular-nums" :class="tooLong ? 'text-red-500' : 'text-muted'">{{ [...text].length }}/500</span>
      <button type="button" class="btn-ghost shrink-0 p-1.5" :disabled="!running" :title="t('chatInput.emotes')" :aria-label="t('chatInput.emotes')" :aria-expanded="pickerOpen" @click.stop="pickerOpen = !pickerOpen">
        <Smile :size="17" />
      </button>
      <button type="button" class="btn-primary shrink-0 px-2.5 py-1.5" :disabled="!running || !text.trim() || sending || tooLong" :title="t('chatInput.send')" :aria-label="t('chatInput.send')" @click="send">
        <Loader2 v-if="sending" :size="15" class="animate-spin" />
        <Send v-else :size="15" />
      </button>
    </div>
  </div>
</template>
