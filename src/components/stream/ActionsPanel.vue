<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Megaphone, Heart, Swords, Scissors, Bookmark, X } from "@lucide/vue";
import { useTwitch } from "../../lib/twitch";

defineProps<{ live: boolean }>();
const { t } = useI18n();
const { app, id, tw, userByLogin } = useTwitch();

const COLORS = ["primary", "blue", "green", "orange", "purple"] as const;
const announcement = ref("");
const color = ref<(typeof COLORS)[number]>("primary");
const shoutoutLogin = ref("");
const raidLogin = ref("");
const raiding = ref(false);
const markerText = ref("");
const busy = ref("");

async function run(key: string, fn: () => Promise<void>) {
  busy.value = key;
  try {
    await fn();
  } finally {
    busy.value = "";
  }
}

const announce = () =>
  run("announce", async () => {
    const r = await tw("POST", "/chat/announcements", { broadcaster_id: id.value, moderator_id: id.value }, { message: announcement.value.trim(), color: color.value });
    if (r !== null) {
      app.toast(t("stream.actions.announced"), "success");
      announcement.value = "";
    }
  });

const shoutout = () =>
  run("shoutout", async () => {
    const u = await userByLogin(shoutoutLogin.value);
    if (!u) return;
    const r = await tw("POST", "/chat/shoutouts", { from_broadcaster_id: id.value, to_broadcaster_id: u.id, moderator_id: id.value });
    if (r !== null) app.toast(t("stream.actions.shoutoutDone", { name: u.display_name }), "success");
  });

const raid = () =>
  run("raid", async () => {
    const u = await userByLogin(raidLogin.value);
    if (!u || !confirm(t("stream.actions.raidConfirm", { name: u.display_name }))) return;
    const r = await tw("POST", "/raids", { from_broadcaster_id: id.value, to_broadcaster_id: u.id });
    if (r !== null) {
      raiding.value = true;
      app.toast(t("stream.actions.raidStarted", { name: u.display_name }), "success", 7000);
    }
  });

const cancelRaid = () =>
  run("raid", async () => {
    const r = await tw("DELETE", "/raids", { broadcaster_id: id.value });
    if (r !== null) {
      raiding.value = false;
      app.toast(t("stream.actions.raidCancelled"), "success");
    }
  });

const clip = () =>
  run("clip", async () => {
    const r = await tw<{ data: { id: string; edit_url: string }[] }>("POST", "/clips", { broadcaster_id: id.value });
    const c = r?.data?.[0];
    if (c) {
      app.toast(t("stream.actions.clipDone"), "success");
      openUrl(c.edit_url);
    }
  });

const marker = () =>
  run("marker", async () => {
    const r = await tw<{ data: { position_seconds: number }[] }>("POST", "/streams/markers", {}, { user_id: id.value, description: markerText.value.trim().slice(0, 140) });
    const m = r?.data?.[0];
    if (m) {
      app.toast(t("stream.actions.markerDone", { s: m.position_seconds }), "success");
      markerText.value = "";
    }
  });
</script>

<template>
  <section class="card-pad space-y-5">
    <h2 class="section-title mb-0">{{ t("stream.actions.title") }}</h2>

    <div>
      <span class="label">{{ t("stream.actions.announcement") }}</span>
      <textarea v-model="announcement" rows="2" maxlength="500" class="input mb-2" :placeholder="t('stream.actions.announcementPh')" />
      <div class="flex flex-wrap items-center gap-2">
        <select v-model="color" class="input w-36">
          <option v-for="c in COLORS" :key="c" :value="c">{{ t(`stream.actions.colors.${c}`) }}</option>
        </select>
        <button class="btn" :disabled="!announcement.trim() || busy === 'announce'" @click="announce"><Megaphone :size="15" /> {{ t("stream.actions.send") }}</button>
      </div>
    </div>

    <div>
      <span class="label">{{ t("stream.actions.shoutout") }}</span>
      <div class="flex gap-2">
        <input v-model="shoutoutLogin" class="input" :placeholder="t('stream.loginPh')" />
        <button class="btn shrink-0" :disabled="!shoutoutLogin.trim() || !live || busy === 'shoutout'" @click="shoutout"><Heart :size="15" /> {{ t("stream.actions.shoutoutBtn") }}</button>
      </div>
      <p class="hint">{{ t("stream.actions.shoutoutHint") }}</p>
    </div>

    <div>
      <span class="label">{{ t("stream.actions.raid") }}</span>
      <div class="flex gap-2">
        <input v-model="raidLogin" class="input" :placeholder="t('stream.loginPh')" />
        <button v-if="!raiding" class="btn shrink-0" :disabled="!raidLogin.trim() || busy === 'raid'" @click="raid"><Swords :size="15" /> {{ t("stream.actions.raidBtn") }}</button>
        <button v-else class="btn-danger shrink-0" :disabled="busy === 'raid'" @click="cancelRaid"><X :size="15" /> {{ t("stream.actions.raidCancel") }}</button>
      </div>
    </div>

    <div class="grid gap-4 md:grid-cols-2">
      <div>
        <span class="label">{{ t("stream.actions.clip") }}</span>
        <button class="btn" :disabled="!live || busy === 'clip'" @click="clip"><Scissors :size="15" /> {{ t("stream.actions.clipBtn") }}</button>
      </div>
      <div>
        <span class="label">{{ t("stream.actions.marker") }}</span>
        <div class="flex gap-2">
          <input v-model="markerText" maxlength="140" class="input" :placeholder="t('stream.actions.markerPh')" />
          <button class="btn shrink-0" :disabled="!live || busy === 'marker'" :title="t('stream.actions.markerBtn')" @click="marker"><Bookmark :size="15" /></button>
        </div>
      </div>
    </div>
    <p v-if="!live" class="hint">{{ t("stream.onlyLive") }}</p>
  </section>
</template>
