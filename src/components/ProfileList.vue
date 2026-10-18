<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Copy, Lock, Plus, Trash2, CircleCheck } from "@lucide/vue";
import { pname } from "../i18n";

interface Item {
  id: string;
  name: string;
  builtin?: boolean;
  description?: string;
}

defineProps<{ items: Item[]; activeId?: string; allowNew?: boolean }>();
const selected = defineModel<string>({ required: true });
const emit = defineEmits<{ new: []; duplicate: [id: string]; delete: [id: string] }>();
const { t } = useI18n();
</script>

<template>
  <div>
    <!-- Narrow screens: compact select -->
    <div class="flex items-center gap-2 lg:hidden">
      <select v-model="selected" class="input">
        <option v-for="i in items" :key="i.id" :value="i.id">{{ i.builtin ? "🔒 " : "" }}{{ pname(i) }}{{ i.id === activeId ? " ✓" : "" }}</option>
      </select>
      <button class="btn" :title="t('common.duplicate')" @click="emit('duplicate', selected)"><Copy :size="16" /></button>
      <button v-if="allowNew !== false" class="btn" :title="t('common.new')" @click="emit('new')"><Plus :size="16" /></button>
    </div>

    <!-- Wide screens: full list -->
    <div class="card hidden overflow-hidden lg:block">
      <div class="flex items-center justify-between border-b border-line px-3 py-2">
        <span class="label mb-0">{{ t("common.profiles") }}</span>
        <button v-if="allowNew !== false" class="btn-ghost" :title="t('common.new')" @click="emit('new')"><Plus :size="16" /></button>
      </div>
      <ul class="max-h-[calc(100vh-14rem)] overflow-y-auto py-1">
        <li v-for="i in items" :key="i.id">
          <div
            class="group flex cursor-pointer items-center gap-2 px-3 py-2 transition"
            :class="selected === i.id ? 'bg-brand-500/12 text-brand-700 dark:text-brand-200' : 'hover:bg-panel-2'"
            @click="selected = i.id"
          >
            <Lock v-if="i.builtin" :size="13" class="shrink-0 text-muted" />
            <span class="min-w-0 flex-1 truncate text-sm font-medium">{{ pname(i) }}</span>
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
