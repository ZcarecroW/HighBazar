---
title: Kosten & Budget
description: Was HighBazar im Betrieb kostet, wie du die KI-Kosten niedrig hältst, das Tagesbudget und die Verbrauchsstatistik.
icon: Coins
---

**HighBazar selbst ist kostenlos.** KI-Anbieter rechnen direkt mit dir ab, was der Bot verbraucht. Mit einem kleinen Modell sind das meist ein paar Cent pro Stream-Stunde; lokale Modelle kosten nichts.

## Grobe Kosten pro Stream-Stunde

Mit den Standard-Einstellungen – etwa 15 Intervall-Nachrichten pro Stunde, ein paar Antworten und Begrüßungen:

| Modellklasse | Begleiter / Stunde | KI-Moderation / Stunde (mäßiger Chat) |
|---|---|---|
| **Klein** – Claude Haiku 5.5, GPT-6 Luna, Gemini Flash-Lite | < 0,02 $ | < 0,02 $ |
| **Mittel** – Claude Sonnet 5.5, GPT-6.1 Sol, Gemini 3.8 Flash | ~ 0,10–0,25 $ | ~ 0,10–0,30 $ |
| **Flaggschiff** – Claude Opus 5.5 | ~ 0,20–0,50 $ | für die Moderation ein kleines Modell nehmen |
| **Lokal** – Ollama, LM Studio | 0 $ | 0 $ |
| **Abo** – Claude Code CLI, Codex CLI | zählt gegen dein Abo | zählt gegen dein Abo |

*Schätzungen – die tatsächlichen Kosten hängen von Chat-Aktivität, Kontextgröße und den Preisen des Anbieters ab.* Websuchen kosten bei bezahlten APIs grob 0,5–1,5 Cent extra.

## Tagesbudget {#daily-budget}

Jeder Kanal hat ein **tägliches KI-Budget** (Standard **2 $**, eingestellt unter [Kanäle](channels.md#daily-budget)). Erreichen die Kosten von heute den Betrag, pausieren alle KI-Funktionen – Begleiter, KI-Moderation, Übersetzung – bis morgen, und der Feed sagt Bescheid. Die Sofort-Regeln der Moderation laufen weiter. `0` heißt unbegrenzt.

Die Karte **KI-Kosten heute** im Dashboard zeigt, wie viel vom Budget verbraucht ist.

## Kosten niedrig halten

- **Ein günstiges Modell für die Moderation.** Stell unter [Kanäle](channels.md#profiles) die *KI für Moderation* auf Claude Haiku 5.5, GPT-6 Luna oder Gemini Flash-Lite – unterscheidet Frotzelei genauso gut von Hass, für einen Bruchteil.
- **Nur verdächtige Nachrichten prüfen.** *KI-Prüfung → Nur verdächtige Nachrichten* ([Moderation](moderation.md#general)) erwischt die meisten Fälle für einen Bruchteil von *Alle Nachrichten*. Eine Obergrenze für KI-Prüfungen pro Minute (Standard 12) schützt vor Spam-Wellen.
- **Weniger Kontext-Nachrichten.** *Chatnachrichten als Kontext* (Begleiter → Timing) ist der größte Teil jeder Anfrage; 30–40 reichen für die meisten Streams.
- **Weniger Emotes in der Liste.** Mit *Emote-Nutzung* auf *Selten* bekommt der Bot pro Anfrage weniger Emote-Namen.
- **Niedriger Denkaufwand.** Für Chat und Moderation ist *Niedrig* oder der Standard des Anbieters meist am besten.
- **Prompt-Caching** läuft automatisch: Der Teil der Anfrage, der jedes Mal gleich ist (die Anweisungen des Bots), wird von Claude, OpenAI, Gemini und OpenRouter zwischengespeichert, und HighBazar rechnet gecachte Tokens zu ihrem echten, viel niedrigeren Preis.

## Verbrauchsstatistik

**KI → Verbrauch** zeigt pro Tag und Kanal: KI-Aufrufe, Tokens, Websuchen, Kosten, gesendete Nachrichten und Moderations-Aktionen.

![Verbrauchsstatistik](ai-usage.webp)

Die Kosten sind aus den Preisen in deinen KI-Profilen geschätzt (OpenRouter meldet exakte Kosten). Die genaue Rechnung siehst du im Dashboard deines Anbieters.
