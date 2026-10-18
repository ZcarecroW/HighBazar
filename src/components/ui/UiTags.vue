<script setup lang="ts">
import { ref } from "vue";
import { X } from "@lucide/vue";

defineProps<{ label?: string; hint?: string; placeholder?: string; disabled?: boolean }>();
const model = defineModel<string[]>({ required: true });
const draft = ref("");

function add() {
  const parts = draft.value
    .split(/[,\n]/)
    .map((s) => s.trim())
    .filter(Boolean);
  for (const p of parts) if (!model.value.includes(p)) model.value.push(p);
  draft.value = "";
}
function remove(i: number) {
  model.value.splice(i, 1);
}
</script>

<template>
  <div class="min-w-0">
    <span v-if="label" class="label">{{ label }}</span>
    <div class="flex flex-wrap items-center gap-1.5 rounded-lg border border-line bg-panel-2 p-1.5" :class="disabled && 'opacity-60'">
      <span v-for="(t, i) in model" :key="t + i" class="chip bg-brand-500/15 text-brand-700 dark:text-brand-200">
        {{ t }}
        <button v-if="!disabled" type="button" class="opacity-60 hover:opacity-100" @click="remove(i)"><X :size="12" /></button>
      </span>
      <input
        v-if="!disabled"
        v-model="draft"
        class="min-w-24 flex-1 bg-transparent px-1 py-0.5 text-sm outline-none placeholder:text-muted"
        :placeholder="placeholder"
        @keydown.enter.prevent="add"
        @blur="add"
      />
    </div>
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>
