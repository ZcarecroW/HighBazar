<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Check, Search } from "../../icons";
import { fuzzySearch } from "../../lib/fuzzy";
import FuzzyText from "./FuzzyText.vue";
import { useCloseOutside } from "../../lib/outside";

// A select with a fuzzy search field – for lists longer than a handful of entries.
export interface ComboOption {
  value: string;
  label: string;
  /** Second line or right-hand hint. */
  sub?: string;
  /** Extra words that should find this entry (not shown). */
  keywords?: string;
  /** Small badge, e.g. "Standard". */
  badge?: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    options: ComboOption[];
    placeholder?: string;
    searchPlaceholder?: string;
    disabled?: boolean;
    /** Accessible name when there is no visible label. */
    label?: string;
    /** Show `sub` under the label instead of on the right. */
    stacked?: boolean;
  }>(),
  { placeholder: "", searchPlaceholder: "", disabled: false, label: "", stacked: false },
);
const model = defineModel<string>({ required: true });
const emit = defineEmits<{ picked: [option: ComboOption] }>();
const { t } = useI18n();

const open = ref(false);
const filter = ref("");
const active = ref(0);
const root = ref<HTMLElement>();
const input = ref<HTMLInputElement>();
const listEl = ref<HTMLElement>();
const uid = `cb-${Math.random().toString(36).slice(2, 8)}`;

const current = computed(() => props.options.find((o) => o.value === model.value));
const rows = computed(() => fuzzySearch(props.options, filter.value, (o) => [o.label, o.sub, o.keywords]));

async function toggle() {
  if (props.disabled) return;
  open.value = !open.value;
  if (!open.value) return;
  filter.value = "";
  active.value = Math.max(0, rows.value.findIndex((r) => r.item.value === model.value));
  await nextTick();
  input.value?.focus();
  scrollActive();
}
function pick(o: ComboOption) {
  if (o.disabled) return;
  model.value = o.value;
  open.value = false;
  emit("picked", o);
}
function scrollActive() {
  void nextTick(() => listEl.value?.querySelector<HTMLElement>(`[data-i="${active.value}"]`)?.scrollIntoView({ block: "nearest" }));
}
function onKey(e: KeyboardEvent) {
  const n = rows.value.length;
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    if (n) active.value = (active.value + (e.key === "ArrowDown" ? 1 : n - 1)) % n;
    scrollActive();
  } else if (e.key === "Home" || e.key === "End") {
    e.preventDefault();
    active.value = e.key === "Home" ? 0 : Math.max(0, n - 1);
    scrollActive();
  } else if (e.key === "Enter") {
    e.preventDefault();
    const r = rows.value[active.value];
    if (r) pick(r.item);
  } else if (e.key === "Escape" || e.key === "Tab") {
    open.value = false;
  }
}
watch(filter, () => (active.value = 0));

useCloseOutside(root, open);
</script>

<template>
  <div ref="root" class="relative">
    <button
      type="button"
      class="input flex min-w-0 items-center gap-2 text-left"
      :class="disabled && 'cursor-not-allowed opacity-70'"
      :disabled="disabled"
      :aria-expanded="open"
      :aria-controls="uid"
      :aria-label="label || undefined"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <span class="min-w-0 flex-1 truncate" :class="!current && 'text-muted'">{{ current?.label ?? (model || placeholder) }}</span>
      <span v-if="current?.badge" class="chip shrink-0 bg-brand-500/15 text-brand-600 dark:text-brand-400">{{ current.badge }}</span>
      <svg class="h-3.5 w-3.5 shrink-0 text-muted" viewBox="0 0 12 12" aria-hidden="true"><path d="M3 4.5 6 7.5 9 4.5" fill="none" stroke="currentColor" stroke-width="1.5" /></svg>
    </button>

    <div v-if="open" class="card absolute z-30 mt-1 w-full min-w-56 overflow-hidden shadow-xl">
      <div class="flex items-center gap-2 border-b border-line px-3 py-2">
        <Search :size="14" class="shrink-0 text-muted" />
        <input
          ref="input"
          v-model="filter"
          class="min-w-0 flex-1 bg-transparent text-sm outline-none"
          :placeholder="searchPlaceholder || t('common.search')"
          spellcheck="false"
          role="combobox"
          aria-autocomplete="list"
          :aria-controls="uid"
          :aria-activedescendant="rows.length ? `${uid}-${active}` : undefined"
          @keydown="onKey"
        />
      </div>
      <ul :id="uid" ref="listEl" role="listbox" class="max-h-72 overflow-y-auto py-1">
        <li
          v-for="(r, i) in rows"
          :id="`${uid}-${i}`"
          :key="r.item.value"
          :data-i="i"
          role="option"
          :aria-selected="r.item.value === model"
          :aria-disabled="r.item.disabled || undefined"
          class="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-sm"
          :class="[i === active ? 'bg-brand-500/12' : '', r.item.disabled && 'cursor-not-allowed opacity-50']"
          @mouseenter="active = i"
          @click="pick(r.item)"
        >
          <Check :size="14" class="shrink-0 text-brand-500" :class="r.item.value === model ? '' : 'invisible'" />
          <span v-if="stacked" class="min-w-0 flex-1">
            <span class="block truncate"><FuzzyText :text="r.item.label" :ranges="r.ranges[0]" /></span>
            <span v-if="r.item.sub" class="block truncate text-xs text-muted"><FuzzyText :text="r.item.sub" :ranges="r.ranges[1]" /></span>
          </span>
          <template v-else>
            <span class="min-w-0 flex-1 truncate"><FuzzyText :text="r.item.label" :ranges="r.ranges[0]" /></span>
            <span v-if="r.item.sub" class="max-w-[50%] shrink-0 truncate text-xs text-muted"><FuzzyText :text="r.item.sub" :ranges="r.ranges[1]" /></span>
          </template>
          <span v-if="r.item.badge" class="chip shrink-0 bg-brand-500/15 text-brand-600 dark:text-brand-400">{{ r.item.badge }}</span>
        </li>
        <li v-if="!rows.length" class="px-3 py-3 text-sm text-muted">{{ t("common.noMatches") }}</li>
      </ul>
    </div>
  </div>
</template>
