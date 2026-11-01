---
title: Websuche & Spielrecherche
description: Wie der Bot aktuelle Fakten nachschlägt, wenn der Chat fragt, wie er dein Spiel recherchiert, welche Suchquellen es gibt und was sie kosten.
icon: Globe
---

KI-Modelle kennen die Welt nur bis zu ihrem Trainingsstand. Für alles Neuere – den heutigen Patch, ein Release-Datum, das Turnier von gestern Abend – lässt HighBazar die KI **im Web suchen**. Das passiert an zwei Stellen:

- **Antworten an den Chat**: Fragt jemand den Bot etwas, das aktuelle Fakten braucht, schlägt er zuerst nach.
- **Spielrecherche**: Wechselst du die Kategorie, liest sich der Bot einmal ins Spiel ein – inklusive des neuesten Updates – und behält Fakten und Tipps für seine Nachrichten.

Beides ist standardmäßig an und lässt sich auf der Seite [Begleiter](companion.md#research) abschalten. Jede Anfrage enthält außerdem **das heutige Datum und deine Uhrzeit**, damit das Modell weiß, wie neu „das Neueste“ sein muss.

## Antworten an den Chat {#answers-to-chat}

Erwähnt jemand den Bot mit @, nutzt `!hb frage`, löst eine *Frag den Bot*-Belohnung ein oder stellt im Chat eine Frage, die der Bot mit einer *Auf den Chat reagieren*-Nachricht aufgreift, darf die KI vor der Antwort suchen. Sie sucht, wenn die Antwort von aktuellen oder speziellen Fakten abhängt – Updates, Patches, Release-Termine, News, Events, Ergebnisse, Statistiken, Preise – und beantwortet Smalltalk sofort. Bei Aktuellem schreibt sie Monat und Jahr in die Suchanfrage, damit alte Patch Notes nicht gewinnen.

Die Antwort ist eine normale Chatnachricht: keine Links, keine Quellenliste, kein „Ich habe nachgeschaut“. Suchergebnisse gelten als nicht vertrauenswürdiger Text – Anweisungen darin befolgt die KI nie.

> [!NOTE]
> Eine Antwort mit Suche dauert ein paar Sekunden länger (grob 5–20 Sekunden, je nach Anbieter und Modell).

## Spielrecherche {#game-research}

Wechselt die Kategorie, sammelt der Bot Material zum neuen Spiel:

1. Die **Wikipedia**-Zusammenfassung des Spiels (kostenlos, ohne Schlüssel) als verlässliche Grundlage.
2. Eine **Websuche** nach dem aktuellen Stand: neuestes Update, Patch, Season oder DLC und aktuelle News.
3. Die KI macht daraus **15 Fakten und 8 Tipps**, jeweils ein kurzer Satz, mit dem neuesten Update samt Datum unter den Fakten.

Das Ergebnis wird pro Spiel gespeichert und 14 Tage wiederverwendet (*Erneuern nach*). Das Dashboard zeigt unter **Spielwissen**, wie viele Fakten und Tipps bereitstehen, und der Feed meldet *„Spiel“ recherchiert: 15 Fakten und 8 Tipps bereit*.

Der Bot nutzt diese Fakten in seinen *Spiel-Fakten*- und *Spiel-Tipps*-Nachrichten und als Hintergrund für Antworten. Kategorien ohne Spiel wie *Just Chatting*, *Music* oder *Art* werden nicht recherchiert.

Schlägt die Recherche fehl – Timeout, Netzwerk-Schluckauf –, sagt der Feed warum, und HighBazar versucht es nach 15 Minuten erneut (bis zu dreimal pro Spiel). *Einstellungen → Cache der Spielrecherche leeren* erzwingt eine frische Recherche.

## Wer sucht?

| KI-Anbieter | Suche |
|---|---|
| Claude, OpenAI, Gemini, Grok, OpenRouter, Perplexity, Groq *compound* | Die **eigene Websuche** des Anbieters. Bezahlte APIs berechnen grob 0,5–1,5 Cent pro Suche. Claude darf bei der Spielrecherche auch eine gefundene Seite öffnen (pro Seite begrenzt). |
| Claude Code CLI, Codex CLI | Die eigene Websuche der CLI – über dein Abo, ohne Zusatzkosten. |
| Ollama, LM Studio, DeepSeek, Mistral, Together AI, andere Groq-Modelle, eigene Server | **Die Websuche von HighBazar** (unten). |

Die Seite KI zeigt neben *Websuche erlauben*, was für ein Profil gilt.

## Die Websuche von HighBazar {#highbazars-own-search}

Für Anbieter ohne eigene Suche sucht HighBazar selbst und gibt die Ergebnisse an das Modell. Bei einer Chat-Frage entscheidet das Modell zuerst, ob es eine Suche braucht und wonach; bei der Spielrecherche läuft die Suche sofort.

![Die Suchquelle wählen](ai-websearch.webp)

Die Quelle wählst du unter **KI → Websuche**:

| Quelle | Schlüssel | Kosten | Hinweise |
|---|---|---|---|
| **You.com** *(Standard)* | keiner | kostenlos | Aktuelle Websuche ohne Anmeldung (etwa 100 Suchen pro Tag). Ist das Tageslimit erreicht, springt Wikipedia ein. Empfohlen. |
| **Wikipedia** | keiner | kostenlos | Gut für Spiele, Lore und Hintergrund – nicht für aktuelle News oder Patches. |
| **Ollama-Websuche** | kostenloses Ollama-Konto | Gratis-Kontingent | Aktuelle Websuche von Ollama. |
| **Exa** | ja | 20 $ Startguthaben plus 10 $ gratis pro Monat, danach etwa 0,7 Cent pro Suche | Sehr genau, mit Textauszügen. |
| **Serper (Google)** | ja | 2.500 Suchen einmalig gratis, danach etwa 0,1 Cent pro Suche | Echte Google-Ergebnisse inklusive Direktantworten. |
| **Tavily** | ja | 1.000 Suchen pro Monat gratis | Websuche speziell für KI. |
| **Brave Search** | ja | monatliches Startguthaben für etwa 1.000 Suchen | Unabhängiger Suchindex. |
| **Eigene SearXNG-Instanz** | keiner | kostenlos | Wenn du [SearXNG](https://docs.searxng.org) selbst betreibst, z. B. mit Docker. Die `settings.yml` muss `json` unter *search → formats* enthalten. |

**Ergebnisse pro Suche** (Standard 5) – mehr Ergebnisse geben dem Modell mehr Material, kosten aber mehr Tokens. **Suche testen** führt eine Anfrage aus und zeigt die Ergebnisse.

## Tipps

- Sagt der Bot, er habe nichts Aktuelles gefunden, probier auf der Seite KI **Mit Websuche testen** mit derselben Frage – das zeigt, wie viele Suchen liefen.
- Mit der Websuche von HighBazar liefert **You.com** aktuelle Ergebnisse; Wikipedia allein kann den Patch dieser Woche nicht kennen.
- Lokale Modelle halten sich weniger zuverlässig an Suchergebnisse als große Cloud-Modelle. Für einen spielelastigen Kanal ist ein kleines Cloud-Modell mit eigener Suche (Claude Haiku 5.5, GPT-6 Luna, Gemini Flash) eine gute, günstige Wahl.
- Websuche kostet bei bezahlten APIs etwas extra. Schalte *Bei Fragen im Chat aktuelle Infos nachschlagen* ab, wenn dein Chat selten nach Aktuellem fragt.
