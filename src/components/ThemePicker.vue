<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Check } from "../icons";
import { THEMES, themeSwatch } from "../lib/themes";

const model = defineModel<string>({ required: true });
const { t } = useI18n();
</script>

<template>
  <div class="grid grid-cols-2 gap-2.5 sm:grid-cols-3 lg:grid-cols-5">
    <button
      v-for="th in THEMES"
      :key="th.id"
      type="button"
      class="group relative overflow-hidden rounded-xl border-2 text-left transition"
      :class="model === th.id ? 'border-brand-500 shadow-md' : 'border-line hover:border-brand-300'"
      :title="t(`settings.themeNames.${th.id}`)"
      @click="model = th.id"
    >
      <!-- light half / dark half preview -->
      <div class="flex h-14">
        <div class="flex flex-1 flex-col justify-end gap-1 p-2" :style="{ background: themeSwatch(th).light }">
          <div class="h-1.5 w-8 rounded-full" :style="{ background: themeSwatch(th).accent }" />
          <div class="h-1.5 w-5 rounded-full" :style="{ background: themeSwatch(th).lightPanel }" />
        </div>
        <div class="flex flex-1 flex-col justify-end gap-1 p-2" :style="{ background: themeSwatch(th).dark }">
          <div class="h-1.5 w-8 rounded-full" :style="{ background: themeSwatch(th).accent2 }" />
          <div class="h-1.5 w-5 rounded-full" :style="{ background: themeSwatch(th).darkPanel }" />
        </div>
      </div>
      <div class="flex items-center gap-1.5 bg-panel px-2.5 py-1.5 text-xs font-semibold">
        <span class="h-2.5 w-2.5 rounded-full" :style="{ background: themeSwatch(th).accent }" />
        {{ t(`settings.themeNames.${th.id}`) }}
        <Check v-if="model === th.id" :size="13" class="ml-auto text-brand-500" />
      </div>
    </button>
  </div>
</template>
