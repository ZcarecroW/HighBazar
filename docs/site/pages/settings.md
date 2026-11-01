---
title: Settings
description: Appearance and themes, performance on gaming PCs, chat display, start and updates, the overlay, advanced options and your data.
icon: Settings
---

![The Settings page](settings.webp)

## Appearance

| Setting | Explanation |
|---|---|
| **Theme** | *System* (follows your operating system), *Dark* or *Light*. |
| **Color theme** | Ten themes – Bazaar, Midnight, Ocean, Aurora, Forest, Gold, Sunset, Crimson, Sakura and Cyber – each in a light and a dark version. All are checked for readable contrast (WCAG AA), and the two-tone icons pick up the theme's colors. |
| **Language** | *Automatic* (your system's language), *English* or *Deutsch*. |
| **Show emotes as images** | Shows Twitch, BTTV, FFZ and 7TV emotes in the live feed as images (loaded from their servers). |

![Ten color themes, each in light and dark](themes.webp)

![The dashboard in the light theme](dashboard-light.webp)

## Performance {#performance}

HighBazar shouldn't slow down your game. The bot itself needs hardly any CPU – most of it goes to the interface, which is why the interface pauses as soon as the window is minimized.

| Setting | Explanation |
|---|---|
| **Games come first** | HighBazar runs with a lower process priority: when a game needs the CPU, it gets it first. You won't notice anything in chat. |
| **Performance mode** | Minimizing or closing the window shuts the interface down completely. HighBazar keeps running invisibly in the notification area – bot, moderation, translation and overlay – with only a few MB of memory (about 9 MB measured) and no measurable CPU load. Click the tray icon to open the window again; quit from its right-click menu. |
| **Start in the notification area** | Together with *Start the bot automatically when the app opens*, HighBazar runs like a background service. |
| **Move to the notification area now** | Does it right away. |

> [!TIP]
> On a weak PC, also switch off *Let new messages glide in* below – that roughly halves the graphics load in a busy chat.

## Chat & size {#chat-and-size}

| Setting | Explanation |
|---|---|
| **Interface size** | Scales the whole app except the chat. <kbd>Ctrl</kbd> + <kbd>+</kbd>/<kbd>−</kbd>, <kbd>Ctrl</kbd> + mouse wheel; <kbd>Ctrl</kbd> + <kbd>0</kbd> resets. |
| **Chat size** | Independent of the interface. In the live feed also <kbd>Ctrl</kbd> + mouse wheel over the chat, or A− / A+. |
| **Write in chat yourself** | The [input field](chat-input.md) under the live chat. |
| **Let new messages glide in** | The chat glides up instead of jumping, so you don't lose your line while reading. Never glides if animations are turned off in your system settings. |
| **Speed** | How long a new message takes to glide in; 350–600 ms suits most people. If messages arrive faster than they glide in, the chat keeps moving without a jerk. |
| **Acceleration and braking** | *Gentle* (soft start and stop – the most comfortable for reading), *Brisk* (starts right away), *Steady* (constant speed) or *Natural* (like a damped spring – especially calm when chat is busy). |

The **preview** next to these settings shows them live; *Simulate a busy chat* shows how it behaves when chat is fast. These settings also apply to the [overlay](overlay.md).

## Start & updates {#start-and-updates}

| Setting | Explanation |
|---|---|
| **Check for updates on start** | Looks for a new version on start and offers to install it. Updates come from the official GitHub releases and are signed ([Installation](installation.md#updates)). **Check now** runs it right away. |
| **Start the bot automatically when the app opens** | Starts the bot for the active channel as soon as HighBazar opens – handy if HighBazar starts with your PC. |
| **Setup wizard** | Opens the [setup wizard](setup-wizard.md) again. |

## Steam overlay

Turns on the chat overlay for the Steam overlay or OBS – see [Chat overlay](overlay.md).

## Advanced

| Setting | Explanation |
|---|---|
| **Twitch Client ID** | Every Twitch app has one, and HighBazar has one built in – you don't need this. Only enter your own if you registered an app at [dev.twitch.tv](https://dev.twitch.tv/console) (see [FAQ](faq.md#can-i-use-my-own-twitch-application)). |
| **Live feed size** | How many entries the live feed keeps (default 200). Fewer use less memory and CPU in a busy chat; older entries simply drop off. |
| **Secret storage** | Where API keys and Twitch logins are kept: your system's secure credential store (Windows Credential Manager, macOS Keychain, Secret Service on Linux) – never the plain settings file. If there is no credential store, a local file only your user can read. |

## Data

- **Open data folder** shows where HighBazar keeps its settings, the game research cache and your channels' data ([Privacy & security](privacy.md#where-your-data-is)).
- **Clear game research cache** forces a fresh [game research](web-search.md#game-research) next time.

## About

Your version, links to GitHub and this documentation, and **Report a problem** (GitHub issues).
