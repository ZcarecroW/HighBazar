<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { Save } from "@lucide/vue";
import { useTwitch } from "../../lib/twitch";

const { t } = useI18n();
const { app, id, tw } = useTwitch();

// Named colors every account may use; hex colors need Turbo or Prime.
const COLORS: Record<string, string> = {
  blue: "#0000FF",
  blue_violet: "#8A2BE2",
  cadet_blue: "#5F9EA0",
  chocolate: "#D2691E",
  coral: "#FF7F50",
  dodger_blue: "#1E90FF",
  firebrick: "#B22222",
  golden_rod: "#DAA520",
  green: "#008000",
  hot_pink: "#FF69B4",
  orange_red: "#FF4500",
  red: "#FF0000",
  sea_green: "#2E8B57",
  spring_green: "#00FF7F",
  yellow_green: "#9ACD32",
};

const user = ref<{ display_name: string; profile_image_url: string; broadcaster_type: string; description: string } | null>(null);
const description = ref("");
const color = ref("");
const hex = ref("#9146FF");
const busy = ref(false);

async function load() {
  const [u, c] = await Promise.all([
    tw<{ data: any[] }>("GET", "/users", { id: id.value }, undefined, true),
    tw<{ data: { color: string }[] }>("GET", "/chat/color", { user_id: id.value }, undefined, true),
  ]);
  user.value = u?.data?.[0] ?? null;
  description.value = user.value?.description ?? "";
  color.value = c?.data?.[0]?.color ?? "";
  if (color.value) hex.value = color.value;
}

async function saveDescription() {
  busy.value = true;
  const r = await tw("PUT", "/users", { description: description.value.trim() });
  busy.value = false;
  if (r !== null) app.toast(t("stream.saved"), "success");
}

async function setColor(value: string) {
  const r = await tw("PUT", "/chat/color", { user_id: id.value, color: value });
  if (r !== null) {
    app.toast(t("stream.saved"), "success");
    await load();
  }
}

onMounted(load);
</script>

<template>
  <section class="card-pad space-y-4">
    <h2 class="section-title mb-0">{{ t("stream.profile.title") }}</h2>
    <div v-if="user" class="flex items-center gap-3">
      <img :src="user.profile_image_url" class="h-12 w-12 rounded-full" alt="" />
      <div>
        <div class="font-semibold">{{ user.display_name }}</div>
        <div class="text-xs text-muted">{{ t(`stream.profile.types.${user.broadcaster_type || "none"}`) }}</div>
      </div>
    </div>
    <div>
      <span class="label">{{ t("stream.profile.description") }}</span>
      <textarea v-model="description" rows="3" maxlength="300" class="input mb-2" />
      <button class="btn" :disabled="busy" @click="saveDescription"><Save :size="15" /> {{ t("common.save") }}</button>
    </div>
    <div>
      <span class="label">{{ t("stream.profile.chatColor") }}</span>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="(c, name) in COLORS"
          :key="name"
          class="h-7 w-7 rounded-full border-2 transition"
          :class="color.toUpperCase() === c ? 'border-fg scale-110' : 'border-transparent'"
          :style="{ background: c }"
          :title="name"
          @click="setColor(name)"
        />
      </div>
      <div class="mt-2 flex items-center gap-2 text-xs text-muted">
        <input v-model="hex" type="color" class="h-7 w-10 rounded border border-line bg-panel-2" />
        <button class="btn-ghost py-0.5 text-xs" @click="setColor(hex)">{{ t("stream.profile.customColor") }}</button>
      </div>
    </div>
  </section>
</template>
