---
title: Moderation
description: Fair, configurable chat protection – instant local rules plus an AI that understands context, with actions per category, escalation, exemptions and a log with undo.
icon: Shield
---

HighBazar protects your chat in two layers:

1. **Instant rules** run on your PC, cost nothing and react in milliseconds: blocked terms, links, caps, repeated messages, emote spam, very long messages.
2. **AI checks** understand context. They tell banter, dark humor and criticism from real hate, harassment or scams – in any language – and judge by your own channel rules.

What happens with a finding is up to you: only log it, suggest an action, or act automatically. A **moderation profile** holds all of this; choose it per channel on [Channels](channels.md#profiles).

![The Moderation page](moderation.webp)

## Built-in profiles

| Profile | What it does |
|---|---|
| **Balanced** *(default)* | Acts only on clear hate, harassment, threats, scams and spam. Banter, swearing and criticism stay allowed. Borderline cases are only logged for you. Mods and VIPs are exempt. |
| **Relaxed** | Maximum freedom. Only the most severe cases (hate, threats, doxxing, scams) are acted on; everything else is logged. |
| **Family Friendly** | Strict. Also removes borderline content, sexual references, excessive caps and links from non-exempt users. |
| **Suggest Only** | Never acts on its own. Flags problematic messages in the dashboard with a suggested action you apply with one click. |
| **Watch & Log** | Only observes and writes a moderation log. No actions, no suggestions. |

## General

| Setting | Explanation |
|---|---|
| **Mode** | *Off*: no moderation. *Watch & log only*: problems are noted in the log. *Suggest actions*: problems show up in the dashboard with a proposed action you apply with one click. *Act automatically*: HighBazar deletes, warns, times out or bans by itself, as set in *Categories*. |
| **AI check** | *Off (rules only)*: only the instant rules – free. *Suspicious messages only*: the AI checks messages with warning signs – cheap, and catches most cases. *All messages (batched)*: every message is checked, several at a time – the most thorough, costs the most. |
| **Strictness** | How sensitive the AI judges: 0 = only extreme content, 50 = balanced, 100 = family friendly. |
| **Min. confidence for automatic actions** | The AI says how sure it is. Automatic actions only happen above this value (default 0.8); less certain cases are just logged. Higher = fewer mistakes, but more slips through. |
| **Max. AI checks per minute** | Upper limit (default 12) so a spam wave can't blow up your AI costs. The instant rules keep working above it. |
| **Your channel rules** | Your chat rules in plain words; the AI judges by them. “No backseating – don't tell me how to play unless I ask. No spoilers for the story. English only.” |

> [!TIP]
> Messages in languages the suspicious-word list doesn't know (it covers English and German) always get the AI check in *Suspicious messages only* mode – so foreign-language insults don't slip through.

## Categories {#categories}

The AI sorts problems into nine categories. For each, you choose an action for a **clear violation** (the AI is sure) and for a **borderline** case (it could be a joke or a misunderstanding).

| Category | Examples |
|---|---|
| **Hate** | Slurs, attacks on origin, religion, gender, sexuality, disability. |
| **Harassment** | Insults and attacks on a person, the streamer included. |
| **Threats** | Threats of violence, calls to harm someone. |
| **Sexual content** | Explicit or harassing sexual messages. |
| **Self-harm** | Encouraging self-harm or suicide. |
| **Spam** | Repeated advertising, viewer-bot offers, floods. |
| **Scams & phishing** | Fake giveaways, “free Nitro”, phishing links. |
| **Doxxing** | Sharing someone's private information. |
| **Extremism** | Extremist propaganda or symbols. |

| Action | Effect on Twitch |
|---|---|
| **Nothing** | Ignored. |
| **Log** | Only noted in the moderation log. |
| **Delete** | Removes the message. |
| **Warn** | Shows the person a warning they must accept before chatting again. |
| **Timeout** | Mutes them for a while (any duration) – they can still watch. |
| **Ban** | Removes them from your chat until you unban them. |

## Repeat offenders {#repeat-offenders}

With **Escalate for repeat offenders**, repeated violations within a time window get stricter actions: the first time maybe a deletion, the second a 10-minute timeout, the third a ban. Add as many steps as you like; only violations inside the window count, and a step never makes an action milder.

## Instant rules {#instant-rules}

These rules need no AI and work even when the [daily budget](costs.md#daily-budget) is used up.

![Instant rules](moderation-rules.webp)

| Rule | Explanation |
|---|---|
| **Blocked terms** | Words or phrases that are always acted on. `*` matches anything (`idiot*` also catches “idiots”), and spellings like `1d10t` (leetspeak) are recognized. |
| **Known spam phrases** | A built-in list of typical viewer-bot and gift-card spam (“cheap viewers”, “free followers”, “free nitro” …). It acts without context, so viewers joking about bots are hit too – **off by default**. With AI moderation these messages are checked in context anyway. |
| **Links** | *Allow* or *Block (except allowed domains)*. Links can be scams or harmful sites; allow the domains you trust, e.g. `youtube.com` or your own website. |
| **Excessive caps** | MESSAGES MOSTLY IN CAPITALS feel like shouting – acts above the set share of capital letters, from a minimum length. |
| **Repeated messages** | The same message several times within 60 seconds. |
| **Emote spam** | More emotes in one message than allowed. |
| **Max. message length** | Very long messages (“walls of text”) push everything else out of chat. `0` switches the rule off. |

Each rule has its own action.

## Exemptions

**Moderators**, **VIPs** and **subscribers** can be exempted, plus **single accounts** – other bots you use (Nightbot, StreamElements) or a friend with a rough tone. **The streamer is always exempt.**

## Messages

| Setting | Explanation |
|---|---|
| **Reason sent to Twitch** | The text Twitch shows to the person and your mods with a timeout or ban. `{reason}` is replaced by what the AI or rule found. Default: `[HighBazar] {reason}`. |
| **Announce automatic actions in chat** | Posts a short note when HighBazar acts on its own (“Removed a message with a link.”) – shows chat that rules are enforced. Rule findings get a short generic reason there, so a blocked term or a spam link is never repeated in chat. Off by default. |

## The moderation log {#log}

The **Log** tab lists every finding: who, what, the category, the verdict and confidence, the source (*AI*, *Rule* or *Manual*) and the status – *logged*, *suggested*, *done*, *simulated* (test mode), *failed*, *dismissed* or *undone*.

![The moderation log](moderation-log.webp)

- **Search** by name, message, translation or reason, and filter for *Actions* or *Suggestions*.
- **Apply** a suggested action with one click, or **dismiss** it.
- **Undo** a timeout or ban – HighBazar lifts it on Twitch.

The bot must be running to apply or undo actions.

## Shield mode and Auto Shield

During a hate raid – many accounts spamming hate at once – Twitch's **shield mode** switches on your strictest chat settings in one go. HighBazar can turn it on by itself: **Auto Shield** activates when several different accounts post hate, harassment, threats or sexual harassment within a short time (as found by moderation), and switches it off again after a calm period. You find it in the [stream manager](stream-manager.md#safety) (*Expert*), next to Twitch's own AutoMod.

## Foreign-language messages

If you set the languages you understand, flagged messages in other languages get a **note for your mods** with the translation and what is really meant – slang, coded insults. See [Translation](translation.md).
