<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Copy, ExternalLink, MonitorPlay, RefreshCw } from "../icons";
import { useApp } from "../stores/app";
import { api } from "../lib/api";
import type { OverlayStatus } from "../lib/types";
import UiToggle from "./ui/UiToggle.vue";
import UiSlider from "./ui/UiSlider.vue";
import UiNumber from "./ui/UiNumber.vue";

// Settings of the chat page for the Steam overlay browser.
const app = useApp();
const { t } = useI18n();
const o = computed(() => app.config!.settings.overlay);
const status = ref<OverlayStatus | null>(null);

function newKey() {
  const a = new Uint8Array(18);
  crypto.getRandomValues(a);
  return Array.from(a, (b) => b.toString(36).padStart(2, "0")).join("").slice(0, 24);
}
// The server follows the saved settings; refresh the status shortly after a change.
let timer = 0;
async function refresh() {
  status.value = await api.overlayStatus().catch(() => null);
}
function later() {
  clearTimeout(timer);
  timer = window.setTimeout(refresh, 900);
}
watch(
  () => [o.value.enabled, o.value.port, o.value.key],
  () => {
    if (o.value.enabled && o.value.key.length < 16) o.value.key = newKey();
    later();
  },
);
let poll = 0;
onMounted(() => {
  void refresh();
  poll = window.setInterval(refresh, 5000);
});
onBeforeUnmount(() => {
  clearInterval(poll);
  clearTimeout(timer);
});

const copied = ref(false);
async function copy() {
  if (!status.value?.url) return;
  await navigator.clipboard.writeText(status.value.url);
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}
function regenerate() {
  if (confirm(t("overlay.regenerateConfirm"))) o.value.key = newKey();
}
const bgPct = computed({
  get: () => Math.round(o.value.background * 100),
  set: (v) => (o.value.background = v / 100),
});
</script>

<template>
  <section class="card-pad space-y-5">
    <h2 class="section-title mb-0 flex items-center gap-2"><MonitorPlay :size="17" class="text-brand-500" /> {{ t("overlay.title") }}</h2>
    <p class="max-w-3xl text-sm text-muted">{{ t("overlay.intro") }}</p>
    <UiToggle v-model="o.enabled" :label="t('overlay.enabled')" :hint="t('overlay.enabledHint')" />

    <template v-if="o.enabled">
      <div>
        <span class="label">{{ t("overlay.link") }}</span>
        <div class="flex flex-wrap gap-2">
          <input :value="status?.url || ''" readonly class="input min-w-0 flex-1 font-mono text-xs" :placeholder="t('overlay.starting')" @focus="($event.target as HTMLInputElement).select()" />
          <button class="btn" :disabled="!status?.url" @click="copy"><Copy :size="15" /> {{ copied ? t("overlay.copied") : t("overlay.copy") }}</button>
          <button class="btn" :disabled="!status?.url" @click="openUrl(status!.url)"><ExternalLink :size="15" /> {{ t("overlay.test") }}</button>
          <button class="btn-ghost" :title="t('overlay.regenerate')" @click="regenerate"><RefreshCw :size="15" /></button>
        </div>
        <p v-if="status?.error" class="hint text-red-500">{{ t("overlay.portError", { port: o.port }) }} ({{ status.error }})</p>
        <p v-else-if="status?.running" class="hint">
          {{ status.clients ? t("overlay.connected", { n: status.clients }) : t("overlay.waiting") }} · {{ t("overlay.keepSecret") }}
        </p>
      </div>

      <div class="rounded-xl bg-panel-2 p-4 text-sm">
        <div class="mb-2 font-semibold">{{ t("overlay.howTo") }}</div>
        <ol class="list-decimal space-y-1 pl-5 text-muted">
          <li>{{ t("overlay.step1") }}</li>
          <li>{{ t("overlay.step2") }}</li>
          <li>{{ t("overlay.step3") }}</li>
          <li>{{ t("overlay.step4") }}</li>
          <li>{{ t("overlay.step5") }}</li>
        </ol>
        <p class="hint mt-2">{{ t("overlay.obs") }}</p>
      </div>

      <div class="grid gap-4 md:grid-cols-3">
        <UiToggle v-model="o.showBot" :label="t('overlay.showBot')" />
        <UiToggle v-model="o.showEvents" :label="t('overlay.showEvents')" />
        <UiToggle v-model="o.showTranslations" :label="t('overlay.showTranslations')" />
      </div>
      <UiToggle v-model="o.chatInput" :label="t('overlay.chatInput')" :hint="t('overlay.chatInputHint')" />
      <div class="grid gap-5 md:grid-cols-2">
        <UiSlider v-model="bgPct" :label="t('overlay.background')" :hint="t('overlay.backgroundHint')" :min="0" :max="100" :step="5" suffix=" %" />
        <UiSlider
          v-model="o.hideAfterSecs"
          :label="t('overlay.hideAfter')"
          :hint="t('overlay.hideAfterHint')"
          :min="0"
          :max="120"
          :step="5"
          :display="o.hideAfterSecs ? `${o.hideAfterSecs} s` : t('overlay.never')"
        />
      </div>
      <p class="hint">{{ t("overlay.usesChatSettings") }}</p>
      <UiNumber v-model="o.port" :label="t('overlay.port')" :hint="t('overlay.portHint')" :min="1024" :max="65535" class="max-w-xs" />
    </template>
  </section>
</template>
