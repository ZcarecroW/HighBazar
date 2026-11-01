# Changelog

All notable changes to HighBazar are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- The documentation is now also in German, with German screenshots: [zcarecrow.github.io/HighBazar/de](https://zcarecrow.github.io/HighBazar/de/). Every page has a language switch, and German-speaking visitors of the English pages get a hint.
- *Settings → About* has a **Documentation** button that opens the documentation in the app's language.

## [1.8.1] – 2026-11-01

Web search that actually gets used, game research that recovers from hiccups, and a few chat fixes. Tested with Claude Code: the real chat and research prompts were run against the Claude Code CLI (Haiku, as in the built-in profile) in English and German, plus all Rust and frontend tests and the UI checks. Builds for Windows, macOS and Linux. New: documentation at [zcarecrow.github.io/HighBazar](https://zcarecrow.github.io/HighBazar/).

### Fixed
- **The bot often said it couldn't look something up** – especially with the Claude Code CLI. The AI never learned today's date (the CLI's own system prompt, which has it, is replaced by HighBazar's), so it didn't know how recent "latest" had to be, and the reply instructions told it to say so playfully when it "can't know" something. Every request now carries today's date and time, questions tell the model to search instead of giving up, and search queries for recent things include the month and year. Measured with the same question: the answer went from "the latest I can confirm is from January … there could be a newer patch" to a concrete patch with its date.
- **Game research didn't search for the latest patch**: it only asked the model to mention one "if you know of" it. With web search on, research now looks up the current update, season or DLC first and records it with its date (live test: the September 2026 patch was found).
- Questions asked in chat without @mentioning the bot (picked up by its "react to chat" messages) can now be looked up too.
- A failed game research (timeout, network) was never retried until the category changed, so a whole stream could run without game knowledge. It is retried after 15 minutes, up to three times per game; research with the Claude Code CLI gets more steps and time.
- After a category change, facts about the previous game were still used until the new research finished; a category change during a running research left the new game without research.
- Claude (API) answers with web search could include the lead-in "Let me look that up" before the answer, and a long search turn that the API paused was cut off. Only the final answer goes to chat now, and paused turns are resumed. OpenAI answers likewise use only the final message.
- A @mention that hit the global cooldown still started the person's own cooldown (they got no answer and had to wait); a chat command used by someone on their own cooldown blocked the command for everyone.
- AI errors of channel point rewards were swallowed (the points were refunded without a note); bot errors and failed research are now also written to the log file, and the log says why an answer was not posted.

### Changed
- Updated: zip 9.2.

## [1.8.0] – 2026-10-31

Smoother chat and emotes, explanations for every setting, current AI models and a faster start. Tested with Claude Code (Rust and frontend tests, suggestion speed measured before and after on an optimized build, live 7TV, BetterTTV and FrankerFaceZ data and 7TV's event stream checked against the real services, the release build started in test mode). Windows build.

### Added
- **Explanations everywhere**: a small ⓘ next to 143 settings and features (and on the main buttons) explains them in plain words – including Twitch terms like raids, VIPs, shoutouts, AutoMod, channel points, pre-rolls or follower-only mode – so newcomers don't have to look anything up. Hover, keyboard focus or a click shows it; it also works in the read-only built-in profiles.
- **7TV live**: emotes the streamer or an editor adds, removes or renames in the channel's 7TV set appear right away in the live feed, the suggestions and the overlay (with a note in the feed), instead of with the next 30-minute refresh. Switching the active 7TV set is noticed too.
- **Zero-width emotes** (7TV's RainTime, SteerR …, BTTV's hats) are drawn on top of the emote before them in the live feed and the overlay, like in the browser extensions.
- AI preset **OpenAI GPT-6.1 Terra** (efficient, strong at writing, half the price of Sol). Reasoning effort "Off" for OpenAI and DeepSeek.

### Changed
- Emote suggestions are about twice as fast for `:` (0.67 ms instead of 1.31 ms per keystroke with ~6,400 emotes), Tab completion six times as fast, the emote picker seven times.
- The window starts with 39 % less JavaScript: only the dashboard and the language in use are loaded at first, the other pages on their first visit.
- The bot prefers your channel's own emotes when picking which emotes to use, instead of a random mix dominated by hundreds of global Twitch emotes.
- AI costs count cached prompt tokens at their real price (a tenth for cache reads), so the daily budget no longer runs out early with Claude and OpenAI.
- Game research with Claude may open the pages it found (web fetch, capped per page).
- Requests to Gemini's generateContent API are no longer stored by Google (new paid projects store them by default since 2026-10-26); OpenRouter caches the system prompt for Claude models.
- Emoji data updated to emojibase 18 (CLDR 49 names and keywords); the 9 new Emoji 18 emojis are held back until systems can draw them.
- Updated: Tauri 2.12.5, tokio 1.54, parking_lot 0.13, Vite 8.3.5 and more; CI also runs the frontend tests.

### Fixed
- **The same emote was suggested several times** when more than one provider had it (e.g. "AngelThump" on 7TV and BTTV, "BibleThump" on Twitch, 7TV and FFZ), or when a set listed it twice. Each name is now suggested once – the one chat shows; `:b`, `:7` … still show a provider's own emote. The live feed also shows the right image for such names (a channel's emote beats a global one of another provider).
- A failed request to one emote provider no longer makes its emotes disappear until the next refresh.
- Follower emotes of other channels (only usable there) are no longer suggested.
- BTTV's emote modifiers ("w!", "h!", "z!" …) are no longer treated as emotes.
- "@" suggestions use the login for people whose display name is in another script, so the mention reaches them.
- Words like "constructor" in chat were drawn as a broken emote image.
- Suggestions follow the caret when it moves with the arrow keys, Home or End.
- The DeepSeek preset used `deepseek-chat`, which DeepSeek retired on 2026-07-24; it now uses `deepseek-flash`, and profiles with the old names keep working.
- OpenAI's GPT-6 models rejected "Minimal" effort and a custom temperature; Gemini 3.8 Flash and Gemini 4 rejected "Minimal" thinking. Both are mapped to what the model accepts.
- When thinking used up "Max. output tokens" before any text, the bot got an empty answer silently; it now says so.

## [1.7.0] – 2026-10-20

Chat fixes, Auto Shield, AutoMod presets, channel points the bot answers, ad automation, translations in chat and fewer AI tokens. Tested with Claude Code (chat input and its suggestions in the app and in the overlay page in a real browser, "@" with the live Twitch chatter list, Auto Shield and translations in chat with the Claude Code CLI in test mode, AI tokens measured before and after with the same test-mode chat, the new Stream panels without writing to Twitch). Windows build.

### Added
- **Auto Shield** (Stream → *Safety*, off by default): turns Twitch's Shield Mode on when several different accounts post hate, harassment, threats or sexual harassment within a short time (as found by moderation), and off again after a calm period. Accounts, time window and calm period are adjustable.
- **Twitch AutoMod explained and with presets**: every category has a tooltip with what it covers, every level a name (0 off … 4 anything suspicious). Six built-in presets (Off, Hate only, Gaming relaxed, Balanced, Strict, Family friendly) plus presets of your own.
- **Channel points the bot answers**: rewards can be linked to an AI action – answer a question, a friendly roast, a rhyme about the viewer, a game fun fact or your own instruction – or to one minute of emote-only chat. Done redemptions leave the queue; ones that can't be done (bot off, budget, error) are refunded automatically. Seven one-click templates plus templates of your own, and a queue to mark redemptions done or refund them.
- **Ad automation** (Stream → *Ads*, off by default): Twitch doesn't let apps change the Ads Manager settings, so HighBazar steers ads itself – runs a mid-roll in time so newcomers don't get a pre-roll, announces ads in chat, pushes the next ad back when a raid comes in and pauses timed bot messages during ad breaks. Plus a button that opens the Ads Manager.
- **Translations in chat**: translations of foreign messages are now also posted in chat by default, e.g. "[HighBazar][Translator] Anna (Spanish): Hello everyone!" (prefix adjustable, can be switched off). Messages flagged by moderation are never posted translated; at most 8 a minute.
- **Profiles ↔ channels both ways**: on the Companion and Moderation pages, tick the channels a profile is used in.
- **"@" suggests everyone in chat**, including people who haven't written yet (Twitch's chatter list), recent chatters first.

### Changed
- **Fewer AI tokens without losing anything**: emote-only messages and short messages the AI just found fine ("gg", "lol") are no longer sent to AI moderation; the same foreign message is translated only once; the chat log in companion prompts is more compact (one time stamp per minute, repeated spam once with a count, the bot's own messages listed only once); fewer emote names when the bot rarely uses emotes. The Claude Code CLI no longer writes ~900 tokens into a prompt cache on every call that could never be read again (measured with CLI 2.1.304).
- "Reply to chatters in their language" is now off by default (also for existing profiles) – you couldn't read those answers yourself; the translator covers foreign messages instead.

### Fixed
- The chat input shows why suggestions are missing: "no matches", "nobody in chat yet" or – for logins from before 1.6.0 – that the emotes of your subscriptions need a reconnect.
- ":D", ":P" and other one-letter smileys are sent with Enter instead of being replaced by the first suggestion; ":)" no longer opens suggestions at all.

## [1.6.0] – 2026-10-20

Web search, Codex CLI, fuzzy search and your own chat input. Tested with Claude Code (web search with the Claude Code CLI and through HighBazar's own search, the Codex CLI against its real binary with a stand-in OpenAI server, raid suggestions with the live Twitch channel search, chat input and emote suggestions in the app and in the overlay page in a real browser – sent in test mode only –, security checks of the overlay's send route, suggestion speed measured on the release build). Windows build.

### Added
- **Write in chat yourself** – under the live feed (Settings → *Chat & size*, on by default) and optionally in the overlay page (Settings → *Steam overlay* → *Chat input in the overlay*, off by default): Shift+Tab into the Steam overlay, type, Enter, back to the game. Messages go out as the streamer; in test mode they only appear in the feed. A spam guard stops more than 20 messages in 30 seconds.
- **Smart suggestions while typing** (same in app and overlay, ranked in the Rust core in well under a millisecond): `:` suggests emotes of every provider – Twitch (including those of your subscriptions), BetterTTV, FrankerFaceZ, 7TV – and all Unicode emojis with English and German names and keywords; `:7 pog`, `:b`, `:f`, `:t`, `:e` limit the search to 7TV, BTTV, FFZ, Twitch or emojis. Favorites come first, then what you use yourself, what chat uses right now and the channel's own emotes; `@` suggests people from chat; **Tab** completes the current word (Tab again for the next match); ↑ brings back your last messages. An emote picker with a tab per provider sits next to the input; right-click (or the star in the suggestions) pins favorites. The emoji data (emojibase, MIT) is only loaded on the first `:`.
- **New web search sources**: **You.com** – current results without a key or sign-up (about 100 searches a day), now the default with Wikipedia as a fallback; **Exa** (accurate, with excerpts; monthly free credit) and **Serper** (Google results including direct answers). Results now carry their date so the model can tell old from new.
- **Web search for chat questions** (Companion → *Research & web search*, on by default): when chat asks the bot about patches, release dates, tournaments and the like, it looks it up on the web before answering; small talk is answered right away. The model decides when a search is needed.
- **Web search for every AI provider.** Providers with their own search use it (Claude, OpenAI, Gemini, OpenRouter's new web search tool, Perplexity, xAI Grok via its Responses API, Groq Compound). For all others – Ollama, LM Studio, DeepSeek, Mistral, Together, custom servers – HighBazar searches itself and passes the results to the model as untrusted reference material. Sources (AI → *Web search*): Wikipedia (free, no key), Ollama web search, Tavily, Brave Search or your own SearXNG instance, with a test search. The number of web searches is shown in the usage table and in the AI test ("Test with web search").
- **Claude Code CLI with web search**: only the web search tool is allowed (plus reading pages during game research); everything else stays switched off.
- **Codex CLI** as a new AI provider (and built-in profile) for your ChatGPT plan: automatic detection (standalone installer, npm, PATH), login check, model list from the CLI's own catalog, reasoning effort, web search. Codex runs isolated – no shell, code, image, agent or app tools, a read-only sandbox, no user config, rules, project instructions or saved sessions, and HighBazar's instructions replace Codex's coding-agent prompt. Verified against codex-cli 0.163.0.
- **Fuzzy search everywhere**: quick search with **Ctrl+K** (pages, sections, settings, profiles, channels and actions such as start/stop bot, raid or tray); raid and shoutout targets are suggested while typing – live channels you follow, people in chat and Twitch's channel search, with avatar, title, category and viewer count; mod/VIP and watch fields suggest people from chat; searchable AI provider, profile, model and language lists (find "German" as "Deutsch", "de" or "german"); search in the profile lists, the moderation log and the mod/VIP/ban lists; keyboard navigation and highlighted matches. Tolerates typos, umlauts and abbreviations.

### Changed
- Web search is now on by default in all built-in AI profiles (it used to be off for providers without their own search).
- Answers written with web search are cleaned for chat: source lists, markdown links and citation marks are removed instead of the whole message being dropped.
- The fuzzy search is up to four times faster on long lists (its cache no longer empties itself with thousands of entries; typo matching skips words that cannot match), and search fields listen for outside clicks only while they are open.
- The channel manager asks Twitch additionally for `user:read:emotes` (all levels: emotes of your subscriptions in the chat suggestions) and, on the *Advanced* level, for `user:read:follows` (live channels you follow as raid suggestions) – reconnect your account once to use it.

## [1.5.0] – 2026-10-20

Performance release for gamers: HighBazar should never slow down your game. Tested with Claude Code (CPU and memory measured on the release build in every state – window visible, minimized and in the tray –, overlay rendered in a real browser, performance mode round trip, all pages without a console error). Windows build.

### Added
- **Performance mode** (Settings → *Performance*): minimizing or closing the window shuts the interface down completely – WebView2 and all its processes end – and HighBazar keeps running in the notification area (bot, moderation, translation, overlay) with about 9 MB of memory and no measurable CPU load. Click the tray icon to open the window again (the chat of the running session is restored); the right-click menu starts/stops the bot and quits. Optionally start directly in the tray, together with "Start the bot automatically" like a background service.
- **Steam chat overlay** (Settings → *Steam overlay*): a local web page (127.0.0.1 only, secret link) that shows the live chat with your chat settings – emotes, smooth gliding, chat size, translations – plus follows/subs/raids and bot messages (each optional), adjustable background and fading of older messages. Pin it in the Steam overlay browser over your game to follow chat on one screen without Alt+Tab; it also works as an OBS browser source. Messages removed by moderation disappear from it as well. Nothing runs while it is off.
- **"Games come first"** (on by default): HighBazar runs with "below normal" process priority, so a game always gets the CPU first.

### Changed
- A minimized window no longer renders anything: the interface pauses (≈1 % instead of ≈17 % of a CPU core with a busy chat) and catches up when it is shown again.
- The live feed is much lighter: rows only re-render when they change (not on every status tick), the moderation buttons are created only for the row under the mouse (≈75 % fewer page elements), times are formatted with a shared formatter. New installations keep 200 instead of 400 feed entries.
- Emote lookups use an index instead of scanning every emote for every chat word.
- The Rust core uses two worker threads instead of one per CPU core.

## [1.4.0] – 2026-10-20

Tested with Claude Code (translation and mod notes in test mode with the Claude Code CLI, model picker for every provider including a live OpenRouter list, smooth scrolling and zoom measured in the running app, all pages at 100 % and 140 % without a console error). Windows build.

### Added
- **Smooth chat:** new messages glide into the live feed instead of jumping, so your eyes don't lose the line while reading. Speed (150–1200 ms) and the acceleration/braking curve (*Gentle*, *Brisk*, *Steady*, *Natural*) are adjustable under Settings → *Chat & size*, with explanations and a live preview. Messages that arrive mid-glide continue the movement without a jerk; a very busy chat skips the backlog instead of scrolling through it. Respects the system's "reduce motion" setting.
- **Zoom:** the chat and the rest of the interface have separate sizes (Settings → *Chat & size*, A− / A+ in the live feed). Shortcuts: Ctrl + plus/minus/0 and Ctrl + mouse wheel – over the chat it changes the chat size.
- **Model picker** for every AI provider: a searchable dropdown instead of typing the exact name. Models load automatically (with readable names where the provider sends them); any other name can still be entered. The Claude Code CLI offers its short names (Haiku, Sonnet, Opus, Fable) with a short description, OpenRouter fills in the prices of the chosen model automatically and lists its models even without a key.
- **Translation of foreign-language chat** (Channels → *Languages & translation*, also in the setup wizard):
  - Set the languages you understand; the first one is the translation target. Until they are set, nothing is translated and the dashboard shows a hint.
  - Messages in other languages show their translation right below them in the live feed (only in the app).
  - Language detection runs locally (script, language-specific letters, common words and statistics), so only foreign messages cost AI tokens. When AI moderation checks the message anyway, the translation comes in the same call.
  - Rule violations in another language are moderated as usual and get a **note with the translation and what is really meant** (slang, coded insults). The note appears in the live feed and the moderation log, and Twitch moderators see it as the timeout or ban reason. A plain deletion becomes a 1-second timeout so the note reaches the mods (can be switched off).
  - With "AI check: suspicious only", foreign messages always get the AI check, because the suspicious-word list only knows German and English.

### Changed
- AI page: effort levels are shown in plain words with an explanation, and the Claude Code CLI also offers "Very high" and "Maximum". Fields the CLI or local models ignore (max tokens, temperature, prices) are hidden. Switching a profile to another provider clears the model, effort and prices of the old one.

### Fixed
- The live feed stopped scrolling along once it reached its entry limit (400 by default).

## [1.3.0] – 2026-10-19

Quality & maintenance pass: every integration was checked against the current Twitch, BetterTTV, FrankerFaceZ, 7TV and AI provider documentation and tested with Claude Code on a live channel.

### Added
- Channel manager catches up with Twitch's API as of October 2026:
  - **Pinned chat messages**: pin any chat message from the live feed, see and unpin the current pin (Advanced).
  - **Clips** with an optional title and a length of 5–60 seconds.
  - **Unban requests**: review, approve or deny with a reply (Expert).
  - **Suspicious users**: put accounts under monitoring or restrict them (Expert).
  - **Hype Train** status and record (Expert), **stream delay** for partners, **channel points voting** in polls, **per-stream / per-viewer limits** and "skip queue" for rewards.
- Reactions to **watch streaks**, paid sub upgrades and pay-it-forward gifts; feed entries for Bits badge tiers and charity donations.
- Long lists (mods, VIPs, banned users) load completely instead of the first 100, banned users can be filtered.

### Fixed
- Connecting a Twitch account in the app could leave it unlinked from the channel (the older local config was saved over the backend's link).
- Commands and bot names with characters whose lowercase form is longer (e.g. "İ") could crash the bot.
- **Shared Chat**: messages and events from the other channels of a shared chat session are no longer moderated, welcomed or thanked; they are marked in the feed.
- Chat messages with GIFs (new Twitch fragment type) keep a "[GIF]" marker instead of losing that part.
- The bot is stopped cleanly when the app closes, so usage numbers and known chatters are saved.
- Panels on the Stream page now reload when you switch to another channel; saving an empty title is blocked; a reward that requires viewer input needs a description.
- "Start bot" (live) is disabled until a streamer account is connected; test mode still works.

### Changed
- Twitch errors are shown with a short explanation in the app's language (missing permission, partner/affiliate only, rate limit …) plus Twitch's original message.
- Network trouble while refreshing the Twitch login no longer claims the session expired.
- Unreadable Twitch name colors (e.g. pure blue on dark backgrounds) are lightened or darkened automatically.
- Claude Code CLI bridge runs with `--safe-mode` in a neutral folder: the user's own Claude Code setup (MCP servers, skills, CLAUDE.md) no longer inflates each request (≈2k instead of up to ≈40k input tokens per message).
- Expert level asks for three more Twitch permissions (unban requests, suspicious users, Hype Train).

## [1.2.0] – 2026-10-19

Tested with Claude Code (update download, contrast of all themes in both modes, icons in every view and the setup wizard). Windows build.

> **Please install this version manually once:** versions 1.0.0–1.1.0 show the update dialog but fail with "Cannot read private member…" when installing. 1.2.0 fixes this, so later updates install from inside the app again.

### Fixed
- In-app updates failed with "Cannot read private member from an object whose class did not declare it" (the update object was wrapped in a reactive proxy).

### Added
- **10 color themes** — Bazaar, Midnight, Ocean, Aurora, Forest, Gold, Sunset, Crimson, Sakura and Cyber — each with a light and a dark version (Settings → Appearance and in the setup wizard). Contrast checked against WCAG AA.
- **Nucleo icons** throughout the app: two-tone icons whose second color follows the theme, plus icons for page titles, section headings, companion/moderation tabs and the setup wizard steps.

### Changed
- Lucide icons removed; README screenshots show the new themes and the channel manager.

## [1.1.0] – 2026-10-19

Tested with Claude Code on a live channel (all read calls, reversible write tests for title/tags, chat settings, blocked terms and chat color, emote images from every source). Windows build.

### Added
- **Stream & channel manager** (new "Stream" page) in three levels, default *Simple*:
  - *Simple*: title, category search with box art, language, tags.
  - *Advanced*: content classification labels, branded content, chat settings, announcements, shoutouts, raids (with cancel), clips, stream markers.
  - *Expert*: polls, predictions, channel point rewards, mods, VIPs, banned users (unban), Twitch AutoMod (overall or per category), shield mode, Twitch blocked terms, ads (run/snooze), stream schedule, channel description, chat color, follower/sub/editor overview.
  - Only the permissions of the chosen level are requested; a banner offers to grant missing ones.
- **Emotes as images** in the live feed and pending messages (Twitch, BetterTTV, FrankerFaceZ, 7TV; also sub emotes from other channels). On by default, switch off under Settings → Appearance.

### Changed
- New Twitch logins also request `channel:manage:broadcast` (needed for the Simple level).

## [1.0.1] – 2026-10-19

Bug-fix release after an end-to-end test of the whole app with Claude Code (live Twitch login and chat, AI replies via the Claude Code CLI, moderation, test mode, settings and setup wizard). Windows build; macOS and Linux follow once the CI builds run again.

### Fixed
- Twitch login failed with "Unexpected device code response" (the device-code answer was parsed in the wrong field case).
- The bot froze on its first AI message (a lock was taken twice while building the prompt); removed a second lock-order risk in the status update.
- Moderation actions appeared twice in the live feed.
- After reloading the window the dashboard showed the bot as stopped while it was still running.
- Busy moments no longer drop AI replies silently; tasks wait for a free slot.
- Announcements of automatic moderation actions no longer repeat a blocked term in chat.
- "Next message" countdown now shows the offline pause right away.

### Added
- Built-in "known spam phrases" rule (viewer-bot and gift-card spam such as "cheap viewers", "free followers", "free nitro") that works without AI. Off by default, because viewers also quote these phrases to joke about bots; with AI moderation such messages are checked in context.
- The Claude Code CLI bridge also finds the CLI bundled with the Claude desktop app and explains how to log it in.

### Changed
- Fixed bot texts (the `!hb` help and moderation announcements) use the stream language (German or English).
- AI messages are asked for the message only (no labels like "Hype moment:") and within the length limit; over-long answers are cut after a full sentence.
- "kys" and similar (self-harm category) now get a 10-minute timeout in the Balanced preset instead of only a deletion.
- Moderation log and feed: rule reasons, verdicts, sources, dates and built-in AI profile names are localized.

## [1.0.0] – 2026-10-18

First public release.

### Companion
- Timed messages with a random interval, adaptive pacing (talks less when chat is busy, pauses when nobody answers) and an "only while live" option.
- Eight message kinds (chat reactions, game facts, tips, trivia, jokes, questions, hype, stream moments) with adjustable weights.
- Replies when addressed (bot name, trigger words, replies to its messages), welcomes first-time chatters, celebrates follows (batched), subs, resubs, gift subs, raids and cheers.
- Chat commands `!hb ask | fact | joke | about` with per-user and global cooldowns and role requirements.
- Language modes: stream language, fixed language or the language most chatters use; optional replies in each chatter's own language.
- Emote awareness for Twitch, BetterTTV, FrankerFaceZ and 7TV, preferring emotes the community actually uses.
- Game research on category change: Wikipedia plus the AI provider's web search, cached per game.
- Preview and approval modes, "say something now", daily AI budget.

### Moderation
- Instant local rules: blocked terms with wildcards and leetspeak detection, links with allow-list, caps, repeated messages, emote spam, message length.
- Context-aware AI classification (batched) with nine categories, strictness, confidence threshold and channel rules in plain language.
- Modes: off, log, suggest, automatic. Escalation for repeat offenders, exemptions, moderation log with undo.
- Five built-in profiles: Balanced (default), Relaxed, Family Friendly, Suggest Only, Watch & Log.

### AI providers
- Anthropic (Messages API), OpenAI (Responses API), Google Gemini (Interactions API with generateContent fallback), OpenRouter, Mistral, Groq, xAI, DeepSeek, Perplexity, Together AI, Ollama, LM Studio, any OpenAI-compatible endpoint, and an optional Claude Code CLI bridge.
- Profiles per provider/model, model list loading, connection test, token and cost tracking.

### App
- Channel profiles for multiple Twitch channels, Twitch login via device code (no password ever touches the app), optional separate bot account.
- Built-in, read-only presets that can be duplicated and edited.
- Setup wizard, chat simulator / test mode, live feed, English and German UI, System/Dark/Light themes.
- Secrets in the OS credential store, automatic updates from GitHub Releases.
