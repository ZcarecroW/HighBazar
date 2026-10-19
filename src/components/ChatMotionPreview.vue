<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { SmoothFollow } from "../lib/smoothFollow";
import { chatScale } from "../lib/zoom";
import type { ChatScroll } from "../lib/types";

// Small fake chat that shows the current scroll settings live.
const props = defineProps<{ scroll: ChatScroll; chatZoom: number; uiZoom: number }>();
const { t } = useI18n();

const NAMES = [
  ["Krähe_42", "#e05d5d"],
  ["pixelpanda", "#4fa3e0"],
  ["NightOwl", "#a970ff"],
  ["lena_plays", "#2fb380"],
  ["Bjoern", "#e0a03a"],
] as const;
const busy = ref(false);
const lines = ref<{ id: number; name: string; color: string; text: string }[]>([]);
const el = ref<HTMLElement>();
const inner = ref<HTMLElement>();
let n = 0;
let timer = 0;
let follower: SmoothFollow | null = null;

function add() {
  const [name, color] = NAMES[n % NAMES.length]!;
  lines.value = [...lines.value.slice(-30), { id: n, name, color, text: t(`settings.preview.msg${(n % 6) + 1}`) }];
  n++;
}
function schedule() {
  clearTimeout(timer);
  timer = window.setTimeout(() => {
    add();
    schedule();
  }, busy.value ? 280 + Math.random() * 250 : 1300 + Math.random() * 600);
}
onMounted(() => {
  for (let i = 0; i < 5; i++) add();
  if (el.value && inner.value) {
    follower = new SmoothFollow(el.value, inner.value, () => props.scroll);
    follower.follow();
  }
  schedule();
});
onBeforeUnmount(() => {
  clearTimeout(timer);
  follower?.destroy();
});
function toggleBusy() {
  busy.value = !busy.value;
  schedule();
}
const fadeMs = computed(() => (props.scroll.smooth ? Math.min(400, props.scroll.durationMs) : 0));
const style = computed(() => ({ zoom: chatScale(props.chatZoom, props.uiZoom), "--feed-in-ms": fadeMs.value + "ms" }));
</script>

<template>
  <div class="overflow-hidden rounded-xl border border-line bg-panel">
    <div class="flex items-center gap-2 border-b border-line px-3 py-1.5 text-xs text-muted">
      <span class="font-semibold text-fg">{{ t("settings.preview.title") }}</span>
      <button type="button" class="chip ml-auto transition" :class="busy ? 'bg-brand-500/15 text-brand-700 dark:text-brand-200' : 'bg-panel-2'" @click="toggleBusy">
        {{ busy ? t("settings.preview.busyOn") : t("settings.preview.busyOff") }}
      </button>
    </div>
    <div ref="el" class="h-44 overflow-hidden px-2 py-1.5">
      <div ref="inner" :style="style">
        <div v-for="l in lines" :key="l.id" class="rounded-lg px-2 py-1 text-[13.5px] leading-snug" :class="fadeMs > 0 && l.id >= 5 ? 'feed-in' : ''">
          <span class="font-semibold" :style="{ color: l.color }">{{ l.name }}</span><span class="text-muted">: </span>{{ l.text }}
        </div>
      </div>
    </div>
  </div>
</template>
