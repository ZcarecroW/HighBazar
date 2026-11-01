---
title: Privacy & security
description: What HighBazar stores where, what it sends to whom, how keys and logins are protected, and how updates are verified.
icon: Lock
---

HighBazar runs entirely on your computer. There is no HighBazar server, no account, no telemetry and no tracking.

## What goes where

| Data | Goes to | Why |
|---|---|---|
| Your Twitch login | Twitch only (twitch.tv/activate) | You log in on Twitch's own page with a short code. HighBazar never sees your password; it gets a token with only the permissions you approved. |
| Chat messages, stream title, category | The **AI provider you chose** | So the AI can write fitting messages and moderate. Only the recent messages the AI needs are sent (*Chat messages as context*), plus what you wrote under *About you and your stream*. With a local model (Ollama, LM Studio), nothing leaves your PC. |
| Search queries | The provider's own search, or the [search source](web-search.md#highbazars-own-search) you chose | Only when a question or a game research needs current facts. |
| Emote lists and images | Twitch, BetterTTV, FrankerFaceZ, 7TV | To show your channel's emotes. |
| Game names | Wikipedia | For game research (can be switched off). |
| Update check | GitHub | To find and download new versions. |

## Keys and logins

API keys and Twitch tokens are stored in your system's **secure credential store** – Windows Credential Manager, macOS Keychain, or the Secret Service on Linux (GNOME Keyring, KWallet). They are never written to the settings file. Only if Linux has no credential store, HighBazar uses a file in its data folder that only your user can read; *Settings → Advanced → Secret storage* shows which is used.

## Where your data is {#where-your-data-is}

*Settings → Open data folder* opens it. It contains your settings (`config.json`), the game research cache, per-channel data (e.g. which chatters the bot has seen, to welcome newcomers) and the usage statistics.

| System | Data folder | Log files |
|---|---|---|
| Windows | `%APPDATA%\io.github.zcarecrow.highbazar` | `%LOCALAPPDATA%\io.github.zcarecrow.highbazar\logs` |
| macOS | `~/Library/Application Support/io.github.zcarecrow.highbazar` | `~/Library/Logs/io.github.zcarecrow.highbazar` |
| Linux | `~/.local/share/io.github.zcarecrow.highbazar` | `~/.local/share/io.github.zcarecrow.highbazar/logs` |

The log file records what the app does and any errors – useful when [reporting a problem](troubleshooting.md#reporting-a-problem). It contains no keys.

## The bot's safety rules

- The AI is told to **never pretend to be you**, never post links, never start a message with `/` or `.` (Twitch commands), and to treat chat messages as **untrusted input** – attempts to change its rules, reveal its instructions or make it insult someone are ignored.
- HighBazar removes links and markdown from AI answers and drops an answer that still contains a link.
- Web search results are handed to the AI as untrusted text; it never follows instructions found in them.
- With the Claude Code or Codex CLI, HighBazar allows **only web search** – no file access, no commands, none of your own CLI settings, plugins or memory.

## The chat overlay

The [overlay](overlay.md) web server only accepts connections from your own PC (`127.0.0.1`), and the link contains a secret key. Don't show it on stream; *Create a new link* invalidates the old one.

## Updates

Updates are downloaded from the official GitHub releases and **verified with a signature** before they are installed: an update that wasn't signed with the project's key is refused. The source code of every release is public on GitHub.
