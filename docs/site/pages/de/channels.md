---
title: Kanäle
description: Kanal-Profile bündeln alles für einen Twitch-Kanal – Accounts, Bot-Name und Präfix, Profile, Sendemodus, Budget und was die KI über dich weiß.
icon: Radio
---

Ein **Kanal-Profil** enthält alles für einen Twitch-Kanal oder ein Setup. Die meisten Streamer brauchen nur eins; leg weitere an, wenn du für einen Freund moderierst, auf zwei Kanälen streamst oder ein eigenes Setup für besondere Streams willst. Gewechselt wird oben im Dashboard.

![Die Seite Kanäle](channels.webp)

## Accounts {#accounts}

| Einstellung | Erklärung |
|---|---|
| **Streamer-Account** | Dein eigener Twitch-Account – der, mit dem du streamst. Du meldest dich auf twitch.tv mit einem kurzen Code an, HighBazar sieht dein Passwort also nie. Er wird gebraucht, um den Chat zu lesen, Follows, Subs und Raids zu sehen und zu moderieren. |
| **Eigener Bot-Account** *(optional)* | Ein zweiter Account nur für den Bot, z. B. *DeinNameBot*. Der Chat sieht den Bot dann unter seinem eigenen Namen. Mach ihn für die Moderations-Funktionen zum Moderator in deinem Chat (`/mod DeinNameBot`) und verbinde ihn dann hier. Die Moderation selbst läuft immer über deinen Streamer-Account. |

Zeigt ein Account **Neu verbinden nötig**, klicke auf **Neu verbinden** und bestätige die Berechtigungen auf Twitch erneut – zum Beispiel nach einer Passwortänderung oder wenn eine Funktion eine Berechtigung braucht, die du noch nicht erteilt hast.

## Bot-Name und Präfix {#bot-name-and-prefix}

| Einstellung | Standard | Erklärung |
|---|---|---|
| **Bot-Name** | `HighBazar` | Wie sich der Bot nennt. Eine Nachricht, die diesen Namen enthält („hey Bazar, was ist das für ein Spiel?“), bekommt eine Antwort. Wähl etwas Kurzes, das niemand aus Versehen tippt. Weitere Spitznamen kommen unter *Weitere Trigger-Wörter* auf die Seite [Begleiter](companion.md#replies-when-addressed). |
| **Nachrichten-Präfix** | `[HighBazar]` | Steht vor jeder Bot-Nachricht, wenn er mit deinem Account schreibt, damit die Zuschauer wissen, dass sie vom Bot kommt – Twitchs Regeln verlangen, dass automatische Nachrichten erkennbar sind. |

## Funktionen

- **Begleiter (Chatten)** – Intervall-Nachrichten, Antworten, Begrüßungen und Dankeschöns. Schalte ihn ab, wenn du nur Moderation willst.
- **Moderation** – schützt deinen Chat nach dem Moderations-Profil.

Beides lässt sich auch live in der *Schnellsteuerung* des Dashboards umschalten.

## Profile {#profiles}

| Einstellung | Erklärung |
|---|---|
| **Begleiter-Profil** | Wie der Bot chattet – siehe [Begleiter](companion.md). |
| **Moderations-Profil** | Wie streng moderiert wird – siehe [Moderation](moderation.md). |
| **KI-Profil** | Das Modell, das die Nachrichten des Bots schreibt – siehe [KI-Anbieter](ai-providers.md). Ohne KI funktionieren nur die lokalen Moderationsregeln. |
| **KI für Moderation** | Die Moderation prüft viele Nachrichten, deshalb spart ein schnelles, günstiges Modell (z. B. Claude Haiku 5.5 oder GPT-6 Luna) viel Geld, ohne schlechter zu sein. *Wie oben* nutzt die Haupt-KI. |

## Verhalten {#behavior}

### Senden {#sending}

| Modus | Was passiert |
|---|---|
| **Automatisch** | Der Bot postet sofort. |
| **Vorschau, dann automatisch senden** | Jede Nachricht wartet im Dashboard für die *Vorschauzeit* (Standard 10 Sekunden); du kannst sie bearbeiten oder verwerfen, bevor sie rausgeht. |
| **Nur nach meiner Freigabe** | Nichts wird gepostet, bis du auf **Senden** klickst – gut, um den Bot kennenzulernen. |

### Tägliches KI-Budget {#daily-budget}

KI-Anbieter rechnen pro Nutzung ab. Erreichen die Kosten von heute diesen Betrag (Standard **2 $**), pausieren alle KI-Funktionen bis morgen; die lokalen Moderationsregeln laufen weiter. `0` heißt unbegrenzt. Siehe [Kosten](costs.md).

### Über dich und deinen Stream

Fakten, die die KI nutzen darf: dein Name oder Spitzname, was du meistens spielst, dein Zeitplan, der Name deiner Community, Running Gags, was deine Emotes bedeuten. Je mehr sie weiß, desto mehr klingt sie wie einer deiner Stammgäste. Bitte nichts Privates – dieser Text geht bei jeder Anfrage an den KI-Anbieter.

## Sprachen & Übersetzung

Der untere Teil der Seite legt fest, welche Sprachen du verstehst und wie fremdsprachige Nachrichten übersetzt werden – siehe [Übersetzung](translation.md).
