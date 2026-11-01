---
title: Channels
description: Channel profiles bundle everything for one Twitch channel – accounts, bot name and prefix, profiles, sending mode, budget and what the AI knows about you.
icon: Radio
---

A **channel profile** holds everything for one Twitch channel or setup. Most streamers need just one; create more if you moderate for a friend, stream on two channels or want a separate setup for special streams. Switch between them with the selector at the top of the dashboard.

![The Channels page](channels.webp)

## Accounts {#accounts}

| Setting | Explanation |
|---|---|
| **Streamer account** | Your own Twitch account – the one you stream with. You log in on twitch.tv with a short code, so HighBazar never sees your password. It is needed to read chat, see follows, subs and raids, and to moderate. |
| **Separate bot account** *(optional)* | A second account only for the bot, e.g. *YourNameBot*. Chat then sees the bot under its own name. Make it a moderator in your chat (`/mod YourNameBot`) for the moderation features, then connect it here. Moderation itself always runs through your streamer account. |

If an account shows **Needs reconnect**, click **Reconnect** and approve the permissions on Twitch again – for example after you changed your password, or when a feature needs a permission you haven't granted yet.

## Bot name and prefix

| Setting | Default | Explanation |
|---|---|---|
| **Bot name** | `HighBazar` | How the bot calls itself. A message that contains this name (“hey Bazar, what game is this?”) gets an answer. Pick something short that nobody types by accident. More nicknames go into *Extra trigger words* on the [Companion](companion.md#replies-when-addressed) page. |
| **Message prefix** | `[HighBazar]` | Shown in front of every bot message when it writes with your account, so viewers know it's the bot – Twitch's rules ask for automated messages to be recognizable. |

## Features

- **Companion (chatting)** – timed messages, answers, welcomes and thank-yous. Switch it off if you only want moderation.
- **Moderation** – protects your chat according to the moderation profile.

Both can also be toggled live from the dashboard's *Quick controls*.

## Profiles

| Setting | Explanation |
|---|---|
| **Companion profile** | How the bot chats – see [Companion](companion.md). |
| **Moderation profile** | How strict moderation is – see [Moderation](moderation.md). |
| **AI profile** | The model that writes the bot's messages – see [AI providers](ai-providers.md). Without an AI only the local moderation rules work. |
| **AI for moderation** | Moderation checks many messages, so a fast, cheap model (e.g. Claude Haiku 5.5 or GPT-6 Luna) saves a lot of money without being worse at it. *Same as above* uses the main AI. |

## Behavior {#behavior}

### Sending {#sending}

| Mode | What happens |
|---|---|
| **Automatic** | The bot posts right away. |
| **Preview, then send automatically** | Each message waits on the dashboard for the *preview time* (default 10 seconds); you can edit or discard it before it goes out. |
| **Only after my approval** | Nothing is posted until you click **Send** – a good way to get to know the bot. |

### Daily AI budget {#daily-budget}

AI providers charge per use. When today's costs reach this amount (default **$2**), all AI features pause until tomorrow; the local moderation rules keep working. `0` means unlimited. See [Costs](costs.md).

### About you and your stream

Facts the AI may use: your name or nickname, what you usually play, your schedule, the name of your community, running jokes, what your emotes mean. The more it knows, the more it sounds like one of your regulars. Nothing private, please – this text goes to the AI provider with every request.

## Languages & translation

The lower part of the page sets the languages you understand and how foreign-language messages are translated – see [Translation](translation.md).
