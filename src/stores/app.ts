import { defineStore } from "pinia";
import { computed, ref, shallowRef, watch } from "vue";
import { onEvent } from "../lib/events";
import { pname } from "../i18n";
import { api, clone, errorText, newId } from "../lib/api";
import type {
  AiProfile,
  Bootstrap,
  BotStatus,
  ChannelProfile,
  CompanionProfile,
  Config,
  FeedItem,
  ModLogEntry,
  ModerationProfile,
  PendingMessage,
  ProviderInfo,
} from "../lib/types";

export type View = "dashboard" | "channels" | "companion" | "moderation" | "ai" | "settings";

export interface Toast {
  id: number;
  kind: "info" | "success" | "error";
  text: string;
}

export const useApp = defineStore("app", () => {
  const ready = ref(false);
  const loadError = ref("");
  const config = ref<Config | null>(null);
  const providers = ref<ProviderInfo[]>([]);
  const version = ref("");
  const secretsBackend = ref<"os" | "file">("os");
  const dataDir = ref("");
  const os = ref("");
  const clientIdConfigured = ref(true);
  const keyStatus = ref<Record<string, boolean>>({});
  const accountStatus = ref<Record<string, boolean>>({});
  const view = ref<View>("dashboard");
  const wizardOpen = ref(false);

  const running = ref<{ channelId: string; sandbox: boolean } | null>(null);
  const status = ref<BotStatus | null>(null);
  const feed = shallowRef<FeedItem[]>([]);
  const pending = ref<PendingMessage[]>([]);
  const modEntries = ref<Map<string, ModLogEntry>>(new Map());
  const toasts = ref<Toast[]>([]);
  const saving = ref(false);

  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let suppressSave = false;
  let toastSeq = 0;

  function toast(text: string, kind: Toast["kind"] = "info", ms = 4000) {
    const id = ++toastSeq;
    toasts.value.push({ id, kind, text });
    setTimeout(() => {
      toasts.value = toasts.value.filter((t) => t.id !== id);
    }, ms);
  }

  function applyBootstrap(b: Bootstrap) {
    suppressSave = true;
    config.value = b.config;
    providers.value = b.providers;
    version.value = b.version;
    secretsBackend.value = b.secretsBackend;
    dataDir.value = b.dataDir;
    os.value = b.os;
    clientIdConfigured.value = b.clientIdConfigured;
    keyStatus.value = b.keyStatus;
    accountStatus.value = b.accountStatus;
    running.value = b.running;
    queueMicrotask(() => (suppressSave = false));
  }

  async function load() {
    try {
      applyBootstrap(await api.bootstrap());
      // The bot outlives a reload of the window.
      running.value = await api.botRunning().catch(() => null);
      ready.value = true;
      if (!config.value!.settings.wizardCompleted) wizardOpen.value = true;
      if (import.meta.env.DEV) {
        // Browser preview helpers, e.g. ?demo&view=companion
        const v = new URLSearchParams(location.search).get("view");
        if (v) view.value = v as View;
      }
    } catch (e) {
      loadError.value = errorText(e);
    }
  }

  async function refresh() {
    applyBootstrap(await api.bootstrap());
  }

  function replaceConfig(c: Config) {
    suppressSave = true;
    config.value = c;
    queueMicrotask(() => (suppressSave = false));
  }

  async function saveNow() {
    if (!config.value) return;
    clearTimeout(saveTimer);
    saving.value = true;
    try {
      await api.saveConfig(clone(config.value));
    } catch (e) {
      toast(errorText(e), "error");
    } finally {
      saving.value = false;
    }
  }

  watch(
    config,
    () => {
      if (suppressSave || !ready.value) return;
      clearTimeout(saveTimer);
      saveTimer = setTimeout(saveNow, 500);
    },
    { deep: true },
  );

  // ---------------------------------------------------------------- getters
  const settings = computed(() => config.value!.settings);
  const activeChannel = computed<ChannelProfile | undefined>(() => {
    const c = config.value;
    if (!c) return undefined;
    return c.channels.find((ch) => ch.id === c.settings.activeChannelId) ?? c.channels[0];
  });
  const isRunning = computed(() => !!running.value);

  function account(id: string | null | undefined) {
    return config.value?.accounts.find((a) => a.id === id);
  }
  function companion(id: string) {
    return config.value?.companionProfiles.find((p) => p.id === id);
  }
  function moderation(id: string) {
    return config.value?.moderationProfiles.find((p) => p.id === id);
  }
  function aiProfile(id: string) {
    return config.value?.aiProfiles.find((p) => p.id === id);
  }
  function provider(id: string) {
    return providers.value.find((p) => p.id === id);
  }
  function keyRefOf(p: AiProfile) {
    return p.keyRef.trim() || p.provider;
  }
  function hasKey(p: AiProfile) {
    const info = provider(p.provider);
    if (info && !info.needsKey) return true;
    return !!keyStatus.value[keyRefOf(p)];
  }

  // ---------------------------------------------------------------- profiles
  function uniqueName(base: string, names: string[]) {
    let name = base;
    let i = 2;
    while (names.includes(name)) name = `${base} (${i++})`;
    return name;
  }

  function duplicateCompanion(src: CompanionProfile, suffix: string): CompanionProfile {
    const p = clone(src);
    p.id = newId();
    p.builtin = false;
    p.name = uniqueName(`${pname(src)} ${suffix}`, config.value!.companionProfiles.map((x) => x.name));
    config.value!.companionProfiles.push(p);
    return p;
  }
  function duplicateModeration(src: ModerationProfile, suffix: string): ModerationProfile {
    const p = clone(src);
    p.id = newId();
    p.builtin = false;
    p.name = uniqueName(`${pname(src)} ${suffix}`, config.value!.moderationProfiles.map((x) => x.name));
    config.value!.moderationProfiles.push(p);
    return p;
  }
  function duplicateAi(src: AiProfile, suffix: string): AiProfile {
    const p = clone(src);
    p.id = newId();
    p.builtin = false;
    p.name = uniqueName(`${src.name} ${suffix}`, config.value!.aiProfiles.map((x) => x.name));
    config.value!.aiProfiles.push(p);
    return p;
  }
  function duplicateChannel(src: ChannelProfile, suffix: string): ChannelProfile {
    const p = clone(src);
    p.id = newId();
    p.name = uniqueName(`${src.name} ${suffix}`, config.value!.channels.map((x) => x.name));
    config.value!.channels.push(p);
    return p;
  }

  // ---------------------------------------------------------------- bot
  async function startBot(sandbox: boolean) {
    const ch = activeChannel.value;
    if (!ch) return;
    await saveNow();
    feed.value = [];
    pending.value = [];
    try {
      await api.botStart(ch.id, sandbox);
      running.value = { channelId: ch.id, sandbox };
    } catch (e) {
      toast(errorText(e), "error", 7000);
    }
  }

  async function stopBot() {
    await api.botStop();
    running.value = null;
    status.value = null;
    pending.value = [];
  }

  async function control(action: string, payload?: unknown) {
    try {
      await api.botControl(action, payload);
    } catch (e) {
      toast(errorText(e), "error");
    }
  }

  function pushFeed(item: FeedItem) {
    const limit = config.value?.settings.feedLimit ?? 400;
    const next = feed.value.length >= limit ? feed.value.slice(feed.value.length - limit + 1) : feed.value.slice();
    next.push(item);
    feed.value = next;
  }

  async function listenAll() {
    await onEvent<FeedItem>("hb:feed", (p) => pushFeed(p));
    await onEvent<BotStatus>("hb:status", (p) => {
      status.value = p.running ? p : null;
      if (!p.running) running.value = null;
      else if (!running.value) running.value = { channelId: p.channelId, sandbox: p.sandbox };
    });
    await onEvent<PendingMessage[]>("hb:pending", (p) => (pending.value = p));
    await onEvent<ModLogEntry>("hb:modlog", (p) => {
      const m = new Map(modEntries.value);
      m.set(p.id, p);
      modEntries.value = m;
    });
  }

  return {
    ready,
    loadError,
    config,
    providers,
    version,
    secretsBackend,
    dataDir,
    os,
    clientIdConfigured,
    keyStatus,
    accountStatus,
    view,
    wizardOpen,
    running,
    status,
    feed,
    pending,
    modEntries,
    toasts,
    saving,
    settings,
    activeChannel,
    isRunning,
    toast,
    load,
    refresh,
    replaceConfig,
    saveNow,
    account,
    companion,
    moderation,
    aiProfile,
    provider,
    keyRefOf,
    hasKey,
    duplicateCompanion,
    duplicateModeration,
    duplicateAi,
    duplicateChannel,
    startBot,
    stopBot,
    control,
    listenAll,
  };
});
