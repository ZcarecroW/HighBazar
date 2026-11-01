---
title: Datenschutz & Sicherheit
description: Was HighBazar wo speichert, was es an wen sendet, wie Schlüssel und Logins geschützt sind und wie Updates geprüft werden.
icon: Lock
---

HighBazar läuft komplett auf deinem Computer. Es gibt keinen HighBazar-Server, kein Konto, keine Telemetrie und kein Tracking.

## Was wohin geht

| Daten | Gehen an | Warum |
|---|---|---|
| Dein Twitch-Login | Nur Twitch (twitch.tv/activate) | Du meldest dich mit einem kurzen Code auf Twitchs eigener Seite an. HighBazar sieht dein Passwort nie; es bekommt ein Token mit genau den Berechtigungen, die du bestätigt hast. |
| Chatnachrichten, Stream-Titel, Kategorie | Den **KI-Anbieter, den du gewählt hast** | Damit die KI passende Nachrichten schreiben und moderieren kann. Gesendet werden nur die aktuellen Nachrichten, die die KI braucht (*Chatnachrichten als Kontext*), plus was du unter *Über dich und deinen Stream* geschrieben hast. Mit einem lokalen Modell (Ollama, LM Studio) verlässt nichts deinen PC. |
| Suchanfragen | Die eigene Suche des Anbieters oder die [Suchquelle](web-search.md#highbazars-own-search), die du gewählt hast | Nur, wenn eine Frage oder eine Spielrecherche aktuelle Fakten braucht. |
| Emote-Listen und -Bilder | Twitch, BetterTTV, FrankerFaceZ, 7TV | Um die Emotes deines Kanals zu zeigen. |
| Spielnamen | Wikipedia | Für die Spielrecherche (abschaltbar). |
| Update-Prüfung | GitHub | Um neue Versionen zu finden und herunterzuladen. |

## Schlüssel und Logins

API-Schlüssel und Twitch-Tokens liegen im **sicheren Anmeldespeicher** deines Systems – Windows-Anmeldeinformationsverwaltung, macOS-Schlüsselbund oder Secret Service unter Linux (GNOME Keyring, KWallet). In die Einstellungsdatei werden sie nie geschrieben. Nur wenn Linux keinen Anmeldespeicher hat, nutzt HighBazar eine Datei in seinem Datenordner, die nur dein Benutzer lesen kann; *Einstellungen → Erweitert → Geheimnisspeicher* zeigt, welcher genutzt wird.

## Wo deine Daten liegen {#where-your-data-is}

*Einstellungen → Datenordner öffnen* öffnet ihn. Er enthält deine Einstellungen (`config.json`), den Cache der Spielrecherche, Daten pro Kanal (z. B. welche Chatter der Bot schon gesehen hat, um Neue zu begrüßen) und die Verbrauchsstatistik.

| System | Datenordner | Logdateien |
|---|---|---|
| Windows | `%APPDATA%\io.github.zcarecrow.highbazar` | `%LOCALAPPDATA%\io.github.zcarecrow.highbazar\logs` |
| macOS | `~/Library/Application Support/io.github.zcarecrow.highbazar` | `~/Library/Logs/io.github.zcarecrow.highbazar` |
| Linux | `~/.local/share/io.github.zcarecrow.highbazar` | `~/.local/share/io.github.zcarecrow.highbazar/logs` |

Die Logdatei hält fest, was die App tut, und alle Fehler – nützlich, wenn du [ein Problem meldest](troubleshooting.md#reporting-a-problem). Sie enthält keine Schlüssel.

## Die Sicherheitsregeln des Bots

- Die KI darf sich **nie als du ausgeben**, nie Links posten, nie eine Nachricht mit `/` oder `.` beginnen (Twitch-Befehle) und behandelt Chatnachrichten als **nicht vertrauenswürdige Eingabe** – Versuche, ihre Regeln zu ändern, ihre Anweisungen zu verraten oder sie jemanden beleidigen zu lassen, werden ignoriert.
- HighBazar entfernt Links und Markdown aus KI-Antworten und verwirft eine Antwort, die trotzdem noch einen Link enthält.
- Ergebnisse der Websuche bekommt die KI als nicht vertrauenswürdigen Text; Anweisungen darin befolgt sie nie.
- Mit der Claude Code oder Codex CLI erlaubt HighBazar **nur die Websuche** – keinen Dateizugriff, keine Befehle, nichts von deinen eigenen CLI-Einstellungen, Plugins oder Erinnerungen.

## Das Chat-Overlay

Der Webserver des [Overlays](overlay.md) nimmt nur Verbindungen von deinem eigenen PC an (`127.0.0.1`), und der Link enthält einen geheimen Schlüssel. Zeig ihn nicht im Stream; *Neuen Link erzeugen* macht den alten ungültig.

## Updates

Updates werden aus den offiziellen GitHub-Releases geladen und vor der Installation **per Signatur geprüft**: Ein Update, das nicht mit dem Schlüssel des Projekts signiert ist, wird abgelehnt. Der Quellcode jeder Version ist öffentlich auf GitHub.
