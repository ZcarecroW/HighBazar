---
title: Chat overlay
description: Show your chat – with emotes, smooth gliding and translations – in the Steam overlay over your game, or as a browser source in OBS.
icon: Gamepad2
---

The chat overlay shows your live chat as a small web page. Pin it in the **Steam overlay** and it stays visible over your game – like a second monitor for your chat, without <kbd>Alt</kbd>+<kbd>Tab</kbd>. The same link works as a **browser source in OBS** or in any browser.

The page runs only on your PC, also works in [performance mode](settings.md#performance), and uses the emotes, smooth gliding and chat size from [Settings → Chat & size](settings.md#chat-and-size).

![Overlay settings](overlay.webp)

## Turn it on

1. Open [Settings](settings.md) and scroll to **Steam overlay**.
2. Switch on **Enable chat overlay**. HighBazar starts a small web server that only your own PC can reach (`127.0.0.1`).
3. Copy **Your overlay link** – or click **Test in browser** to see it.

> [!WARNING]
> The link contains a secret key. **Don't show it on stream.** If it leaks, click *Create a new link* – the old one stops working.

## Use it in Steam

1. Copy the link.
2. In your game, press <kbd>Shift</kbd>+<kbd>Tab</kbd> and open the **web browser** at the bottom of the Steam overlay.
3. Paste the link into the address bar and press <kbd>Enter</kbd>.
4. Move the window where you want it and resize it.
5. Click the **pin** at the top of the window and set its opacity. The window now stays visible over the game even when the overlay is closed – you just keep playing.

## Use it in OBS

Add a **Browser** source, paste the link as the URL and set the size you like. The background darkness is set in HighBazar (see below), so you don't need custom CSS.

## Settings

| Setting | Default | Explanation |
|---|---|---|
| **Show bot messages** | on | Include the bot's own messages. |
| **Show follows, subs, raids** | on | Include events. |
| **Show translations** | on | Show [translations](translation.md) below foreign-language messages. |
| **Chat input in the overlay** | off | An input field at the bottom of the overlay – see below. |
| **Background** | 55 % | How dark the background behind the text is. In Steam you can also set the opacity of the whole window. |
| **Hide messages after** | never | Older messages fade out so less of the game is covered. |
| **Port** | 47823 | Only change it if another program uses this port – HighBazar tells you when it's taken. |

The settings page shows how many overlay windows are connected.

## Chat input in the overlay

With **Chat input in the overlay** on, the overlay gets the same input field as the app – with the same [emote and emoji suggestions](chat-input.md): <kbd>Shift</kbd>+<kbd>Tab</kbd> into the Steam overlay, type your reply, <kbd>Enter</kbd> – and back to the game. Sending only works through your secret link and only from your PC.
