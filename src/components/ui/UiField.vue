<script setup lang="ts">
defineProps<{ label?: string; hint?: string }>();
</script>

<template>
  <div class="min-w-0">
    <span v-if="label" class="label">{{ label }}</span>
    <slot />
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>
