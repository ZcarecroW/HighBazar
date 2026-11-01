---
title: Dein erster Stream
description: Alles gefahrlos im Testmodus mit dem Chat-Simulator ausprobieren und dann mit einer kurzen Checkliste live gehen.
icon: Play
---

Bevor HighBazar in deinem echten Chat redet, probier es im **Testmodus** aus: Der Bot läuft genau wie live, aber nichts wird an Twitch gesendet – Antworten, Begrüßungen und Moderations-Aktionen erscheinen nur in deinem Live-Feed.

## Testmodus starten

Klicke im [Dashboard](dashboard.md) auf **Testmodus**. Ein Banner zeigt, dass Nachrichten und Moderations-Aktionen nur simuliert werden.

> [!NOTE]
> Der Testmodus funktioniert auch ohne verbundenen Twitch-Account. Er verbindet sich gar nicht mit deinem echten Chat: Der Chat-Simulator unten ist dein Publikum. Titel, Kategorie und Emotes werden trotzdem geladen, wenn ein Account verbunden ist – so passen die Nachrichten zu deinem Stream.

## Mit dem Chat-Simulator Zuschauer spielen {#chat-simulator}

Mit dem **Chat-Simulator** rechts im Dashboard schreibst du als Fake-Zuschauer. Wähle einen Namen und eine Rolle (Zuschauer, Abonnent, VIP, Moderator), tippe eine Nachricht und drück Enter. Probier das aus:

| Probier | Was passieren sollte |
|---|---|
| `hey HighBazar, was ist das für ein Spiel?` | Der Bot antwortet dir mit Namen (er reagiert auf seinen Namen und die Trigger-Wörter). |
| `@HighBazar was war im letzten Patch?` | Der Bot schlägt die Antwort im Web nach und antwortet mit den Fakten und ihrem Datum. |
| `!hb frage wer ist der Endboss?` | Ein Chat-Befehl – wie eine Frage, mit eigenem Cooldown. |
| `!hb fakt` / `!hb witz` | Ein Fun Fact zum Spiel oder ein Witz. |
| `!hb hilfe` | Eine kurze Vorstellung mit den verfügbaren Befehlen (ohne KI). |
| Eine Nachricht unter neuem Namen | Der Bot begrüßt den Erst-Chatter nach ein paar Sekunden. |
| `free followers at spam.example` | Die Moderation greift ein (je nach Profil). |

Unter dem Eingabefeld schickt **Event simulieren** einen Follow, Sub, Resub, Sub-Geschenke oder einen Raid – der Bot bedankt sich wie live.

Mit **Jetzt etwas sagen** oben holst du dir sofort eine Intervall-Nachricht, statt auf den Countdown zu warten.

## Prüfen, wie er klingt

Passt eine Nachricht nicht zu deinem Stream:

- Ändere die **Persönlichkeit**, die Regler für Humor, Energie und Frechheit oder ergänze **Zusätzliche Anweisungen** auf der Seite [Begleiter](companion.md).
- Erzähl ihm von deinem Stream (**Über dich und deinen Stream** unter [Kanäle](channels.md#behavior)) – Community-Name, Running Gags, was deine Emotes bedeuten.
- Stell **Senden** auf *Vorschau* oder *Nur nach meiner Freigabe*, um jede Nachricht vor dem Versand zu prüfen ([Kanäle](channels.md#sending)).

## Live gehen – eine Checkliste

1. **Streamer-Account verbunden** (Kanäle) – das Dashboard zeigt *Verbunden*.
2. **KI getestet** – *KI → Testen* antwortet.
3. **Budget gesetzt** – das tägliche KI-Budget (Standard 2 $) pausiert die KI-Funktionen, wenn es erreicht ist.
4. **Moderations-Profil gewählt** – *Ausgewogen* passt für die meisten Streams; *Nur Vorschläge*, wenn du jede Aktion selbst freigeben willst.
5. **Kategorie auf Twitch gesetzt** – der Bot recherchiert das Spiel, wenn sie wechselt.
6. Klicke auf **Bot starten**.

Antworten, Moderation und Overlay funktionieren sofort. **Intervall-Nachrichten** warten, bis du live bist (wenn *Nur während des Streams* an ist), und pausieren, wenn im Chat sehr viel los ist oder niemand auf die letzten geantwortet hat.

> [!TIP]
> Soll HighBazar mit deinem PC starten und im Hintergrund laufen? Schalte *Bot beim Öffnen der App automatisch starten* und *Direkt im Infobereich starten* unter [Einstellungen](settings.md#performance) ein.

## Während des Streams

- Der **Live-Feed** zeigt alles: Chat, Bot-Nachrichten mit Grund (Intervall, Antwort, Begrüßung, Event, Befehl), Follows, Subs, Raids und Moderation. Zeig auf eine Nachricht, um sie anzupinnen, zu löschen, der Person ein Timeout zu geben oder sie zu bannen.
- **Wartet auf Versand** listet Bot-Nachrichten im Vorschau- oder Freigabe-Modus – bearbeiten, senden oder verwerfen.
- Die Schalter **Begleiter** und **Moderation** unter *Schnellsteuerung* schalten jeden Teil sofort ab und wieder an.

Beendest du den Stream, hören die Intervall-Nachrichten von selbst auf. **Stoppen** trennt den Bot ganz.
