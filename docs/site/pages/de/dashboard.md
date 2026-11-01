---
title: Dashboard
description: Der Live-Feed, die Steuerung des Bots, die Zahlen von heute, Nachrichten, die auf Freigabe warten, und der Chat-Simulator.
icon: LayoutDashboard
---

Im Dashboard verbringst du deinen Stream: alles, was im Chat passiert, was der Bot tut, und die Steuerung dafür.

![Das Dashboard während eines Streams](dashboard.webp)

## Bot starten und stoppen

| Schaltfläche | Was sie tut |
|---|---|
| **Bot starten** | Verbindet sich mit deinem Twitch-Chat und startet den Bot mit dem aktiven Kanal-Profil. Antworten, Moderation und Overlay funktionieren sofort; Intervall-Nachrichten warten, bis du live bist (wenn *Nur während des Streams* an ist). |
| **Testmodus** | Lässt den Bot laufen, ohne etwas an Twitch zu schreiben – siehe [Dein erster Stream](first-stream.md). |
| **Stoppen** | Trennt den Bot. |
| **Jetzt etwas sagen** | Holt sofort eine Intervall-Nachricht, statt auf den Countdown zu warten. |
| **Stream-Infos aktualisieren** | Lädt Titel, Kategorie und Zuschauer neu und alle Emotes von Twitch, BetterTTV, FrankerFaceZ und 7TV. |
| **In den Hintergrund** | Schließt die Oberfläche; der Bot läuft im Infobereich weiter ([Performance-Modus](settings.md#performance)). |

Oben stehen der Kanal, ob du **LIVE** bist (mit Laufzeit), die Kategorie und die Zahl der Zuschauer.

## Der Live-Feed {#live-feed}

Alles, was passiert, der Reihe nach: Chatnachrichten, die Nachrichten des Bots, Follows, Subs, Raids, Systemhinweise und Moderations-Aktionen.

- **Bot-Nachrichten** zeigen ihren Grund: *Intervall*, *Antwort*, *Begrüßung*, *Event*, *Befehl* oder *Moderation*.
- **Erst-Chatter** sind als *neu* markiert; Nachrichten aus einem [Shared Chat](twitch-glossary.md#shared-chat) sind gekennzeichnet und werden nicht moderiert.
- **Emotes** von Twitch, BetterTTV, FrankerFaceZ und 7TV erscheinen als Bilder – Zero-Width-Emotes wie 7TVs RainTime liegen über dem Emote davor. Unter [Einstellungen](settings.md#chat-and-size) lassen sich die Bilder abschalten.
- **Übersetzungen** fremdsprachiger Nachrichten stehen direkt darunter, wenn die [Übersetzung](translation.md) an ist.
- **Moderations-Einträge** zeigen Kategorie, Aktion und Begründung, mit **Rückgängig**, wo Twitch das erlaubt.

Zeig auf eine Chatnachricht für schnelle Aktionen: im Chat **anpinnen**, **löschen**, der Person **10 Minuten Timeout** geben oder sie **bannen**.

Die Schaltflächen über dem Feed filtern, was angezeigt wird (*Chat*, *Bot*, *Moderation*, *System*), und schalten **Auto-Scroll**. **A− / A+** (oder <kbd>Strg</kbd> + Mausrad über dem Chat) ändern die Chatgröße unabhängig vom Rest der Oberfläche. Neue Nachrichten rollen sanft ein – Tempo und Kurve stellst du in den [Einstellungen](settings.md#chat-and-size) ein.

Unter dem Feed kannst du [selbst im Chat schreiben](chat-input.md), mit Vorschlägen für Emotes und Emojis.

## Die Zahlen von heute

| Karte | Bedeutung |
|---|---|
| **Nächste Nachricht** | Countdown bis zur nächsten Intervall-Nachricht. Pausiert, wenn du offline bist, im Chat sehr viel los ist, während einer Werbepause oder wenn niemand auf die letzten Nachrichten geantwortet hat – die Karte sagt, warum. |
| **KI-Kosten heute** | Was die KI-Aufrufe heute gekostet haben, geschätzt aus den Preisen in deinem KI-Profil. Der Balken zeigt, wie viel vom [Tagesbudget](costs.md#daily-budget) verbraucht ist. |
| **Nachrichten heute** | Wie viele Nachrichten der Bot gesendet hat. |
| **Chat / Min.** | Zuschauer-Nachrichten pro Minute. Über der Grenze *Sehr aktiv ab* (Begleiter → Timing) hält sich der Bot mit Intervall-Nachrichten zurück. |
| **Emotes** | Wie viele Emotes dein Kanal kennt – Twitch plus BetterTTV, FrankerFaceZ und 7TV. Änderungen an deinem 7TV-Set kommen live an. |
| **Spielwissen** | Fakten und Tipps, die der Bot zum aktuellen Spiel nachgeschlagen hat, oder *Recherchiert…*, während er sich einliest ([Websuche](web-search.md#game-research)). |

## Schnellsteuerung

- **Begleiter** und **Moderation** schalten jeden Teil sofort ab und an, ohne den Bot zu stoppen.
- **Senden** ändert, wie Bot-Nachrichten rausgehen: *Automatisch*, *Vorschau, dann automatisch senden* oder *Nur nach meiner Freigabe* (dieselbe Einstellung wie unter [Kanäle](channels.md#sending)).

## Wartet auf Versand {#pending}

Im Vorschau- oder Freigabe-Modus landen Bot-Nachrichten zuerst hier. Bei *Vorschau* gehen sie von selbst raus, wenn der Timer abläuft; bei *Freigabe* erst, wenn du auf **Senden** klickst. Jede Nachricht kannst du **bearbeiten**, **senden** oder **verwerfen**.

## Chat-Simulator

Schreib als Fake-Zuschauer und simuliere Events (Follow, Sub, Resub, Sub-Geschenke, Raid), um zu sehen, wie der Bot reagiert. Das geht im Testmodus und live – simulierte Nachrichten bleiben auf deinem PC und erreichen Twitch nie. Ideen findest du unter [Dein erster Stream](first-stream.md#chat-simulator).

## Schnellsuche

Drück überall <kbd>Strg</kbd> + <kbd>K</kbd> (oder klicke in der Seitenleiste auf **Suchen**), um zu jeder Seite, Einstellung, jedem Profil oder jeder Aktion zu springen – *Bot starten*, *Titel bearbeiten*, *Dunkles Design*. Tippfehler und Abkürzungen werden verziehen.

![Die Schnellsuche findet Seiten, Einstellungen und Aktionen](palette.webp)
