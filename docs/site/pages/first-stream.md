---
title: Your first stream
description: Try everything safely in test mode with the chat simulator, then go live with a short checklist.
icon: Play
---

Before HighBazar talks in your real chat, try it in **test mode**: the bot runs exactly as it would live, but nothing is sent to Twitch – replies, welcomes and moderation actions only appear in your live feed.

## Start test mode

On the [Dashboard](dashboard.md), click **Test mode**. A banner shows that messages and moderation actions are only simulated.

> [!NOTE]
> Test mode works even without a connected Twitch account. It doesn't connect to your real chat at all: the chat simulator below is your audience. Stream title, category and emotes are still loaded when an account is connected, so messages fit your stream.

## Play viewer with the chat simulator {#chat-simulator}

The **Chat simulator** on the right of the dashboard lets you write as a fake viewer. Pick a name and a role (viewer, subscriber, VIP, moderator), type a message and press Enter. Try these:

| Try | What should happen |
|---|---|
| `hey HighBazar, what game is this?` | The bot answers you by name (it reacts to its name and the trigger words). |
| `@HighBazar what was in the latest patch?` | The bot looks the answer up on the web and replies with the facts and their date. |
| `!hb ask who is the final boss?` | A chat command – the same as asking, with its own cooldown. |
| `!hb fact` / `!hb joke` | A fun fact about the game, or a joke. |
| `!hb about` | A short introduction with the available commands (no AI needed). |
| A message from a new name | The bot welcomes the first-time chatter after a few seconds. |
| `free followers at spam.example` | Moderation steps in (depending on your profile). |

Below the input, **Simulate event** sends a follow, sub, resub, gift subs or a raid – the bot thanks them like it would live.

Use **Say something now** at the top to get a timed message right away instead of waiting for the countdown.

## Check how it sounds

If a message doesn't fit your stream:

- Change the **personality**, the humor, energy and teasing sliders, or add **extra instructions** on the [Companion](companion.md) page.
- Tell it about your stream (**About you and your stream** on [Channels](channels.md#behavior)) – community name, running jokes, what your emotes mean.
- Set **Sending** to *Preview* or *Only after my approval* to check every message before it goes out ([Channels](channels.md#sending)).

## Going live – a checklist

1. **Streamer account connected** (Channels) – the dashboard shows *Connected*.
2. **AI tested** – *AI → Test* answers.
3. **Budget set** – the daily AI budget (default $2) pauses AI features when it is reached.
4. **Moderation profile chosen** – *Balanced* fits most streams; *Suggest Only* if you want to approve every action.
5. **Category set** on Twitch – the bot researches the game when it changes.
6. Click **Start bot**.

Answers, moderation and the overlay work right away. **Timed messages** wait until you are live (if *Only while live* is on) and pause when chat is very busy or nobody answered the last few.

> [!TIP]
> Want HighBazar to start with your PC and run in the background? Turn on *Start the bot automatically when the app opens* and *Start in the notification area* under [Settings](settings.md#performance).

## During the stream

- The **live feed** shows everything: chat, bot messages with the reason (timed, reply, welcome, event, command), follows, subs, raids and moderation. Point at a message to pin it, delete it, time the person out or ban them.
- **Waiting to be sent** lists bot messages in preview or approval mode – edit, send or discard them.
- The **Companion** and **Moderation** switches under *Quick controls* turn each part off and on instantly.

When you end the stream, timed messages stop by themselves. Click **Stop** to disconnect the bot completely.
