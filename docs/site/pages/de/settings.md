---
title: Einstellungen
description: Darstellung und Farbschemas, Leistung auf Gaming-PCs, Chat-Anzeige, Start und Updates, das Overlay, erweiterte Optionen und deine Daten.
icon: Settings
---

![Die Seite Einstellungen](settings.webp)

## Darstellung {#appearance}

| Einstellung | Erklärung |
|---|---|
| **Design** | *System* (folgt deinem Betriebssystem), *Dunkel* oder *Hell*. |
| **Farbschema** | Zehn Schemas – Bazaar, Midnight, Ocean, Aurora, Forest, Gold, Sunset, Crimson, Sakura und Cyber –, jedes in hell und dunkel. Alle sind auf gut lesbaren Kontrast geprüft (WCAG AA), und die zweifarbigen Icons übernehmen die Farben des Schemas. |
| **Sprache** | *Automatisch* (die Sprache deines Systems), *English* oder *Deutsch*. |
| **Emotes als Bilder anzeigen** | Zeigt Emotes von Twitch, BTTV, FFZ und 7TV im Live-Feed als Bilder (von deren Servern geladen). |

![Zehn Farbschemas, jedes in hell und dunkel](themes.webp)

![Das Dashboard im hellen Design](dashboard-light.webp)

## Leistung {#performance}

HighBazar soll dein Spiel nicht ausbremsen. Der Bot selbst braucht kaum CPU – das meiste geht an die Oberfläche. Deshalb pausiert die Oberfläche, sobald das Fenster minimiert ist.

| Einstellung | Erklärung |
|---|---|
| **Spiele haben Vorrang** | HighBazar läuft mit niedrigerer Prozess-Priorität: Braucht ein Spiel die CPU, bekommt es sie zuerst. Im Chat merkst du davon nichts. |
| **Performance-Modus** | Minimieren oder Schließen des Fensters beendet die Oberfläche ganz. HighBazar läuft unsichtbar im Infobereich weiter – Bot, Moderation, Übersetzung und Overlay – mit nur ein paar MB Speicher (gemessen rund 9 MB) und ohne messbare CPU-Last. Ein Klick aufs Symbol öffnet das Fenster wieder; beenden über das Rechtsklick-Menü. |
| **Direkt im Infobereich starten** | Zusammen mit *Bot beim Öffnen der App automatisch starten* läuft HighBazar wie ein Hintergrunddienst. |
| **Jetzt in den Infobereich** | Macht es sofort. |

> [!TIP]
> Auf einem schwachen PC hilft es außerdem, unten *Neue Nachrichten sanft einrollen* abzuschalten – das halbiert die Grafiklast in einem vollen Chat ungefähr.

## Chat & Größe {#chat-and-size}

| Einstellung | Erklärung |
|---|---|
| **Größe der Oberfläche** | Skaliert die ganze App außer dem Chat. <kbd>Strg</kbd> + <kbd>+</kbd>/<kbd>−</kbd>, <kbd>Strg</kbd> + Mausrad; <kbd>Strg</kbd> + <kbd>0</kbd> setzt zurück. |
| **Größe des Chats** | Unabhängig von der Oberfläche. Im Live-Feed auch mit <kbd>Strg</kbd> + Mausrad über dem Chat oder mit A− / A+. |
| **Selbst im Chat schreiben** | Das [Eingabefeld](chat-input.md) unter dem Live-Chat. |
| **Neue Nachrichten sanft einrollen** | Der Chat gleitet nach oben statt zu springen, so verlierst du beim Lesen nicht die Zeile. Rollt nie, wenn Animationen in deinen Systemeinstellungen aus sind. |
| **Geschwindigkeit** | Wie lange eine neue Nachricht zum Einrollen braucht; 350–600 ms passen für die meisten. Kommen Nachrichten schneller, als sie einrollen, bleibt der Chat ohne Ruckeln in Bewegung. |
| **Anfahren und Bremsen** | *Sanft* (weicher Start und Stopp – am angenehmsten zum Mitlesen), *Zügig* (startet sofort), *Gleichmäßig* (konstantes Tempo) oder *Natürlich* (wie eine gedämpfte Feder – besonders ruhig bei vollem Chat). |

Die **Vorschau** neben diesen Einstellungen zeigt sie live; *Viel los simulieren* zeigt das Verhalten in einem schnellen Chat. Diese Einstellungen gelten auch fürs [Overlay](overlay.md).

## Start & Updates {#start-and-updates}

| Einstellung | Erklärung |
|---|---|
| **Beim Start nach Updates suchen** | Sucht beim Start nach einer neuen Version und bietet die Installation an. Updates kommen aus den offiziellen GitHub-Releases und sind signiert ([Installation](installation.md#updates)). **Jetzt prüfen** startet die Suche sofort. |
| **Bot beim Öffnen der App automatisch starten** | Startet den Bot für den aktiven Kanal, sobald HighBazar öffnet – praktisch, wenn HighBazar mit deinem PC startet. |
| **Einrichtungsassistent** | Öffnet die [Einrichtung](setup-wizard.md) erneut. |

## Steam-Overlay

Schaltet das Chat-Overlay für das Steam-Overlay oder OBS ein – siehe [Chat-Overlay](overlay.md).

## Erweitert

| Einstellung | Erklärung |
|---|---|
| **Twitch Client ID** | Jede Twitch-App hat eine, und HighBazar bringt eine mit – du brauchst das nicht. Trag nur eine eigene ein, wenn du unter [dev.twitch.tv](https://dev.twitch.tv/console) eine App registriert hast (siehe [FAQ](faq.md#can-i-use-my-own-twitch-application)). |
| **Größe des Live-Feeds** | Wie viele Einträge der Live-Feed behält (Standard 200). Weniger Einträge brauchen in einem vollen Chat weniger Speicher und CPU; ältere fallen einfach weg. |
| **Geheimnisspeicher** | Wo API-Schlüssel und Twitch-Logins liegen: im sicheren Anmeldespeicher deines Systems (Windows-Anmeldeinformationsverwaltung, macOS-Schlüsselbund, Secret Service unter Linux) – nie in der einfachen Einstellungsdatei. Gibt es keinen Anmeldespeicher, eine lokale Datei, die nur dein Benutzer lesen kann. |

## Daten

- **Datenordner öffnen** zeigt, wo HighBazar seine Einstellungen, den Cache der Spielrecherche und die Daten deiner Kanäle ablegt ([Datenschutz & Sicherheit](privacy.md#where-your-data-is)).
- **Cache der Spielrecherche leeren** erzwingt beim nächsten Mal eine frische [Spielrecherche](web-search.md#game-research).

## Über

Deine Version, Links zu GitHub und zu dieser Dokumentation und **Problem melden** (GitHub-Issues).
