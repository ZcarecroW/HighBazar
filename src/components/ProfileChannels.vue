<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Check } from "../icons";
import { useApp } from "../stores/app";
import type { ChannelProfile } from "../lib/types";

// Which channels use a profile – the other way round than in Channels: tick a
// channel to give it this profile. A channel always needs one, so unticking
// falls back to the built-in default.
const props = defineProps<{
  profileId: string;
  field: "companionProfileId" | "moderationProfileId";
  fallbackId: string;
}>();

const app = useApp();
const { t } = useI18n();
const channels = computed(() => app.config?.channels ?? []);
const uses = (ch: ChannelProfile) => ch[props.field] === props.profileId;

function toggle(ch: ChannelProfile) {
  if (uses(ch)) {
    if (props.profileId !== props.fallbackId) ch[props.field] = props.fallbackId;
  } else {
    ch[props.field] = props.profileId;
  }
}
</script>

<template>
  <div class="min-w-60">
    <span class="label">{{ t("profiles.usedIn") }}</span>
    <div class="flex flex-wrap gap-1.5">
      <button
        v-for="ch in channels"
        :key="ch.id"
        type="button"
        role="checkbox"
        :aria-checked="uses(ch)"
        class="chip cursor-pointer gap-1 py-1"
        :class="uses(ch) ? 'bg-brand-500 text-white' : 'bg-panel-2 hover:bg-brand-500/15'"
        :disabled="uses(ch) && profileId === fallbackId"
        :title="uses(ch) ? (profileId === fallbackId ? t('profiles.isDefault') : t('profiles.untick')) : t('profiles.tick')"
        @click="toggle(ch)"
      >
        <Check v-if="uses(ch)" :size="12" />
        {{ ch.name }}
      </button>
    </div>
  </div>
</template>
