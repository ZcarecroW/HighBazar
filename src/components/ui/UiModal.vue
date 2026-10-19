<script setup lang="ts">
import { X } from "../../icons";

withDefaults(defineProps<{ title?: string; width?: string; closable?: boolean }>(), { width: "max-w-lg", closable: true });
const emit = defineEmits<{ close: [] }>();
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm" @click.self="closable && emit('close')">
      <div class="card flex max-h-full w-full flex-col overflow-hidden shadow-2xl" :class="width">
        <header v-if="title || closable" class="flex items-center justify-between border-b border-line px-5 py-3">
          <h2 class="text-base font-semibold">{{ title }}</h2>
          <button v-if="closable" class="btn-ghost" @click="emit('close')"><X :size="18" /></button>
        </header>
        <div class="min-h-0 flex-1 overflow-y-auto p-5">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="flex justify-end gap-2 border-t border-line px-5 py-3">
          <slot name="footer" />
        </footer>
      </div>
    </div>
  </Teleport>
</template>
