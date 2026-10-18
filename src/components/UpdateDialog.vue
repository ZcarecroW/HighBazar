<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { Download, Loader2 } from "@lucide/vue";
import UiModal from "./ui/UiModal.vue";
import { installUpdate, pendingUpdate, updateInstalling, updateProgress } from "../lib/updater";
import { errorText } from "../lib/api";
import { useApp } from "../stores/app";

const { t } = useI18n();
const app = useApp();
const busy = ref(false);

async function install() {
  busy.value = true;
  try {
    if (app.isRunning) await app.stopBot();
    await installUpdate();
  } catch (e) {
    app.toast(t("update.error", { e: errorText(e) }), "error", 8000);
    busy.value = false;
  }
}
</script>

<template>
  <UiModal v-if="pendingUpdate" :title="t('update.available', { v: pendingUpdate.version })" :closable="!busy" @close="pendingUpdate = null">
    <div class="space-y-3">
      <div v-if="pendingUpdate.body">
        <span class="label">{{ t("update.notes") }}</span>
        <div class="selectable max-h-64 overflow-y-auto rounded-lg bg-panel-2 p-3 text-sm whitespace-pre-wrap">{{ pendingUpdate.body }}</div>
      </div>
      <div v-if="busy" class="flex items-center gap-2 text-sm text-muted">
        <Loader2 :size="16" class="animate-spin" />
        <span v-if="updateInstalling">{{ t("update.installing") }}</span>
        <span v-else>{{ t("update.downloading", { p: updateProgress === null ? "" : updateProgress + "%" }) }}</span>
      </div>
      <div v-if="busy && updateProgress !== null" class="h-1.5 overflow-hidden rounded-full bg-panel-2">
        <div class="h-full bg-brand-500 transition-all" :style="{ width: updateProgress + '%' }" />
      </div>
    </div>
    <template #footer>
      <button class="btn" :disabled="busy" @click="pendingUpdate = null">{{ t("update.later") }}</button>
      <button class="btn-primary" :disabled="busy" @click="install"><Download :size="16" /> {{ t("update.install") }}</button>
    </template>
  </UiModal>
</template>
