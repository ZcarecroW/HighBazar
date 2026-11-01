// Browser-preview mock of the Tauri backend (dev only, `npm run dev` in a
// normal browser). Never included in production builds.
import cfgJson from "./mock-config.json";
import { mockEmit } from "./events";
import { scopesFor } from "./twitch";
import pkg from "../../package.json";
import type { BotStatus, Config, FeedItem, ModLogEntry, PendingMessage, ProviderInfo } from "./types";
import type { Suggestion } from "./chatInput";

const providers: ProviderInfo[] = [
  { id: "anthropic", name: "Anthropic (Claude)", baseUrl: "https://api.anthropic.com", needsKey: true, webSearch: true, keyUrl: "https://platform.claude.com/settings/keys", local: false, reasoningOptions: ["", "low", "medium", "high", "xhigh", "max"] },
  { id: "openai", name: "OpenAI", baseUrl: "https://api.openai.com/v1", needsKey: true, webSearch: true, keyUrl: "https://platform.openai.com/api-keys", local: false, reasoningOptions: ["", "none", "low", "medium", "high", "xhigh", "max"] },
  { id: "gemini", name: "Google Gemini", baseUrl: "https://generativelanguage.googleapis.com/v1beta", needsKey: true, webSearch: true, keyUrl: "https://aistudio.google.com/apikey", local: false, reasoningOptions: ["", "minimal", "low", "medium", "high"] },
  { id: "openrouter", name: "OpenRouter", baseUrl: "https://openrouter.ai/api/v1", needsKey: true, webSearch: true, keyUrl: "https://openrouter.ai/settings/keys", local: false, reasoningOptions: ["", "low", "medium", "high"] },
  { id: "mistral", name: "Mistral AI", baseUrl: "https://api.mistral.ai/v1", needsKey: true, webSearch: false, keyUrl: "", local: false, reasoningOptions: [""] },
  { id: "deepseek", name: "DeepSeek", baseUrl: "https://api.deepseek.com/v1", needsKey: true, webSearch: false, keyUrl: "", local: false, reasoningOptions: ["", "none", "low", "high", "max"] },
  { id: "perplexity", name: "Perplexity", baseUrl: "https://api.perplexity.ai", needsKey: true, webSearch: true, keyUrl: "", local: false, reasoningOptions: [""] },
  { id: "ollama", name: "Ollama (local)", baseUrl: "http://localhost:11434/v1", needsKey: false, webSearch: false, keyUrl: "", local: true, reasoningOptions: [""] },
  { id: "lmstudio", name: "LM Studio (local)", baseUrl: "http://localhost:1234/v1", needsKey: false, webSearch: false, keyUrl: "", local: true, reasoningOptions: [""] },
  { id: "claudeCli", name: "Claude Code CLI (subscription)", baseUrl: "", needsKey: false, webSearch: true, keyUrl: "", local: true, reasoningOptions: ["", "low", "medium", "high"] },
  { id: "codexCli", name: "Codex CLI (ChatGPT subscription)", baseUrl: "", needsKey: false, webSearch: true, keyUrl: "https://learn.chatgpt.com/docs/codex-cli", local: true, reasoningOptions: ["", "low", "medium", "high", "xhigh", "max", "ultra"] },
];

let config: Config = JSON.parse(JSON.stringify(cfgJson));
config.accounts = [{ id: "1001", login: "zcarecrow", displayName: "ZcarecroW", profileImageUrl: "", scopes: [] }];
config.channels[0]!.name = "ZcarecroW";
config.channels[0]!.streamerAccountId = "1001";
config.channels[0]!.aboutStream = "Horror & souls-like games, cozy community called 'Krähen'.";
config.channels[0]!.translation ??= { enabled: true, languages: ["de", "en"], noteOnDelete: true, postToChat: true, chatPrefix: "[HighBazar][Translator]" };
const params = new URLSearchParams(location.search);
config.settings.wizardCompleted = !params.has("wizard");
if (params.get("theme") === "light" || params.get("theme") === "dark") config.settings.theme = params.get("theme") as "light" | "dark";
if (params.get("lang") === "de" || params.get("lang") === "en") config.settings.language = params.get("lang") as "de" | "en";
if (params.get("color")) config.settings.colorTheme = params.get("color")!;
if (params.get("level")) config.settings.channelManagerLevel = params.get("level") as "simple" | "advanced" | "expert";
config.accounts[0]!.scopes = scopesFor("expert");

// A few real emotes (images from the providers' CDNs) for the feed and the chat input.
const tw = (id: number) => `https://static-cdn.jtvnw.net/emoticons/v2/${id}/default/dark/1.0`;
const ffz = (id: number) => `https://cdn.frankerfacez.com/emote/${id}/1`;
const demoEmotes: Suggestion[] = [
  { insert: "Pog", label: "Pog", kind: "emote", provider: "ffz", url: ffz(210748), scope: "channel", favorite: true },
  { insert: "PogChamp", label: "PogChamp", kind: "emote", provider: "twitch", url: tw(305954156), scope: "global", favorite: false },
  { insert: "POGGERS", label: "POGGERS", kind: "emote", provider: "ffz", url: ffz(214129), scope: "channel", favorite: false },
  { insert: "KEKW", label: "KEKW", kind: "emote", provider: "ffz", url: ffz(381875), scope: "channel", favorite: true },
  { insert: "OMEGALUL", label: "OMEGALUL", kind: "emote", provider: "ffz", url: ffz(128054), scope: "global", favorite: false },
  { insert: "catJam", label: "catJam", kind: "emote", provider: "7tv", url: "https://cdn.7tv.app/emote/01F6ME9FRG0005TFYTWP1H8R42/1x.webp", scope: "channel", favorite: false },
  { insert: "Kappa", label: "Kappa", kind: "emote", provider: "twitch", url: tw(25), scope: "global", favorite: false },
  { insert: "Kreygasm", label: "Kreygasm", kind: "emote", provider: "twitch", url: tw(41), scope: "global", favorite: false },
];
const demoPeople = ["Mira_plays", "kraehe_42", "NightOwlTV", "pixelpanda"];

function demoSuggest(query: string, mode: string, limit: number): Suggestion[] {
  const q = query.toLowerCase().replace(/^[7bfte] /, "");
  if (mode === "mention") {
    return demoPeople.filter((n) => n.toLowerCase().includes(q)).map((n) => ({ insert: `@${n}`, label: n, kind: "user", provider: "user", url: "", scope: "", favorite: false }));
  }
  return demoEmotes.filter((e) => mode === "browse" || e.label.toLowerCase().includes(q)).slice(0, limit);
}

let running: { channelId: string; sandbox: boolean } | null = null;
let seq = 0;
const now = () => Date.now();

function item(kind: FeedItem["kind"], text: string, extra: Partial<FeedItem> = {}): FeedItem {
  return { id: String(++seq), ts: now(), kind, user: "", login: "", userId: "", messageId: "m" + seq, color: "", badges: [], text, meta: null, ...extra };
}

const status = (): BotStatus => ({
  running: true,
  sandbox: running?.sandbox ?? false,
  connected: true,
  channelId: config.channels[0]!.id,
  channelLogin: "zcarecrow",
  channelName: "ZcarecroW",
  senderLogin: "zcarecrow",
  live: true,
  game: "Elden Ring Nightreign",
  title: "Blind run – no summons! !hb",
  language: "de",
  viewers: 37,
  startedAt: new Date(now() - 83 * 60000).toISOString(),
  nextMessageAt: now() + 142000,
  pausedReason: "",
  chatRate: 9,
  emoteCount: 812,
  knowledgeItems: 23,
  researching: false,
  costToday: 0.184,
  budget: 2,
  messagesToday: 41,
  lastError: "",
});

function demoFeed() {
  const chat = (user: string, color: string, text: string, badges: string[] = [], meta: any = null) =>
    mockEmit("hb:feed", item("chat", text, { user, login: user.toLowerCase(), userId: user, color, badges, meta }));
  const bot = (text: string, reason: string) => mockEmit("hb:feed", item("bot", text, { user: "HighBazar", meta: { reason } }));
  mockEmit("hb:feed", item("system", "", { meta: { key: "connected", params: { channel: "zcarecrow" } } }));
  mockEmit("hb:feed", item("system", "", { meta: { key: "researched", params: { game: "Elden Ring Nightreign", facts: 15, tips: 8 } } }));
  // German with ?lang=de, English otherwise (the documentation screenshots).
  const de = config.settings.language === "de";
  const say = (en: string, deText: string) => (de ? deText : en);
  chat("NightOwl92", "#1E90FF", "this boss is actually insane monkaS", ["subscriber"]);
  chat("Kraehenmama", "#FF7F50", say("you got this!! Pog", "du schaffst das!! Pog"), ["vip"]);
  bot(say("Fun fact: this boss has a hidden weakness to bleed damage – maybe time for a dagger? KEKW", "Fun Fact: Der Boss hat eine versteckte Schwäche gegen Blutungsschaden – vielleicht Zeit für ein Messer? KEKW"), "timed");
  chat("lurker_lena", "#9ACD32", say("hi, I'm new here :)", "hi, bin neu hier :)"), [], { firstMsg: true });
  bot(say("@lurker_lena Welcome to the Crows! Make yourself at home – this boss is eating souls for breakfast right now Sadge", "@lurker_lena Willkommen bei den Krähen! Mach's dir gemütlich – der Boss hier frisst gerade Seelen zum Frühstück Sadge"), "welcome");
  mockEmit("hb:feed", item("event", "", { meta: { key: "raid", params: { user: "PixelPirat", viewers: 23 } } }));
  bot(say("@PixelPirat brings 23 raiders! Welcome, legends – you're just in time for the boss fight catJam", "@PixelPirat bringt 23 Raider mit! Willkommen, ihr Legenden – ihr kommt genau rechtzeitig zum Bossfight catJam"), "event");
  chat("BackseatBernd", "#DAA520", say("HighBazar which weapon is best here?", "HighBazar welche waffe ist die beste hier?"));
  bot(say("@BackseatBernd Fast weapons with bleed are the favorite against this one – but honestly, skill > weapon Kappa", "@BackseatBernd Gegen den hier schwören viele auf schnelle Waffen mit Blutung – aber ganz ehrlich, Skill > Waffe Kappa"), "reply");
  for (const entry of demoModlog()) {
    mockEmit("hb:modlog", entry);
    if (entry.status === "executed") mockEmit("hb:feed", item("mod", "", { user: entry.displayName, meta: entry }));
  }
  chat("NightOwl92", "#1E90FF", "LETS GOOOO", ["subscriber"]);
  const pend: PendingMessage[] = [
    { id: "p1", text: say("Who here actually beat this boss on the first try? Honest answers only 👀", "Wer von euch hat den Boss eigentlich schon beim ersten Versuch gelegt? Ehrliche Antworten only 👀"), reason: "timed", created: now(), sendAt: now() + 8000 },
  ];
  mockEmit("hb:pending", pend);
}

function demoModlog(): ModLogEntry[] {
  const de = config.settings.language === "de";
  const e = (id: string, mins: number, name: string, text: string, category: string, verdict: string, confidence: number, reason: string, source: string, kind: ModLogEntry["action"]["kind"], secs: number, status: string): ModLogEntry => ({
    id, ts: now() - mins * 60000, userId: name, login: name.toLowerCase(), displayName: name, messageId: `m-${id}`, text, category, verdict, confidence, reason, source, action: { kind, durationSecs: secs }, status, error: "",
  });
  return [
    e("mod4", 31, "spam_bot_4821", "cheap viewers and followers at strea…", "spam", "violation", 1, de ? "bekannte Spam-Phrase" : "known spam phrase", "rule:scam", "ban", 0, "executed"),
    e("mod3", 18, "CapsCarl", "WHY IS NOBODY TALKING ABOUT THE LORE", "spam", "violation", 1, de ? "zu viele Großbuchstaben" : "excessive caps", "rule:caps", "delete", 0, "executed"),
    e("mod2", 9, "edgy_eddie", "you play like my grandma lol", "harassment", "borderline", 0.55, de ? "Frotzelei, eher Spaß" : "banter, most likely a joke", "ai", "log", 0, "logged"),
    e("mod1", 2, "toxic_tim", "[removed insult]", "harassment", "violation", 0.94, de ? "Beleidigung gegen den Streamer" : "insult aimed at the streamer", "ai", "timeout", 600, "executed"),
  ];
}

export async function mockInvoke(cmd: string, args: any): Promise<any> {
  switch (cmd) {
    case "get_bootstrap":
      if (params.has("demo") && !running) {
        running = { channelId: config.channels[0]!.id, sandbox: false };
        setTimeout(() => {
          mockEmit("hb:status", status());
          demoFeed();
        }, 300);
      }
      return {
        config: JSON.parse(JSON.stringify(config)),
        providers,
        version: pkg.version,
        secretsBackend: "os",
        dataDir: "C:\\Users\\you\\AppData\\Roaming\\io.github.zcarecrow.highbazar",
        clientIdConfigured: true,
        keyStatus: { anthropic: true },
        accountStatus: { "1001": true },
        running,
        os: "windows",
      };
    case "save_config":
      config = args.config;
      return config;
    case "get_key_status":
      return { anthropic: true };
    case "ai_test":
      await new Promise((r) => setTimeout(r, 600));
      return { text: "Hey Chat! Ich bin HighBazar – euer neuer Lieblings-Mitbewohner im Chat. Wer hat Snacks? 🍿", inputTokens: 412, outputTokens: 38, costUsd: 0.00241, millis: 912, model: "claude-opus-5-5" };
    case "get_recent_feed":
      return { items: [], translations: [] };
    case "overlay_status":
      return { running: config.settings.overlay?.enabled ?? false, url: "http://127.0.0.1:47823/?k=demo", error: "", clients: 0 };
    case "app_to_tray":
      return null;
    case "ai_list_models":
      return (args.profile as { provider: string }).provider === "claudeCli"
        ? ["haiku", "sonnet", "opus", "fable"].map((id) => ({ id, name: "" }))
        : [
            { id: "claude-opus-5-5", name: "Claude Opus 5.5" },
            { id: "claude-sonnet-5-5", name: "Claude Sonnet 5.5" },
            { id: "claude-haiku-5-5", name: "Claude Haiku 5.5" },
            { id: "claude-fable-5-1", name: "Claude Fable 5.1" },
          ];
    case "bot_start":
      running = { channelId: args.channelId, sandbox: args.sandbox };
      setTimeout(() => {
        mockEmit("hb:status", status());
        demoFeed();
      }, 200);
      return;
    case "bot_stop":
      running = null;
      mockEmit("hb:status", { running: false });
      return;
    case "bot_running":
      return running;
    case "get_modlog":
      return demoModlog();
    case "get_emote_urls":
      return { urls: Object.fromEntries(demoEmotes.map((e) => [e.label, e.url])), zeroWidth: [] };
    case "chat_suggest":
      return demoSuggest(String(args.query ?? ""), args.mode as string, Number(args.limit ?? 10));
    case "chat_status":
      return { subEmotesMissing: false, emotes: 6412, people: 23 };
    case "twitch_api":
      return mockTwitch(args.method as string, args.path as string);
    case "get_usage":
      return { "2026-10-18": { [config.channels[0]!.id]: { calls: 312, inputTokens: 401233, outputTokens: 18321, costUsd: 0.184, messagesSent: 41, modActions: 2 } } };
    default:
      return null;
  }
}

// Channel manager data for the browser preview.
function mockTwitch(method: string, path: string): unknown {
  if (method !== "GET") return null;
  switch (path) {
    case "/channels":
      return {
        data: [
          {
            title: "Elden Ring Nightreign — blind run with the Krähen",
            game_id: "1",
            game_name: "ELDEN RING NIGHTREIGN",
            broadcaster_language: "de",
            tags: ["Deutsch", "Soulslike", "Blind", "Community"],
            content_classification_labels: ["ViolentGraphic"],
            is_branded_content: false,
          },
        ],
      };
    case "/streams":
      return { data: [{ viewer_count: 87, started_at: new Date(Date.now() - 5_400_000).toISOString() }] };
    case "/content_classification_labels":
      return {
        data: [
          { id: "ViolentGraphic", name: "Drastische Gewaltdarstellungen", description: "Realistische Gewalt, Blut, extreme Verletzungen." },
          { id: "ProfanityVulgarity", name: "Obszöne und vulgäre Sprache", description: "Wiederholter Gebrauch von Schimpfwörtern." },
        ],
      };
    case "/chat/settings":
      return {
        data: [
          {
            emote_mode: false,
            follower_mode: true,
            follower_mode_duration: 10,
            slow_mode: false,
            slow_mode_wait_time: 30,
            subscriber_mode: false,
            unique_chat_mode: true,
            non_moderator_chat_delay: false,
            non_moderator_chat_delay_duration: 2,
          },
        ],
      };
    case "/channels/followers":
      return { data: [], total: 1284 };
    case "/subscriptions":
      return { data: [], total: 42, points: 47 };
    default:
      return { data: [] };
  }
}