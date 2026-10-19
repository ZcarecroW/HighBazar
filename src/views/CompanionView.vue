<script setup lang="ts">
import { Bot, Smile, Clock, Tag, Globe, Heart, Bell, Chat, Brain } from "../icons";
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { useApp } from "../stores/app";
import { pdesc, pname } from "../i18n";
import type { CommandConfig, CompanionProfile } from "../lib/types";
import ProfileList from "../components/ProfileList.vue";
import BuiltinBanner from "../components/BuiltinBanner.vue";
import UiToggle from "../components/ui/UiToggle.vue";
import UiField from "../components/ui/UiField.vue";
import UiSlider from "../components/ui/UiSlider.vue";
import UiNumber from "../components/ui/UiNumber.vue";
import UiTags from "../components/ui/UiTags.vue";

const app = useApp();
const { t, tm, locale } = useI18n();
const cfg = computed(() => app.config!);

const selectedId = ref(app.activeChannel?.companionProfileId ?? cfg.value.companionProfiles[0]!.id);
watch(
  () => cfg.value.companionProfiles.map((p) => p.id),
  (ids) => {
    if (!ids.includes(selectedId.value)) selectedId.value = ids[0]!;
  },
);
const p = computed<CompanionProfile>(() => cfg.value.companionProfiles.find((x) => x.id === selectedId.value)!);
const ro = computed(() => p.value.builtin);
const isUsed = computed(() => app.activeChannel?.companionProfileId === p.value.id);

const sections = ["personality", "timing", "content", "language", "emotes", "interactions", "commands", "research"] as const;
const sectionIcons = { personality: Smile, timing: Clock, content: Tag, language: Globe, emotes: Heart, interactions: Bell, commands: Chat, research: Brain };
const section = ref<(typeof sections)[number]>("personality");

function duplicate(id = selectedId.value) {
  const src = cfg.value.companionProfiles.find((x) => x.id === id);
  if (src) selectedId.value = app.duplicateCompanion(src, t("common.copySuffix")).id;
}
function create() {
  duplicate(cfg.value.companionProfiles[0]!.id);
}
function remove(id: string) {
  const prof = cfg.value.companionProfiles.find((x) => x.id === id);
  if (!prof || prof.builtin || !confirm(t("common.confirmDelete", { name: prof.name }))) return;
  cfg.value.companionProfiles = cfg.value.companionProfiles.filter((x) => x.id !== id);
}
function useIt() {
  if (app.activeChannel) app.activeChannel.companionProfileId = p.value.id;
}

const mixKeys = ["chatReaction", "gameFact", "gameTip", "trivia", "joke", "question", "hype", "streamMoment"] as const;
const languages = ["en", "de", "es", "fr", "it", "pt", "nl", "pl", "tr", "ru", "uk", "sv", "da", "no", "fi", "cs", "hu", "ro", "el", "ja", "ko", "zh", "ar", "hi", "th", "vi", "id"];
const langName = (code: string) => {
  try {
    return new Intl.DisplayNames([locale.value], { type: "language" }).of(code) ?? code;
  } catch {
    return code;
  }
};
const roles = ["everyone", "subscriber", "vip", "moderator", "broadcaster"] as const;
const commands: { key: "ask" | "fact" | "joke" | "about"; label: string }[] = [
  { key: "ask", label: "companion.cmdAsk" },
  { key: "fact", label: "companion.cmdFact" },
  { key: "joke", label: "companion.cmdJoke" },
  { key: "about", label: "companion.cmdAbout" },
];
function cmd(key: "ask" | "fact" | "joke" | "about"): CommandConfig {
  return p.value.commands[key];
}

const fmtSecs = (s: number) => (s >= 60 ? `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")} min` : `${s}s`);
const emoteLevels = computed(() => tm("companion.emoteLevels") as unknown as string[]);
</script>

<template>
  <div class="mx-auto max-w-[1500px] space-y-5 p-4 xl:p-6">
    <header>
      <h1 class="flex items-center gap-2 text-xl font-bold"><Bot :size="22" class="text-brand-500" /> {{ t("companion.title") }}</h1>
      <p class="mt-1 text-sm text-muted">{{ t("companion.intro") }}</p>
    </header>

    <div class="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]">
      <ProfileList
        v-model="selectedId"
        :items="cfg.companionProfiles"
        :active-id="app.activeChannel?.companionProfileId"
        @new="create"
        @duplicate="duplicate"
        @delete="remove"
      />

      <div v-if="p" class="min-w-0 space-y-4">
        <BuiltinBanner v-if="ro" @duplicate="duplicate()" />

        <div class="card-pad flex flex-wrap items-start gap-4">
          <UiField :label="t('common.name')" class="min-w-60 flex-1">
            <input v-if="!ro" v-model="p.name" class="input" />
            <input v-else :value="pname(p)" class="input" disabled />
          </UiField>
          <UiField :label="t('common.description')" class="min-w-72 flex-[2]">
            <input v-if="!ro" v-model="p.description" class="input" />
            <input v-else :value="pdesc(p)" class="input" disabled />
          </UiField>
          <div class="self-end">
            <span v-if="isUsed" class="chip bg-emerald-500/15 py-1 text-emerald-600">{{ t("common.active") }}</span>
            <button v-else class="btn" @click="useIt">{{ t("common.setActive") }}</button>
          </div>
        </div>

        <!-- Section tabs -->
        <div class="flex flex-wrap gap-1 rounded-xl border border-line bg-panel p-1">
          <button
            v-for="s in sections"
            :key="s"
            class="inline-flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition"
            :class="section === s ? 'bg-brand-500 text-white' : 'text-muted hover:bg-panel-2 hover:text-fg'"
            @click="section = s"
          >
            <component :is="sectionIcons[s]" :size="15" />
            {{ t(`companion.sections.${s}`) }}
          </button>
        </div>

        <fieldset :disabled="ro" class="card-pad min-w-0">
          <!-- Personality -->
          <div v-if="section === 'personality'" class="space-y-5">
            <UiField :label="t('companion.persona')" :hint="t('companion.personaHint')">
              <textarea v-model="p.persona" rows="3" class="input" />
            </UiField>
            <div class="grid gap-5 md:grid-cols-3">
              <UiSlider v-model="p.humor" :label="t('companion.humor')" :disabled="ro" />
              <UiSlider v-model="p.energy" :label="t('companion.energy')" :disabled="ro" />
              <UiSlider v-model="p.snark" :label="t('companion.snark')" :disabled="ro" />
            </div>
            <UiSlider v-model="p.maxLength" :label="t('companion.maxLength')" :min="60" :max="480" :step="10" :suffix="' ' + t('common.chars')" :disabled="ro" />
            <UiField :label="t('companion.customInstructions')" :hint="t('companion.customInstructionsHint')">
              <textarea v-model="p.customInstructions" rows="3" class="input" />
            </UiField>
            <UiField :label="t('companion.avoidTopics')">
              <textarea v-model="p.avoidTopics" rows="2" class="input" />
            </UiField>
          </div>

          <!-- Timing -->
          <div v-else-if="section === 'timing'" class="space-y-5">
            <UiToggle v-model="p.intervalEnabled" :label="t('companion.intervalEnabled')" :disabled="ro" />
            <div class="grid gap-5 md:grid-cols-2">
              <UiSlider
                v-model="p.intervalMinSecs"
                :label="`${t('companion.interval')} – ${t('companion.from')}`"
                :min="30"
                :max="1800"
                :step="15"
                :display="fmtSecs(p.intervalMinSecs)"
                :disabled="ro || !p.intervalEnabled"
                @update:model-value="(v: number) => { if (p.intervalMaxSecs < v) p.intervalMaxSecs = v }"
              />
              <UiSlider
                v-model="p.intervalMaxSecs"
                :label="`${t('companion.interval')} – ${t('companion.to')}`"
                :min="30"
                :max="3600"
                :step="15"
                :display="fmtSecs(p.intervalMaxSecs)"
                :disabled="ro || !p.intervalEnabled"
                @update:model-value="(v: number) => { if (p.intervalMinSecs > v) p.intervalMinSecs = v }"
              />
            </div>
            <p class="hint">{{ t("companion.intervalHint") }}</p>
            <div class="grid gap-5 md:grid-cols-2">
              <UiToggle v-model="p.onlyWhenLive" :label="t('companion.onlyWhenLive')" :disabled="ro" />
              <UiToggle v-model="p.adaptive" :label="t('companion.adaptive')" :hint="t('companion.adaptiveHint')" :disabled="ro" />
              <UiNumber v-model="p.busyThreshold" :label="t('companion.busyThreshold')" :min="0" :max="1000" :disabled="ro || !p.adaptive" />
              <UiNumber v-model="p.maxUnanswered" :label="t('companion.maxUnanswered')" :hint="t('companion.maxUnansweredHint')" :min="0" :max="50" :disabled="ro" />
              <UiSlider v-model="p.contextMessages" :label="t('companion.contextMessages')" :min="0" :max="200" :step="5" :disabled="ro" />
              <UiSlider v-model="p.contextMaxAgeMins" :label="t('companion.contextAge')" :min="1" :max="120" :suffix="' min'" :disabled="ro" />
            </div>
          </div>

          <!-- Content mix -->
          <div v-else-if="section === 'content'" class="space-y-4">
            <p class="hint mt-0">{{ t("companion.mixHint") }}</p>
            <div class="grid gap-x-8 gap-y-4 md:grid-cols-2">
              <UiSlider v-for="k in mixKeys" :key="k" v-model="p.mix[k]" :label="t(`companion.mix.${k}`)" :min="0" :max="10" :disabled="ro" />
            </div>
          </div>

          <!-- Language -->
          <div v-else-if="section === 'language'" class="grid gap-5 md:grid-cols-2">
            <UiField :label="t('companion.languageMode')">
              <select v-model="p.language.mode" class="input">
                <option v-for="m in ['stream', 'fixed', 'chat']" :key="m" :value="m">{{ t(`companion.languageModes.${m}`) }}</option>
              </select>
            </UiField>
            <UiField :label="t('companion.fixedLanguage')">
              <select v-model="p.language.fixed" class="input">
                <option v-for="l in languages" :key="l" :value="l">{{ langName(l) }}</option>
              </select>
            </UiField>
            <UiToggle v-model="p.language.replyInUserLanguage" :label="t('companion.replyInUserLanguage')" :disabled="ro" />
            <UiToggle v-model="p.language.allowSlang" :label="t('companion.allowSlang')" :disabled="ro" />
          </div>

          <!-- Emotes -->
          <div v-else-if="section === 'emotes'" class="space-y-5">
            <UiSlider
              v-model="p.emoteLevel"
              :label="t('companion.emoteLevel')"
              :min="0"
              :max="3"
              :display="emoteLevels[p.emoteLevel]"
              :disabled="ro"
            />
            <div>
              <span class="label">{{ t("companion.emoteSources") }}</span>
              <div class="grid gap-3 sm:grid-cols-4">
                <UiToggle v-model="p.emotes.twitch" label="Twitch" :disabled="ro" />
                <UiToggle v-model="p.emotes.bttv" label="BetterTTV" :disabled="ro" />
                <UiToggle v-model="p.emotes.ffz" label="FrankerFaceZ" :disabled="ro" />
                <UiToggle v-model="p.emotes.seventv" label="7TV" :disabled="ro" />
              </div>
            </div>
            <div class="grid gap-5 md:grid-cols-2">
              <UiToggle v-model="p.emotes.preferPopular" :label="t('companion.preferPopular')" :disabled="ro" />
              <UiNumber v-model="p.emotes.maxPerMessage" :label="t('companion.maxEmotes')" :min="0" :max="10" :disabled="ro" />
            </div>
          </div>

          <!-- Interactions -->
          <div v-else-if="section === 'interactions'" class="space-y-6">
            <div class="space-y-4">
              <UiToggle v-model="p.mentions.enabled" :disabled="ro"><span class="font-semibold">{{ t("companion.mentions") }}</span></UiToggle>
              <div class="grid gap-5 md:grid-cols-2" :class="!p.mentions.enabled && 'opacity-60'">
                <UiTags v-model="p.mentions.triggerWords" :label="t('companion.triggerWords')" :hint="t('companion.triggerWordsHint')" :disabled="ro" />
                <UiToggle v-model="p.mentions.replies" :label="t('companion.replies')" :disabled="ro" />
                <UiNumber v-model="p.mentions.userCooldownSecs" :label="t('companion.userCooldown')" :suffix="t('common.seconds')" :max="3600" :disabled="ro" />
                <UiNumber v-model="p.mentions.globalCooldownSecs" :label="t('companion.globalCooldown')" :suffix="t('common.seconds')" :max="3600" :disabled="ro" />
                <UiField :label="t('companion.minRole')">
                  <select v-model="p.mentions.minRole" class="input">
                    <option v-for="r in roles" :key="r" :value="r">{{ t(`roles.${r}`) }}</option>
                  </select>
                </UiField>
              </div>
            </div>
            <hr class="border-line" />
            <div class="space-y-4">
              <UiToggle v-model="p.welcome.enabled" :disabled="ro"><span class="font-semibold">{{ t("companion.welcome") }}</span></UiToggle>
              <div class="grid gap-5 md:grid-cols-2" :class="!p.welcome.enabled && 'opacity-60'">
                <UiField :label="t('companion.welcomeScope')" :hint="p.welcome.scope === 'firstEver' ? t('companion.welcomeLearning') : ''">
                  <select v-model="p.welcome.scope" class="input">
                    <option value="firstEver">{{ t("companion.welcomeScopes.firstEver") }}</option>
                    <option value="firstThisStream">{{ t("companion.welcomeScopes.firstThisStream") }}</option>
                  </select>
                </UiField>
                <UiField :label="t('companion.welcomeInstructions')">
                  <input v-model="p.welcome.instructions" class="input" />
                </UiField>
                <UiNumber v-model="p.welcome.delaySecs" :label="t('companion.welcomeDelay')" :suffix="t('common.seconds')" :max="120" :disabled="ro" />
                <UiNumber v-model="p.welcome.maxPerMinute" :label="t('companion.welcomeMax')" :min="1" :max="30" :disabled="ro" />
              </div>
            </div>
            <hr class="border-line" />
            <div class="space-y-4">
              <span class="font-semibold">{{ t("companion.events") }}</span>
              <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                <UiToggle v-model="p.events.follow" :label="t('companion.eventFollow')" :disabled="ro" />
                <UiToggle v-model="p.events.sub" :label="t('companion.eventSub')" :disabled="ro" />
                <UiToggle v-model="p.events.resub" :label="t('companion.eventResub')" :disabled="ro" />
                <UiToggle v-model="p.events.gift" :label="t('companion.eventGift')" :disabled="ro" />
                <UiToggle v-model="p.events.raid" :label="t('companion.eventRaid')" :disabled="ro" />
                <UiToggle v-model="p.events.cheer" :label="t('companion.eventCheer')" :disabled="ro" />
                <UiToggle v-model="p.events.watchStreak" :label="t('companion.eventWatchStreak')" :disabled="ro" />
              </div>
              <div class="grid gap-5 md:grid-cols-2">
                <UiNumber v-model="p.events.minCheerBits" :label="t('companion.minBits')" :min="1" :max="100000" :disabled="ro" />
                <UiNumber v-model="p.events.followBatchSecs" :label="t('companion.followBatch')" :suffix="t('common.seconds')" :min="0" :max="600" :disabled="ro" />
              </div>
            </div>
          </div>

          <!-- Commands -->
          <div v-else-if="section === 'commands'" class="space-y-5">
            <div class="flex flex-wrap items-end gap-5">
              <UiToggle v-model="p.commands.enabled" :label="t('companion.commandsEnabled')" :disabled="ro" />
              <UiField :label="t('companion.commandPrefix')">
                <input v-model="p.commands.prefix" class="input w-32 font-mono" />
              </UiField>
            </div>
            <div class="overflow-x-auto">
              <table class="w-full min-w-[640px] text-sm">
                <thead>
                  <tr class="text-left text-xs text-muted uppercase">
                    <th class="py-2 pr-3 font-semibold"></th>
                    <th class="py-2 pr-3 font-semibold">Command</th>
                    <th class="py-2 pr-3 font-semibold">{{ t("companion.userCooldown") }}</th>
                    <th class="py-2 pr-3 font-semibold">{{ t("companion.globalCooldown") }}</th>
                    <th class="py-2 font-semibold">{{ t("companion.minRole") }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="c in commands" :key="c.key" class="border-t border-line">
                    <td class="py-2 pr-3"><UiToggle v-model="cmd(c.key).enabled" :disabled="ro" /></td>
                    <td class="py-2 pr-3">
                      <code class="font-mono text-brand-600 dark:text-brand-300">{{ p.commands.prefix }} {{ c.key }}</code>
                      <div class="text-xs text-muted">{{ t(c.label) }}</div>
                    </td>
                    <td class="py-2 pr-3"><UiNumber v-model="cmd(c.key).userCooldownSecs" :suffix="t('common.seconds')" :max="86400" :disabled="ro" /></td>
                    <td class="py-2 pr-3"><UiNumber v-model="cmd(c.key).globalCooldownSecs" :suffix="t('common.seconds')" :max="86400" :disabled="ro" /></td>
                    <td class="py-2">
                      <select v-model="cmd(c.key).minRole" class="input">
                        <option v-for="r in roles" :key="r" :value="r">{{ t(`roles.${r}`) }}</option>
                      </select>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Research -->
          <div v-else-if="section === 'research'" class="space-y-5">
            <UiToggle v-model="p.research.enabled" :label="t('companion.researchEnabled')" :hint="t('companion.researchHint')" :disabled="ro" />
            <div class="grid gap-5 md:grid-cols-3" :class="!p.research.enabled && 'opacity-60'">
              <UiToggle v-model="p.research.webSearch" :label="t('companion.webSearch')" :disabled="ro" />
              <UiToggle v-model="p.research.wikipedia" :label="t('companion.wikipedia')" :disabled="ro" />
              <UiNumber v-model="p.research.refreshDays" :label="t('companion.refreshDays')" :min="1" :max="365" :disabled="ro" />
            </div>
          </div>
        </fieldset>
      </div>
    </div>
  </div>
</template>
