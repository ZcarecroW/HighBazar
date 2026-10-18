<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { Eye, EyeOff, Trash2 } from "@lucide/vue";
import { api, errorText } from "../lib/api";
import { useApp } from "../stores/app";

const props = defineProps<{ keyRef: string }>();
const { t } = useI18n();
const app = useApp();
const value = ref("");
const reveal = ref(false);
const busy = ref(false);

async function save() {
  if (!value.value.trim()) return;
  busy.value = true;
  try {
    await api.setAiKey(props.keyRef, value.value.trim());
    value.value = "";
    app.keyStatus = await api.keyStatus();
    app.toast(t("common.saved"), "success", 2000);
  } catch (e) {
    app.toast(errorText(e), "error");
  } finally {
    busy.value = false;
  }
}

async function clear() {
  busy.value = true;
  try {
    await api.setAiKey(props.keyRef, "");
    app.keyStatus = await api.keyStatus();
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="flex gap-2">
    <div class="relative flex-1">
      <input
        v-model="value"
        :type="reveal ? 'text' : 'password'"
        class="input pr-9 font-mono"
        :placeholder="app.keyStatus[keyRef] ? '••••••••••••••••' : t('ai.enterKey')"
        autocomplete="off"
        spellcheck="false"
        @keydown.enter="save"
      />
      <button type="button" class="absolute top-1/2 right-2 -translate-y-1/2 text-muted hover:text-fg" @click="reveal = !reveal">
        <EyeOff v-if="reveal" :size="15" />
        <Eye v-else :size="15" />
      </button>
    </div>
    <button type="button" class="btn-primary shrink-0" :disabled="busy || !value.trim()" @click="save">{{ t("common.save") }}</button>
    <button v-if="app.keyStatus[keyRef]" type="button" class="btn shrink-0" :disabled="busy" :title="t('common.remove')" @click="clear"><Trash2 :size="15" /></button>
  </div>
</template>
