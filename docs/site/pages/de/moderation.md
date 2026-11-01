---
title: Moderation
description: Fairer, einstellbarer Schutz für deinen Chat – Sofort-Regeln plus eine KI, die den Zusammenhang versteht, mit Aktionen pro Kategorie, Eskalation, Ausnahmen und einem Protokoll mit Rückgängig.
icon: Shield
---

HighBazar schützt deinen Chat in zwei Stufen:

1. **Sofort-Regeln** laufen auf deinem PC, kosten nichts und reagieren in Millisekunden: gesperrte Begriffe, Links, Großschreibung, wiederholte Nachrichten, Emote-Spam, sehr lange Nachrichten.
2. **KI-Prüfungen** verstehen den Zusammenhang. Sie unterscheiden Frotzelei, schwarzen Humor und Kritik von echtem Hass, Belästigung oder Betrug – in jeder Sprache – und urteilen nach deinen eigenen Kanalregeln.

Was mit einem Fund passiert, entscheidest du: nur protokollieren, eine Aktion vorschlagen oder automatisch handeln. Ein **Moderations-Profil** fasst das alles zusammen; du wählst es pro Kanal unter [Kanäle](channels.md#profiles).

![Die Seite Moderation](moderation.webp)

## Eingebaute Profile

| Profil | Was es tut |
|---|---|
| **Ausgewogen** *(Standard)* | Handelt nur bei eindeutigem Hass, Belästigung, Drohungen, Betrug und Spam. Frotzeleien, Fluchen und Kritik bleiben erlaubt. Grenzfälle werden nur für dich protokolliert. Mods und VIPs sind ausgenommen. |
| **Locker** | Maximale Freiheit. Nur die schwersten Fälle (Hass, Drohungen, Doxxing, Betrug) werden behandelt; alles andere wird protokolliert. |
| **Familienfreundlich** | Streng. Entfernt auch Grenzfälle, sexuelle Anspielungen, übermäßige Großschreibung und Links von nicht ausgenommenen Nutzern. |
| **Nur Vorschläge** | Handelt nie selbst. Markiert problematische Nachrichten im Dashboard mit einer vorgeschlagenen Aktion, die du mit einem Klick ausführst. |
| **Nur beobachten** | Beobachtet nur und führt ein Moderations-Protokoll. Keine Aktionen, keine Vorschläge. |

## Allgemein {#general}

| Einstellung | Erklärung |
|---|---|
| **Modus** | *Aus*: keine Moderation. *Nur beobachten & protokollieren*: Probleme landen im Protokoll. *Aktionen vorschlagen*: Probleme erscheinen im Dashboard mit einer vorgeschlagenen Aktion, die du mit einem Klick ausführst. *Automatisch handeln*: HighBazar löscht, verwarnt, gibt Timeouts oder bannt selbst – so wie unter *Kategorien* eingestellt. |
| **KI-Prüfung** | *Aus (nur Regeln)*: nur die Sofort-Regeln – kostenlos. *Nur verdächtige Nachrichten*: Die KI prüft Nachrichten mit Warnzeichen – günstig und erwischt die meisten Fälle. *Alle Nachrichten (gebündelt)*: Jede Nachricht wird geprüft, mehrere auf einmal – am gründlichsten, kostet am meisten. |
| **Strenge** | Wie empfindlich die KI urteilt: 0 = nur extreme Inhalte, 50 = ausgewogen, 100 = familienfreundlich. |
| **Min. Sicherheit für automatische Aktionen** | Die KI sagt, wie sicher sie ist. Automatische Aktionen gibt es nur über diesem Wert (Standard 0,8); unsicherere Fälle werden nur protokolliert. Höher = weniger Fehler, aber mehr rutscht durch. |
| **Max. KI-Prüfungen pro Minute** | Obergrenze (Standard 12), damit eine Spam-Welle deine KI-Kosten nicht in die Höhe treibt. Die Sofort-Regeln arbeiten darüber hinaus weiter. |
| **Deine Kanalregeln** | Deine Chat-Regeln in einfachen Worten; die KI urteilt danach. „Kein Backseating – sag mir nicht, wie ich spielen soll, außer ich frage. Keine Spoiler zur Story. Nur Deutsch.“ |

> [!TIP]
> Nachrichten in Sprachen, die die Liste verdächtiger Wörter nicht kennt (sie deckt Deutsch und Englisch ab), bekommen im Modus *Nur verdächtige Nachrichten* immer die KI-Prüfung – so rutschen fremdsprachige Beleidigungen nicht durch.

## Kategorien {#categories}

Die KI ordnet Probleme in neun Kategorien ein. Für jede wählst du eine Aktion für einen **eindeutigen Verstoß** (die KI ist sicher) und für einen **Grenzfall** (es könnte ein Witz oder ein Missverständnis sein).

| Kategorie | Beispiele |
|---|---|
| **Hass** | Beleidigungen wegen Herkunft, Religion, Geschlecht, Sexualität, Behinderung. |
| **Belästigung** | Beleidigungen und Angriffe auf eine Person, auch auf den Streamer. |
| **Drohungen** | Gewaltandrohungen, Aufrufe, jemandem zu schaden. |
| **Sexuelle Inhalte** | Explizite oder belästigende sexuelle Nachrichten. |
| **Selbstverletzung** | Ermutigung zu Selbstverletzung oder Suizid. |
| **Spam** | Wiederholte Werbung, Angebote für Viewer-Bots, Fluten. |
| **Betrug & Phishing** | Falsche Gewinnspiele, „Free Nitro“, Phishing-Links. |
| **Doxxing** | Private Daten von jemandem verbreiten. |
| **Extremismus** | Extremistische Propaganda oder Symbole. |

| Aktion | Wirkung auf Twitch |
|---|---|
| **Nichts** | Wird ignoriert. |
| **Protokollieren** | Wird nur im Moderations-Protokoll vermerkt. |
| **Löschen** | Entfernt die Nachricht. |
| **Verwarnen** | Zeigt der Person eine Verwarnung, die sie bestätigen muss, bevor sie weiterchatten kann. |
| **Timeout** | Schaltet sie für eine Weile stumm (beliebige Dauer) – zuschauen kann sie weiter. |
| **Bannen** | Entfernt sie aus deinem Chat, bis du sie entbannst. |

## Wiederholungstäter {#repeat-offenders}

Mit **Bei Wiederholung verschärfen** bekommen wiederholte Verstöße innerhalb eines Zeitfensters strengere Aktionen: beim ersten Mal vielleicht eine Löschung, beim zweiten ein Timeout von 10 Minuten, beim dritten ein Bann. Füg so viele Stufen hinzu, wie du willst; nur Verstöße innerhalb des Fensters zählen, und eine Stufe macht eine Aktion nie milder.

## Sofort-Regeln {#instant-rules}

Diese Regeln brauchen keine KI und funktionieren auch, wenn das [Tagesbudget](costs.md#daily-budget) aufgebraucht ist.

![Sofort-Regeln](moderation-rules.webp)

| Regel | Erklärung |
|---|---|
| **Gesperrte Begriffe** | Wörter oder Ausdrücke, die immer behandelt werden. `*` passt auf alles (`idiot*` erwischt auch „Idioten“), und Schreibweisen wie `1d10t` (Leetspeak) werden erkannt. |
| **Bekannte Spam-Phrasen** | Eine eingebaute Liste typischen Viewer-Bot- und Gutschein-Spams („cheap viewers“, „free followers“, „free nitro“ …). Sie handelt ohne Zusammenhang, trifft also auch Zuschauer, die über Bots witzeln – **standardmäßig aus**. Mit KI-Moderation werden solche Nachrichten ohnehin im Zusammenhang geprüft. |
| **Links** | *Erlauben* oder *Blockieren (außer erlaubte Domains)*. Links können Betrug oder schädliche Seiten sein; erlaube die Domains, denen du vertraust, z. B. `youtube.com` oder deine eigene Website. |
| **Übermäßige Großschreibung** | NACHRICHTEN FAST NUR IN GROSSBUCHSTABEN wirken wie Schreien – greift über dem eingestellten Anteil an Großbuchstaben, ab einer Mindestlänge. |
| **Wiederholte Nachrichten** | Dieselbe Nachricht mehrmals innerhalb von 60 Sekunden. |
| **Emote-Spam** | Mehr Emotes in einer Nachricht als erlaubt. |
| **Max. Nachrichtenlänge** | Sehr lange Nachrichten („Textwände“) schieben alles andere aus dem Chat. `0` schaltet die Regel ab. |

Jede Regel hat ihre eigene Aktion.

## Ausnahmen

**Moderatoren**, **VIPs** und **Abonnenten** lassen sich ausnehmen, dazu **einzelne Accounts** – andere Bots, die du nutzt (Nightbot, StreamElements), oder ein Freund mit rauem Ton. **Der Streamer ist immer ausgenommen.**

## Nachrichten

| Einstellung | Erklärung |
|---|---|
| **Begründung an Twitch** | Der Text, den Twitch der Person und deinen Mods bei einem Timeout oder Bann zeigt. `{reason}` wird durch das ersetzt, was KI oder Regel gefunden haben. Standard: `[HighBazar] {reason}`. |
| **Automatische Aktionen im Chat ankündigen** | Postet einen kurzen Hinweis, wenn HighBazar selbst handelt („Nachricht mit Link entfernt.“) – zeigt dem Chat, dass Regeln durchgesetzt werden. Regel-Funde bekommen dort eine kurze, allgemeine Begründung, damit ein gesperrter Begriff oder ein Spam-Link nie im Chat wiederholt wird. Standardmäßig aus. |

## Das Moderations-Protokoll {#log}

Der Tab **Protokoll** listet jeden Fund: wer, was, die Kategorie, das Urteil und die Sicherheit, die Quelle (*KI*, *Regel* oder *Manuell*) und den Status – *protokolliert*, *vorgeschlagen*, *ausgeführt*, *simuliert* (Testmodus), *fehlgeschlagen*, *verworfen* oder *rückgängig*.

![Das Moderations-Protokoll](moderation-log.webp)

- **Suche** nach Name, Nachricht, Übersetzung oder Begründung und filtere nach *Aktionen* oder *Vorschläge*.
- Eine vorgeschlagene Aktion mit einem Klick **ausführen** oder **verwerfen**.
- Ein Timeout oder einen Bann **rückgängig** machen – HighBazar hebt ihn auf Twitch auf.

Zum Ausführen oder Rückgängigmachen muss der Bot laufen.

## Schild-Modus und Auto-Schild

Bei einem Hate-Raid – viele Accounts spammen gleichzeitig Hass – schaltet Twitchs **Schild-Modus** deine strengsten Chat-Einstellungen auf einen Schlag ein. HighBazar kann ihn selbst einschalten: **Auto-Schild bei Angriffen** springt an, wenn mehrere verschiedene Accounts in kurzer Zeit Hass, Belästigung, Drohungen oder sexuelle Belästigung posten (wie von der Moderation erkannt), und schaltet ihn nach einer ruhigen Phase wieder ab. Du findest ihn in der [Stream-Verwaltung](stream-manager.md#safety) (*Experte*), neben Twitchs eigenem AutoMod.

## Fremdsprachige Nachrichten

Hast du die Sprachen eingetragen, die du verstehst, bekommen markierte Nachrichten in anderen Sprachen einen **Vermerk für deine Mods** mit der Übersetzung und dem, was wirklich gemeint ist – Slang, versteckte Beleidigungen. Siehe [Übersetzung](translation.md).
