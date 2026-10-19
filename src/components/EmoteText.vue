<script setup lang="ts">
import { computed } from "vue";
import { useApp } from "../stores/app";

// Renders chat text with Twitch/BTTV/FFZ/7TV emotes as images (if enabled).
const props = defineProps<{ text: string; emotes?: Record<string, string> }>();
const app = useApp();

const parts = computed(() => {
  if (!app.config?.settings.showEmotes) return [{ text: props.text, url: "" }];
  const local = props.emotes ?? {};
  const known = app.emoteUrls;
  const out: { text: string; url: string }[] = [];
  let buf = "";
  for (const token of props.text.split(/(\s+)/)) {
    const url = local[token] ?? (Object.hasOwn(known, token) ? known[token] : "");
    if (url) {
      if (buf) out.push({ text: buf, url: "" });
      buf = "";
      out.push({ text: token, url });
    } else {
      buf += token;
    }
  }
  if (buf) out.push({ text: buf, url: "" });
  return out;
});
</script>

<template>
  <template v-for="(p, i) in parts" :key="i">
    <img v-if="p.url" :src="p.url" :alt="p.text" :title="p.text" class="emote" loading="lazy" decoding="async" /><template v-else>{{ p.text }}</template>
  </template>
</template>
