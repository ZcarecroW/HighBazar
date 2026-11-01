---
title: Companion
description: How the bot chats – personality, timing, topics, language, emotes, reactions, chat commands and game research.
icon: Bot
---

The companion is the chatting part of HighBazar. A **companion profile** decides its personality, how often it writes on its own and about what, how it reacts to your community and which commands it answers. Choose the profile per channel on [Channels](channels.md#profiles).

![The Companion page with the personality section](companion.webp)

## Built-in profiles

| Profile | In short |
|---|---|
| **Balanced Companion** *(default)* | Friendly, witty all-rounder. Reacts to chat, shares game facts, asks questions. Works for most streams. |
| **Small Stream Booster** | Talks a bit more often and asks lots of questions to get lurkers chatting. Ideal for streams with few viewers. |
| **Cozy & Chill** | Calm, warm and laid-back. Fewer messages, gentle humor. Great for cozy games, art and just chatting. |
| **Hype Buddy** | High energy, loves big moments, celebrates every play and every new chatter. |
| **Game Expert** | Knows the game inside out: facts, lore, tips and trivia. Researches the current game online. |
| **Comedian** | Quick-witted and playful with light teasing. Jokes, puns and funny takes on what happens in chat. |
| **Quiet Helper** | No timed messages. Only answers when addressed, welcomes newcomers, reacts to events and commands. |

Built-in profiles are read-only and improve with updates. Click **Duplicate & edit** to make your own; the list on the left shows which channels use which profile.

## Personality

| Setting | Explanation |
|---|---|
| **Personality** | Who the bot is, written like a character description: “A friendly retro-gaming nerd who loves bad puns and cheers for every new viewer.” The AI keeps this voice in every message. |
| **Humor** | 0 = factual and calm, 100 = jokes and puns whenever it fits. |
| **Energy** | How excited it sounds: low = relaxed and quiet, high = lots of hype and exclamation marks. |
| **Teasing** | How much it teases you and chat in a friendly way. Keep it low for young or sensitive communities – it never insults anyone or punches down. |
| **Max. message length** | Upper limit for one message (default 220 characters). Twitch allows 500, but short messages fit chat better: 150–200 is a good start. |
| **Extra instructions** | Rules that always apply, in plain words: “Never spoil the story”, “Call the viewers ‘Crows’”, “Answer in a pirate voice on Fridays”. |
| **Topics to avoid** | Topics it never brings up and steers away from. Default: politics, religion, real-world tragedies, viewer counts, begging for follows or subs. |

> [!TIP]
> The bot also uses what you wrote under **About you and your stream** on [Channels](channels.md#behavior) – community name, schedule, running jokes. That's often the quickest way to make it sound like one of your regulars.

## Timing

Timed messages are what the bot writes on its own to keep chat alive: questions, facts, jokes, reactions.

![Timing: interval, adaptive pacing and context](companion-timing.webp)

| Setting | Default | Explanation |
|---|---|---|
| **Timed messages** | on | Switch off if the bot should only answer when spoken to. |
| **Interval** | 3–5 min | Before each timed message the bot waits a random time between these values. Small streams can go a bit shorter. |
| **Only while live** | on | No talking into an empty offline chat. Answers, commands and moderation also work offline. |
| **Adaptive** | on | Talks less when chat is lively and pauses when it's very busy. |
| **Busy above** | 25 messages/min | From this many viewer messages per minute, chat counts as busy and the bot holds back – people are talking anyway. |
| **Pause after unanswered messages** | 4 | If nobody reacts to this many bot messages in a row, chat is probably empty or busy watching; the bot pauses until someone writes again. `0` = never. |
| **Chat messages as context** | 40 | How many recent messages the AI reads before it writes. More context gives better-fitting messages but costs more tokens; 30–60 is a good range. |
| **Max. age of context** | 20 min | Older messages are left out, so the bot doesn't react to something from an hour ago. |

Timed messages also pause during **ad breaks** when the [ad automation](stream-manager.md#ads-and-schedule) is set to do so.

## Topics {#topics}

The **mix** decides how often each kind of timed message is chosen. The values are weights, not percentages: a topic at 8 comes about twice as often as one at 4, and 0 switches it off.

| Kind | What the bot does |
|---|---|
| **React to chat** | A witty comment or follow-up on what's going on in chat. If chat asks something current, it can look it up on the web. |
| **Game facts** | A surprising or little-known fact about the current game – from the [game research](web-search.md#game-research) when it's there. |
| **Game tips** | A short, useful tip, framed as a fun “did you know” – never backseating. |
| **Trivia** | General gaming or pop-culture trivia that fits the stream. |
| **Jokes** | A short, clever joke or pun about the stream, the game or the conversation. |
| **Questions to chat** | An easy question that invites lurkers to answer. |
| **Hype** | A short burst of positive energy for the moment. |
| **About the stream** | A comment on the stream itself – the game, the title, the vibe, how long you've been live. |

The last two kinds it used are picked less often, and the bot checks its recent messages so it doesn't say the same thing twice.

## Language

| Setting | Explanation |
|---|---|
| **Language** | *Stream language* – the language set in your Twitch stream info. *Fixed language* – always the language below. *Language most chatters use* – follows your chat, good for international communities. |
| **Language / fallback** | Used with *Fixed language*, and as a fallback when the stream language isn't known. |
| **Reply to chatters in their language** | Off is recommended: you couldn't read answers in languages you don't speak. The [translator](translation.md) translates foreign messages for you instead. |
| **Allow English gaming slang** | Lets the bot use common words like “GG”, “clutch” or “poggers” in other languages too – that's how Twitch chats talk. |

## Emotes

| Setting | Explanation |
|---|---|
| **Emote usage** | *Never*, *Rarely*, *Sometimes* or *Often*. Emotes are the small pictures of Twitch chat: words like “Kappa” or “catJAM” are shown as images. |
| **Emote sources** | Twitch, BetterTTV (BTTV), FrankerFaceZ (FFZ), 7TV. The last three are browser extensions – viewers only see their emotes as images if they use the extension. Switch off sources most of your chat doesn't use. |
| **Prefer emotes the chat uses** | The bot prefers emotes your chat is using right now – that feels the most natural. Otherwise it prefers your channel's own emotes over the hundreds of global ones. |
| **Max. emotes per message** | Upper limit, so it never spams emotes. |

## Reactions {#reactions}

![Reactions: replies, welcomes and events](companion-reactions.webp)

### Replies when addressed {#replies-when-addressed}

The bot answers when someone writes its **name** (see [Channels](channels.md#bot-name-and-prefix)) or one of the **extra trigger words** – nicknames like “Bazar”.

| Setting | Default | Explanation |
|---|---|---|
| **Treat replies to bot messages as mentions** | on | On Twitch you can reply to a specific message; a reply to a bot message counts as talking to the bot. |
| **Cooldown per user** | 30 s | How long one person waits before the bot answers them again. |
| **Global cooldown** | 4 s | Minimum pause between any two answers, no matter who asks. |
| **Who can use it** | Everyone | Or only subscribers, VIPs, moderators or just you. |

When a question needs current facts – a patch, a release date, a tournament result – the bot looks it up first (see [Web search](web-search.md#answers-to-chat)).

### Welcome new chatters

Greets people who write in your chat for the first time – a warm welcome makes newcomers much more likely to stay.

| Setting | Default | Explanation |
|---|---|---|
| **Who counts as new** | First message ever | *First message ever*: only real newcomers – the bot learns who your regulars are. *First message in this stream*: everyone's first message of each stream, regulars included – nice for very small streams. |
| **Delay** | 4 s | A few seconds of waiting feels more natural and lets you greet them yourself first. |
| **Max. welcomes per minute** | 3 | Upper limit, e.g. when a raid brings many newcomers at once. |
| **Welcome style** | – | How the welcome should sound: “mention that we're doing a no-hit run today”, “keep it to one short sentence”. |

During the first 20 minutes the bot learns who your regulars are and only welcomes Twitch's official *first-time chatters*.

### Event reactions

The bot answers these Twitch events with a short message (each can be switched off):

| Event | What it is |
|---|---|
| **Follows** *(batched)* | Someone clicked *Follow* to get notified when you go live. Follows are collected for 30 seconds and thanked together, so a follow wave doesn't flood chat. |
| **Subs** | A paid monthly subscription (Prime Gaming gives one free sub a month). |
| **Resubs** | Someone shares how long they've been subscribed. |
| **Gift subs** | Someone buys subscriptions for others. |
| **Raids** | Another streamer sends their viewers to you – the bot also mentions what the raider was playing. |
| **Cheers** | A tip with Bits, Twitch's own currency. Cheers below *Min. bits* (default 100 – about one US dollar for you) get no thank-you. |
| **Watch streaks** | Someone watched several of your streams in a row. |

## Commands {#commands}

Chat commands start with a **prefix** (default `!hb`). If you already use a bot like Nightbot with `!` commands, keep a prefix that doesn't clash with them.

| Command | German alias | What it does |
|---|---|---|
| `!hb ask <question>` | `!hb frage` | The bot answers the question – with a web search if needed. Anything after `!hb` that isn't another command counts as a question too. |
| `!hb fact` | `!hb fakt` | A fun fact about the current game. |
| `!hb joke` | `!hb witz` | A joke that fits the stream. |
| `!hb about` | `!hb hilfe`, `!hb info` | A short introduction and the list of available commands – no AI needed. |

Each command has its own switch, **cooldowns** (global and per user) and **who can use it**. Channel point rewards can trigger the same AI actions – see [Stream manager](stream-manager.md#channel-point-rewards).

## Research & web search {#research}

![Research & web search settings](companion-research.webp)

| Setting | Default | Explanation |
|---|---|---|
| **Research the current game** | on | When the category changes, the bot reads up once and keeps 15 facts and 8 tips for its messages, cached per game. |
| **Search the web while researching** | on | Looks up the game's current state – latest patch, season or DLC – with the AI provider's own search or [HighBazar's web search](web-search.md#highbazars-own-search). |
| **Look up current facts when chat asks something** | on | Patches, release dates, tournaments and the like are checked on the web before the bot answers; small talk is answered right away. |
| **Use Wikipedia** | on | The game's Wikipedia article as a reliable base – free, no key. |
| **Refresh after (days)** | 14 | After this many days a game is researched again – patches and seasons change things. |

How all this works in detail – and what it costs – is explained on [Web search & game research](web-search.md).
