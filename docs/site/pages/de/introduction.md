---
title: Einführung
description: Was HighBazar ist, was es für deinen Stream tut und wie die Teile zusammenspielen.
icon: Sparkles
---

HighBazar ist eine kleine Desktop-App, die neben deinem Stream läuft und Leben in deinen Twitch-Chat bringt. Sie nutzt ein KI-Modell deiner Wahl – Claude, GPT, Gemini, ein lokales Modell und mehr – und macht vier Dinge gleichzeitig:

- **Sie chattet wie ein netter Stammgast.** Alle paar Minuten sagt sie etwas Lustiges, Interessantes oder stellt eine Frage, die zu Chat und Spiel passt.
- **Sie reagiert auf deine Community.** Sie antwortet, wenn man sie anspricht, begrüßt Erst-Chatter und bedankt sich für Follows, Subs, Sub-Geschenke, Raids und Cheers.
- **Sie kennt dein Spiel.** Wechselst du die Kategorie, liest sie sich ins Spiel ein – Wikipedia und Websuche – und hat Fakten, Tipps und den neuesten Patch parat. Fragt der Chat nach Aktuellem, schlägt sie nach.
- **Sie hält den Chat sauber.** Sofort-Regeln erwischen Spam, Links und gesperrte Wörter kostenlos; eine KI, die den Zusammenhang versteht, unterscheidet Frotzelei von echtem Hass und handelt so, wie du es festlegst.

![Das Dashboard: Live-Feed, Statistik, Schnellsteuerung und Chat-Simulator](dashboard.webp)

HighBazar ist für **kleine Streamer** gemacht, die vom ersten Zuschauer an einen lebendigen Chat wollen – und wächst mit **großen Kanälen** mit: adaptives Tempo, gebündelte KI-Prüfungen und ein Tagesbudget.

## Wer schreibt die Nachrichten?

Standardmäßig schreibt der Bot mit **deinem eigenen Streamer-Account**, und jede Bot-Nachricht beginnt mit einem Präfix wie `[HighBazar]`, damit die Zuschauer sie von dir unterscheiden können. Wenn du magst, verbindest du einen **eigenen Bot-Account** (zum Beispiel *DeinNameBot*); der Chat sieht den Bot dann unter seinem eigenen Namen. Siehe [Kanäle](channels.md#accounts).

> [!NOTE]
> HighBazar gibt sich nie als du aus. Die KI darf nie in deinem Namen sprechen, und Twitchs Regeln verlangen, dass automatische Nachrichten erkennbar sind – genau dafür ist das Präfix da.

## Wie die Teile zusammenspielen

HighBazar ist in **Profilen** organisiert. So wechselst du ganze Setups mit einem Klick, und die eingebauten Vorlagen bleiben als sichere Grundlage erhalten:

| Profil | Was es enthält | Seite |
|---|---|---|
| **Kanal-Profil** | Ein Twitch-Kanal oder Setup: die Accounts, Bot-Name und Präfix, welches Begleiter-, Moderations- und KI-Profil genutzt wird, wie Nachrichten gesendet werden, das Tagesbudget, die Übersetzung. | [Kanäle](channels.md) |
| **Begleiter-Profil** | Wie der Bot chattet: Persönlichkeit, Timing, Themen, Reaktionen, Befehle, Spielrecherche. Sieben eingebaute Stile. | [Begleiter](companion.md) |
| **Moderations-Profil** | Wie streng der Chat geschützt wird: Modus, KI-Prüfung, Kategorien und Aktionen, Wiederholungstäter, Sofort-Regeln, Ausnahmen. Fünf eingebaute Profile. | [Moderation](moderation.md) |
| **KI-Profil** | Welcher Anbieter und welches Modell die Nachrichten schreibt – mit Schlüssel, Denkaufwand, Grenzen, Preisen und Websuche. Vorlagen für jeden Anbieter. | [KI-Anbieter](ai-providers.md) |

Eingebaute Profile sind schreibgeschützt und werden mit Updates besser. Zum Ändern klickst du auf **Duplizieren & bearbeiten** – deine Kopie gehört dir.

## Was du damit machen kannst

<div class="cards">
<a href="../companion/"><strong>{icon:Bot} Begleiter</strong><span>Intervall-Nachrichten, Antworten, Begrüßungen, Danke für Events, Chat-Befehle und Spielrecherche.</span></a>
<a href="../moderation/"><strong>{icon:Shield} Moderation</strong><span>Lokale Regeln, KI-Prüfung in neun Kategorien, Eskalation, Vorschläge und ein Protokoll mit Rückgängig.</span></a>
<a href="../translation/"><strong>{icon:Languages} Übersetzung</strong><span>Fremdsprachige Nachrichten für dich übersetzt, mit Vermerken für deine Mods.</span></a>
<a href="../stream-manager/"><strong>{icon:MonitorPlay} Stream-Verwaltung</strong><span>Titel, Kategorie, Chat-Modi, Raids, Umfragen, Kanalpunkte, AutoMod, Werbung.</span></a>
<a href="../chat-input/"><strong>{icon:Chat} Selbst chatten</strong><span>Im Chat schreiben, mit Emote- und Emoji-Vorschlägen aller Anbieter.</span></a>
<a href="../overlay/"><strong>{icon:Gamepad2} Chat-Overlay</strong><span>Dein Chat im Steam-Overlay oder in OBS – mitlesen ohne Alt+Tab.</span></a>
</div>

## Überall erklärt

Du musst Twitch nicht in- und auswendig kennen. Neben jeder Einstellung steht ein kleines **ⓘ** – zeig darauf, fokussiere es mit der Tastatur oder klick es an, damit es offen bleibt. Es erklärt die Einstellung in einfachen Worten, auch Twitch-Begriffe wie Raids, VIPs, AutoMod oder Kanalpunkte. Das [Twitch-Glossar](twitch-glossary.md) sammelt alle an einem Ort.

![Ein Tooltip erklärt die Moderations-Modi](tooltip.webp)

## Kostenlos, privat und leicht

- **Kostenlos und Open Source** (MIT). KI-Anbieter rechnen direkt mit dir ab, was der Bot verbraucht – mit einem kleinen Modell meist ein paar Cent pro Stream-Stunde, mit einem lokalen Modell nichts. Siehe [Kosten](costs.md).
- **Deine Daten bleiben auf deinem PC.** API-Schlüssel und Twitch-Logins liegen im Anmeldespeicher deines Systems; Chatnachrichten gehen nur an den KI-Anbieter, den du gewählt hast. Siehe [Datenschutz & Sicherheit](privacy.md).
- **Gemacht für Gaming-PCs.** Ein nativer Rust-Kern statt eines mitgelieferten Browsers, niedrigere Prozess-Priorität und ein Performance-Modus, in dem der Bot mit rund 9 MB Speicher im Hintergrund läuft. Siehe [Einstellungen](settings.md#performance).

## Wie geht es weiter?

1. [HighBazar installieren](installation.md) – auf Windows, macOS oder Linux.
2. Die [Einrichtung](setup-wizard.md) durchgehen – etwa fünf Minuten.
3. Alles gefahrlos im [Testmodus](first-stream.md) ausprobieren, bevor du das nächste Mal streamst.
