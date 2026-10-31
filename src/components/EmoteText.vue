<script setup lang="ts">
import { computed } from "vue";
import { useApp } from "../stores/app";
import { emoteParts } from "../lib/emoteParts";

// Renders chat text with Twitch/BTTV/FFZ/7TV emotes as images (if enabled).
const props = defineProps<{ text: string; emotes?: Record<string, string> }>();
const app = useApp();

const parts = computed(() =>
  app.config?.settings.showEmotes ? emoteParts(props.text, props.emotes, app.emoteUrls, app.zeroWidth) : [{ text: props.text, url: "" }],
);
</script>

<template>
  <template v-for="(p, i) in parts" :key="i">
    <span v-if="p.over" class="emote-stack" :title="[p.text, ...p.over.map((o) => o.text)].join(' ')"
      ><img :src="p.url" :alt="p.text" class="emote" loading="lazy" decoding="async" /><img
        v-for="(o, j) in p.over"
        :key="j"
        :src="o.url"
        :alt="o.text"
        class="emote"
        loading="lazy"
        decoding="async"
    /></span>
    <img v-else-if="p.url" :src="p.url" :alt="p.text" :title="p.text" class="emote" loading="lazy" decoding="async" /><template v-else>{{ p.text }}</template>
  </template>
</template>
