// Mirrors of the Rust types (serde camelCase).

export interface Config {
  version: number;
  settings: AppSettings;
  accounts: TwitchAccount[];
  channels: ChannelProfile[];
  companionProfiles: CompanionProfile[];
  moderationProfiles: ModerationProfile[];
  aiProfiles: AiProfile[];
}

export type ChannelLevel = "simple" | "advanced" | "expert";

export interface AppSettings {
  theme: "system" | "dark" | "light";
  colorTheme: string;
  language: "auto" | "en" | "de";
  checkUpdatesOnStart: boolean;
  wizardCompleted: boolean;
  activeChannelId: string | null;
  startBotOnLaunch: boolean;
  twitchClientId: string;
  feedLimit: number;
  showEmotes: boolean;
  channelManagerLevel: ChannelLevel;
  /** Size of the live chat (1 = 100 %), independent of uiZoom. */
  chatZoom: number;
  /** Size of the rest of the app (1 = 100 %). */
  uiZoom: number;
  chatScroll: ChatScroll;
  /** Run with "below normal" priority so games get the CPU first. */
  lowPriority: boolean;
  /** Minimizing or closing closes the whole UI; the bot keeps running in the tray. */
  performanceMode: boolean;
  startInTray: boolean;
  overlay: OverlaySettings;
  /** HighBazar's own web search for AI providers without a built-in one. */
  webSearch: WebSearchSettings;
  chatInput: ChatInputSettings;
}

export interface ChatInputSettings {
  enabled: boolean;
  /** Emote names / emojis pinned to the top of the suggestions. */
  favorites: string[];
}

export type WebSearchSource = "youcom" | "wikipedia" | "exa" | "serper" | "tavily" | "brave" | "ollama" | "searxng";

export interface WebSearchSettings {
  source: WebSearchSource;
  searxngUrl: string;
  maxResults: number;
}

export interface SearchHit {
  title: string;
  url: string;
  snippet: string;
}

export interface CliStatus {
  path: string | null;
  /** e.g. "Logged in using ChatGPT"; null = not logged in. */
  login: string | null;
  error: string;
}

export interface TwitchAccount {
  id: string;
  login: string;
  displayName: string;
  profileImageUrl: string;
  scopes: string[];
}

export interface ChannelProfile {
  id: string;
  name: string;
  streamerAccountId: string | null;
  botAccountId: string | null;
  botName: string;
  prefixEnabled: boolean;
  prefix: string;
  companionEnabled: boolean;
  moderationEnabled: boolean;
  companionProfileId: string;
  moderationProfileId: string;
  aiProfileId: string;
  moderationAiProfileId: string;
  sendMode: "auto" | "preview" | "approve";
  previewDelaySecs: number;
  aboutStream: string;
  dailyBudgetUsd: number;
  translation: TranslationSettings;
}

export interface TranslationSettings {
  enabled: boolean;
  /** Languages the streamer understands (ISO 639-1); the first is the translation target. */
  languages: string[];
  noteOnDelete: boolean;
}

export interface ChatTranslation {
  lang: string;
  text: string;
}

export interface ContentMix {
  chatReaction: number;
  gameFact: number;
  gameTip: number;
  trivia: number;
  joke: number;
  question: number;
  hype: number;
  streamMoment: number;
}

export interface CommandConfig {
  enabled: boolean;
  userCooldownSecs: number;
  globalCooldownSecs: number;
  minRole: Role;
}

export type Role = "everyone" | "subscriber" | "vip" | "moderator" | "broadcaster";

export interface CompanionProfile {
  id: string;
  name: string;
  description: string;
  builtin: boolean;
  persona: string;
  humor: number;
  energy: number;
  snark: number;
  emoteLevel: number;
  customInstructions: string;
  avoidTopics: string;
  maxLength: number;
  intervalEnabled: boolean;
  intervalMinSecs: number;
  intervalMaxSecs: number;
  onlyWhenLive: boolean;
  adaptive: boolean;
  busyThreshold: number;
  maxUnanswered: number;
  contextMessages: number;
  contextMaxAgeMins: number;
  mix: ContentMix;
  language: {
    mode: "stream" | "fixed" | "chat";
    fixed: string;
    replyInUserLanguage: boolean;
    allowSlang: boolean;
  };
  emotes: {
    twitch: boolean;
    bttv: boolean;
    ffz: boolean;
    seventv: boolean;
    preferPopular: boolean;
    maxPerMessage: number;
  };
  mentions: {
    enabled: boolean;
    triggerWords: string[];
    replies: boolean;
    userCooldownSecs: number;
    globalCooldownSecs: number;
    minRole: Role;
  };
  welcome: {
    enabled: boolean;
    scope: "firstEver" | "firstThisStream";
    delaySecs: number;
    maxPerMinute: number;
    instructions: string;
  };
  events: {
    follow: boolean;
    sub: boolean;
    resub: boolean;
    gift: boolean;
    raid: boolean;
    cheer: boolean;
    watchStreak: boolean;
    minCheerBits: number;
    followBatchSecs: number;
  };
  commands: {
    enabled: boolean;
    prefix: string;
    ask: CommandConfig;
    fact: CommandConfig;
    joke: CommandConfig;
    about: CommandConfig;
  };
  research: {
    enabled: boolean;
    webSearch: boolean;
    /** Look up current facts when chat asks the bot something. */
    answerSearch: boolean;
    wikipedia: boolean;
    refreshDays: number;
  };
}

export type ModActionKind = "none" | "log" | "delete" | "warn" | "timeout" | "ban";

export interface ModAction {
  kind: ModActionKind;
  durationSecs: number;
}

export interface CategoryRule {
  category: string;
  enabled: boolean;
  clearAction: ModAction;
  borderlineAction: ModAction;
}

export interface ModerationProfile {
  id: string;
  name: string;
  description: string;
  builtin: boolean;
  mode: "off" | "log" | "suggest" | "auto";
  aiCheck: "off" | "suspicious" | "all";
  strictness: number;
  minConfidence: number;
  maxAiCallsPerMin: number;
  channelRules: string;
  exempt: { moderators: boolean; vips: boolean; subscribers: boolean; users: string[] };
  categories: CategoryRule[];
  escalation: { enabled: boolean; windowMins: number; steps: ModAction[] };
  rules: {
    blockedTerms: string[];
    blockedTermsAction: ModAction;
    scamPhrasesEnabled: boolean;
    scamPhrasesAction: ModAction;
    links: "allow" | "block";
    allowedDomains: string[];
    linksAction: ModAction;
    capsEnabled: boolean;
    capsMinLength: number;
    capsRatio: number;
    capsAction: ModAction;
    repeatEnabled: boolean;
    repeatCount: number;
    repeatAction: ModAction;
    emoteSpamEnabled: boolean;
    maxEmotes: number;
    emoteSpamAction: ModAction;
    maxLength: number;
    lengthAction: ModAction;
  };
  reasonTemplate: string;
  announceActions: boolean;
}

export interface HeaderPair {
  name: string;
  value: string;
}

export interface AiProfile {
  id: string;
  name: string;
  description: string;
  builtin: boolean;
  provider: string;
  baseUrl: string;
  model: string;
  keyRef: string;
  maxOutputTokens: number;
  temperature: number | null;
  reasoning: string;
  webSearch: boolean;
  timeoutSecs: number;
  priceInputPerMtok: number;
  priceOutputPerMtok: number;
  extraHeaders: HeaderPair[];
  openaiResponses: boolean;
  geminiInteractions: boolean;
  cliPath: string;
}

export interface ProviderInfo {
  id: string;
  name: string;
  baseUrl: string;
  needsKey: boolean;
  webSearch: boolean;
  keyUrl: string;
  local: boolean;
  reasoningOptions: string[];
}

export interface Bootstrap {
  config: Config;
  providers: ProviderInfo[];
  version: string;
  secretsBackend: "os" | "file";
  dataDir: string;
  clientIdConfigured: boolean;
  keyStatus: Record<string, boolean>;
  accountStatus: Record<string, boolean>;
  running: { channelId: string; sandbox: boolean } | null;
  os: string;
}

export interface FeedItem {
  id: string;
  ts: number;
  kind: "chat" | "bot" | "mod" | "event" | "system" | "error";
  user: string;
  login: string;
  userId: string;
  messageId: string;
  color: string;
  badges: string[];
  text: string;
  meta: any;
}

export interface BotStatus {
  running: boolean;
  sandbox: boolean;
  connected: boolean;
  channelId: string;
  channelLogin: string;
  channelName: string;
  senderLogin: string;
  live: boolean;
  game: string;
  title: string;
  language: string;
  viewers: number;
  startedAt: string | null;
  nextMessageAt: number | null;
  pausedReason: string;
  chatRate: number;
  emoteCount: number;
  knowledgeItems: number;
  researching: boolean;
  costToday: number;
  budget: number;
  messagesToday: number;
  lastError: string;
}

export interface PendingMessage {
  id: string;
  text: string;
  reason: string;
  created: number;
  sendAt: number | null;
}

export interface ModLogEntry {
  id: string;
  ts: number;
  userId: string;
  login: string;
  displayName: string;
  messageId: string;
  text: string;
  category: string;
  verdict: string;
  confidence: number;
  reason: string;
  source: string;
  action: ModAction;
  status: string;
  error: string;
  sourceLang?: string;
  translation?: string;
  meaning?: string;
}

export interface UsageStats {
  calls: number;
  inputTokens: number;
  outputTokens: number;
  costUsd: number;
  messagesSent: number;
  modActions: number;
  webSearches: number;
}

export interface LoginStart {
  loginId: string;
  userCode: string;
  verificationUri: string;
  expiresIn: number;
}

export interface LoginEvent {
  loginId: string;
  status: "done" | "error" | "expired" | "cancelled";
  account: TwitchAccount | null;
  error: string;
}

export interface AiTestResult {
  text: string;
  inputTokens: number;
  outputTokens: number;
  costUsd: number;
  millis: number;
  model: string;
  searches: number;
}

export interface ModelInfo {
  id: string;
  /** Display name from the provider; empty = use the id. */
  name: string;
  /** USD per million tokens, when the provider publishes prices (OpenRouter). */
  priceIn?: number | null;
  priceOut?: number | null;
}

export type ScrollEasing = "easeInOut" | "easeOut" | "linear" | "spring";

export interface ChatScroll {
  /** false = new messages appear instantly. */
  smooth: boolean;
  durationMs: number;
  easing: ScrollEasing;
}

export interface OverlaySettings {
  enabled: boolean;
  port: number;
  /** Random part of the overlay URL. */
  key: string;
  showBot: boolean;
  showEvents: boolean;
  showTranslations: boolean;
  /** Fade out after this many seconds (0 = stay). */
  hideAfterSecs: number;
  /** Background opacity 0–1. */
  background: number;
  /** Input field to write in chat from the overlay page. */
  chatInput: boolean;
}

export interface OverlayStatus {
  running: boolean;
  url: string;
  error: string;
  clients: number;
}
