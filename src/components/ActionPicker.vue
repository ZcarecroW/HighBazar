<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { ModAction } from "../lib/types";

defineProps<{ disabled?: boolean; allowNone?: boolean }>();
const model = defineModel<ModAction>({ required: true });
const { t } = useI18n();

const kinds = ["none", "log", "delete", "warn", "timeout", "ban"] as const;
const durations = [10, 30, 60, 300, 600, 1800, 3600, 21600, 86400, 604800, 1209600];

function fmt(s: number) {
  if (s < 60) return `${s}s`;
  if (s < 3600) return `${s / 60}m`;
  if (s < 86400) return `${s / 3600}h`;
  return `${s / 86400}d`;
}

const kind = computed({
  get: () => model.value.kind,
  set: (k) => {
    model.value = { kind: k, durationSecs: k === "timeout" ? model.value.durationSecs || 600 : 0 };
  },
});
const duration = computed({
  get: () => model.value.durationSecs,
  set: (d) => (model.value = { ...model.value, durationSecs: d }),
});
</script>

<template>
  <div class="flex items-center gap-1.5">
    <select v-model="kind" class="input w-auto py-1 text-xs" :disabled="disabled">
      <option v-for="k in kinds" :key="k" :value="k">{{ t(`moderation.actions.${k}`) }}</option>
    </select>
    <select v-if="kind === 'timeout'" v-model.number="duration" class="input w-auto py-1 text-xs" :disabled="disabled">
      <option v-for="d in durations" :key="d" :value="d">{{ fmt(d) }}</option>
      <option v-if="!durations.includes(duration)" :value="duration">{{ fmt(duration) }}</option>
    </select>
  </div>
</template>
