<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Copy, Lock, Plus, Trash2, CircleCheck, Search } from "../icons";
import { pdesc, pname } from "../i18n";
import { fuzzySearch } from "../lib/fuzzy";
import FuzzyText from "./ui/FuzzyText.vue";
import UiCombobox from "./ui/UiCombobox.vue";

interface Item {
  id: string;
  name: string;
  builtin?: boolean;
  description?: string;
}

const props = defineProps<{ items: Item[]; activeId?: string; allowNew?: boolean }>();
const selected = defineModel<string>({ required: true });
const emit = defineEmits<{ new: []; duplicate: [id: string]; delete: [id: string] }>();
const { t } = useI18n();

const filter = ref("");
const results = computed(() => fuzzySearch(props.items, filter.value, (i) => [pname(i), pdesc(i)]));
const options = computed(() =>
  props.items.map((i) => ({
    value: i.id,
    label: pname(i),
    sub: pdesc(i),
    badge: i.id === props.activeId ? t("common.active") : i.builtin ? t("common.builtin") : undefined,
  })),
);
</script>

<template>
  <div>
    <!-- Narrow screens: compact select -->
    <div class="flex items-center gap-2 lg:hidden">
      <UiCombobox v-model="selected" class="min-w-0 flex-1" :options="options" :label="t('common.profiles')" stacked />
      <button class="btn" :title="t('common.duplicate')" @click="emit('duplicate', selected)"><Copy :size="16" /></button>
      <button v-if="allowNew !== false" class="btn" :title="t('common.new')" @click="emit('new')"><Plus :size="16" /></button>
    </div>

    <!-- Wide screens: full list -->
    <div class="card hidden overflow-hidden lg:block">
      <div class="flex items-center justify-between border-b border-line px-3 py-2">
        <span class="label mb-0">{{ t("common.profiles") }}</span>
        <button v-if="allowNew !== false" class="btn-ghost" :title="t('common.new')" @click="emit('new')"><Plus :size="16" /></button>
      </div>
      <div v-if="items.length > 6" class="relative border-b border-line">
        <Search :size="13" class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-muted" />
        <input
          v-model="filter"
          class="w-full bg-transparent py-2 pr-3 pl-8 text-sm outline-none"
          :placeholder="t('common.search')"
          :aria-label="t('common.search')"
          spellcheck="false"
          @keydown.esc="filter = ''"
          @keydown.enter="results[0] && (selected = results[0].item.id)"
        />
      </div>
      <ul class="max-h-[calc(100vh-14rem)] overflow-y-auto py-1">
        <li v-if="!results.length" class="px-3 py-2 text-sm text-muted">{{ t("common.noMatches") }}</li>
        <li v-for="{ item: i, ranges } in results" :key="i.id">
          <div
            class="group flex cursor-pointer items-center gap-2 px-3 py-2 transition"
            :class="selected === i.id ? 'bg-brand-500/12 text-brand-700 dark:text-brand-200' : 'hover:bg-panel-2'"
            @click="selected = i.id"
          >
            <Lock v-if="i.builtin" :size="13" class="shrink-0 text-muted" />
            <span class="min-w-0 flex-1 truncate text-sm font-medium"><FuzzyText :text="pname(i)" :ranges="ranges[0]" /></span>
            <CircleCheck v-if="i.id === activeId" :size="15" class="shrink-0 text-emerald-500" :title="t('common.active')" />
            <span class="hidden gap-0.5 group-hover:flex">
              <button class="rounded p-1 text-muted hover:text-fg" :title="t('common.duplicate')" @click.stop="emit('duplicate', i.id)"><Copy :size="13" /></button>
              <button v-if="!i.builtin" class="rounded p-1 text-muted hover:text-red-500" :title="t('common.delete')" @click.stop="emit('delete', i.id)"><Trash2 :size="13" /></button>
            </span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>
