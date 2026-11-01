---
title: Aus dem Quellcode bauen
description: HighBazar selbst bauen und starten, die Tests ausführen, an der Oberfläche im Browser arbeiten und diese Dokumentation bauen.
icon: Hammer
---

HighBazar ist eine [Tauri-2](https://v2.tauri.app)-App: ein Rust-Kern mit einer Vue-Oberfläche, angezeigt in der Web-Ansicht, die zu deinem System gehört.

## Voraussetzungen

- [Rust](https://rustup.rs) (stable, 1.95 oder neuer)
- [Node.js](https://nodejs.org) LTS mit npm
- Die [Tauri-Voraussetzungen](https://v2.tauri.app/start/prerequisites/) für dein System – unter Windows die *Microsoft C++ Build Tools*, unter Linux WebKitGTK und ein paar Bibliotheken

## Bauen und starten

```bash
git clone https://github.com/ZcarecroW/HighBazar.git
cd HighBazar
npm install
npm run tauri dev     # im Entwicklungsmodus starten
npm run tauri build   # Installer nach src-tauri/target/release/bundle bauen
```

Für den Twitch-Login braucht der Build eine Client ID: Setz die Umgebungsvariable `HIGHBAZAR_TWITCH_CLIENT_ID` oder trag in der laufenden App unter *Einstellungen → Erweitert* deine eigene ein.

## Die Oberfläche im Browser

`npm run dev` zeigt die Oberfläche in einem normalen Browser mit einem simulierten Backend – praktisch für Arbeit an der Oberfläche, nichts berührt Twitch oder eine KI:

```text
http://localhost:1420/?demo                  Dashboard
http://localhost:1420/?demo&view=companion   jede Seite: dashboard, stream, channels, companion, moderation, ai, settings
http://localhost:1420/?demo&wizard           die Einrichtung
http://localhost:1420/?demo&theme=light&lang=de&color=ocean&level=expert
```

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # Rust-Kern
npm test                                          # Chat-Eingabe, unscharfe Suche, Emotes, Übersetzungen
npx vue-tsc --noEmit                              # Typprüfung
```

Live-Tests schicken die echten Chat- und Recherche-Prompts an eine KI und nutzen deinen Schlüssel oder dein Abo, deshalb werden sie standardmäßig übersprungen:

```bash
cargo test --manifest-path src-tauri/Cargo.toml live_ -- --ignored --nocapture --test-threads 1
```

Sie nutzen die Claude Code CLI mit Haiku; `HIGHBAZAR_LIVE_PROVIDER`, `HIGHBAZAR_LIVE_MODEL`, `HIGHBAZAR_LIVE_GAME`, `HIGHBAZAR_LIVE_QUESTION` und `HIGHBAZAR_LIVE_LANG` ändern das.

## Technik

| Ebene | Technologie |
|---|---|
| App-Hülle | Tauri 2.12 (Rust 2024 Edition) |
| Kern | Rust: tokio, reqwest (rustls), tokio-tungstenite, keyring |
| Oberfläche | Vue 3.5, Pinia 4, vue-i18n 11, Tailwind CSS 4.3, Vite 8, TypeScript 6 |
| Twitch | Device Code Flow, EventSub-WebSocket, Helix API |
| Emotes | Twitch, BetterTTV, FrankerFaceZ und 7TV (mit dem Live-Event-Stream von 7TV) |
| Updates | Tauri-Updater mit signierten GitHub-Release-Dateien |

## Projektaufbau

```text
src/                  Vue-Oberfläche (Ansichten, Komponenten, Stores, i18n)
src-tauri/src/
  ai/                 Anbieter-Clients (Anthropic, OpenAI, Gemini, OpenAI-kompatibel, Claude und Codex CLI)
  bot/                Engine, Prompts, Moderation, Recherche, Übersetzung, Verbrauch
  twitch/             Login (Device Code), Helix, EventSub, Emotes, 7TV-Live-Updates
  config/             Einstellungen, Profile und eingebaute Vorlagen
  overlay/            der lokale Webserver des Chat-Overlays
docs/site/            diese Dokumentation
```

## Diese Dokumentation

Die Website entsteht aus Markdown-Dateien – Englisch in `docs/site/pages`, Deutsch in `docs/site/pages/de`:

```bash
node docs/site/shots.mjs            # Screenshots aus der Browser-Vorschau, Englisch und Deutsch (braucht Microsoft Edge)
node docs/site/build.mjs --serve    # bauen und unter http://localhost:4173/HighBazar/ ansehen
```

Der Build bricht bei kaputten Links oder Ankern ab. Ein Push auf `main` veröffentlicht sie auf GitHub Pages.

## Versionen veröffentlichen

Ein Tag `vX.Y.Z` lässt GitHub Actions die Installer für Windows, macOS und Linux bauen, die Updates signieren, prüfen, dass die Signaturen zum Schlüssel passen, dem installierte Apps vertrauen, und die Version veröffentlichen.

## Mitmachen

Issues und Pull Requests sind auf [GitHub](https://github.com/ZcarecroW/HighBazar) willkommen. Beschreib bitte, was du geändert hast, und lass die Tests laufen, bevor du einen Pull Request öffnest.
