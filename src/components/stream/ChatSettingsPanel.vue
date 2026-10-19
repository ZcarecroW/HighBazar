<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Loader2, Save, Chat } from "../../icons";
import { useTwitch } from "../../lib/twitch";
import UiToggle from "../ui/UiToggle.vue";
import UiNumber from "../ui/UiNumber.vue";

const { t } = useI18n();
const { app, id, tw } = useTwitch();

interface ChatSettings {
  emote_mode: boolean;
  follower_mode: boolean;
  follower_mode_duration: number | null;
  slow_mode: boolean;
  slow_mode_wait_time: number | null;
  subscriber_mode: boolean;
  unique_chat_mode: boolean;
  non_moderator_chat_delay: boolean;
  non_moderator_chat_delay_duration: number | null;
}
const s = ref<ChatSettings | null>(null);
const busy = ref(false);

async function load() {
  const r = await tw<{ data: ChatSettings[] }>("GET", "/chat/settings", { broadcaster_id: id.value, moderator_id: id.value });
  const d = r?.data?.[0];
  if (d) {
    d.follower_mode_duration ??= 0;
    d.slow_mode_wait_time ??= 30;
    d.non_moderator_chat_delay ??= false;
    d.non_moderator_chat_delay_duration ??= 2;
    s.value = d;
  }
}

async function save() {
  if (!s.value) return;
  busy.value = true;
  const v = s.value;
  const body: Record<string, unknown> = {
    emote_mode: v.emote_mode,
    follower_mode: v.follower_mode,
    slow_mode: v.slow_mode,
    subscriber_mode: v.subscriber_mode,
    unique_chat_mode: v.unique_chat_mode,
    non_moderator_chat_delay: v.non_moderator_chat_delay,
  };
  if (v.follower_mode) body.follower_mode_duration = v.follower_mode_duration ?? 0;
  if (v.slow_mode) body.slow_mode_wait_time = v.slow_mode_wait_time ?? 30;
  if (v.non_moderator_chat_delay) body.non_moderator_chat_delay_duration = v.non_moderator_chat_delay_duration ?? 2;
  const r = await tw<{ data: ChatSettings[] }>("PATCH", "/chat/settings", { broadcaster_id: id.value, moderator_id: id.value }, body);
  busy.value = false;
  if (r) {
    app.toast(t("stream.saved"), "success");
    await load();
  }
}

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-4">
    <h2 class="section-title mb-0 flex items-center gap-2"><Chat :size="17" class="text-brand-500" /> {{ t("stream.chat.title") }}</h2>
    <p v-if="!s" class="text-sm text-muted">{{ t("common.loading") }}</p>
    <template v-else>
      <UiToggle v-model="s.emote_mode" :label="t('stream.chat.emoteOnly')" />
      <UiToggle v-model="s.subscriber_mode" :label="t('stream.chat.subOnly')" />
      <UiToggle v-model="s.unique_chat_mode" :label="t('stream.chat.unique')" :hint="t('stream.chat.uniqueHint')" />
      <div class="flex flex-wrap items-end gap-4">
        <UiToggle v-model="s.follower_mode" :label="t('stream.chat.followerOnly')" class="flex-1" />
        <UiNumber v-if="s.follower_mode" v-model="s.follower_mode_duration!" :min="0" :max="129600" :suffix="t('stream.chat.minutesFollowed')" />
      </div>
      <div class="flex flex-wrap items-end gap-4">
        <UiToggle v-model="s.slow_mode" :label="t('stream.chat.slow')" class="flex-1" />
        <UiNumber v-if="s.slow_mode" v-model="s.slow_mode_wait_time!" :min="3" :max="120" :suffix="t('stream.seconds')" />
      </div>
      <div class="flex flex-wrap items-end gap-4">
        <UiToggle v-model="s.non_moderator_chat_delay" :label="t('stream.chat.delay')" :hint="t('stream.chat.delayHint')" class="flex-1" />
        <select v-if="s.non_moderator_chat_delay" v-model.number="s.non_moderator_chat_delay_duration" class="input w-28">
          <option :value="2">2 s</option>
          <option :value="4">4 s</option>
          <option :value="6">6 s</option>
        </select>
      </div>
      <button class="btn-primary" :disabled="busy" @click="save">
        <Loader2 v-if="busy" :size="15" class="animate-spin" /><Save v-else :size="15" /> {{ t("common.save") }}
      </button>
    </template>
  </section>
</template>
