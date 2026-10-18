<script setup lang="ts">
defineProps<{ label?: string; hint?: string; disabled?: boolean }>();
const model = defineModel<boolean>({ required: true });
</script>

<template>
  <label class="flex items-start gap-3" :class="disabled ? 'cursor-not-allowed opacity-60' : 'cursor-pointer'">
    <button
      type="button"
      role="switch"
      :aria-checked="model"
      :disabled="disabled"
      class="relative mt-0.5 inline-flex h-5 w-9 shrink-0 items-center rounded-full transition"
      :class="model ? 'bg-brand-500' : 'bg-[var(--line)]'"
      @click="model = !model"
    >
      <span
        class="inline-block h-4 w-4 rounded-full bg-white shadow transition"
        :class="model ? 'translate-x-4.5' : 'translate-x-0.5'"
      />
    </button>
    <span v-if="label || hint || $slots.default" class="min-w-0">
      <span class="block text-sm text-fg">
        <slot>{{ label }}</slot>
      </span>
      <span v-if="hint" class="hint block">{{ hint }}</span>
    </span>
  </label>
</template>
