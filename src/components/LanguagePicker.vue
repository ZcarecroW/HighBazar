<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { X } from "../icons";
import { languageOptions } from "../lib/languages";
import UiCombobox from "./ui/UiCombobox.vue";

// Languages the streamer understands; the first one is the translation target.
// Same list as the backend detector (src-tauri/src/bot/translate.rs).
const CODES = ["de", "en", "es", "fr", "it", "pt", "nl", "pl", "ru", "uk", "tr", "sv", "no", "da", "fi", "cs", "hu", "ro", "el", "ja", "ko", "zh", "ar", "hi", "th", "vi", "id"];

const model = defineModel<string[]>({ required: true });
const { t, locale } = useI18n();
const adding = ref("");

function name(code: string) {
  try {
    return new Intl.DisplayNames([locale.value], { type: "language" }).of(code) ?? code;
  } catch {
    return code;
  }
}
const available = computed(() => CODES.filter((c) => !model.value.includes(c)).sort((a, b) => name(a).localeCompare(name(b))));

const options = computed(() => languageOptions(available.value, locale.value));
function add() {
  if (adding.value && !model.value.includes(adding.value)) model.value = [...model.value, adding.value];
  adding.value = "";
}
function remove(code: string) {
  model.value = model.value.filter((c) => c !== code);
}
function makeTarget(code: string) {
  model.value = [code, ...model.value.filter((c) => c !== code)];
}
</script>

<template>
  <div>
    <div class="flex flex-wrap items-center gap-1.5">
      <span
        v-for="(c, i) in model"
        :key="c"
        class="chip flex items-center gap-1 py-1 pl-2.5"
        :class="i === 0 ? 'bg-brand-500 text-white' : 'bg-panel-2'"
      >
        <button
          type="button"
          :class="i === 0 ? 'cursor-default' : 'hover:underline'"
          :title="i === 0 ? t('translation.target') : t('translation.makeTarget')"
          @click="makeTarget(c)"
        >
          {{ name(c) }}<template v-if="i === 0"> · {{ t("translation.target") }}</template>
        </button>
        <button type="button" class="rounded-full p-0.5 opacity-70 hover:opacity-100" :aria-label="t('common.remove')" @click="remove(c)"><X :size="12" /></button>
      </span>
      <UiCombobox
        v-model="adding"
        class="w-56"
        :options="options"
        :placeholder="t('translation.addLanguage')"
        :label="t('translation.addLanguage')"
        @picked="add"
      />
    </div>
  </div>
</template>
