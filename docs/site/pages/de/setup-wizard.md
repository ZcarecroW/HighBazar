---
title: Einrichtung
description: Der geführte erste Start – Twitch verbinden, eine KI wählen, festlegen, wie der Bot sich verhält und wie streng der Chat geschützt wird.
icon: Wand
---

Beim ersten Start öffnet HighBazar einen kurzen Einrichtungsassistenten. Er dauert etwa fünf Minuten, jeder Schritt lässt sich überspringen, und du kannst ihn jederzeit wieder öffnen – in der Seitenleiste (**Einrichtung**) oder unter *Einstellungen → Start & Updates*.

![Der erste Schritt der Einrichtung](wizard.webp)

## 1. Willkommen

Wähle die **Sprache** der Oberfläche (Deutsch oder Englisch, oder *Automatisch* nach deinem System), das **Design** (System, Dunkel, Hell) und ein **Farbschema**. Alles lässt sich später unter [Einstellungen](settings.md#appearance) ändern.

## 2. Twitch – dein Streamer-Account {#twitch}

Klicke auf **Verbinden**. HighBazar zeigt einen kurzen Code und öffnet `twitch.tv/activate`, wo du ihn eingibst und die Berechtigungen bestätigst. Du meldest dich auf Twitchs eigener Seite an – **HighBazar sieht dein Passwort nie**.

Der Streamer-Account ist der, mit dem du streamst. HighBazar liest damit deinen Chat, sieht Follows, Subs und Raids und moderiert. Standardmäßig schreibt der Bot auch damit.

> [!TIP]
> HighBazar fragt Twitch nur nach den Berechtigungen, die es gerade braucht. Stellst du die [Stream-Verwaltung](stream-manager.md) später auf *Erweitert* oder *Experte*, fragt es einmal nach den zusätzlichen.

## 3. Bot-Account (optional) {#bot-account}

Du kannst einen **zweiten Twitch-Account nur für den Bot** verbinden, zum Beispiel *DeinNameBot*. Der Chat sieht den Bot dann unter seinem eigenen Namen statt unter deinem mit Präfix.

1. Leg auf twitch.tv einen kostenlosen Account an.
2. Mach ihn in deinem Chat zum Moderator: Tippe `/mod DeinNameBot` in deinen Chat.
3. Verbinde ihn hier genauso wie deinen Streamer-Account.

Die meisten Streamer starten ohne. Ohne Bot-Account sorgt der Schalter **Bot-Nachrichten mit Präfix kennzeichnen** dafür, dass die Nachrichten des Bots erkennbar bleiben (Standard-Präfix `[HighBazar]`).

## 4. KI

Wähle die KI, die die Nachrichten des Bots schreibt. Die Liste enthält fertige Profile für jeden Anbieter – Claude, OpenAI, Gemini, OpenRouter, Mistral, DeepSeek, Perplexity, lokale Modelle mit Ollama oder LM Studio sowie die Kommandozeilen-Tools Claude Code und Codex, die dein vorhandenes Abo nutzen.

![Ein KI-Profil in der Einrichtung wählen](wizard-ai.webp)

Für einen Cloud-Anbieter fügst du einen **API-Schlüssel** ein – eine Art Passwort für den Anbieter, das du auf dessen Website erstellst (der Link **Schlüssel holen** führt dorthin; meist hinterlegst du eine Zahlungsart und zahlst nur, was der Bot verbraucht). Klicke dann auf **Verbindung testen**: Die KI schreibt einen kurzen Gruß, und du siehst, dass der Schlüssel funktioniert.

> [!TIP]
> Unsicher, welche? Ein kleines, schnelles Modell wie **Claude Haiku 5.5**, **GPT-6 Luna** oder **Gemini 3.5 Flash-Lite** kostet ein paar Cent pro Stream-Stunde und ist ideal für die Moderation. Größere Modelle schreiben witzigere Nachrichten. [KI-Anbieter](ai-providers.md) vergleicht sie, und du kannst zwei kombinieren – ein gutes zum Chatten und ein günstiges zum Moderieren.

## 5. Begleiter

Wähle einen **Begleiter-Stil** – wie sich der Bot verhält – und ungefähr, wie oft er von sich aus schreibt:

| Stil | Kurz gesagt |
|---|---|
| **Ausgewogener Begleiter** | Freundlicher, witziger Allrounder. Reagiert auf den Chat, teilt Spiel-Fakten, stellt Fragen. |
| **Kleiner-Stream-Booster** | Redet etwas öfter und stellt viele Fragen, damit Lurker ins Gespräch kommen. |
| **Gemütlich & entspannt** | Ruhig, herzlich und gelassen. Weniger Nachrichten, sanfter Humor. |
| **Hype-Buddy** | Viel Energie, feiert jeden Spielzug und jeden neuen Chatter. |
| **Spiel-Experte** | Fakten, Lore, Tipps und Trivia – recherchiert das aktuelle Spiel online. |
| **Komiker** | Schlagfertig und verspielt, mit leichtem Necken. |
| **Stiller Helfer** | Keine Intervall-Nachrichten – nur Antworten, Begrüßungen, Events und Befehle. |

Du kannst der KI auch etwas **über dich und deinen Stream** erzählen – was du spielst, wie deine Community heißt, Running Gags. Alles lässt sich später auf der Seite [Begleiter](companion.md) feinjustieren.

## 6. Moderation

Wähle, wie streng der Chat geschützt wird:

| Profil | Was es tut |
|---|---|
| **Ausgewogen** *(Standard)* | Handelt nur bei eindeutigem Hass, Belästigung, Drohungen, Betrug und Spam. Frotzeleien, Fluchen und Kritik bleiben erlaubt; Grenzfälle werden nur protokolliert. |
| **Locker** | Maximale Freiheit: Nur die schwersten Fälle werden behandelt, alles andere protokolliert. |
| **Familienfreundlich** | Streng: Entfernt auch Grenzfälle, sexuelle Anspielungen, übermäßige Großschreibung und Links. |
| **Nur Vorschläge** | Handelt nie selbst – markiert Nachrichten mit einer vorgeschlagenen Aktion, die du mit einem Klick ausführst. |
| **Nur beobachten** | Beobachtet nur und führt ein Protokoll. |

Sprichst du nur bestimmte Sprachen, trag hier die **Sprachen, die du verstehst** ein – Nachrichten in anderen Sprachen werden dann für dich übersetzt ([Übersetzung](translation.md)).

## 7. Fertig

Klicke auf **Im Testmodus ausprobieren**, um den Bot in Aktion zu sehen, ohne etwas an Twitch zu senden – die Anleitung [Dein erster Stream](first-stream.md) zeigt, was du ausprobieren kannst – oder auf **Bot starten**, um direkt loszulegen.
