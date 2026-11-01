---
title: Setup wizard
description: The guided first start – connect Twitch, pick an AI, choose how the bot behaves and how strictly chat is protected.
icon: Wand
---

On the first start HighBazar opens a short wizard. It takes about five minutes, every step can be skipped, and you can reopen it any time from the sidebar (**Setup wizard**) or under *Settings → Start & updates*.

![The first step of the setup wizard](wizard.webp)

## 1. Welcome

Pick the interface **language** (English or German, or *Automatic* to follow your system) and a **color theme**. Both can be changed later under [Settings](settings.md#appearance).

## 2. Twitch – your streamer account {#twitch}

Click **Connect**. HighBazar shows a short code and opens `twitch.tv/activate`, where you enter it and approve the permissions. You log in on Twitch's own page, so **HighBazar never sees your password**.

The streamer account is the one you stream with. HighBazar reads your chat with it, sees follows, subs and raids, and moderates with it. By default the bot also writes with it.

> [!TIP]
> HighBazar only asks Twitch for the permissions it needs right now. When you later switch the [stream manager](stream-manager.md) to *Advanced* or *Expert*, it asks once more for the extra permissions.

## 3. Bot account (optional) {#bot-account}

You can connect a **second Twitch account just for the bot**, for example *YourNameBot*. Chat then sees the bot under its own name instead of your name with a prefix.

1. Create a free account on twitch.tv.
2. Make it a moderator in your chat: type `/mod YourNameBot` in your chat.
3. Connect it here the same way as your streamer account.

Most streamers start without one. Without a bot account, the switch **Mark bot messages with a prefix** keeps the bot's messages recognizable (default prefix `[HighBazar]`).

## 4. AI

Pick the AI that writes the bot's messages. The list contains ready-made profiles for every provider – Claude, OpenAI, Gemini, OpenRouter, Mistral, DeepSeek, Perplexity, local models with Ollama or LM Studio, and the Claude Code and Codex command-line tools that use your existing subscription.

![Choosing an AI profile in the wizard](wizard-ai.webp)

For a cloud provider, paste an **API key** – a password for the provider that you create on their website (the **Get a key** link takes you there; you usually add a payment method and pay only for what the bot uses). Then click **Test connection**: the AI writes a short greeting, and you see that the key works.

> [!TIP]
> Not sure which to pick? A small, fast model such as **Claude Haiku 5.5**, **GPT-6 Luna** or **Gemini 3.5 Flash-Lite** costs a few cents per stream hour and is great for moderation. Larger models write wittier messages. [AI providers](ai-providers.md) compares them, and you can combine two – a great one for chatting and a cheap one for moderation.

## 5. Companion

Choose a **companion style** – how the bot behaves – and roughly how often it should write on its own:

| Style | In short |
|---|---|
| **Balanced Companion** | Friendly, witty all-rounder. Reacts to chat, shares game facts, asks questions. |
| **Small Stream Booster** | Talks a bit more often and asks lots of questions to get lurkers chatting. |
| **Cozy & Chill** | Calm, warm and laid-back. Fewer messages, gentle humor. |
| **Hype Buddy** | High energy, celebrates every play and every new chatter. |
| **Game Expert** | Facts, lore, tips and trivia – researches the current game online. |
| **Comedian** | Quick-witted and playful, with light teasing. |
| **Quiet Helper** | No timed messages – only answers, welcomes, events and commands. |

You can also tell the AI a bit **about you and your stream** – what you play, your community's name, running jokes. Everything can be fine-tuned later on the [Companion](companion.md) page.

## 6. Moderation

Choose how strictly chat is protected:

| Profile | What it does |
|---|---|
| **Balanced** *(default)* | Acts only on clear hate, harassment, threats, scams and spam. Banter, swearing and criticism stay allowed; borderline cases are only logged. |
| **Relaxed** | Maximum freedom: only the most severe cases are acted on, everything else is logged. |
| **Family Friendly** | Strict: also removes borderline content, sexual references, excessive caps and links. |
| **Suggest Only** | Never acts on its own – flags messages with a suggested action you apply with one click. |
| **Watch & Log** | Only observes and keeps a log. |

If you speak only some languages, set the **languages you understand** here – messages in other languages are then translated for you ([Translation](translation.md)).

## 7. Done

Click **Try in test mode** to see the bot in action without sending anything to Twitch – the [first stream guide](first-stream.md) shows what to try – or **Start bot** to go live right away.
