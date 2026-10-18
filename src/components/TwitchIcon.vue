<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 16 });
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
    <path d="M4.3 3 3 6.4v13.1h4.5V22h2.6l2.4-2.5h3.6L21 14.6V3H4.3Zm15 10.7-2.8 2.8h-4.5l-2.4 2.4v-2.4H5.8V4.7h13.5v9Zm-2.8-5.6v4.9h-1.7V8.1h1.7Zm-4.5 0v4.9h-1.7V8.1H12Z" />
  </svg>
</template>
