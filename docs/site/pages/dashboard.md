---
title: Dashboard
description: The live feed, the bot controls, today's numbers, messages waiting for approval and the chat simulator.
icon: LayoutDashboard
---

The dashboard is where you spend your stream: everything that happens in chat, what the bot does, and the controls to steer it.

![The dashboard while live](dashboard.webp)

## Starting and stopping the bot

| Button | What it does |
|---|---|
| **Start bot** | Connects to your Twitch chat and starts the bot with the active channel profile. Answers, moderation and the overlay work right away; timed messages wait until you are live (if *Only while live* is on). |
| **Test mode** | Runs the bot without writing anything to Twitch – see [Your first stream](first-stream.md). |
| **Stop** | Disconnects the bot. |
| **Say something now** | Asks the bot for a timed message right away instead of waiting for the countdown. |
| **Refresh stream info** | Reloads title, category and viewers, and all emotes of Twitch, BetterTTV, FrankerFaceZ and 7TV. |
| **Run in background** | Closes the interface; the bot keeps running in the notification area ([performance mode](settings.md#performance)). |

The header shows the channel, whether you are **LIVE** (with uptime), the category and the number of viewers.

## The live feed {#live-feed}

Everything that happens, in order: chat messages, the bot's messages, follows, subs, raids, system notes and moderation actions.

- **Bot messages** carry their reason: *timed*, *reply*, *welcome*, *event*, *command* or *moderation*.
- **First-time chatters** are marked *new*; messages from a [shared chat](twitch-glossary.md#shared-chat) partner are marked as such and are not moderated.
- **Emotes** of Twitch, BetterTTV, FrankerFaceZ and 7TV are shown as images – zero-width emotes such as 7TV's RainTime sit on top of the emote before them. You can switch images off under [Settings](settings.md#chat-and-size).
- **Translations** of foreign-language messages appear right below them when [translation](translation.md) is on.
- **Moderation entries** show the category, the action and the reason, with **Undo** where Twitch allows it.

Point at a chat message for quick actions: **pin it** in chat, **delete** it, **time the person out** for 10 minutes or **ban** them.

The buttons above the feed filter what is shown (*Chat*, *Bot*, *Moderation*, *System*) and switch **auto-scroll**. **A− / A+** (or <kbd>Ctrl</kbd> + mouse wheel over the chat) change the chat size independently of the rest of the interface. New messages glide in smoothly – speed and curve are adjustable in [Settings](settings.md#chat-and-size).

Under the feed you can [write in chat yourself](chat-input.md), with emote and emoji suggestions.

## Today's numbers

| Card | Meaning |
|---|---|
| **Next message** | Countdown to the next timed bot message. It pauses while you are offline, when chat is very busy, during an ad break, or when nobody answered the last messages – the card says why. |
| **AI cost today** | What today's AI calls cost, estimated from the prices in your AI profile. The bar shows how much of the [daily budget](costs.md#daily-budget) is used. |
| **Messages today** | How many messages the bot sent. |
| **Chat / min** | Viewer messages per minute. Above the *busy* limit (Companion → Timing) the bot holds back its timed messages. |
| **Emotes** | How many emotes your channel knows – Twitch plus BetterTTV, FrankerFaceZ and 7TV. Changes to your 7TV set arrive live. |
| **Game knowledge** | Facts and tips the bot looked up about the current game, or *Researching…* while it reads up ([web search](web-search.md#game-research)). |

## Quick controls

- **Companion** and **Moderation** switch each part off and on instantly, without stopping the bot.
- **Sending** changes how bot messages go out: *Automatic*, *Preview, then send automatically* or *Only after my approval* (the same setting as on [Channels](channels.md#sending)).

## Waiting to be sent {#pending}

In *Preview* or *Approval* mode, bot messages land here first. In *Preview* they go out by themselves when the timer runs out; in *Approval* only when you click **Send**. You can **edit**, **send** or **discard** each one.

## Chat simulator

Write as a fake viewer and simulate events (follow, sub, resub, gift subs, raid) to see how the bot reacts. It works in test mode and while live – simulated messages stay on your PC and never reach Twitch. See [Your first stream](first-stream.md#chat-simulator) for ideas.

## Quick search

Press <kbd>Ctrl</kbd> + <kbd>K</kbd> anywhere (or click **Search** in the sidebar) to jump to any page, setting, profile or action – *start the bot*, *edit title*, *dark theme*. Typos and abbreviations are forgiven.

![Quick search finds pages, settings and actions](palette.webp)
