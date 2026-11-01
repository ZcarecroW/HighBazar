---
title: Installation
description: HighBazar auf Windows, macOS oder Linux herunterladen und installieren, beim ersten Mal öffnen und aktuell halten.
icon: Download
---

HighBazar läuft auf **Windows 10/11**, **macOS 11 oder neuer** (Apple Silicon und Intel) und **Linux** (x86-64). Lade die Datei für dein System von der [neuesten Version auf GitHub](https://github.com/ZcarecroW/HighBazar/releases/latest) herunter:

| System | Datei | Hinweise |
|---|---|---|
| **Windows 10/11** | `HighBazar_x.y.z_x64-setup.exe` | Installiert sich für den aktuellen Benutzer – keine Admin-Rechte nötig. |
| **macOS 11+** | `HighBazar_x.y.z_universal.dmg` | Eine App für Apple Silicon und Intel. |
| **Linux** | `HighBazar_x.y.z_amd64.AppImage` | Läuft auf den meisten Distributionen und aktualisiert sich selbst. |
| | `HighBazar_x.y.z_amd64.deb` / `HighBazar-x.y.z-1.x86_64.rpm` | Für Debian/Ubuntu und Fedora/openSUSE. |

Die Installer sind klein (unter Windows rund 5 MB): HighBazar nutzt die Web-Ansicht, die schon zu deinem System gehört, statt einen eigenen Browser mitzubringen.

## Windows {#windows}

1. Starte `HighBazar_x.y.z_x64-setup.exe`.
2. HighBazar installiert sich in deinen Benutzerordner und startet.

### „Der Computer wurde durch Windows geschützt“ {#windows-protected-your-pc}

HighBazar ist Open Source, aber nicht mit einem kostenpflichtigen Code-Signing-Zertifikat signiert. Deshalb kann Microsoft SmartScreen beim ersten Mal warnen. Klicke auf **Weitere Informationen → Trotzdem ausführen**. Der Download kommt direkt aus den GitHub-Releases des Projekts, und der Quellcode ist öffentlich.

> [!TIP]
> HighBazar braucht Microsofts **WebView2**-Laufzeit, die zu Windows 10 und 11 gehört. Fehlt sie auf einer älteren Installation, lädt der Installer sie für dich herunter.

## macOS {#macos}

1. Öffne die `.dmg`-Datei und zieh **HighBazar** in **Programme**.
2. Starte es aus dem Programme-Ordner.

### „HighBazar ist beschädigt und kann nicht geöffnet werden“

Die App ist nicht von Apple notarisiert (dafür braucht es ein kostenpflichtiges Entwicklerkonto), deshalb blockiert macOS sie zunächst. Nachdem du sie nach *Programme* gezogen hast, führe einmal im Terminal aus:

```bash
xattr -cr /Applications/HighBazar.app
```

Danach öffnest du HighBazar ganz normal.

## Linux {#linux}

Das **AppImage** läuft auf fast jeder Distribution und ist das einzige Linux-Format, das sich selbst aktualisiert:

```bash
chmod +x HighBazar_*.AppImage
./HighBazar_*.AppImage
```

Oder installiere ein Paket:

```bash
sudo apt install ./HighBazar_*.deb       # Debian, Ubuntu, Mint …
sudo dnf install ./HighBazar-*.rpm       # Fedora, openSUSE …
```

API-Schlüssel und Twitch-Logins werden über den **Secret Service** gespeichert (GNOME Keyring, KWallet …). Hat dein Desktop keinen, nutzt HighBazar eine Datei in seinem Datenordner, die nur dein Benutzer lesen kann – *Einstellungen → Erweitert* zeigt, welcher Speicher genutzt wird.

## Updates {#updates}

HighBazar sucht beim Start nach einer neuen Version und bietet an, sie mit einem Klick zu installieren (*Installieren & neu starten*). Updates kommen aus den offiziellen GitHub-Releases und sind **signiert** – HighBazar lehnt ein Update ab, dessen Signatur nicht passt. Du kannst die Prüfung unter [Einstellungen → Start & Updates](settings.md#start-and-updates) abschalten oder von Hand starten.

## Deinstallieren

- **Windows:** *Einstellungen → Apps → Installierte Apps → HighBazar → Deinstallieren*.
- **macOS:** HighBazar aus Programme in den Papierkorb ziehen.
- **Linux:** das AppImage löschen oder das Paket mit deinem Paketmanager entfernen.

Deine Einstellungen bleiben im Datenordner, bis du ihn löschst; [Datenschutz & Sicherheit](privacy.md#where-your-data-is) zeigt, wo er liegt. API-Schlüssel und Twitch-Logins bleiben im Anmeldespeicher des Systems – entferne sie vorher in der App (*Kanäle → Trennen*, *KI → API-Schlüssel*), wenn sie weg sollen.

## Weiter

Starte HighBazar – beim ersten Start öffnet sich die [Einrichtung](setup-wizard.md).
