---
title: Begleiter
description: Wie der Bot chattet – Persönlichkeit, Timing, Themen, Sprache, Emotes, Reaktionen, Chat-Befehle und Spielrecherche.
icon: Bot
---

Der Begleiter ist der chattende Teil von HighBazar. Ein **Begleiter-Profil** legt seine Persönlichkeit fest, wie oft er von sich aus schreibt und worüber, wie er auf deine Community reagiert und welche Befehle er beantwortet. Das Profil wählst du pro Kanal unter [Kanäle](channels.md#profiles).

![Die Seite Begleiter mit dem Bereich Persönlichkeit](companion.webp)

## Eingebaute Profile

| Profil | Kurz gesagt |
|---|---|
| **Ausgewogener Begleiter** *(Standard)* | Freundlicher, witziger Allrounder. Reagiert auf den Chat, teilt Spiel-Fakten, stellt Fragen. Passt für die meisten Streams. |
| **Kleiner-Stream-Booster** | Redet etwas öfter und stellt viele Fragen, damit Lurker ins Gespräch kommen. Ideal für Streams mit wenigen Zuschauern. |
| **Gemütlich & entspannt** | Ruhig, herzlich und gelassen. Weniger Nachrichten, sanfter Humor. Toll für Cozy Games, Kunst und Just Chatting. |
| **Hype-Buddy** | Viel Energie, liebt große Momente, feiert jeden Spielzug und jeden neuen Chatter. |
| **Spiel-Experte** | Kennt das Spiel in- und auswendig: Fakten, Lore, Tipps und Trivia. Recherchiert das aktuelle Spiel online. |
| **Komiker** | Schlagfertig und verspielt mit leichtem Necken. Witze, Wortspiele und lustige Kommentare zum Chat. |
| **Stiller Helfer** | Keine Intervall-Nachrichten. Antwortet nur, wenn man ihn anspricht, begrüßt Neue, reagiert auf Events und Befehle. |

Eingebaute Profile sind schreibgeschützt und werden mit Updates besser. Mit **Duplizieren & bearbeiten** machst du dein eigenes; die Liste links zeigt, welche Kanäle welches Profil nutzen.

## Persönlichkeit

| Einstellung | Erklärung |
|---|---|
| **Persönlichkeit** | Wer der Bot ist, wie eine Figurenbeschreibung: „Ein freundlicher Retro-Gaming-Nerd, der schlechte Wortspiele liebt und jeden neuen Zuschauer feiert.“ Die KI hält diese Stimme in jeder Nachricht. |
| **Humor** | 0 = sachlich und ruhig, 100 = Witze und Wortspiele, wann immer es passt. |
| **Energie** | Wie aufgedreht der Bot klingt: niedrig = entspannt und leise, hoch = viel Hype und Ausrufezeichen. |
| **Frechheit** | Wie sehr er dich und den Chat freundlich neckt. Für junge oder empfindliche Communities niedrig lassen – er beleidigt nie jemanden und tritt nie nach unten. |
| **Max. Nachrichtenlänge** | Obergrenze für eine Nachricht (Standard 220 Zeichen). Twitch erlaubt 500, aber kurze Nachrichten passen besser in den Chat: 150–200 sind ein guter Start. |
| **Zusätzliche Anweisungen** | Regeln, die immer gelten, in einfachen Worten: „Nie die Story spoilern“, „Nenn die Zuschauer ‚Krähen‘“, „Freitags in Piratensprache antworten“. |
| **Themen vermeiden** | Themen, die er nie anspricht und von denen er weglenkt. Standard: Politik, Religion, reale Tragödien, Zuschauerzahlen, Betteln um Follows oder Subs. |

> [!TIP]
> Der Bot nutzt auch, was du unter **Über dich und deinen Stream** unter [Kanäle](channels.md#behavior) geschrieben hast – Community-Name, Zeitplan, Running Gags. Oft der schnellste Weg, damit er wie einer deiner Stammgäste klingt.

## Timing {#timing}

Intervall-Nachrichten schreibt der Bot von sich aus, um den Chat am Leben zu halten: Fragen, Fakten, Witze, Reaktionen.

![Timing: Intervall, adaptives Tempo und Kontext](companion-timing.webp)

| Einstellung | Standard | Erklärung |
|---|---|---|
| **Intervall-Nachrichten** | an | Abschalten, wenn der Bot nur antworten soll, wenn man ihn anspricht. |
| **Intervall** | 3–5 Min. | Vor jeder Intervall-Nachricht wartet der Bot eine zufällige Zeit zwischen diesen Werten. Kleine Streams können etwas kürzer gehen. |
| **Nur während des Streams** | an | Kein Reden in einen leeren Offline-Chat. Antworten, Befehle und Moderation funktionieren auch offline. |
| **Adaptiv** | an | Redet weniger, wenn der Chat lebhaft ist, und pausiert, wenn sehr viel los ist. |
| **Sehr aktiv ab** | 25 Nachrichten/Min. | Ab so vielen Zuschauer-Nachrichten pro Minute gilt der Chat als voll, und der Bot hält sich zurück – die Leute reden ja schon. |
| **Pause nach unbeantworteten Nachrichten** | 4 | Reagiert niemand auf so viele Bot-Nachrichten hintereinander, ist der Chat vermutlich leer oder schaut gebannt zu; der Bot pausiert, bis wieder jemand schreibt. `0` = nie. |
| **Chatnachrichten als Kontext** | 40 | Wie viele aktuelle Nachrichten die KI liest, bevor sie schreibt. Mehr Kontext gibt passendere Nachrichten, kostet aber mehr Tokens; 30–60 ist ein guter Bereich. |
| **Max. Alter des Kontexts** | 20 Min. | Ältere Nachrichten fallen weg, damit der Bot nicht auf etwas von vor einer Stunde reagiert. |

Intervall-Nachrichten pausieren auch während **Werbepausen**, wenn die [Werbe-Automatik](stream-manager.md#ads-and-schedule) so eingestellt ist.

## Themen {#topics}

Die **Mischung** bestimmt, wie oft jede Art von Intervall-Nachricht gewählt wird. Die Werte sind Gewichte, keine Prozente: Ein Thema mit 8 kommt etwa doppelt so oft wie eins mit 4, und 0 schaltet es ab.

| Art | Was der Bot tut |
|---|---|
| **Auf den Chat reagieren** | Ein witziger Kommentar oder eine Nachfrage zu dem, was im Chat los ist. Fragt der Chat nach etwas Aktuellem, kann er im Web nachschlagen. |
| **Spiel-Fakten** | Ein überraschender oder wenig bekannter Fakt zum aktuellen Spiel – aus der [Spielrecherche](web-search.md#game-research), wenn es eine gibt. |
| **Spiel-Tipps** | Ein kurzer, nützlicher Tipp, verpackt als lockeres „Wusstest du schon“ – nie als Backseating. |
| **Trivia** | Allgemeines Gaming- oder Popkultur-Wissen, das zum Stream passt. |
| **Witze** | Ein kurzer, cleverer Witz oder ein Wortspiel zu Stream, Spiel oder Gespräch. |
| **Fragen an den Chat** | Eine leichte Frage, die Lurker zum Antworten einlädt. |
| **Hype** | Ein kurzer Schub positiver Energie für den Moment. |
| **Über den Stream** | Ein Kommentar zum Stream selbst – Spiel, Titel, Stimmung, wie lange du schon live bist. |

Die letzten beiden Arten kommen seltener dran, und der Bot prüft seine letzten Nachrichten, damit er nichts doppelt sagt.

## Sprache

| Einstellung | Erklärung |
|---|---|
| **Sprache** | *Stream-Sprache (von Twitch)* – die Sprache aus deinen Twitch-Stream-Infos. *Feste Sprache* – immer die Sprache darunter. *Sprache der meisten Chatter* – folgt deinem Chat, gut für internationale Communities. |
| **Sprache / Fallback** | Gilt bei *Feste Sprache* und als Ersatz, wenn die Stream-Sprache nicht bekannt ist. |
| **Chattern in ihrer Sprache antworten** | Aus ist empfohlen: Antworten in Sprachen, die du nicht sprichst, könntest du nicht lesen. Der [Übersetzer](translation.md) übersetzt dir stattdessen fremde Nachrichten. |
| **Englischen Gaming-Slang erlauben** | Erlaubt gängige Wörter wie „GG“, „clutch“ oder „poggers“ auch in anderen Sprachen – so redet Twitch-Chat nun mal. |

## Emotes

| Einstellung | Erklärung |
|---|---|
| **Emote-Nutzung** | *Nie*, *Selten*, *Manchmal* oder *Oft*. Emotes sind die kleinen Bilder im Twitch-Chat: Wörter wie „Kappa“ oder „catJAM“ erscheinen als Bild. |
| **Emote-Quellen** | Twitch, BetterTTV (BTTV), FrankerFaceZ (FFZ), 7TV. Die letzten drei sind Browser-Erweiterungen – Zuschauer sehen deren Emotes nur als Bild, wenn sie die Erweiterung nutzen. Schalte Quellen ab, die der Großteil deines Chats nicht nutzt. |
| **Emotes bevorzugen, die der Chat nutzt** | Der Bot nimmt bevorzugt Emotes, die dein Chat gerade nutzt – das wirkt am natürlichsten. Sonst bevorzugt er die eigenen Emotes deines Kanals vor den Hunderten globalen. |
| **Max. Emotes pro Nachricht** | Obergrenze, damit er nie mit Emotes spammt. |

## Reaktionen {#reactions}

![Reaktionen: Antworten, Begrüßungen und Events](companion-reactions.webp)

### Antworten, wenn angesprochen {#replies-when-addressed}

Der Bot antwortet, wenn jemand seinen **Namen** schreibt (siehe [Kanäle](channels.md#bot-name-and-prefix)) oder eines der **weiteren Trigger-Wörter** – Spitznamen wie „Bazar“.

| Einstellung | Standard | Erklärung |
|---|---|---|
| **Antworten auf Bot-Nachrichten zählen als Ansprache** | an | Auf Twitch kann man auf eine bestimmte Nachricht antworten; eine Antwort auf eine Bot-Nachricht gilt als Ansprache. |
| **Cooldown pro Nutzer** | 30 s | Wie lange eine Person warten muss, bis der Bot ihr wieder antwortet. |
| **Globaler Cooldown** | 4 s | Mindestpause zwischen zwei Antworten, egal wer fragt. |
| **Wer darf** | Alle | Oder nur Abonnenten, VIPs, Moderatoren oder nur du. |

Braucht eine Frage aktuelle Fakten – einen Patch, ein Release-Datum, ein Turnierergebnis –, schlägt der Bot zuerst nach (siehe [Websuche](web-search.md#answers-to-chat)).

### Neue Chatter begrüßen

Begrüßt Leute, die zum ersten Mal in deinem Chat schreiben – eine herzliche Begrüßung macht es viel wahrscheinlicher, dass Neue bleiben.

| Einstellung | Standard | Erklärung |
|---|---|---|
| **Wer gilt als neu** | Erste Nachricht überhaupt | *Erste Nachricht überhaupt im Kanal*: nur echte Neulinge – der Bot lernt, wer deine Stammgäste sind. *Erste Nachricht in diesem Stream*: die erste Nachricht jedes Streams von allen, Stammgäste eingeschlossen – schön für sehr kleine Streams. |
| **Verzögerung** | 4 s | Ein paar Sekunden Warten wirkt natürlicher und lässt dich zuerst grüßen. |
| **Max. Begrüßungen pro Minute** | 3 | Obergrenze, z. B. wenn ein Raid viele Neue auf einmal bringt. |
| **Begrüßungsstil** | – | Wie die Begrüßung klingen soll: „erwähne, dass wir heute einen No-Hit-Run machen“, „nur ein kurzer Satz“. |

In den ersten 20 Minuten lernt der Bot, wer deine Stammgäste sind, und begrüßt nur Twitchs offizielle *Erst-Chatter*.

### Event-Reaktionen

Auf diese Twitch-Events antwortet der Bot mit einer kurzen Nachricht (jedes lässt sich abschalten):

| Event | Was das ist |
|---|---|
| **Follows** *(gesammelt)* | Jemand hat auf *Folgen* geklickt, um benachrichtigt zu werden, wenn du live gehst. Follows werden 30 Sekunden gesammelt und gemeinsam bedankt, damit eine Follow-Welle den Chat nicht flutet. |
| **Subs** | Ein bezahltes Monats-Abo (Prime Gaming gibt einen kostenlosen Sub pro Monat). |
| **Resubs** | Jemand teilt mit, wie lange er schon abonniert ist. |
| **Sub-Geschenke** | Jemand kauft Abos für andere. |
| **Raids** | Ein anderer Streamer schickt dir seine Zuschauer – der Bot erwähnt auch, was der Raider gespielt hat. |
| **Cheers** | Ein Trinkgeld mit Bits, Twitchs eigener Währung. Cheers unter *Min. Bits* (Standard 100 – etwa ein US-Dollar für dich) bekommen kein Dankeschön. |
| **Zuschauer-Serien** | Jemand hat mehrere deiner Streams hintereinander geschaut. |

## Befehle {#commands}

Chat-Befehle beginnen mit einem **Präfix** (Standard `!hb`). Nutzt du schon einen Bot wie Nightbot mit `!`-Befehlen, wähl ein Präfix, das sich nicht damit überschneidet.

| Befehl | Englisch | Was er tut |
|---|---|---|
| `!hb frage <Frage>` | `!hb ask` | Der Bot beantwortet die Frage – mit Websuche, wenn nötig. Alles nach `!hb`, das kein anderer Befehl ist, gilt ebenfalls als Frage. |
| `!hb fakt` | `!hb fact` | Ein Fun Fact zum aktuellen Spiel. |
| `!hb witz` | `!hb joke` | Ein Witz, der zum Stream passt. |
| `!hb hilfe`, `!hb info` | `!hb about` | Eine kurze Vorstellung und die Liste der verfügbaren Befehle – ohne KI. |

Jeder Befehl hat seinen eigenen Schalter, **Cooldowns** (global und pro Nutzer) und **Wer darf**. Kanalpunkte-Belohnungen können dieselben KI-Aktionen auslösen – siehe [Stream-Verwaltung](stream-manager.md#channel-point-rewards).

## Recherche & Websuche {#research}

![Einstellungen für Recherche & Websuche](companion-research.webp)

| Einstellung | Standard | Erklärung |
|---|---|---|
| **Das aktuelle Spiel recherchieren** | an | Wechselt die Kategorie, liest sich der Bot einmal ein und behält 15 Fakten und 8 Tipps für seine Nachrichten, gespeichert pro Spiel. |
| **Beim Recherchieren im Web suchen** | an | Schlägt den aktuellen Stand des Spiels nach – neuester Patch, Season oder DLC – mit der eigenen Suche des KI-Anbieters oder der [Websuche von HighBazar](web-search.md#highbazars-own-search). |
| **Bei Fragen im Chat aktuelle Infos nachschlagen** | an | Patches, Release-Termine, Turniere und Ähnliches werden im Web geprüft, bevor der Bot antwortet; Smalltalk beantwortet er sofort. |
| **Wikipedia nutzen** | an | Der Wikipedia-Artikel zum Spiel als verlässliche Grundlage – kostenlos, ohne Schlüssel. |
| **Erneuern nach (Tagen)** | 14 | Nach so vielen Tagen wird ein Spiel neu recherchiert – Patches und Seasons verändern einiges. |

Wie das im Detail funktioniert – und was es kostet –, steht unter [Websuche & Spielrecherche](web-search.md).
