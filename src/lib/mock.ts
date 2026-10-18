// Browser-preview mock of the Tauri backend (dev only, `npm run dev` in a
// normal browser). Never included in production builds.
import cfgJson from "./mock-config.json";
import { mockEmit } from "./events";
import type { BotStatus, Config, FeedItem, ModLogEntry, PendingMessage, ProviderInfo } from "./types";

const providers: ProviderInfo[] = [
  { id: "anthropic", name: "Anthropic (Claude)", baseUrl: "https://api.anthropic.com", needsKey: true, webSearch: true, keyUrl: "https://platform.claude.com/settings/keys", local: false, reasoningOptions: ["", "low", "medium", "high", "xhigh", "max"] },
  { id: "openai", name: "OpenAI", baseUrl: "https://api.openai.com/v1", needsKey: true, webSearch: true, keyUrl: "https://platform.openai.com/api-keys", local: false, reasoningOptions: ["", "minimal", "low", "medium", "high"] },
  { id: "gemini", name: "Google Gemini", baseUrl: "https://generativelanguage.googleapis.com/v1beta", needsKey: true, webSearch: true, keyUrl: "https://aistudio.google.com/apikey", local: false, reasoningOptions: ["", "minimal", "low", "medium", "high"] },
  { id: "openrouter", name: "OpenRouter", baseUrl: "https://openrouter.ai/api/v1", needsKey: true, webSearch: true, keyUrl: "https://openrouter.ai/settings/keys", local: false, reasoningOptions: ["", "low", "medium", "high"] },
  { id: "mistral", name: "Mistral AI", baseUrl: "https://api.mistral.ai/v1", needsKey: true, webSearch: false, keyUrl: "", local: false, reasoningOptions: [""] },
  { id: "deepseek", name: "DeepSeek", baseUrl: "https://api.deepseek.com/v1", needsKey: true, webSearch: false, keyUrl: "", local: false, reasoningOptions: [""] },
  { id: "perplexity", name: "Perplexity", baseUrl: "https://api.perplexity.ai", needsKey: true, webSearch: true, keyUrl: "", local: false, reasoningOptions: [""] },
  { id: "ollama", name: "Ollama (local)", baseUrl: "http://localhost:11434/v1", needsKey: false, webSearch: false, keyUrl: "", local: true, reasoningOptions: [""] },
  { id: "lmstudio", name: "LM Studio (local)", baseUrl: "http://localhost:1234/v1", needsKey: false, webSearch: false, keyUrl: "", local: true, reasoningOptions: [""] },
  { id: "claudeCli", name: "Claude Code CLI (subscription)", baseUrl: "", needsKey: false, webSearch: false, keyUrl: "", local: true, reasoningOptions: ["", "low", "medium", "high"] },
];

let config: Config = JSON.parse(JSON.stringify(cfgJson));
config.accounts = [{ id: "1001", login: "zcarecrow", displayName: "ZcarecroW", profileImageUrl: "", scopes: [] }];
config.channels[0]!.name = "ZcarecroW";
config.channels[0]!.streamerAccountId = "1001";
config.channels[0]!.aboutStream = "Horror & souls-like games, cozy community called 'Krähen'.";
const params = new URLSearchParams(location.search);
config.settings.wizardCompleted = !params.has("wizard");
if (params.get("theme") === "light" || params.get("theme") === "dark") config.settings.theme = params.get("theme") as "light" | "dark";
if (params.get("lang") === "de" || params.get("lang") === "en") config.settings.language = params.get("lang") as "de" | "en";

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
  chat("NightOwl92", "#1E90FF", "this boss is actually insane monkaS", ["subscriber"]);
  chat("Kraehenmama", "#FF7F50", "du schaffst das!! Pog", ["vip"]);
  bot("Fun Fact: Der Boss hat eine versteckte Schwäche gegen Blutungsschaden – vielleicht Zeit für ein Messer? KEKW", "timed");
  chat("lurker_lena", "#9ACD32", "hi, bin neu hier :)", [], { firstMsg: true });
  bot("@lurker_lena Willkommen bei den Krähen! Mach's dir gemütlich – der Boss hier frisst gerade Seelen zum Frühstück Sadge", "welcome");
  mockEmit("hb:feed", item("event", "", { meta: { key: "raid", params: { user: "PixelPirat", viewers: 23 } } }));
  bot("@PixelPirat bringt 23 Raider mit! Willkommen, ihr Legenden – ihr kommt genau rechtzeitig zum Bossfight catJAM", "event");
  chat("BackseatBernd", "#DAA520", "HighBazar welche waffe ist die beste hier?");
  bot("@BackseatBernd Gegen den hier schwören viele auf schnelle Waffen mit Blutung – aber ganz ehrlich, Skill > Waffe Kappa", "reply");
  const entry: ModLogEntry = {
    id: "mod1", ts: now(), userId: "x", login: "toxic_tim", displayName: "toxic_tim", messageId: "m-x",
    text: "[removed insult]", category: "harassment", verdict: "violation", confidence: 0.94,
    reason: "Beleidigung gegen den Streamer", source: "ai", action: { kind: "timeout", durationSecs: 600 }, status: "executed", error: "",
  };
  mockEmit("hb:modlog", entry);
  mockEmit("hb:feed", item("mod", "", { user: "toxic_tim", meta: entry }));
  chat("NightOwl92", "#1E90FF", "LETS GOOOO", ["subscriber"]);
  const pend: PendingMessage[] = [
    { id: "p1", text: "Wer von euch hat den Boss eigentlich schon beim ersten Versuch gelegt? Ehrliche Antworten only 👀", reason: "timed", created: now(), sendAt: now() + 8000 },
  ];
  mockEmit("hb:pending", pend);
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
        version: "1.0.0",
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
    case "ai_list_models":
      return ["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5", "claude-fable-5-1"];
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
      return [];
    case "get_usage":
      return { "2026-10-18": { [config.channels[0]!.id]: { calls: 312, inputTokens: 401233, outputTokens: 18321, costUsd: 0.184, messagesSent: 41, modActions: 2 } } };
    default:
      return null;
  }
}
