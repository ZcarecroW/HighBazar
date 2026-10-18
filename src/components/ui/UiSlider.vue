<script setup lang="ts">
withDefaults(
  defineProps<{ label?: string; hint?: string; min?: number; max?: number; step?: number; suffix?: string; disabled?: boolean; display?: string }>(),
  { min: 0, max: 100, step: 1, suffix: "" },
);
const model = defineModel<number>({ required: true });
</script>

<template>
  <div class="min-w-0">
    <div v-if="label" class="mb-1 flex items-baseline justify-between gap-2">
      <span class="label mb-0">{{ label }}</span>
      <span class="text-xs font-semibold tabular-nums text-brand-600 dark:text-brand-300">{{ display ?? model + suffix }}</span>
    </div>
    <input
      v-model.number="model"
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :disabled="disabled"
      class="w-full accent-brand-500 disabled:opacity-60"
    />
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>
