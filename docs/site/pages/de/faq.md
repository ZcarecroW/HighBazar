---
title: FAQ
description: Kurze Antworten auf die Fragen, die Streamer am häufigsten zu HighBazar stellen.
icon: Info
---

## Gibt sich der Bot als ich aus? {#does-the-bot-pretend-to-be-me}

Nein. Nachrichten über deinen Account tragen ein Präfix (Standard `[HighBazar]`), und die KI darf nie in deinem Namen sprechen. Du kannst auch einen eigenen Bot-Account verbinden, damit der Chat den Bot unter seinem eigenen Namen sieht – siehe [Kanäle](channels.md#accounts).

## Spammt er meinen Chat zu? {#will-it-spam-my-chat}

Nein. Intervall-Nachrichten kommen mit Abstand (standardmäßig alle 3–5 Minuten), pausieren, wenn im Chat sehr viel los ist, hören nach ein paar unbeantworteten Nachrichten auf und laufen nur, während du live bist. Antworten und Begrüßungen haben Cooldowns und Obergrenzen. *Stiller Helfer* schreibt gar keine Intervall-Nachrichten. Siehe [Begleiter → Timing](companion.md#timing).

## Wie streng ist die Standard-Moderation?

Das Profil *Ausgewogen* handelt nur bei **eindeutigem** Hass, Belästigung, Drohungen, Betrug und Spam automatisch (Löschen oder Timeout, verschärft bei Wiederholungstätern). Fluchen, Frotzeleien, schwarzer Humor und Kritik sind erlaubt; Grenzfälle werden nur für dich protokolliert. Mods und VIPs sind ausgenommen. Probier *Nur Vorschläge*, wenn du jede Aktion selbst freigeben willst. Siehe [Moderation](moderation.md).

## Was kostet das?

HighBazar ist kostenlos. KI-Anbieter rechnen direkt mit dir ab: Mit einem kleinen Modell kostet eine Stream-Stunde ein paar Cent, lokale Modelle nichts, und ein Tagesbudget (Standard 2 $) setzt die Grenze. Siehe [Kosten](costs.md).

## Kann ich mein Claude-Pro- oder ChatGPT-Abo nutzen?

Drittanbieter-Apps dürfen sich nicht mit einem Claude.ai-Abo anmelden; der offizielle Weg ist ein **API-Schlüssel** aus der [Claude Console](https://platform.claude.com) – Bezahlung nach Nutzung, und Claude Haiku kostet Bruchteile eines Cents pro Nachricht. Als optionale Alternative kann HighBazar deine lokal installierte und angemeldete **Claude Code CLI** aufrufen, die über dein Abo läuft, oder die **Codex CLI** mit deinem ChatGPT-Abo. Ob ein ständig laufender Chat-Bot von den Bedingungen deines Abos gedeckt ist, liegt in deiner Verantwortung. Siehe [KI-Anbieter](ai-providers.md#claude-code-cli).

## Welche KI soll ich nehmen?

Für die meisten Streams: ein mittleres oder kleines Cloud-Modell zum Chatten (Claude Sonnet oder Haiku 5.5, GPT-6.1 Terra, Gemini 3.8 Flash) und ein kleines für die Moderation. Lokale Modelle sind kostenlos und privat, brauchen aber eine starke Grafikkarte und halten sich weniger zuverlässig an Suchergebnisse. Siehe [KI-Anbieter](ai-providers.md).

## Funktioniert es ohne KI?

Teilweise: Die **Sofort-Regeln der Moderation** (gesperrte Begriffe, Links, Großschreibung, Spam) funktionieren ganz ohne KI, ebenso die Stream-Verwaltung, das Overlay und das Schreiben im Chat. Chatten, KI-Moderation, Übersetzung und Spielrecherche brauchen ein KI-Profil.

## Funktioniert es in meiner Sprache?

Die Oberfläche gibt es auf Deutsch und Englisch. Der Bot schreibt in der Sprache deines Streams, in einer festen Sprache oder in der Sprache der meisten Chatter – in jeder Sprache, die das KI-Modell spricht. Nachrichten in Sprachen, die du nicht verstehst, lassen sich für dich [übersetzen](translation.md).

## Bremst es mein Spiel aus?

Nein – der Bot selbst braucht kaum CPU. HighBazar läuft mit niedrigerer Prozess-Priorität, die Oberfläche pausiert, wenn das Fenster minimiert ist, und im Performance-Modus läuft es mit rund 9 MB Speicher im Hintergrund. Siehe [Einstellungen → Leistung](settings.md#performance).

## Wo werden meine Daten gespeichert?

Alles bleibt auf deinem Computer: Einstellungen im Datenordner der App, API-Schlüssel und Twitch-Tokens im Anmeldespeicher deines Systems. Chatnachrichten gehen nur an den KI-Anbieter, den du gewählt hast. Siehe [Datenschutz & Sicherheit](privacy.md).

## Kann ich es für mehrere Kanäle nutzen?

Ja – leg für jeden ein [Kanal-Profil](channels.md) an. Der Bot läuft für das aktive.

## Kann ich meine eigene Twitch-Anwendung nutzen? {#can-i-use-my-own-twitch-application}

Ja: Registriere auf [dev.twitch.tv](https://dev.twitch.tv/console) eine **öffentliche** Anwendung (Kategorie *Chat Bot*, beliebige Redirect-URL wie `http://localhost:17563`) und trag ihre Client ID unter *Einstellungen → Erweitert* ein.

## Ist HighBazar Open Source?

Ja, unter der MIT-Lizenz – der Code liegt auf [GitHub](https://github.com/ZcarecroW/HighBazar). Die Icons sind von [Nucleo](https://nucleoapp.com) und nur für HighBazar an den Autor lizenziert; die MIT-Lizenz gilt für sie nicht.

## Gehört es zu Twitch?

Nein. HighBazar ist ein unabhängiges Projekt und nicht mit Twitch, Anthropic, OpenAI, Google oder anderen Anbietern verbunden.
