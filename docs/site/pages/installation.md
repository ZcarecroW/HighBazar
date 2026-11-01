---
title: Installation
description: Download and install HighBazar on Windows, macOS or Linux, open it the first time and keep it up to date.
icon: Download
---

HighBazar runs on **Windows 10/11**, **macOS 11 or newer** (Apple Silicon and Intel) and **Linux** (x86-64). Download the file for your system from the [latest release on GitHub](https://github.com/ZcarecroW/HighBazar/releases/latest):

| System | File | Notes |
|---|---|---|
| **Windows 10/11** | `HighBazar_x.y.z_x64-setup.exe` | Installs for the current user – no admin rights needed. |
| **macOS 11+** | `HighBazar_x.y.z_universal.dmg` | One app for Apple Silicon and Intel. |
| **Linux** | `HighBazar_x.y.z_amd64.AppImage` | Works on most distributions and updates itself. |
| | `HighBazar_x.y.z_amd64.deb` / `HighBazar-x.y.z-1.x86_64.rpm` | For Debian/Ubuntu and Fedora/openSUSE. |

The installers are small (about 5 MB on Windows): HighBazar uses the web view that is already part of your system instead of shipping its own browser.

## Windows

1. Run `HighBazar_x.y.z_x64-setup.exe`.
2. HighBazar installs into your user folder and starts.

### “Windows protected your PC”

HighBazar is open source but not signed with a paid code-signing certificate, so Microsoft SmartScreen may warn you the first time. Click **More info → Run anyway**. The download comes straight from the project's GitHub releases, and the source code is public.

> [!TIP]
> HighBazar needs Microsoft's **WebView2** runtime, which is part of Windows 10 and 11. If it is missing on an older installation, the installer downloads it for you.

## macOS

1. Open the `.dmg` file and drag **HighBazar** into **Applications**.
2. Start it from Applications.

### “HighBazar is damaged and can't be opened”

The app is not notarized by Apple (that requires a paid developer account), so macOS blocks it at first. After moving it to *Applications*, run this once in the Terminal app:

```bash
xattr -cr /Applications/HighBazar.app
```

Then open HighBazar normally.

## Linux

The **AppImage** runs on almost every distribution and is the only Linux format that updates itself:

```bash
chmod +x HighBazar_*.AppImage
./HighBazar_*.AppImage
```

Or install a package:

```bash
sudo apt install ./HighBazar_*.deb       # Debian, Ubuntu, Mint …
sudo dnf install ./HighBazar-*.rpm       # Fedora, openSUSE …
```

API keys and Twitch logins are stored through the **Secret Service** (GNOME Keyring, KWallet …). If your desktop has none, HighBazar falls back to a file in its data folder that only your user can read – *Settings → Advanced* shows which one is used.

## Updates {#updates}

HighBazar checks for a new version when it starts and offers to install it with one click (*Install & restart*). Updates come from the official GitHub releases and are **signed** – HighBazar refuses an update whose signature doesn't match. You can switch the check off or run it by hand under [Settings → Start & updates](settings.md#start-and-updates).

## Uninstalling

- **Windows:** *Settings → Apps → Installed apps → HighBazar → Uninstall*.
- **macOS:** drag HighBazar from Applications to the Trash.
- **Linux:** delete the AppImage, or remove the package with your package manager.

Your settings stay in the data folder until you delete it; [Privacy & security](privacy.md#where-your-data-is) lists where it is. API keys and Twitch logins stay in the system credential store – remove them in the app first (*Channels → Disconnect*, *AI → API keys*) if you want them gone.

## Next

Start HighBazar – the [setup wizard](setup-wizard.md) opens on the first start.
