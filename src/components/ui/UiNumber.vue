<script setup lang="ts">
const props = withDefaults(
  defineProps<{ label?: string; hint?: string; min?: number; max?: number; step?: number; suffix?: string; disabled?: boolean }>(),
  { min: 0, max: 1_000_000, step: 1 },
);
const model = defineModel<number>({ required: true });

function onInput(e: Event) {
  const raw = (e.target as HTMLInputElement).valueAsNumber;
  if (Number.isNaN(raw)) return;
  model.value = Math.min(props.max, Math.max(props.min, raw));
}
</script>

<template>
  <div class="min-w-0">
    <span v-if="label" class="label">{{ label }}</span>
    <div class="flex items-center gap-2">
      <input
        :value="model"
        type="number"
        :min="min"
        :max="max"
        :step="step"
        :disabled="disabled"
        class="input w-28 tabular-nums"
        @change="onInput"
      />
      <span v-if="suffix" class="text-xs text-muted">{{ suffix }}</span>
    </div>
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>
