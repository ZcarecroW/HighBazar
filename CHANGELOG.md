# Changelog

All notable changes to HighBazar are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed
- Twitch login failed with "Unexpected device code response" (the device-code answer was parsed in the wrong field case).
- The bot froze on its first AI message (a lock was taken twice while building the prompt); removed a second lock-order risk in the status update.
- Moderation actions appeared twice in the live feed.
- After reloading the window the dashboard showed the bot as stopped while it was still running.
- Busy moments no longer drop AI replies silently; tasks wait for a free slot.
- Announcements of automatic moderation actions no longer repeat a blocked term in chat.
- "Next message" countdown now shows the offline pause right away.

### Added
- Built-in "known spam phrases" rule (viewer-bot and gift-card spam such as "cheap viewers", "free followers", "free nitro") that works without AI; on by default.
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
