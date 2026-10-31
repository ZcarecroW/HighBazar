<script setup lang="ts">
import UiTip from "./UiTip.vue";
defineProps<{ label?: string; hint?: string; tip?: string }>();
</script>

<template>
  <div class="min-w-0">
    <span v-if="label" class="label">{{ label }}<UiTip v-if="tip" :text="tip" /></span>
    <slot />
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>
