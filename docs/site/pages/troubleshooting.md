---
title: Troubleshooting
description: Solutions for the most common problems – connecting Twitch, AI errors, web search, missing emotes, moderation, the overlay and updates.
icon: Activity
---

Most problems show up as a red line in the **live feed** – start there. Errors are also written to the [log file](privacy.md#where-your-data-is).

## Twitch

### “Needs reconnect” or features don't work

Click **Reconnect** next to the account on [Channels](channels.md#accounts) and approve the permissions on Twitch. This is needed after a password change, when Twitch revoked the login, or when a feature needs a permission you haven't approved yet (for example a higher level of the [stream manager](stream-manager.md)).

### The bot doesn't react to follows, or can't moderate

Reconnect the **streamer account** – it needs the moderation and follower permissions. If you use a separate bot account for writing, moderation still runs through the streamer account. For the bot account to delete or time out messages in its own name, it must be a moderator in your chat (`/mod YourNameBot`).

### “Twitch event … unavailable”

Some events need extra permissions or a specific status. Channel point redemptions and ad breaks, for example, need the permissions of the stream manager's *Expert* level and are only available to affiliates and partners. Switch the [stream manager](stream-manager.md) to *Expert* and approve the permissions if you want them. The feed note is informational – everything else keeps working.

### The login code expired

Codes are valid for a few minutes. Click **Connect** again for a new one.

## AI

### “No AI profile selected”

Choose an **AI profile** for the channel on [Channels](channels.md#profiles). Without one, only the local moderation rules work.

### Authentication errors (401, invalid key)

The API key is wrong, expired or has no credit. Paste a new one on *AI → API keys*, and check your account balance on the provider's website. **Test** on the AI page shows whether it works.

### “The model used up Max. output tokens before writing an answer”

Reasoning models count their thinking toward the output limit. Raise **Max. output tokens** in the AI profile, or choose a lower **reasoning effort**.

### Timeouts

The provider didn't answer in time. Raise the profile's **Timeout** – especially for local models – or pick a faster model.

### “Daily AI budget reached”

AI features pause until tomorrow. Raise the budget on [Channels](channels.md#daily-budget) or set it to 0 for no limit. See [Costs](costs.md).

### The bot doesn't answer someone

- Check the **cooldowns** (per user 30 s, global 4 s by default) and **Who can use it** under [Companion → Reactions](companion.md#replies-when-addressed).
- The message must contain the bot name or a trigger word – or be a reply to a bot message.
- In *Preview* or *Approval* mode, answers wait under [Waiting to be sent](dashboard.md#pending).
- The AI may decide that nothing fits and skip. An answer that contains a link is never posted – the log file says why a message wasn't sent.

## Web search

### The bot says it couldn't find something current

- Check that **Allow web search** is on in the AI profile and *Look up current facts when chat asks something* on the [Companion](companion.md#research) page.
- Try **Test with web search** on the AI page with the same question: it shows the answer and how many searches ran.
- With a provider that uses HighBazar's own search, use **You.com** or a search with a key; **Wikipedia** alone can't know this week's patch ([Web search](web-search.md#highbazars-own-search)).
- Small or local models follow search results less reliably. A small cloud model with its own search (Claude Haiku 5.5, GPT-6 Luna, Gemini Flash) is a cheap, reliable choice.

### “Research for … failed”

The feed shows the reason. HighBazar retries after 15 minutes, up to three times per game. With the Claude Code CLI, make sure it is logged in (run `claude` in a terminal once and type `/login`). *Settings → Clear game research cache* forces a fresh start.

## Claude Code CLI and Codex CLI

- **“Claude Code CLI not found”** – install Claude Code, or set the path to the `claude` executable in the AI profile and click **Detect**. The CLI bundled with the Claude desktop app is found too.
- **“Claude Code CLI is not logged in”** – open a terminal, run `claude` and type `/login` once.
- **“Codex CLI is not logged in”** – run `codex login` in a terminal once, then click **Detect**.

## Emotes

- **Emotes show as words:** switch on *Show emotes as images* ([Settings](settings.md#appearance)), and click **Refresh stream info** on the dashboard to reload them.
- **Your subscription emotes are missing in the chat input:** Twitch needs a newer permission – the input shows a link to reconnect your account.
- **7TV changes don't arrive:** they come live while the bot runs. If 7TV was unreachable, the next refresh (every 30 minutes, or *Refresh stream info*) catches up.

## Moderation

- **Too strict or too loose:** adjust **Strictness** and **Min. confidence**, write your **channel rules** in plain words, or start from another built-in profile ([Moderation](moderation.md)). *Suggest Only* lets you approve every action while you tune it.
- **“AI moderation is at its per-minute limit”:** raise *Max. AI checks per minute*, or use *Suspicious messages only*. The instant rules keep working.
- **An action failed:** the bot (or its account) needs moderator rights in the channel; the log shows the error from Twitch.

## Overlay

- **The overlay doesn't load:** check that *Enable chat overlay* is on and the bot – or at least HighBazar – is running. If the port is taken, pick another one under *Port*.
- **“Port … is in use”:** another program uses the port; choose another one (e.g. 47824).
- **The link stopped working:** you created a new link; copy the current one.

## Updates and installation

- **The update check fails:** check your internet connection or firewall; *Check now* on the Settings page retries. You can always download the newest version from the [releases page](https://github.com/ZcarecroW/HighBazar/releases/latest).
- **Windows warns about the installer** or **macOS says the app is damaged:** see [Installation](installation.md#windows-protected-your-pc).

## Reporting a problem {#reporting-a-problem}

Open an issue on [GitHub](https://github.com/ZcarecroW/HighBazar/issues) (*Settings → Report a problem*) and include:

1. your HighBazar version (*Settings → About*) and your system,
2. what you did and what happened instead,
3. the red line from the live feed, and the relevant part of the [log file](privacy.md#where-your-data-is) – it contains no keys, but check it for anything private before you post it.
