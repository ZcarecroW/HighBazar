<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Info } from "../../icons";

// A small "i" next to a setting: hover, focus or click shows the explanation
// (drawn by TipLayer). A span, not a button: it has to work inside the
// disabled fieldsets of built-in profiles too. Stops the click so a
// surrounding label isn't toggled.
defineProps<{ text: string }>();
const { t } = useI18n();
</script>

<template>
  <span
    role="button"
    tabindex="0"
    class="tip-btn"
    :data-tip="text"
    :aria-label="`${t('common.moreInfo')}: ${text}`"
    @click.prevent.stop
    @keydown.enter.space.prevent="($event.currentTarget as HTMLElement).click()"
  >
    <Info :size="13" />
  </span>
</template>
