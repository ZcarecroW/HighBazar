<script setup lang="ts">
defineProps<{ options: { value: string; label: string }[]; disabled?: boolean; label?: string }>();
const model = defineModel<string>({ required: true });
</script>

<template>
  <div class="min-w-0">
    <span v-if="label" class="label">{{ label }}</span>
    <div class="inline-flex flex-wrap gap-1 rounded-lg border border-line bg-panel-2 p-1" :class="disabled && 'opacity-60'">
      <button
        v-for="o in options"
        :key="o.value"
        type="button"
        :disabled="disabled"
        class="rounded-md px-2.5 py-1 text-xs font-medium transition"
        :class="model === o.value ? 'bg-brand-500 text-white shadow-sm' : 'text-muted hover:text-fg'"
        @click="model = o.value"
      >
        {{ o.label }}
      </button>
    </div>
  </div>
</template>
