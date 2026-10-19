# Changelog

All notable changes to HighBazar are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

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
