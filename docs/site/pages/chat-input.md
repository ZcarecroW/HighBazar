---
title: Writing in chat
description: Chat yourself from HighBazar, with emotes from every provider, emojis, mentions, Tab completion and favorites.
icon: Chat
---

Under the live feed there is an input field to write in your chat yourself – handy when HighBazar is on your second screen or when you follow chat in the [overlay](overlay.md). It needs the bot to be running (in [test mode](first-stream.md) your messages only show up locally). Switch it off under [Settings → Chat & size](settings.md#chat-and-size) if you don't need it.

![Typing ":pog" suggests emotes from every provider](chat-input.webp)

## Emotes and emojis

Type **`:`** at the start of a word and HighBazar suggests emotes from **every provider** – Twitch (including the emotes of channels you are subscribed to), BetterTTV, FrankerFaceZ and 7TV – and **emojis**, which you can find by English and German keywords (`:fire`, `:herz`).

- <kbd>↑</kbd> <kbd>↓</kbd> choose, <kbd>Enter</kbd> or <kbd>Tab</kbd> inserts.
- Each name is suggested once – the emote chat will actually show, even if several providers have one with that name.
- Your **favorites** and the emotes you use most come first.

Limit the suggestions to one provider by starting with its letter:

| Type | Searches |
|---|---|
| `:7 pog` | only 7TV |
| `:b pog` | only BetterTTV |
| `:f pog` | only FrankerFaceZ |
| `:t pog` | only Twitch |
| `:e heart` | only emojis |

> [!NOTE]
> BetterTTV, FrankerFaceZ and 7TV are browser extensions: viewers only see those emotes as images if they use the extension. Everyone sees Twitch emotes.

## Mentions

Type **`@`** to get suggestions of people who are in chat right now. For names in other scripts (for example Japanese display names), the login is inserted so the mention reaches the person.

## Tab completion and history

- <kbd>Tab</kbd> completes the word you are typing – an emote name or a person in chat – without typing `:` or `@`.
- <kbd>↑</kbd> in an empty field brings back your last message.
- <kbd>Enter</kbd> sends.

## The emote picker

The smiley button next to the field opens the **emote picker** with a tab per provider and a *★ Mine* tab for favorites and recently used emotes. **Right-click** an emote to pin it as a favorite or remove it.

> [!TIP]
> If your subscription emotes are missing, Twitch needs a newer permission for them – HighBazar shows a note with a link to reconnect your account on [Channels](channels.md#accounts).

## In the overlay

The same input – with the same suggestions – is available inside the [chat overlay](overlay.md#chat-input-in-the-overlay): <kbd>Shift</kbd>+<kbd>Tab</kbd> into the Steam overlay, type, <kbd>Enter</kbd>, and you're back in the game.
