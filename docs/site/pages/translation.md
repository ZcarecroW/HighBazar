---
title: Translation
description: Messages in languages you don't speak are translated right below them – and rule breaks get a note with the real meaning for you and your mods.
icon: Languages
---

International chats are great – until someone writes in a language you can't read. HighBazar translates those messages for you, right in the live feed, and can post the translation in chat so everyone can follow. If such a message breaks your rules, moderation acts as usual and adds a **note for your mods** with the translation and what is really meant: slang, coded insults, double meanings.

![Languages & translation on the Channels page](translation.webp)

## Set it up

On [Channels](channels.md), scroll to **Languages & translation**:

1. Add the **languages you understand**. The first one is the **target**: translations are written in it. Click another language to make it the target.
2. Switch on **Translate foreign-language messages**.

Nothing is translated until you've set your languages – the dashboard reminds you.

## Settings

| Setting | Explanation |
|---|---|
| **Translate foreign-language messages** | Uses the channel's *AI for moderation* profile. If AI moderation runs anyway, translating adds very little cost. |
| **Post translations in chat, too** | Posts each translation as a message of its own, e.g. `[HighBazar][Translator] Anna (Spanish): Hello everyone!` – at most 8 a minute. Messages that moderation flags are **never** posted translated. |
| **Prefix in chat** | Goes in front of every translated message (default `[HighBazar][Translator]`). |
| **Send the note to mods on deletions, too** | Twitch has no reason field for plain deletions. With this on, a 1-second timeout is used instead: it removes the message just the same and shows your mods the translation as the reason. |

## How it works

- **Language detection runs locally** on your PC, so only messages in foreign languages cost AI tokens. Emotes, short reactions (“gg”, “lol”) and the streamer's own messages are never translated.
- Messages that the AI moderation checks anyway get their translation from the same request.
- Translations appear right below the original in the live feed (and, if you like, in the [overlay](overlay.md)).
- For a flagged message, the moderation log stores the original, the translation and the meaning – searchable like everything else.

> [!TIP]
> Leave *Reply to chatters in their language* (Companion → Language) off: the bot then answers in your stream's language, and you can read every reply. The translator covers the other direction.
