<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { useI18n } from "vue-i18n";
import { onEvent } from "../lib/events";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, Loader2 } from "../icons";
import TwitchIcon from "./TwitchIcon.vue";
import { api, errorText } from "../lib/api";
import type { LoginEvent, LoginStart, TwitchAccount } from "../lib/types";
import { useApp } from "../stores/app";
import UiModal from "./ui/UiModal.vue";

const props = defineProps<{ kind: "streamer" | "bot"; label?: string; primary?: boolean; level?: string }>();
const emit = defineEmits<{ connected: [account: TwitchAccount] }>();
const { t } = useI18n();
const app = useApp();

const flow = ref<LoginStart | null>(null);
const busy = ref(false);
const message = ref("");
let unlisten: (() => void) | undefined;

async function start() {
  busy.value = true;
  message.value = "";
  try {
    unlisten?.();
    unlisten = await onEvent<LoginEvent>("hb:login", async (payload) => {
      if (!flow.value || payload.loginId !== flow.value.loginId) return;
      if (payload.status === "done" && payload.account) {
        await app.saveNow();
        await app.refresh();
        app.toast(t("channels.loginDone", { name: payload.account.displayName }), "success");
        emit("connected", payload.account);
        close();
      } else if (payload.status === "expired") {
        message.value = t("channels.loginExpired");
      } else if (payload.status === "error") {
        message.value = payload.error;
      }
    });
    flow.value = await api.twitchLoginStart(props.kind, props.level);
    await openUrl(flow.value.verificationUri);
  } catch (e) {
    app.toast(errorText(e), "error", 7000);
    flow.value = null;
  } finally {
    busy.value = false;
  }
}

function close() {
  if (flow.value) api.twitchLoginCancel(flow.value.loginId).catch(() => {});
  flow.value = null;
  unlisten?.();
  unlisten = undefined;
}

onBeforeUnmount(close);
</script>

<template>
  <button :class="primary ? 'btn-primary' : 'btn'" :disabled="busy" @click="start">
    <Loader2 v-if="busy" :size="16" class="animate-spin" />
    <TwitchIcon v-else :size="16" />
    {{ label ?? t("common.connect") }}
  </button>

  <UiModal v-if="flow" :title="t('channels.loginTitle')" @close="close">
    <div class="space-y-4 text-center">
      <p class="text-sm text-muted">{{ t("channels.loginStep1") }}</p>
      <div class="selectable mx-auto w-fit rounded-xl border-2 border-dashed border-brand-400 px-6 py-3 font-mono text-3xl font-bold tracking-[0.3em] text-brand-600 dark:text-brand-300">
        {{ flow.userCode }}
      </div>
      <button class="btn-primary" @click="openUrl(flow.verificationUri)">
        <ExternalLink :size="16" /> {{ t("channels.loginOpen") }}
      </button>
      <p v-if="!message" class="flex items-center justify-center gap-2 text-sm text-muted">
        <Loader2 :size="14" class="animate-spin" /> {{ t("channels.loginWaiting") }}
      </p>
      <p v-else class="text-sm text-red-500">{{ message }}</p>
    </div>
  </UiModal>
</template>
