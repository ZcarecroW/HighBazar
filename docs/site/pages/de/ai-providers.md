---
title: KI-Anbieter
description: Die KI wählen, die die Nachrichten des Bots schreibt – Cloud-Anbieter mit API-Schlüssel, lokale Modelle oder dein Claude- bzw. ChatGPT-Abo – und sie mit KI-Profilen einstellen.
icon: Cpu
---

HighBazar funktioniert mit fast jedem KI-Anbieter. Jeder wird über ein **KI-Profil** eingestellt: Anbieter, Modell, Schlüssel, Denkaufwand, Grenzen, Preise und Websuche. Das Profil wählst du pro Kanal unter [Kanäle](channels.md#profiles) – und auf Wunsch ein zweites, günstigeres nur für die Moderation.

![Die Seite KI mit dem Profil-Editor](ai.webp)

## Anbieter

| Anbieter | Schlüssel | Websuche | Hinweise |
|---|---|---|---|
| **Anthropic (Claude)** | API-Schlüssel | eigene | Messages API mit Prompt-Caching und Websuche (bei der Spielrecherche auch gefundene Seiten lesen). |
| **OpenAI** | API-Schlüssel | eigene | Responses API (oder Chat Completions). |
| **Google Gemini** | API-Schlüssel | eigene (Google-Suche) | Interactions API mit automatischem Rückfall auf generateContent. Google speichert die Anfragen nicht. |
| **OpenRouter** | API-Schlüssel | eigene | Ein Schlüssel für Hunderte Modelle, exakte Kostenangabe. |
| **xAI (Grok)** | API-Schlüssel | eigene | |
| **Perplexity** | API-Schlüssel | immer | Antwortet mit eingebauter Live-Websuche. |
| **Groq** | API-Schlüssel | eigene bei *groq/compound*-Modellen | Andere Groq-Modelle nutzen die Suche von HighBazar. |
| **Mistral, DeepSeek, Together AI** | API-Schlüssel | von HighBazar | Über ihre OpenAI-kompatiblen Schnittstellen. |
| **Ollama, LM Studio** | keiner | von HighBazar | Lokale Modelle auf deinem eigenen PC – kostenlos, nichts außer Twitch verlässt deinen PC. Brauchen eine starke Grafikkarte. |
| **Custom (OpenAI-compatible)** | optional | von HighBazar | Jeder kompatible Server: eigene Basis-URL und Header. |
| **Claude Code CLI** | dein Claude-Abo | eigene | Nutzt deine installierte, angemeldete `claude`-CLI – siehe [unten](#claude-code-cli). |
| **Codex CLI** | dein ChatGPT-Abo | eigene | Nutzt deine installierte Codex CLI – siehe [unten](#codex-cli). |

„Von HighBazar“ heißt: HighBazar sucht selbst im Web und gibt die Ergebnisse an das Modell weiter – siehe [Websuche](web-search.md#highbazars-own-search).

## Eingebaute Profile

| Profil | Modell | Kurz gesagt |
|---|---|---|
| **Claude Opus 5.5** | `claude-opus-5-5` | Anthropics Flaggschiff. Beste Qualität und bester Humor; höhere Kosten. |
| **Claude Sonnet 5.5** | `claude-sonnet-5-5` | Sehr gute Qualität zu moderatem Preis. |
| **Claude Haiku 5.5** | `claude-haiku-5-5` | Schnell und sehr günstig. Empfohlen für Moderation und volle Chats. |
| **OpenAI GPT-6.1 Sol** | `gpt-6.1-sol` | OpenAIs ausgewogenes Modell. |
| **OpenAI GPT-6.1 Terra** | `gpt-6.1-terra` | Effizient und stark beim Schreiben – halb so teuer wie Sol. |
| **OpenAI GPT-6 Luna** | `gpt-6-luna` | Schnell und günstig. |
| **Gemini 3.8 Flash** | `gemini-3.8-flash` | Googles empfohlenes Allzweck-Modell mit Google-Suche. |
| **Gemini 3.5 Flash-Lite** | `gemini-3.5-flash-lite` | Googles günstigstes Modell für hohe Mengen. |
| **OpenRouter Auto** | `openrouter/auto` | Lässt OpenRouter ein Modell wählen. |
| **Mistral Small** | `mistral-small-latest` | Mistrals effizientes Modell (EU-Anbieter). |
| **DeepSeek Flash** | `deepseek-flash` | Günstig, antwortet ohne Denkpause. |
| **Perplexity Sonar** | `sonar` | Antwortet mit eingebauter Live-Websuche. |
| **Ollama (lokal)** / **LM Studio (lokal)** | | Deine eigenen Modelle. Wähl ein installiertes Modell in einer Kopie des Profils. |
| **Claude Code CLI (Abo)** | `haiku` | Dein Claude-Abo über die CLI. |
| **Codex CLI (ChatGPT-Abo)** | `gpt-6-luna` | Dein ChatGPT-Abo über die CLI. |

Eingebaute Profile sind schreibgeschützt und werden mit Updates besser; mit **Duplizieren & bearbeiten** änderst du eins.

## API-Schlüssel

Ein API-Schlüssel ist wie ein Passwort für einen KI-Anbieter. Du erstellst ihn auf der Website des Anbieters – der Link **Schlüssel holen** führt dorthin. Meist hinterlegst du dort eine Zahlungsart und zahlst nur, was du nutzt.

Schlüssel fügst du im Tab **API-Schlüssel** ein (oder direkt im Profil). Standardmäßig teilen sich alle Profile eines Anbieters einen Schlüssel; ein Profil kann auch einen **eigenen Schlüssel** nutzen – zum Beispiel ein zweites Anthropic-Konto nur für die Moderation. Schlüssel liegen im Anmeldespeicher deines Systems, nie in der Einstellungsdatei ([Datenschutz & Sicherheit](privacy.md)).

## Profil-Einstellungen

| Einstellung | Erklärung |
|---|---|
| **Anbieter** | Die Firma oder das Programm, dessen KI die Nachrichten schreibt. |
| **Modell** | Aus der Liste wählen, die vom Anbieter geladen wird (einfach tippen zum Suchen – Tippfehler werden verziehen), oder einen beliebigen Modellnamen eingeben. Größere Modelle schreiben besser, kosten aber mehr und brauchen länger; kleine sind schnell und günstig – ideal für die Moderation. |
| **Denkaufwand** | Wie lange das Modell nachdenkt, bevor es antwortet. Mehr Aufwand hilft bei schweren Fragen, ist aber langsamer und teurer. Für Chat und Moderation ist *Niedrig* oder der Standard des Anbieters meist am besten. Welche Stufen es gibt, hängt vom Anbieter ab (z. B. *Aus* … *Maximum*). |
| **Max. Ausgabe-Tokens** | Obergrenze für die Länge einer Antwort, inklusive des Nachdenkens. Tokens sind Wortteile, etwa ¾ Wort. Zu niedrig, und eine Antwort kann leer bleiben – HighBazar sagt dir dann Bescheid. |
| **Temperatur** | Zufälligkeit. Aktuelle Denk-Modelle ignorieren sie oder erlauben sie nicht; lass sie auf Standard, außer du weißt, was du willst. |
| **Websuche erlauben** | Lässt die KI nachschlagen, wenn eine Frage aktuelle Fakten braucht – siehe [Websuche](web-search.md). |
| **Timeout** | Wie lange HighBazar auf eine Antwort wartet (Standard 90 s). Für langsame lokale Modelle erhöhen. Die Spielrecherche darf von sich aus länger dauern. |
| **Preise** | Die Preise deines Anbieters pro Million Tokens (Eingabe und Ausgabe), für die Kostenanzeige und das Tagesbudget. Eingebaute Profile haben sie schon eingetragen; OpenRouter meldet die genauen Kosten selbst. |
| **Basis-URL** | Nur für eigene oder selbst betriebene Server; leer = die normale Adresse des Anbieters. |
| **Zusätzliche HTTP-Header** | Für spezielle Server oder Proxys, z. B. eine Organisations-ID. Meist nicht nötig. |
| **Responses API** *(OpenAI)* / **Interactions API** *(Gemini)* | Die neueren Schnittstellen (empfohlen). Nur abschalten, wenn ein Proxy oder ein älteres Modell sie nicht unterstützt. |

### Testen

**Testen** schickt eine kurze Nachricht mit diesem Profil und zeigt Antwort, Zeit, Tokens und Kosten – der schnellste Weg zu sehen, ob Schlüssel und Modell funktionieren. **Mit Websuche testen** stellt eine Frage, die eine Suche braucht (z. B. *Was steht in den neuesten Patch Notes zu Elden Ring Nightreign?*), und zeigt, wie viele Suchen dabei liefen.

## Zwei KIs: eine zum Chatten, eine zum Moderieren

Die Moderation prüft viele Nachrichten, deshalb kannst du unter [Kanäle](channels.md#profiles) eine eigene **KI für Moderation** wählen. Ein schnelles, günstiges Modell (Claude Haiku 5.5, GPT-6 Luna, Gemini Flash-Lite) unterscheidet Frotzelei genauso gut von Hass – und kostet einen Bruchteil. Auch die Übersetzung nutzt die Moderations-KI.

## Lokale Modelle (Ollama, LM Studio)

Lokale Modelle laufen auf deiner eigenen Grafikkarte: kostenlos, privat und unabhängig von jedem Dienst.

1. Installiere [Ollama](https://ollama.com/download) oder [LM Studio](https://lmstudio.ai) und lade ein Modell herunter.
2. Dupliziere in HighBazar das Profil *Ollama (lokal)* oder *LM Studio (lokal)* und wähle dein Modell aus der Liste.
3. Erhöhe den **Timeout**, wenn dein Modell langsam ist.

Lokale Modelle haben keine eigene Websuche – HighBazar sucht für sie ([Websuche](web-search.md#highbazars-own-search)).

## Claude Code CLI {#claude-code-cli}

Hast du ein Claude-Abo und die **Claude Code CLI** installiert und angemeldet, kann HighBazar sie statt eines API-Schlüssels nutzen (`claude -p`). HighBazar findet die CLI automatisch – auch die, die mit der Claude-Desktop-App kommt – oder du trägst ihren Pfad ein und klickst auf **Suchen**. Das Modell wählst du über den Kurznamen: *Haiku* (schnell und sparsam – gut für die Moderation), *Sonnet* (ausgewogen), *Opus* (besonders fähig) oder *Standard* (wie in Claude Code eingestellt).

HighBazar startet die CLI **nur mit Websuche** (und für die Spielrecherche mit dem Lesen von Webseiten) – keine Dateien, keine Befehle, nichts von deinen eigenen Claude-Code-Einstellungen, Plugins oder Erinnerungen. Die Nutzung zählt gegen dein Abo; die Kostenanzeige zeigt 0 $.

> [!IMPORTANT]
> Anthropics Bedingungen für Abos decken die persönliche Nutzung von Claude Code ab. Ob ein ständig laufender Chat-Bot darunter fällt, liegt in deiner Verantwortung – der Weg über einen API-Schlüssel ist der offiziell unterstützte.

## Codex CLI {#codex-cli}

Mit einem ChatGPT-Abo und installierter **Codex CLI** kann HighBazar auch diese nutzen. Melde dich einmal in einem Terminal mit `codex login` an und klicke dann auf **Suchen**. HighBazar startet Codex **ohne Befehle, Dateizugriff oder andere Werkzeuge** – nur Antworten und Websuche. Die Nutzung zählt gegen die Codex-Kontingente deines Abos.

## Verbrauch

Der Tab **Verbrauch** zeigt pro Tag und Kanal: KI-Aufrufe, Tokens, Websuchen, Kosten, gesendete Nachrichten und Moderations-Aktionen. Siehe [Kosten](costs.md).
