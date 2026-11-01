---
title: Übersetzung
description: Nachrichten in Sprachen, die du nicht sprichst, werden direkt darunter übersetzt – und bei Regelverstößen bekommen du und deine Mods einen Vermerk mit der echten Bedeutung.
icon: Languages
---

Internationale Chats sind toll – bis jemand in einer Sprache schreibt, die du nicht lesen kannst. HighBazar übersetzt solche Nachrichten für dich, direkt im Live-Feed, und kann die Übersetzung im Chat posten, damit alle mitkommen. Verstößt so eine Nachricht gegen deine Regeln, handelt die Moderation wie gewohnt und ergänzt einen **Vermerk für deine Mods** mit der Übersetzung und dem, was wirklich gemeint ist: Slang, versteckte Beleidigungen, Doppeldeutigkeiten.

![Sprachen & Übersetzung auf der Seite Kanäle](translation.webp)

## Einrichten

Scroll unter [Kanäle](channels.md) zu **Sprachen & Übersetzung**:

1. Trag die **Sprachen, die du verstehst** ein. Die erste ist die **Zielsprache**: In sie wird übersetzt. Klick auf eine andere Sprache, um sie zur Zielsprache zu machen.
2. Schalte **Fremdsprachige Nachrichten übersetzen** ein.

Solange du keine Sprachen eingetragen hast, wird nichts übersetzt – das Dashboard erinnert dich daran.

## Einstellungen

| Einstellung | Erklärung |
|---|---|
| **Fremdsprachige Nachrichten übersetzen** | Nutzt das Profil *KI für Moderation* des Kanals. Läuft die KI-Moderation ohnehin, kostet das Übersetzen kaum etwas extra. |
| **Übersetzung auch im Chat posten** | Postet jede Übersetzung als eigene Nachricht, z. B. `[HighBazar][Translator] Anna (Spanisch): Hallo zusammen!` – höchstens 8 pro Minute. Nachrichten, die die Moderation markiert, werden **nie** übersetzt gepostet. |
| **Präfix im Chat** | Steht vor jeder übersetzten Nachricht (Standard `[HighBazar][Translator]`). |
| **Vermerk auch beim Löschen an die Mods** | Twitch hat für einfache Löschungen kein Begründungsfeld. Ist das an, wird stattdessen ein Timeout von 1 Sekunde genutzt: Es entfernt die Nachricht genauso und zeigt deinen Mods die Übersetzung als Begründung. |

## So funktioniert es

- **Die Spracherkennung läuft lokal** auf deinem PC, deshalb kosten nur Nachrichten in fremden Sprachen KI-Tokens. Emotes, kurze Reaktionen („gg“, „lol“) und deine eigenen Nachrichten werden nie übersetzt.
- Nachrichten, die die KI-Moderation sowieso prüft, bekommen ihre Übersetzung aus derselben Anfrage.
- Übersetzungen stehen im Live-Feed direkt unter dem Original (und auf Wunsch im [Overlay](overlay.md)).
- Bei einer markierten Nachricht speichert das Moderations-Protokoll Original, Übersetzung und Bedeutung – durchsuchbar wie alles andere.

> [!TIP]
> Lass *Chattern in ihrer Sprache antworten* (Begleiter → Sprache) aus: Der Bot antwortet dann in der Sprache deines Streams, und du kannst jede Antwort lesen. Der Übersetzer deckt die andere Richtung ab.
