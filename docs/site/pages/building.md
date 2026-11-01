---
title: Building from source
description: Build and run HighBazar yourself, run the tests, work on the interface in a browser and build this documentation.
icon: Hammer
---

HighBazar is a [Tauri 2](https://v2.tauri.app) app: a Rust core with a Vue interface, shown in the web view that is part of your system.

## Requirements

- [Rust](https://rustup.rs) (stable, 1.95 or newer)
- [Node.js](https://nodejs.org) LTS with npm
- The [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system – on Windows the *Microsoft C++ Build Tools*, on Linux WebKitGTK and a few libraries

## Build and run

```bash
git clone https://github.com/ZcarecroW/HighBazar.git
cd HighBazar
npm install
npm run tauri dev     # run in development mode
npm run tauri build   # build installers into src-tauri/target/release/bundle
```

Twitch login needs a Client ID at build time: set the environment variable `HIGHBAZAR_TWITCH_CLIENT_ID`, or enter your own under *Settings → Advanced* in the running app.

## The interface in a browser

`npm run dev` serves the interface in a normal browser with a mocked backend – handy for UI work, nothing touches Twitch or an AI:

```text
http://localhost:1420/?demo                  dashboard
http://localhost:1420/?demo&view=companion   any page: dashboard, stream, channels, companion, moderation, ai, settings
http://localhost:1420/?demo&wizard           the setup wizard
http://localhost:1420/?demo&theme=light&lang=de&color=ocean&level=expert
```

## Tests

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # Rust core
npm test                                          # chat input, fuzzy search, emotes, translations
npx vue-tsc --noEmit                              # type check
```

Live tests run the real chat and research prompts against an AI and use your key or subscription, so they are skipped by default:

```bash
cargo test --manifest-path src-tauri/Cargo.toml live_ -- --ignored --nocapture --test-threads 1
```

They use the Claude Code CLI with Haiku; `HIGHBAZAR_LIVE_PROVIDER`, `HIGHBAZAR_LIVE_MODEL`, `HIGHBAZAR_LIVE_GAME`, `HIGHBAZAR_LIVE_QUESTION` and `HIGHBAZAR_LIVE_LANG` change that.

## Tech stack

| Layer | Technology |
|---|---|
| App shell | Tauri 2.12 (Rust 2024 edition) |
| Core | Rust: tokio, reqwest (rustls), tokio-tungstenite, keyring |
| Interface | Vue 3.5, Pinia 4, vue-i18n 11, Tailwind CSS 4.3, Vite 8, TypeScript 6 |
| Twitch | Device code flow, EventSub WebSocket, Helix API |
| Emotes | Twitch, BetterTTV, FrankerFaceZ and 7TV (with 7TV's live event stream) |
| Updates | Tauri updater with signed GitHub release artifacts |

## Project layout

```text
src/                  Vue interface (views, components, stores, i18n)
src-tauri/src/
  ai/                 provider clients (Anthropic, OpenAI, Gemini, OpenAI-compatible, Claude and Codex CLI)
  bot/                engine, prompts, moderation, research, translation, usage
  twitch/             login (device code), Helix, EventSub, emotes, 7TV live updates
  config/             settings, profiles and built-in presets
  overlay/            the chat overlay's local web server
docs/site/            this documentation
```

## This documentation

The site is generated from Markdown files in `docs/site/pages`:

```bash
node docs/site/shots.mjs            # screenshots from the browser preview (needs Microsoft Edge)
node docs/site/build.mjs --serve    # build and preview on http://localhost:4173/HighBazar/
```

The build fails on broken links or anchors. A push to `main` publishes it to GitHub Pages.

## Releases

Pushing a tag `vX.Y.Z` makes GitHub Actions build the installers for Windows, macOS and Linux, sign the updates, check that the signatures match the key installed apps trust, and publish the release.

## Contributing

Issues and pull requests are welcome on [GitHub](https://github.com/ZcarecroW/HighBazar). Please describe what you changed and run the tests before opening a pull request.
