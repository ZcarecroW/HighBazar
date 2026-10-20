<div align="center">

<img src="app-icon.svg" width="112" alt="HighBazar logo" />

# HighBazar

**An AI companion and fair moderator for your Twitch chat.**
It chats, welcomes newcomers, celebrates your community, knows your game — and keeps chat safe.

[![Release](https://img.shields.io/github/v/release/ZcarecroW/HighBazar?color=9146ff)](https://github.com/ZcarecroW/HighBazar/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-555)
![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24c8db)

[**⬇ Download**](https://github.com/ZcarecroW/HighBazar/releases/latest) · [Features](#features) · [Installation](#installation) · [Quick start](#quick-start) · [FAQ](#faq) · [🇩🇪 Deutsch](README.de.md)

<img src="docs/screenshots/dashboard-dark.png" alt="HighBazar dashboard" width="900" />

</div>

---

## What is HighBazar?

HighBazar is a small desktop app that sits next to your stream and brings life into your chat. It uses
an AI model of your choice (Claude, GPT, Gemini, local models and more) to:

- **chat like a friendly regular** — every few minutes it says something fun, interesting or engaging
  that fits what's happening in chat and in your game,
- **react to your community** — answers when addressed, welcomes first-time chatters, thanks follows,
  subs, gifts, raids and cheers,
- **know your game** — when you switch categories it researches facts and tips (Wikipedia + web search),
- **moderate fairly** — instant local rules plus context-aware AI that tells banter from real hate.

It's built for **small streamers** who want a lively chat from the first viewer on — and it scales to
**big channels** with adaptive pacing, batching and budgets.

The bot writes with **your streamer account** by default (marked with a configurable prefix like
`[HighBazar]`), or with a **separate bot account** if you connect one.

## Features

### 🤖 Companion
| | |
|---|---|
| **Timed messages** | Random interval (e.g. every 3–5 min), only while live, adaptive: talks less when chat is busy, pauses when nobody answers. |
| **Content mix** | React to chat · game facts · game tips · trivia · jokes · questions to chat · hype · stream moments — each with its own weight. |
| **Context-aware** | Reads the last *N* chat messages (configurable), stream title, category, uptime and viewers, plus everything you tell it about your stream. |
| **Personality** | Persona text plus humor / energy / teasing sliders, custom instructions and topics to avoid. Seven built-in presets. |
| **Language** | Stream language, a fixed language, or whatever most chatters use. Optionally answers each chatter in *their* language. |
| **Emotes** | Knows Twitch, **BetterTTV**, **FrankerFaceZ** and **7TV** emotes of your channel and prefers the ones your community actually uses. |
| **Reactions** | Replies when addressed, welcomes first-time chatters, thanks follows (batched), subs, resubs, gift subs, raids (mentions what the raider played) and cheers. |
| **Commands** | `!hb ask <question>`, `!hb fact`, `!hb joke`, `!hb about` — cooldowns and role requirements per command. |
| **Game research** | On category change: Wikipedia + a web search → 15 facts & 8 tips, cached per game. |
| **Current answers** | When chat asks about patches, release dates, tournaments and the like, the bot looks it up on the web first (small talk is answered right away). On by default, can be switched off. |
| **Control** | Automatic sending, *preview then send*, or *approve every message*. "Say something now" button. Daily AI budget. |

### 🛡️ Moderation
- **Instant local rules** (free, no AI): blocked terms with `*` wildcards and leetspeak detection, links with allow-list, caps, repeated messages, emote spam, length.
- **AI classification** that understands context — batched for efficiency — in nine categories: hate, harassment, threats, sexual content, self-harm, spam, scams, doxxing, extremism.
- **Actions per category** for *clear* violations and *borderline* cases: nothing, log, delete, warn, timeout (any duration), ban.
- **Escalation** for repeat offenders, **exemptions** (mods, VIPs, subs, specific users), channel rules in plain language, strictness slider, confidence threshold.
- **Modes**: off · watch & log · suggest actions (one click to apply) · act automatically. Full **moderation log with undo**.
- Built-in profiles: **Balanced** (default – acts only on clear cases), Relaxed, Family Friendly, Suggest Only, Watch & Log.
- **Translation of foreign-language chat**: set the languages you understand, and messages in any other language are translated right below them in the live feed. If such a message breaks the rules, moderation acts as usual and adds a **note with the translation and what is really meant** (slang, coded insults). Your mods see it as the timeout or ban reason on Twitch. Language detection runs locally, so only foreign messages cost AI tokens. By default the translation is also posted in chat ("[HighBazar][Translator] …"), so everyone can follow – never for flagged messages.

### 🧠 AI providers
| Provider | Notes |
|---|---|
| **Anthropic Claude** | Messages API, prompt caching, web search. Presets: Opus 5.5, Sonnet 5.5, Haiku 5.5. |
| **OpenAI** | Responses API (or Chat Completions). Presets: GPT‑6.1 Sol, GPT‑6 Luna. |
| **Google Gemini** | Interactions API with automatic fallback to generateContent, Google Search grounding. |
| **OpenRouter** | One key for hundreds of models, exact cost reporting, web search tool. |
| **Mistral, Groq, xAI, DeepSeek, Perplexity, Together AI** | Via their OpenAI-compatible endpoints. |
| **Ollama, LM Studio** | Local models, free, nothing leaves your PC (except Twitch). |
| **Any OpenAI-compatible endpoint** | Custom base URL and headers. |
| **Claude Code CLI** *(optional)* | Uses your locally installed, logged-in `claude` CLI and thus your Claude subscription, with web search. See the [note below](#can-i-use-my-claude-pro-subscription). |
| **Codex CLI** *(optional)* | Uses your locally installed OpenAI Codex CLI logged in with your ChatGPT plan (counts against its Codex usage limits), with web search. HighBazar runs it without commands, file access or other tools. |

**Web search for every provider.** Claude, OpenAI, Gemini, Grok, OpenRouter, Perplexity, Groq Compound and both CLIs use
their own web search. For everything else – Ollama, LM Studio, DeepSeek, Mistral, Together, custom servers – HighBazar
searches itself and hands the results to the model: with **You.com** (free, no key – the default, with Wikipedia
as a fallback), **Wikipedia**, **Exa**, **Serper** (Google results), **Tavily**, **Brave Search**, the **Ollama web
search** or your own **SearXNG** (AI → *Web search*).

Every provider is configured through **AI profiles** (model, reasoning effort, max tokens, temperature, timeout,
prices, extra headers…). Pick the model from a searchable list loaded from the provider (or type any name), test the connection, see tokens and costs.
You can use a **different, cheaper model for moderation**.

### 🎛️ Stream & channel manager
Change your stream right from HighBazar, in three levels (default: *Simple*):
- **Simple** — title, category (search with box art), language and tags.
- **Advanced** — plus content classification labels and branded content, chat settings (emote-only, followers-only, slow mode, subs-only, unique chat, chat delay), announcements, shoutouts, raids, clips (with title and length), stream markers and pinned chat messages.
- **Expert** — everything Twitch allows: polls (with channel points voting) and predictions, channel point rewards with limits, one-click templates and AI actions (the bot answers questions, roasts, rhymes or shares a fun fact; unhandled redemptions are refunded) plus a redemption queue, mods, VIPs, bans and unban requests, suspicious-user monitoring, Twitch AutoMod levels with explanations and presets, shield mode with an optional Auto Shield against hate raids and blocked terms, ads (run / snooze) with an optional ad automation (no pre-rolls for newcomers, announcements in chat, snooze on raids), stream schedule, stream delay (partners), channel description and chat color, follower/sub numbers and the Hype Train.

HighBazar only asks Twitch for the permissions of the level you pick; switching to a higher level asks you to approve the extra ones once.

### 🎨 10 themes, light and dark
Bazaar, Midnight, Ocean, Aurora, Forest, Gold, Sunset, Crimson, Sakura and Cyber — each in a light and a dark version (or following your system). All themes are checked for readable contrast (WCAG AA). Two-tone icons pick up the theme's colors.

<img src="docs/screenshots/themes.png" alt="Theme picker" width="700" />

### 🧩 Profiles everywhere
- **Channel profiles** (global profiles) — one per Twitch channel or setup, each with its own accounts, prefix, companion, moderation and AI profiles. Switch any time.
- **Companion, moderation and AI profiles** — built-in presets are read-only and improve with updates; duplicate one and make it yours, or start from scratch.

### 🎮 Made for gaming PCs
- **Performance mode**: minimize or close the window and the whole interface shuts down – HighBazar keeps running in the notification area with about **9 MB of memory and no measurable CPU load** (bot, moderation, translation and overlay keep working). A minimized window pauses its interface even without performance mode.
- **Steam chat overlay**: a local link that shows your live chat – with emotes, smooth gliding and translations – in the Steam overlay browser, pinned over your game. Follow chat on one screen without Alt+Tab (also works as an OBS browser source).
- **Games come first**: HighBazar runs with lower process priority, so your game always gets the CPU first.

### ✨ And also
- **Chat yourself** – in the app and, if you like, right in the Steam/OBS overlay (Shift+Tab, type, Enter, back to the game). Type `:` for emotes of **every provider** (Twitch incl. your subscriptions, BTTV, FFZ, 7TV) and **emojis** with English and German keywords (`:herz`, `:fire`); `:7 pog` searches only 7TV (`:b` BTTV, `:f` FFZ, `:t` Twitch, `:e` emoji). Your favorites and what you use most come first, `@` suggests people from chat, **Tab** completes the current word, ↑ brings back your last message. An emote picker with a tab per provider is one click away; right-click pins a favorite.
- **Fuzzy search everywhere**: **Ctrl+K** jumps to any page, setting, profile or action; raid and shoutout targets are suggested while you type – live channels you follow, people in chat and Twitch's channel search, with avatar, title, category and viewers; profiles, models, languages and the moderation log are searchable too. Typos, umlauts and abbreviations are forgiven ("minecarft", "jc" → Just Chatting).
- **Setup wizard** on first start (skippable, can be reopened any time).
- **Test mode + chat simulator** — try everything without sending anything to Twitch.
- **Live feed** with chat, bot messages, events and moderation; delete / timeout / ban with one click. Twitch, BetterTTV, FrankerFaceZ and 7TV **emotes are shown as images** (can be switched off).
- **Calm reading**: new chat messages **glide in** instead of jumping (speed and acceleration curve adjustable, with live preview), and the chat and the rest of the interface can be **zoomed separately** (Ctrl + mouse wheel, Ctrl + plus/minus).
- **System, Dark and Light** themes, **English and German** UI, responsive layout optimized for 1920×1080.
- **Automatic updates** from GitHub Releases (check on start, configurable).
- **Lightweight**: native Rust core and the system's own webview instead of Electron; small installers.

<details>
<summary><b>More screenshots</b></summary>

| | |
|---|---|
| ![Companion](docs/screenshots/companion.png) | ![Moderation](docs/screenshots/moderation.png) |
| ![AI](docs/screenshots/ai.png) | ![Channels](docs/screenshots/channels.png) |
| ![Light mode, German](docs/screenshots/dashboard-light.png) | ![Setup wizard](docs/screenshots/wizard.png) |
| ![Stream & channel manager](docs/screenshots/stream.png) | ![Themes](docs/screenshots/themes.png) |

</details>

## Installation

Download the file for your system from the **[latest release](https://github.com/ZcarecroW/HighBazar/releases/latest)**:

| System | File | Notes |
|---|---|---|
| **Windows 10/11** | `HighBazar_x.y.z_x64-setup.exe` | Installs for the current user, no admin rights needed. |
| **macOS 11+** (Apple Silicon & Intel) | `HighBazar_x.y.z_universal.dmg` | Open the DMG and drag HighBazar into *Applications*. |
| **Linux** | `.AppImage`, `.deb` or `.rpm` | AppImage works everywhere and supports auto-updates. |

<details>
<summary><b>Windows: "Windows protected your PC"</b></summary>

HighBazar is open source but not signed with a paid code-signing certificate, so SmartScreen may warn you.
Click **More info → Run anyway**. You can verify the download against the source on GitHub.
</details>

<details>
<summary><b>macOS: "HighBazar is damaged and can't be opened"</b></summary>

The app isn't notarized by Apple (that requires a paid developer account). After moving it to *Applications*, run once in Terminal:

```bash
xattr -cr /Applications/HighBazar.app
```

Then open it normally.
</details>

<details>
<summary><b>Linux</b></summary>

```bash
chmod +x HighBazar_*.AppImage
./HighBazar_*.AppImage
```

or install the `.deb` (`sudo apt install ./HighBazar_*.deb`) / `.rpm` (`sudo dnf install ./HighBazar-*.rpm`).
API keys are stored via the Secret Service (GNOME Keyring, KWallet…). Without one, HighBazar falls back to a
file in its data folder that only your user can read.
</details>

## Quick start

1. **Start HighBazar** — the setup wizard opens.
2. **Connect Twitch** — click *Connect*, enter the shown code on `twitch.tv/activate`. HighBazar never sees your password.
3. *(Optional)* **Connect a bot account** — otherwise the bot writes with your account and the prefix.
4. **Pick an AI** and paste an API key (or choose a local model) → *Test connection*.
5. **Pick a companion style** and a **moderation profile**, optionally tell the AI a bit about your stream.
6. **Try it in test mode**: use the chat simulator, mention the bot, type `!hb ask …`, simulate a raid.
7. **Start the bot** and stream. 🎉

## Costs

HighBazar itself is free. AI providers bill you directly; the dashboard shows today's cost and you can set a
**daily budget** per channel (default $2). Rough orders of magnitude for one hour of streaming with default
settings (~15 timed messages, a few replies and welcomes):

| Model class | Companion / hour | AI moderation / hour (moderate chat) |
|---|---|---|
| Small models (Claude Haiku 5.5, GPT‑6 Luna, Gemini Flash-Lite) | < $0.02 | < $0.02 |
| Mid models (Claude Sonnet 5.5, GPT‑6.1 Sol, Gemini 3.8 Flash) | ~ $0.10–0.25 | ~ $0.10–0.30 |
| Flagships (Claude Opus 5.5) | ~ $0.20–0.50 | use a small model for moderation |
| Local (Ollama / LM Studio) | $0 | $0 |

*Estimates — actual costs depend on chat activity, context size and prices.* Tip: use a great model for the
companion and a fast, cheap one for moderation (Channels → *AI for moderation*).

## FAQ

<details>
<summary><b>Can I use my Claude Pro subscription?</b></summary>

Third-party apps may not sign in with a Claude.ai subscription; the official way is an **API key** from the
[Claude Console](https://platform.claude.com) (pay per use — Haiku costs fractions of a cent per message).
As an *optional* alternative, HighBazar can call your locally installed and logged-in **Claude Code CLI**
(`claude -p`), which runs on your subscription. Whether an always-on chat bot is covered by your plan's terms
is your responsibility — the API key route is the supported one.
</details>

<details>
<summary><b>Does the bot pretend to be me?</b></summary>

No. Messages sent through your account carry the prefix (default `[HighBazar]`), and the AI is instructed
never to speak in your name. You can also connect a separate bot account.
</details>

<details>
<summary><b>Will it spam my chat?</b></summary>

Timed messages are spread out (default 3–5 min), pause when chat is very busy, stop after a few messages without
any answer, and only run while you're live. Replies and welcomes have cooldowns and rate limits.
</details>

<details>
<summary><b>How strict is the default moderation?</b></summary>

The *Balanced* profile only acts automatically on **clear** hate, harassment, threats, scams and spam (delete or
timeout, escalating for repeat offenders). Swearing, banter, dark humor and criticism are allowed; borderline cases
are only logged for you. Mods and VIPs are exempt. Try *Suggest Only* if you want to approve every action.
</details>

<details>
<summary><b>Where is my data stored?</b></summary>

Everything stays on your computer: settings in the app config folder, API keys and Twitch tokens in your OS
credential store (Windows Credential Manager, macOS Keychain, Secret Service). Chat messages are only sent to
the AI provider you chose. *Settings → Open data folder* shows the location.
</details>

<details>
<summary><b>The bot doesn't react to follows / can't moderate</b></summary>

Reconnect the streamer account in *Channels* (it needs the moderation and follower scopes). If you use a separate
bot account for writing, moderation still runs through the streamer account.
</details>

<details>
<summary><b>Can I use my own Twitch application?</b></summary>

Yes: register a **Public** application on [dev.twitch.tv](https://dev.twitch.tv/console) (category *Chat Bot*,
any redirect URL such as `http://localhost:17563`) and paste its Client ID under *Settings → Advanced*.
</details>

## Building from source

Requirements: [Rust](https://rustup.rs) (stable), [Node.js](https://nodejs.org) LTS and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/ZcarecroW/HighBazar.git
cd HighBazar
npm install
npm run tauri dev     # run in development mode
npm run tauri build   # build installers into src-tauri/target/release/bundle
```

`npm run dev` alone serves the UI in a normal browser with a mocked backend (handy for UI work:
`http://localhost:1420/?demo`). Tests: `cargo test --manifest-path src-tauri/Cargo.toml`.

### Tech stack

| Layer | Technology |
|---|---|
| App shell | Tauri 2.12 (Rust 2024 edition) |
| Core | Rust: tokio, reqwest (rustls), tokio-tungstenite, keyring |
| UI | Vue 3.5, Pinia 4, vue-i18n 11, Tailwind CSS 4.3, Vite 8, TypeScript 6 |
| Twitch | Device Code Flow, EventSub WebSocket, Helix API |
| Updates | Tauri updater with signed GitHub release artifacts |

### Project layout

```
src/                 Vue UI (views, components, stores, i18n)
src-tauri/src/
  ai/                provider clients (Anthropic, OpenAI, Gemini, OpenAI-compatible, Claude CLI)
  bot/               engine, prompts, moderation, research, usage
  twitch/            auth (device code), Helix, EventSub, emotes
  config/            settings, profiles and built-in presets
```

## Contributing

Issues and pull requests are welcome! Please describe what you changed and run the tests before opening a PR.

## License

[MIT](LICENSE) © ZcarecroW. HighBazar is not affiliated with Twitch, Anthropic, OpenAI, Google or any other provider.

Icons: [Nucleo](https://nucleoapp.com) — licensed to the author for use in HighBazar and **not** covered by the MIT license; they may not be extracted or reused as an icon set (src/icons/nucleo.ts).

Emoji names and keywords: [emojibase](https://emojibase.dev) (MIT), based on Unicode CLDR data (src-tauri/data/emoji.tsv, rebuilt with `node scripts/emoji/build.mjs`).
