---
title: AI providers
description: Pick the AI that writes the bot's messages – cloud providers with an API key, local models, or your Claude or ChatGPT subscription – and tune it with AI profiles.
icon: Cpu
---

HighBazar works with almost every AI provider. Each one is configured through an **AI profile**: provider, model, key, reasoning effort, limits, prices and web search. Pick the profile per channel on [Channels](channels.md#profiles) – and optionally a second, cheaper one just for moderation.

![The AI page with the profile editor](ai.webp)

## Providers

| Provider | Key | Web search | Notes |
|---|---|---|---|
| **Anthropic (Claude)** | API key | own | Messages API with prompt caching and web search (plus reading found pages during game research). |
| **OpenAI** | API key | own | Responses API (or Chat Completions). |
| **Google Gemini** | API key | own (Google Search) | Interactions API with automatic fallback to generateContent. Requests are not stored by Google. |
| **OpenRouter** | API key | own | One key for hundreds of models, exact cost reporting. |
| **xAI (Grok)** | API key | own | |
| **Perplexity** | API key | always | Answers with built-in live web search. |
| **Groq** | API key | own with *groq/compound* models | Other Groq models use HighBazar's search. |
| **Mistral, DeepSeek, Together AI** | API key | HighBazar's | Via their OpenAI-compatible endpoints. |
| **Ollama, LM Studio** | none | HighBazar's | Local models on your own PC – free, nothing leaves your PC except Twitch. Need a strong graphics card. |
| **Custom (OpenAI-compatible)** | optional | HighBazar's | Any compatible server: your own base URL and headers. |
| **Claude Code CLI** | your Claude subscription | own | Uses your installed, logged-in `claude` CLI – see [below](#claude-code-cli). |
| **Codex CLI** | your ChatGPT plan | own | Uses your installed Codex CLI – see [below](#codex-cli). |

“HighBazar's” search means HighBazar searches the web itself and passes the results to the model – see [Web search](web-search.md#highbazars-own-search).

## Built-in profiles

| Profile | Model | In short |
|---|---|---|
| **Claude Opus 5.5** | `claude-opus-5-5` | Anthropic's flagship. Best quality and humor; higher cost. |
| **Claude Sonnet 5.5** | `claude-sonnet-5-5` | Great quality at a moderate price. |
| **Claude Haiku 5.5** | `claude-haiku-5-5` | Fast and very cheap. Recommended for moderation and busy chats. |
| **OpenAI GPT-6.1 Sol** | `gpt-6.1-sol` | OpenAI's balanced model. |
| **OpenAI GPT-6.1 Terra** | `gpt-6.1-terra` | Efficient and strong at writing – half the price of Sol. |
| **OpenAI GPT-6 Luna** | `gpt-6-luna` | Fast, low-cost. |
| **Gemini 3.8 Flash** | `gemini-3.8-flash` | Google's recommended general-purpose model with Google Search grounding. |
| **Gemini 3.5 Flash-Lite** | `gemini-3.5-flash-lite` | Google's cheapest high-volume model. |
| **OpenRouter Auto** | `openrouter/auto` | Lets OpenRouter pick a model. |
| **Mistral Small** | `mistral-small-latest` | Mistral's efficient model (EU provider). |
| **DeepSeek Flash** | `deepseek-flash` | Low-cost, answers without a thinking pause. |
| **Perplexity Sonar** | `sonar` | Answers with built-in live web search. |
| **Ollama (local)** / **LM Studio (local)** | | Your own models. Pick an installed model in a copy of the profile. |
| **Claude Code CLI (subscription)** | `haiku` | Your Claude subscription through the CLI. |
| **Codex CLI (ChatGPT subscription)** | `gpt-6-luna` | Your ChatGPT plan through the CLI. |

Built-in profiles are read-only and improve with updates; **Duplicate & edit** to change one.

## API keys

An API key is like a password for an AI provider. Create it on the provider's website – the **Get a key** link takes you there. You usually add a payment method there and pay only for what you use.

Paste keys on the **API keys** tab (or right in the profile). By default, all profiles of a provider share one key; a profile can also use a **separate key** – for example a second Anthropic account just for moderation. Keys are stored in your system's credential store, never in the settings file ([Privacy & security](privacy.md)).

## Profile settings

| Setting | Explanation |
|---|---|
| **Provider** | The company or program whose AI writes the messages. |
| **Model** | Pick from the list loaded from the provider (just type to search – typos are forgiven) or type any model name. Bigger models write better but cost more and take longer; small ones are fast and cheap – ideal for moderation. |
| **Reasoning / effort** | How long the model thinks before it answers. More effort helps with hard questions but is slower and costs more. For chat and moderation, *Low* or the provider default is usually best. Options depend on the provider (e.g. *Off*, *Low* … *Maximum*). |
| **Max. output tokens** | Upper limit for one answer, including the model's thinking. Tokens are word pieces, about ¾ of a word each. Too low and an answer can end up empty – HighBazar tells you when that happens. |
| **Temperature** | Randomness. Current reasoning models ignore it or don't allow it; leave it at the default unless you know what you want. |
| **Allow web search** | Lets the AI look things up when a question needs current facts – see [Web search](web-search.md). |
| **Timeout** | How long HighBazar waits for an answer (default 90 s). Raise it for slow local models. Game research may take longer by itself. |
| **Prices** | Your provider's prices per million tokens (input and output), for the cost display and the daily budget. Built-in profiles have them filled in; OpenRouter reports exact costs itself. |
| **Base URL** | Only for your own or self-hosted servers; empty = the provider's normal address. |
| **Extra HTTP headers** | For special servers or proxies, e.g. an organization ID. Usually not needed. |
| **Use the Responses API** *(OpenAI)* / **Interactions API** *(Gemini)* | The newer interfaces (recommended). Only switch off if a proxy or an older model doesn't support them. |

### Test

**Test** sends a short message with this profile and shows the answer, the time, the tokens and the cost – the quickest way to see whether key and model work. **Test with web search** asks a question that needs a search (e.g. *What's in the latest Elden Ring Nightreign patch notes?*) and shows how many searches it used.

## Two AIs: one for chat, one for moderation

Moderation checks many messages, so on [Channels](channels.md#profiles) you can pick a separate **AI for moderation**. A fast, cheap model (Claude Haiku 5.5, GPT-6 Luna, Gemini Flash-Lite) is just as good at telling banter from hate – and costs a fraction. Translation uses the moderation AI too.

## Local models (Ollama, LM Studio)

Local models run on your own graphics card: free, private, and independent of any service.

1. Install [Ollama](https://ollama.com/download) or [LM Studio](https://lmstudio.ai) and download a model.
2. In HighBazar, duplicate the *Ollama (local)* or *LM Studio (local)* profile and pick your model from the list.
3. Raise the **timeout** if your model is slow.

Local models have no web search of their own – HighBazar searches for them ([Web search](web-search.md#highbazars-own-search)).

## Claude Code CLI {#claude-code-cli}

If you have a Claude subscription and the **Claude Code CLI** installed and logged in, HighBazar can use it instead of an API key (`claude -p`). HighBazar finds the CLI automatically – including the one bundled with the Claude desktop app – or you set its path and click **Detect**. Choose the model by its short name: *Haiku* (fast and economical – good for moderation), *Sonnet* (balanced), *Opus* (especially capable) or *Default* (as set in Claude Code).

HighBazar runs the CLI with **only web search** (and reading web pages for game research) – no files, no commands, none of your own Claude Code settings, plugins or memory. Usage counts against your subscription; the cost display shows $0.

> [!IMPORTANT]
> Anthropic's terms for subscriptions cover personal use of Claude Code. Whether an always-on chat bot is covered is your responsibility – the API key route is the officially supported way.

## Codex CLI {#codex-cli}

With a ChatGPT plan and the **Codex CLI** installed, HighBazar can use it too. Log in once in a terminal with `codex login`, then click **Detect**. HighBazar runs Codex **without commands, file access or other tools** – only answers and web search. Usage counts against your plan's Codex usage limits.

## Usage

The **Usage** tab shows per day and channel: AI calls, tokens, web searches, cost, messages sent and moderation actions. See [Costs](costs.md).
