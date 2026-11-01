---
title: Costs & budget
description: What HighBazar costs to run, how to keep AI costs low, the daily budget and the usage statistics.
icon: Coins
---

**HighBazar itself is free.** AI providers bill you directly for what the bot uses. With a small model that's usually a few cents per stream hour; local models cost nothing.

## Rough costs per stream hour

With default settings – about 15 timed messages an hour, a few replies and welcomes:

| Model class | Companion / hour | AI moderation / hour (moderate chat) |
|---|---|---|
| **Small** – Claude Haiku 5.5, GPT-6 Luna, Gemini Flash-Lite | < $0.02 | < $0.02 |
| **Mid** – Claude Sonnet 5.5, GPT-6.1 Sol, Gemini 3.8 Flash | ~ $0.10–0.25 | ~ $0.10–0.30 |
| **Flagship** – Claude Opus 5.5 | ~ $0.20–0.50 | use a small model for moderation |
| **Local** – Ollama, LM Studio | $0 | $0 |
| **Subscription** – Claude Code CLI, Codex CLI | counts against your plan | counts against your plan |

*Estimates – actual costs depend on chat activity, context size and the provider's prices.* Web searches add roughly 0.5–1.5 cents each on paid APIs.

## Daily budget {#daily-budget}

Each channel has a **daily AI budget** (default **$2**, set on [Channels](channels.md#daily-budget)). When today's costs reach it, all AI features – companion, AI moderation, translation – pause until tomorrow, and the feed says so. The instant moderation rules keep working. `0` means unlimited.

The dashboard's **AI cost today** card shows how much of the budget is used.

## Keeping costs low

- **Use a cheap model for moderation.** On [Channels](channels.md#profiles), set *AI for moderation* to Claude Haiku 5.5, GPT-6 Luna or Gemini Flash-Lite – just as good at telling banter from hate, for a fraction.
- **Check only suspicious messages.** *AI check → Suspicious messages only* ([Moderation](moderation.md#general)) catches most cases at a fraction of *All messages*. A cap on AI checks per minute (default 12) protects you from spam waves.
- **Fewer context messages.** *Chat messages as context* (Companion → Timing) is the biggest part of each request; 30–40 is plenty for most streams.
- **Fewer emotes listed.** The bot gets fewer emote names per request when *Emote usage* is set to *Rarely*.
- **Low reasoning effort.** For chat and moderation, *Low* or the provider default is usually best.
- **Prompt caching** is automatic: the part of the request that is the same every time (the bot's instructions) is cached by Claude, OpenAI, Gemini and OpenRouter, and HighBazar counts cached tokens at their real, much lower price.

## Usage statistics

**AI → Usage** shows per day and channel: AI calls, tokens, web searches, cost, messages sent and moderation actions.

![Usage statistics](ai-usage.webp)

The costs are estimated from the prices in your AI profiles (OpenRouter reports exact costs). For the exact bill, check your provider's dashboard.
