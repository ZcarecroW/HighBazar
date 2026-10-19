<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Lock } from "../icons";

const emit = defineEmits<{ duplicate: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 rounded-xl border border-amber-400/40 bg-amber-400/10 px-4 py-3 text-sm">
    <Lock :size="16" class="shrink-0 text-amber-500" />
    <span class="min-w-0 flex-1">{{ t("common.builtinHint") }}</span>
    <button class="btn-primary" @click="emit('duplicate')">{{ t("common.duplicateEdit") }}</button>
  </div>
</template>
