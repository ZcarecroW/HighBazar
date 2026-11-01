---
title: Introduction
description: What HighBazar is, what it does for your stream and how its parts fit together.
icon: Sparkles
---

HighBazar is a small desktop app that sits next to your stream and brings life into your Twitch chat. It uses an AI model of your choice – Claude, GPT, Gemini, a local model and more – and does four things at once:

- **It chats like a friendly regular.** Every few minutes it says something fun, interesting or engaging that fits what is happening in chat and in your game.
- **It reacts to your community.** It answers when someone talks to it, welcomes first-time chatters and thanks follows, subs, gift subs, raids and cheers.
- **It knows your game.** When you switch the category, it reads up on the game – Wikipedia and a web search – and keeps facts, tips and the latest patch at hand. When chat asks about something current, it looks it up.
- **It keeps chat safe.** Instant local rules catch spam, links and blocked words for free; an AI that understands context tells banter from real hate and acts the way you decide.

![The dashboard: live feed, stats, quick controls and the chat simulator](dashboard.webp)

It is built for **small streamers** who want a lively chat from the very first viewer – and it scales to **big channels** with adaptive pacing, batched AI checks and a daily budget.

## Who writes the messages?

By default the bot writes with **your own streamer account**, and every bot message starts with a prefix such as `[HighBazar]` so viewers can tell it apart from you. If you prefer, you can connect a **separate bot account** (for example *YourNameBot*); chat then sees the bot under its own name. See [Channels](channels.md#accounts).

> [!NOTE]
> HighBazar never pretends to be you. The AI is told to never speak in your name, and Twitch's rules ask for automated messages to be recognizable – that's what the prefix is for.

## How the parts fit together

HighBazar is organized in **profiles**, so you can switch whole setups with one click and keep the built-in presets as a safe fallback:

| Profile | What it holds | Page |
|---|---|---|
| **Channel profile** | One Twitch channel or setup: the accounts, the bot name and prefix, which companion, moderation and AI profile to use, how messages are sent, the daily budget, translation. | [Channels](channels.md) |
| **Companion profile** | How the bot chats: personality, timing, topics, reactions, commands, game research. Seven built-in styles. | [Companion](companion.md) |
| **Moderation profile** | How strictly chat is protected: mode, AI check, categories and actions, repeat offenders, instant rules, exemptions. Five built-in profiles. | [Moderation](moderation.md) |
| **AI profile** | Which provider and model writes the messages, with key, reasoning effort, limits, prices and web search. Presets for every provider. | [AI providers](ai-providers.md) |

Built-in profiles are read-only and improve with updates. To change one, click **Duplicate & edit** – your copy is yours to keep.

## What you can do with it

<div class="cards">
<a href="../companion/"><strong>{icon:Bot} Companion</strong><span>Timed messages, replies, welcomes, event thank-yous, chat commands and game research.</span></a>
<a href="../moderation/"><strong>{icon:Shield} Moderation</strong><span>Local rules, AI checks in nine categories, escalation, suggestions and a log with undo.</span></a>
<a href="../translation/"><strong>{icon:Languages} Translation</strong><span>Foreign-language messages translated for you, with notes for your mods.</span></a>
<a href="../stream-manager/"><strong>{icon:MonitorPlay} Stream manager</strong><span>Title, category, chat modes, raids, polls, channel points, AutoMod, ads.</span></a>
<a href="../chat-input/"><strong>{icon:Chat} Chat yourself</strong><span>Write in chat with emote and emoji suggestions from every provider.</span></a>
<a href="../overlay/"><strong>{icon:Gamepad2} Chat overlay</strong><span>Your chat in the Steam overlay or OBS – follow it without Alt+Tab.</span></a>
</div>

## Explained everywhere

You don't need to know Twitch inside out to use HighBazar. Every setting has a small **ⓘ** next to it – point at it, focus it with the keyboard or click it to keep it open. It explains the setting in plain words, including Twitch terms such as raids, VIPs, AutoMod or channel points. The [Twitch glossary](twitch-glossary.md) collects them all in one place.

![A tooltip explains the moderation modes](tooltip.webp)

## Free, private and light

- **Free and open source** (MIT). AI providers bill you directly for what the bot uses – usually a few cents per stream hour with a small model, nothing with a local model. See [Costs](costs.md).
- **Your data stays on your PC.** API keys and Twitch logins are kept in your system's credential store; chat messages only go to the AI provider you chose. See [Privacy & security](privacy.md).
- **Made for gaming PCs.** A native Rust core instead of a bundled browser, a lower process priority, and a performance mode that runs the bot with about 9 MB of memory in the background. See [Settings](settings.md#performance).

## Next steps

1. [Install HighBazar](installation.md) on Windows, macOS or Linux.
2. Walk through the [setup wizard](setup-wizard.md) – about five minutes.
3. Try everything safely in [test mode](first-stream.md) before your next stream.
