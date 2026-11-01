---
title: Web search & game research
description: How the bot looks up current facts when chat asks, how it researches your game, which search sources there are and what they cost.
icon: Globe
---

AI models only know the world up to their training date. For anything newer – today's patch, a release date, last night's tournament – HighBazar lets the AI **search the web**. It happens in two places:

- **Answers to chat**: when someone asks the bot something that needs current facts, it looks it up first.
- **Game research**: when you switch the category, the bot reads up on the game once – including the latest update – and keeps facts and tips for its messages.

Both are on by default and can be switched off on the [Companion](companion.md#research) page. Every request also carries **today's date and your local time**, so the model knows how recent “latest” has to be.

## Answers to chat {#answers-to-chat}

When someone @mentions the bot, uses `!hb ask`, redeems an *Ask the bot* reward, or asks a question in chat that the bot picks up with a *React to chat* message, the AI may search before it answers. It searches when the answer depends on current or specific facts – updates, patches, release dates, news, events, results, stats, prices – and answers small talk right away. For recent things it puts the current month and year into the query, so old patch notes don't win.

The answer is a normal chat message: no links, no source list, no “I searched for that”. Search results count as untrusted text – the AI never follows instructions found in them.

> [!NOTE]
> An answer with a search takes a few seconds longer (roughly 5–20 seconds, depending on the provider and the model).

## Game research {#game-research}

When the category changes, the bot gathers material about the new game:

1. The game's **Wikipedia** summary (free, no key) as a reliable base.
2. A **web search** for the current state: the latest update, patch, season or DLC and recent news.
3. The AI turns this into **15 facts and 8 tips**, each one short sentence, with the newest update and its date among the facts.

The result is cached per game and reused for 14 days (*Refresh after*). The dashboard shows how many facts and tips are ready under **Game knowledge**, and the feed notes *Researched “Game”: 15 facts and 8 tips ready*.

The bot uses these facts in its *Game facts* and *Game tips* messages and as background for answers. Non-game categories such as *Just Chatting*, *Music* or *Art* aren't researched.

If the research fails – a timeout, a network hiccup – the feed says why, and HighBazar tries again after 15 minutes (up to three times per game). *Settings → Clear game research cache* forces a fresh research.

## Who searches?

| AI provider | Search |
|---|---|
| Claude, OpenAI, Gemini, Grok, OpenRouter, Perplexity, Groq *compound* | The provider's **own web search**. Paid APIs charge roughly 0.5–1.5 cents per search. Claude may also open a found page during game research (capped per page). |
| Claude Code CLI, Codex CLI | The CLI's own web search – through your subscription, at no extra cost. |
| Ollama, LM Studio, DeepSeek, Mistral, Together AI, other Groq models, custom servers | **HighBazar's own search** (below). |

The AI page shows which applies to a profile, next to *Allow web search*.

## HighBazar's own search {#highbazars-own-search}

For providers without a search of their own, HighBazar searches and hands the results to the model. For a chat question, the model first decides whether it needs a search and what to search for; for game research the search runs right away.

![Choosing the search source](ai-websearch.webp)

Choose the source under **AI → Web search**:

| Source | Key | Cost | Notes |
|---|---|---|---|
| **You.com** *(default)* | none | free | Current web search without sign-up (about 100 searches a day). When the daily limit is reached, Wikipedia steps in. Recommended. |
| **Wikipedia** | none | free | Good for games, lore and background – not for current news or patches. |
| **Ollama web search** | free Ollama account | free tier | Current web search by Ollama. |
| **Exa** | yes | $20 starter credit plus $10 free every month, then about 0.7 cents per search | Very accurate, with text excerpts. |
| **Serper (Google)** | yes | 2,500 searches free once, then about 0.1 cents per search | Real Google results including direct answers. |
| **Tavily** | yes | 1,000 searches a month free | Web search made for AI. |
| **Brave Search** | yes | monthly starter credit for about 1,000 searches | Independent search index. |
| **Own SearXNG instance** | none | free | If you run [SearXNG](https://docs.searxng.org) yourself, e.g. with Docker. Its `settings.yml` must list `json` under *search → formats*. |

**Results per search** (default 5) – more results give the model more to work with but cost more tokens. **Test the search** runs a query and shows the results.

## Tips

- If the bot says it couldn't find something current, try **Test with web search** on the AI page with the same question – it shows how many searches ran.
- With HighBazar's own search, **You.com** gives current results; Wikipedia alone can't know this week's patch.
- Local models follow search results less reliably than large cloud models. For a game-heavy channel, a small cloud model with its own search (Claude Haiku 5.5, GPT-6 Luna, Gemini Flash) is a good, cheap choice.
- Web search costs a little extra on paid APIs. Switch *Look up current facts when chat asks something* off if your chat rarely asks about current things.
