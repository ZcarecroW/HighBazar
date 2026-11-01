---
title: FAQ
description: Short answers to the questions streamers ask most about HighBazar.
icon: Info
---

## Does the bot pretend to be me?

No. Messages sent through your account carry a prefix (default `[HighBazar]`), and the AI is instructed never to speak in your name. You can also connect a separate bot account so chat sees the bot under its own name – see [Channels](channels.md#accounts).

## Will it spam my chat?

No. Timed messages are spread out (3–5 minutes by default), pause when chat is very busy, stop after a few messages without any answer, and only run while you're live. Replies and welcomes have cooldowns and rate limits. *Quiet Helper* has no timed messages at all. See [Companion → Timing](companion.md#timing).

## How strict is the default moderation?

The *Balanced* profile only acts automatically on **clear** hate, harassment, threats, scams and spam (delete or timeout, escalating for repeat offenders). Swearing, banter, dark humor and criticism are allowed; borderline cases are only logged for you. Mods and VIPs are exempt. Try *Suggest Only* if you want to approve every action. See [Moderation](moderation.md).

## What does it cost?

HighBazar is free. AI providers bill you directly: with a small model a stream hour costs a few cents, local models nothing, and a daily budget (default $2) caps it. See [Costs](costs.md).

## Can I use my Claude Pro or ChatGPT subscription?

Third-party apps may not sign in with a Claude.ai subscription; the official way is an **API key** from the [Claude Console](https://platform.claude.com) – pay per use, and Claude Haiku costs fractions of a cent per message. As an optional alternative, HighBazar can call your locally installed and logged-in **Claude Code CLI**, which runs on your subscription, or the **Codex CLI** with your ChatGPT plan. Whether an always-on chat bot is covered by your plan's terms is your responsibility. See [AI providers](ai-providers.md#claude-code-cli).

## Which AI should I use?

For most streams: a mid-sized or small cloud model for chatting (Claude Sonnet or Haiku 5.5, GPT-6.1 Terra, Gemini 3.8 Flash) and a small one for moderation. Local models are free and private but need a strong graphics card and follow web search results less reliably. See [AI providers](ai-providers.md).

## Does it work without an AI?

Partly: the **instant moderation rules** (blocked terms, links, caps, spam) work without any AI, as do the stream manager, the overlay and writing in chat. Chatting, AI moderation, translation and game research need an AI profile.

## Does it work in my language?

The interface is in English and German. The bot writes in your stream's language, a fixed language, or the language most chatters use – any language the AI model speaks. Messages in languages you don't understand can be [translated](translation.md) for you.

## Does it slow down my game?

No – the bot itself needs hardly any CPU. HighBazar runs with a lower process priority, the interface pauses when the window is minimized, and in performance mode it runs in the background with about 9 MB of memory. See [Settings → Performance](settings.md#performance).

## Where is my data stored?

Everything stays on your computer: settings in the app's data folder, API keys and Twitch tokens in your system's credential store. Chat messages are only sent to the AI provider you chose. See [Privacy & security](privacy.md).

## Can I use it for several channels?

Yes – create a [channel profile](channels.md) for each. The bot runs for the active one.

## Can I use my own Twitch application?

Yes: register a **Public** application on [dev.twitch.tv](https://dev.twitch.tv/console) (category *Chat Bot*, any redirect URL such as `http://localhost:17563`) and paste its Client ID under *Settings → Advanced*.

## Is HighBazar open source?

Yes, under the MIT license – the code is on [GitHub](https://github.com/ZcarecroW/HighBazar). The icons are from [Nucleo](https://nucleoapp.com) and licensed to the author for HighBazar only; they are not covered by the MIT license.

## Is it affiliated with Twitch?

No. HighBazar is an independent project and not affiliated with Twitch, Anthropic, OpenAI, Google or any other provider.
