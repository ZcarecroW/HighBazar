<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Languages, Shield } from "../icons";
import type { ModLogEntry } from "../lib/types";

// Translation note of a moderated foreign-language message. Only the streamer
// sees it here; Twitch moderators get it as the timeout/ban reason.
const props = defineProps<{ entry: ModLogEntry; noteOnDelete: boolean }>();
const { t } = useI18n();

const sentToMods = computed(() => {
  const e = props.entry;
  if (e.status !== "executed" && e.status !== "simulated") return false;
  return ["timeout", "ban"].includes(e.action.kind) || (e.action.kind === "delete" && props.noteOnDelete);
});
</script>

<template>
  <div v-if="entry.translation" class="mt-1 rounded-md border border-line bg-panel-2/60 px-2 py-1.5 text-xs leading-relaxed">
    <div class="mb-0.5 flex flex-wrap items-center gap-1.5 font-medium">
      <Languages :size="12" class="text-brand-500" />
      {{ t("translation.noteTitle") }}
      <span v-if="sentToMods" class="chip flex items-center gap-1 bg-brand-500/12 text-brand-600 dark:text-brand-300">
        <Shield :size="11" />{{ entry.status === "simulated" ? t("translation.sentToModsTest") : t("translation.sentToMods") }}
      </span>
    </div>
    <div class="selectable">
      <span class="text-muted">{{ t("translation.translatedFrom", { lang: entry.sourceLang || "?" }) }}:</span> „{{ entry.translation }}“
    </div>
    <div v-if="entry.meaning" class="selectable"><span class="text-muted">{{ t("translation.meaning") }}:</span> {{ entry.meaning }}</div>
  </div>
</template>
