---
title: Problemlösung
description: Lösungen für die häufigsten Probleme – Twitch verbinden, KI-Fehler, Websuche, fehlende Emotes, Moderation, Overlay und Updates.
icon: Activity
---

Die meisten Probleme erscheinen als rote Zeile im **Live-Feed** – fang dort an. Fehler landen außerdem in der [Logdatei](privacy.md#where-your-data-is).

## Twitch

### „Neu verbinden nötig“ oder Funktionen gehen nicht

Klicke unter [Kanäle](channels.md#accounts) neben dem Account auf **Neu verbinden** und bestätige die Berechtigungen auf Twitch. Das ist nötig nach einer Passwortänderung, wenn Twitch den Login widerrufen hat oder wenn eine Funktion eine Berechtigung braucht, die du noch nicht erteilt hast (zum Beispiel eine höhere Stufe der [Stream-Verwaltung](stream-manager.md)).

### Der Bot reagiert nicht auf Follows oder kann nicht moderieren

Verbinde den **Streamer-Account** neu – er braucht die Berechtigungen für Moderation und Follower. Schreibt ein eigener Bot-Account, läuft die Moderation trotzdem über den Streamer-Account. Damit der Bot-Account in eigenem Namen löschen oder Timeouts geben kann, muss er in deinem Chat Moderator sein (`/mod DeinNameBot`).

### „Twitch-Event … nicht verfügbar“

Manche Events brauchen zusätzliche Berechtigungen oder einen bestimmten Status. Kanalpunkte-Einlösungen und Werbepausen zum Beispiel brauchen die Berechtigungen der Stufe *Experte* der Stream-Verwaltung und gibt es nur für Affiliates und Partner. Stell die [Stream-Verwaltung](stream-manager.md) auf *Experte* und bestätige die Berechtigungen, wenn du sie willst. Der Hinweis im Feed ist nur zur Info – alles andere läuft weiter.

### Der Anmeldecode ist abgelaufen

Codes gelten ein paar Minuten. Klick noch einmal auf **Verbinden** für einen neuen.

## KI

### „Für diesen Kanal ist kein KI-Profil ausgewählt“

Wähle unter [Kanäle](channels.md#profiles) ein **KI-Profil** für den Kanal. Ohne eins funktionieren nur die lokalen Moderationsregeln.

### Anmeldefehler (401, ungültiger Schlüssel)

Der API-Schlüssel ist falsch, abgelaufen oder ohne Guthaben. Füg unter *KI → API-Schlüssel* einen neuen ein und prüf dein Guthaben auf der Website des Anbieters. **Testen** auf der Seite KI zeigt, ob es funktioniert.

### „The model used up “Max. output tokens” before writing an answer“

Diese Meldung kommt auf Englisch: Das Modell hat sein Token-Limit verbraucht, bevor es eine Antwort schrieb. Denk-Modelle rechnen ihr Nachdenken in die Ausgabegrenze ein. Erhöhe **Max. Ausgabe-Tokens** im KI-Profil oder wähle einen niedrigeren **Denkaufwand**.

### Timeouts

Der Anbieter hat nicht rechtzeitig geantwortet. Erhöhe den **Timeout** des Profils – besonders bei lokalen Modellen – oder wähle ein schnelleres Modell.

### „Tägliches KI-Budget von … erreicht“

Die KI-Funktionen pausieren bis morgen. Erhöhe das Budget unter [Kanäle](channels.md#daily-budget) oder setze es auf 0 für kein Limit. Siehe [Kosten](costs.md).

### Der Bot antwortet jemandem nicht

- Prüf die **Cooldowns** (standardmäßig pro Nutzer 30 s, global 4 s) und **Wer darf** unter [Begleiter → Reaktionen](companion.md#replies-when-addressed).
- Die Nachricht muss den Bot-Namen oder ein Trigger-Wort enthalten – oder eine Antwort auf eine Bot-Nachricht sein.
- Im Modus *Vorschau* oder *Freigabe* warten Antworten unter [Wartet auf Versand](dashboard.md#pending).
- Die KI kann entscheiden, dass nichts passt, und aussetzen. Eine Antwort mit Link wird nie gepostet – die Logdatei sagt, warum eine Nachricht nicht rausging.

## Websuche

### Der Bot sagt, er habe nichts Aktuelles gefunden

- Prüf, dass **Websuche erlauben** im KI-Profil an ist und *Bei Fragen im Chat aktuelle Infos nachschlagen* auf der Seite [Begleiter](companion.md#research).
- Probier auf der Seite KI **Mit Websuche testen** mit derselben Frage: Das zeigt die Antwort und wie viele Suchen liefen.
- Bei einem Anbieter, der die Websuche von HighBazar nutzt, nimm **You.com** oder eine Suche mit Schlüssel; **Wikipedia** allein kann den Patch dieser Woche nicht kennen ([Websuche](web-search.md#highbazars-own-search)).
- Kleine oder lokale Modelle halten sich weniger zuverlässig an Suchergebnisse. Ein kleines Cloud-Modell mit eigener Suche (Claude Haiku 5.5, GPT-6 Luna, Gemini Flash) ist eine günstige, verlässliche Wahl.

### „Recherche zu … fehlgeschlagen“

Der Feed zeigt den Grund. HighBazar versucht es nach 15 Minuten erneut, bis zu dreimal pro Spiel. Mit der Claude Code CLI: Stell sicher, dass sie angemeldet ist (einmal `claude` in einem Terminal starten und `/login` tippen). *Einstellungen → Cache der Spielrecherche leeren* erzwingt einen Neustart.

## Claude Code CLI und Codex CLI

- **„Claude Code CLI nicht gefunden“** – installiere Claude Code oder trag im KI-Profil den Pfad zur `claude`-Datei ein und klick auf **Suchen**. Die CLI aus der Claude-Desktop-App wird ebenfalls gefunden.
- **„Claude Code CLI is not logged in“** (Meldung auf Englisch) – öffne ein Terminal, starte `claude` und tippe einmal `/login`.
- **„Codex CLI is not logged in“** im Feed oder **„Gefunden (…), aber nicht angemeldet“** auf der Seite KI – führe einmal `codex login` in einem Terminal aus und klick dann auf **Suchen**.

## Emotes

- **Emotes erscheinen als Wörter:** Schalte *Emotes als Bilder anzeigen* ein ([Einstellungen](settings.md#appearance)) und klick im Dashboard auf **Stream-Infos aktualisieren**, um sie neu zu laden.
- **Die Emotes deiner Abos fehlen in der Chat-Eingabe:** Twitch braucht eine neuere Berechtigung – die Eingabe zeigt einen Link, um deinen Account neu zu verbinden.
- **7TV-Änderungen kommen nicht an:** Sie kommen live, solange der Bot läuft. War 7TV nicht erreichbar, holt die nächste Aktualisierung (alle 30 Minuten oder *Stream-Infos aktualisieren*) sie nach.

## Moderation

- **Zu streng oder zu locker:** Pass **Strenge** und **Min. Sicherheit** an, schreib deine **Kanalregeln** in einfachen Worten oder starte von einem anderen eingebauten Profil aus ([Moderation](moderation.md)). Mit *Nur Vorschläge* gibst du jede Aktion selbst frei, während du nachjustierst.
- **„Die KI-Moderation ist am Minutenlimit“:** Erhöhe *Max. KI-Prüfungen pro Minute* oder nutze *Nur verdächtige Nachrichten*. Die Sofort-Regeln laufen weiter.
- **Eine Aktion ist fehlgeschlagen:** Der Bot (bzw. sein Account) braucht Moderationsrechte im Kanal; das Protokoll zeigt den Fehler von Twitch.

## Overlay

- **Das Overlay lädt nicht:** Prüf, dass *Chat-Overlay aktivieren* an ist und HighBazar läuft. Ist der Port belegt, wähl unter *Port* einen anderen.
- **„Port … ist belegt“:** Ein anderes Programm nutzt den Port; wähl einen anderen (z. B. 47824).
- **Der Link funktioniert nicht mehr:** Du hast einen neuen Link erzeugt; kopier den aktuellen.

## Updates und Installation

- **Die Update-Prüfung schlägt fehl:** Prüf deine Internetverbindung oder Firewall; *Jetzt prüfen* in den Einstellungen versucht es erneut. Die neueste Version kannst du jederzeit von der [Release-Seite](https://github.com/ZcarecroW/HighBazar/releases/latest) laden.
- **Windows warnt vor dem Installer** oder **macOS sagt, die App sei beschädigt:** siehe [Installation](installation.md#windows-protected-your-pc).

## Ein Problem melden {#reporting-a-problem}

Eröffne ein Issue auf [GitHub](https://github.com/ZcarecroW/HighBazar/issues) (*Einstellungen → Problem melden*) und gib an:

1. deine HighBazar-Version (*Einstellungen → Über*) und dein System,
2. was du gemacht hast und was stattdessen passiert ist,
3. die rote Zeile aus dem Live-Feed und den passenden Teil der [Logdatei](privacy.md#where-your-data-is) – sie enthält keine Schlüssel, aber schau vor dem Posten nach, ob etwas Privates drinsteht.
