---
title: Stream manager
description: Change your stream right from HighBazar – from title and tags up to chat modes, raids, polls, channel point rewards, AutoMod, shield mode and ads.
icon: MonitorPlay
---

The **Stream** page changes your stream and channel directly on Twitch. It comes in three levels so it never shows more than you need:

| Level | What you get |
|---|---|
| **Simple** *(default)* | Title, category, language and tags. |
| **Advanced** | Plus content classification labels and branded content, chat settings, announcements, shoutouts, raids, clips, stream markers and pinned messages. |
| **Expert** | Everything Twitch allows: polls and predictions, channel point rewards with AI actions and a redemption queue, mods, VIPs, bans and unban requests, suspicious users, AutoMod, shield mode, ads and ad automation, schedule, stream delay, channel description and chat color. |

Twitch apps only get the permissions you approve. HighBazar asks only for what the chosen level needs; switching to a higher level asks you once for the extra permissions (**Grant permissions**). You can go back to a lower level at any time.

![The stream manager on the Simple level](stream.webp)

## Stream info

| Field | Explanation |
|---|---|
| **Title** | The title above your stream and in the Twitch directory. Short and specific gets more clicks: say what you're doing today. |
| **Category** | The game or topic – search with box art. Viewers browse Twitch by category, and the bot [researches](web-search.md#game-research) the game you pick. |
| **Language** | The stream's language; the bot uses it unless you set another one. |
| **Tags** | Up to 10 keywords viewers can filter by, e.g. *English*, *Chill* or *NoBackseating* – letters and numbers only, 25 characters each. |
| **Content classification labels** *(Advanced)* | Warn viewers about mature content such as violence, drugs, gambling or sexual themes. Twitch expects you to set them when your stream contains such content. “Mature-rated game” is set by Twitch automatically. |
| **Branded content** *(Advanced)* | Tick this when the stream contains paid promotion or sponsored products – Twitch then shows viewers a notice. |
| **Stream delay** *(Expert, partners)* | Delays your stream for viewers, e.g. against stream sniping in competitive games. Chat lags behind you too. 0–900 seconds. |

Changes are saved on Twitch with **Save**.

## Chat settings *(Advanced)*

![Chat settings and actions on the Advanced level](stream-advanced.webp)

| Mode | Explanation |
|---|---|
| **Emote-only** | Only messages made of emotes – fun for hype moments, or to calm a spam wave. |
| **Subscribers only** | Only subscribers can write. Strong protection against troll raids, but new viewers can't chat. |
| **Unique chat** | Also called R9K mode: a message that is the same as a recent one is blocked. Stops copy-paste spam. |
| **Followers only** | Only people who have followed you for at least the set time can write. Very effective against bot and hate raids; 10 minutes is common, 0 means any follower. |
| **Slow mode** | Each person can send one message every few seconds. Calms a very fast chat. |
| **Chat delay for non-mods** | Messages of normal viewers appear a few seconds later, so your mods can remove bad ones before anyone sees them. |

## Actions *(Advanced)*

| Action | Explanation |
|---|---|
| **Announcement** | A highlighted message with a colored bar – for important things like a giveaway or a schedule change. |
| **Shoutout** | Twitch's official shoutout: a card in your chat that promotes another streamer with a Follow button. Common after a raid to thank the raider. Only while live, at most every 2 minutes. |
| **Raid** | Sends your viewers to another live channel at the end of your stream. Type a name, a game or a title – live channels you follow, people in chat and Twitch's search are suggested with avatar, category and viewers. After Twitch's countdown everyone moves over together; you can cancel before that. |
| **Clip** | Saves a short video of the last seconds of your stream (with title and length). |
| **Stream marker** | A bookmark in your recording (VOD) at the current moment, so you find it again for highlights. |
| **Pinned message** | A message pinned at the top of chat. Pin one with the pin icon in the live feed; unpin it here. |

## Polls and predictions *(Expert)* {#polls-and-predictions}

For affiliates and partners. A **poll** lets viewers vote between up to 5 options – optionally with extra votes for channel points. A **prediction** lets viewers bet channel points on an outcome (“Will I beat the boss on the first try?”); the winners share the losers' points. Lock it, pick the winning outcome, or cancel and refund.

## Channel point rewards *(Expert)* {#channel-point-rewards}

![Channel point rewards with AI actions](stream-expert.webp)

Channel points are free, virtual points viewers earn by watching and chatting, and spend on rewards you create. HighBazar lets you create and edit rewards (cost, cooldown, limits per stream and per viewer, whether the viewer must enter text) and lets the **bot fulfill them itself**:

| Action | What the bot does |
|---|---|
| **AI: answer the question** | Answers the viewer's question in chat – with a web search if needed. |
| **AI: friendly roast** | Teases the viewer in one good-natured line – never hurtful, never about looks or origin. |
| **AI: rhyme about the viewer** | A short, funny rhyme about the viewer and the stream. |
| **AI: fun fact about the game** | A surprising, true fact about the current game. |
| **AI: your own instruction** | Does what you describe (“Invent an epic title for the viewer”). |
| **Emote-only for 1 minute** | Chat goes emote-only for a minute and opens again by itself. |

If the bot can't do it (bot off, AI budget reached, error), the viewer **gets the points back automatically**. One click on a **template** (*Ask the bot*, *Roast me*, *Rhyme about me*, *Game fun fact*, *Hydrate!*, *Game wish*, *Emote-only*) fills the form; save your own templates too.

Rewards that wait for you appear in the **queue**: mark them done, or reject and refund the points. Twitch only lets apps edit rewards they created – other rewards are shown read-only.

## Community *(Expert)*

Mods, VIPs, banned users and **unban requests** (approve or deny, optionally with a reply), a **watch list** for suspicious accounts (*Monitor* highlights their messages for you and your mods, *Restrict* hides them from everyone else), your editors, follower and sub numbers (with sub points) and the current **Hype Train**.

## Safety *(Expert)* {#safety}

| Tool | Explanation |
|---|---|
| **Shield mode** | Your emergency button during a hate raid: switches on your strictest chat settings in one go. |
| **Auto Shield on attacks** | Turns shield mode on by itself when several different accounts post hate, harassment, threats or sexual harassment within a short time (as found by [moderation](moderation.md)), and off again after a calm period. You choose how many accounts, within how many seconds, and how long the calm must last. |
| **Twitch AutoMod** | Twitch's own filter: it holds risky messages back until a moderator allows or denies them – it works even when HighBazar isn't running. Pick a preset (*Hate only*, *Gaming, relaxed*, *Balanced*, *Strict*, *Family friendly*) or set the eight categories individually from 0 (off) to 4 (anything suspicious); save your own presets. |
| **Blocked terms (Twitch)** | Terms Twitch blocks itself – in addition to HighBazar's own rules. |

## Ads and schedule *(Expert)* {#ads-and-schedule}

Run an ad (with confirmation), snooze the next one, and see when the next ad and the next pre-roll-free period are due. **Pre-rolls** are ads new viewers see before your stream starts – many leave instead of waiting. After a mid-roll, Twitch skips pre-rolls for a while.

**Ad automation** – Twitch doesn't let apps change the Ads Manager settings, so HighBazar decides itself when ads run:

| Setting | Explanation |
|---|---|
| **No pre-rolls for newcomers** | When the pre-roll-free period runs out, HighBazar starts an ad in time – unless Twitch has one scheduled shortly anyway. |
| **Announce ads in chat** | A short message beforehand, so nobody thinks the stream crashed (your own text or a default one in the stream's language). |
| **Push back on raids** | When a raid comes in and an ad is due within 10 minutes, it's pushed back by 5 minutes – raiders shouldn't land in an ad. |
| **Bot pauses during ads** | No timed messages while viewers watch ads. |

The automation needs the bot running. **Schedule** edits your stream schedule on Twitch (weekly segments with start, duration and title).

## Profile *(Expert)*

Your channel description (the “About” text) and chat color – any color for Turbo and Prime users.
