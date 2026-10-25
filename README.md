# Music Discord RPC

![Discord rich presence showing the current song, artist, album art and progress bar](example.png?v=2)

Shows whatever you're playing as Discord Rich Presence. It reads the current session from Windows media controls, so it works with any player Windows knows about — Spotify, YouTube Music in your browser, VLC, foobar2000, Winamp, and the rest. The artwork label and the **Listen Along** / **View Artist** buttons follow the detected player.

## Install

One-liner in PowerShell:

```powershell
irm https://raw.githubusercontent.com/pepestunaa/music-discord-rpc/main/install.ps1 | iex
```

Downloads the latest release to `%LOCALAPPDATA%` and starts it. If no release is published yet it builds from source instead, which needs [Rust](https://rustup.rs).

From source, without cloning:

```powershell
cargo install --git https://github.com/pepestunaa/music-discord-rpc
```

Either way, Discord's **desktop** app has to be running for the activity to show.

## Autostart

Each launch registers the exe in `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` as `music_discord_rpc`. Undo it in Task Manager → Startup apps, or delete that value in `regedit`. There's no window or tray icon; end `music_discord_rpc.exe` in Task Manager to stop it.

## Use your own Discord app (optional)

The default build uses a bundled application ID. To brand the presence yourself, create an app at [discord.com/developers/applications](https://discord.com/developers/applications) and point `MUSIC_RPC_CLIENT_ID` at it (or edit `DEFAULT_CLIENT_ID` in `src/main.rs`):

```powershell
$env:MUSIC_RPC_CLIENT_ID = "1234567890123456789"
.\music_discord_rpc.exe
```

## Notes

- Presence clears when playback pauses or stops, and returns when you hit play. Nothing showing up? Check Discord desktop is running and the player actually reports to Windows media controls.
- Album art comes from a title + artist lookup and is cached, so obscure tracks and local files can show the wrong cover or the default gif.
- Windows 10 1809 or newer.
