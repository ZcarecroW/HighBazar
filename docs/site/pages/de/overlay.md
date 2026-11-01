---
title: Chat-Overlay
description: Deinen Chat – mit Emotes, sanftem Einrollen und Übersetzungen – im Steam-Overlay über deinem Spiel zeigen oder als Browserquelle in OBS.
icon: Gamepad2
---

Das Chat-Overlay zeigt deinen Live-Chat als kleine Webseite. Pinn sie im **Steam-Overlay** an, und sie bleibt über deinem Spiel sichtbar – wie ein zweiter Monitor für deinen Chat, ohne <kbd>Alt</kbd>+<kbd>Tab</kbd>. Derselbe Link funktioniert als **Browserquelle in OBS** oder in jedem Browser.

Die Seite läuft nur auf deinem PC, funktioniert auch im [Performance-Modus](settings.md#performance) und übernimmt Emotes, sanftes Einrollen und Chatgröße aus [Einstellungen → Chat & Größe](settings.md#chat-and-size).

![Overlay-Einstellungen](overlay.webp)

## Einschalten

1. Öffne die [Einstellungen](settings.md) und scroll zu **Steam-Overlay**.
2. Schalte **Chat-Overlay aktivieren** ein. HighBazar startet einen kleinen Webserver, den nur dein eigener PC erreicht (`127.0.0.1`).
3. Kopiere **Dein Overlay-Link** – oder klicke auf **Im Browser testen**.

> [!WARNING]
> Der Link enthält einen geheimen Schlüssel. **Zeig ihn nicht im Stream.** Ist er doch rausgegangen, klick auf *Neuen Link erzeugen* – der alte funktioniert dann nicht mehr.

## In Steam nutzen

1. Kopiere den Link.
2. Drück im Spiel <kbd>Umschalt</kbd>+<kbd>Tab</kbd> und öffne unten im Steam-Overlay den **Webbrowser**.
3. Füg den Link in die Adresszeile ein und drück <kbd>Enter</kbd>.
4. Schieb das Fenster, wohin du willst, und pass die Größe an.
5. Klick oben im Fenster auf die **Pinnnadel** und stell die Deckkraft ein. Das Fenster bleibt jetzt über dem Spiel sichtbar, auch wenn das Overlay geschlossen ist – du spielst einfach weiter.

## In OBS nutzen

Füg eine **Browser**-Quelle hinzu, trag den Link als URL ein und stell die gewünschte Größe ein. Wie dunkel der Hintergrund ist, stellst du in HighBazar ein (siehe unten) – eigenes CSS brauchst du nicht.

## Einstellungen

| Einstellung | Standard | Erklärung |
|---|---|---|
| **Bot-Nachrichten anzeigen** | an | Die eigenen Nachrichten des Bots einblenden. |
| **Follows, Subs, Raids anzeigen** | an | Events einblenden. |
| **Übersetzungen anzeigen** | an | [Übersetzungen](translation.md) unter fremdsprachigen Nachrichten zeigen. |
| **Chat-Eingabe im Overlay** | aus | Ein Eingabefeld unten im Overlay – siehe unten. |
| **Hintergrund** | 55 % | Wie dunkel der Hintergrund hinter dem Text ist. In Steam kannst du zusätzlich die Deckkraft des ganzen Fensters einstellen. |
| **Nachrichten ausblenden nach** | nie | Ältere Nachrichten blenden aus, damit weniger vom Spiel verdeckt wird. |
| **Port** | 47823 | Nur ändern, wenn ein anderes Programm diesen Port nutzt – HighBazar sagt dir, wenn er belegt ist. |

Die Einstellungsseite zeigt, wie viele Overlay-Fenster verbunden sind.

## Chat-Eingabe im Overlay {#chat-input-in-the-overlay}

Ist **Chat-Eingabe im Overlay** an, bekommt das Overlay dasselbe Eingabefeld wie die App – mit denselben [Emote- und Emoji-Vorschlägen](chat-input.md): <kbd>Umschalt</kbd>+<kbd>Tab</kbd> ins Steam-Overlay, Antwort tippen, <kbd>Enter</kbd> – und zurück ins Spiel. Senden geht nur über deinen geheimen Link und nur von deinem PC aus.
