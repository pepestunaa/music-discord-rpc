# Music Discord RPC

![Discord rich presence showing the current song, artist, album art and progress bar](example.png?v=2)

Shows whatever you're listening to as Discord Rich Presence. It reads the current media session from Windows System Media Transport Controls (SMTC), so it works with **any** player Windows knows about — Spotify, YouTube Music in your browser, VLC, foobar2000, Winamp, AIMP, MediaMonkey, and anything else that shows up in the Windows media popup.

The presence adapts to the real player: the label on the album art comes from the detected app, and the **Listen Along** / **View Artist** buttons point at a matching search (Spotify search for Spotify, YouTube Music search for everything else).

## Features

- Live "Listening to" activity with title, artist, album art and progress bar
- Per-player detection and branding, including a sensible fallback for unknown apps
- Album art looked up by title + artist, cached so repeat tracks cost nothing
- Presence clears itself when playback pauses, stops, or the player closes
- No config files, no accounts, no API keys

## Install

### One-liner

Paste into PowerShell:

```powershell
irm https://raw.githubusercontent.com/pepestunaa/music-discord-rpc/main/install.ps1 | iex
```

The script grabs the latest release build and installs it to `%LOCALAPPDATA%\music_discord_rpc.exe`, then starts it. If no release exists yet (or the download fails), it builds from source instead — that path needs [Rust](https://rustup.rs) installed and takes a minute.

Pass `-InstallDir` if you want it somewhere else, in which case run the script directly rather than piping it:

```powershell
irm https://raw.githubusercontent.com/pepestunaa/music-discord-rpc/main/install.ps1 -OutFile install.ps1
.\install.ps1 -InstallDir 'C:\Tools'
```

### From source

Requires [Rust](https://rustup.rs). Fastest way, no clone needed:

```powershell
cargo install --git https://github.com/pepestunaa/music-discord-rpc
music_discord_rpc
```

Or clone it and build locally:

```powershell
git clone https://github.com/pepestunaa/music-discord-rpc
cd music-discord-rpc
cargo build --release
.\target\release\music_discord_rpc.exe
```

Either route, the Discord **desktop** app has to be running for the activity to show up (the browser version of Discord can't receive it).

## Autostart

On every run the app registers itself in `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` under the name `music_discord_rpc`, so it comes back after you log in.

To turn that off:

- Task Manager → Startup apps → disable `music_discord_rpc`, or
- delete the `music_discord_rpc` value in that key with `regedit`

To stop it once, just end `music_discord_rpc.exe` in Task Manager. There's no tray icon or window — when it's running, that's the whole UI.

## Use your own Discord application (optional)

By default this uses the bundled Discord application ID. To brand the presence with your own app:

- go to [discord.com/developers/applications](https://discord.com/developers/applications)
- create a new application and copy its Application ID
- run the exe with the environment variable `MUSIC_RPC_CLIENT_ID` set to that ID, or edit `DEFAULT_CLIENT_ID` in `src/main.rs` and rebuild

In PowerShell:

```powershell
$env:MUSIC_RPC_CLIENT_ID = "1234567890123456789"
.\music_discord_rpc.exe
```

## Troubleshooting

**Nothing shows up on my profile.** Make sure the Discord desktop app is running and that music is actually playing. Discord only renders the activity while the track is playing — pausing clears it on purpose.

**My player isn't detected.** The app can only see players that register with Windows media controls. Check that pressing the media keys on your keyboard controls the player; if Windows doesn't see it, neither does this.

**"Unknown text" instead of a title or artist.** That's what gets shown when the player reports empty metadata — some browser sites and older players do this.

**Wrong album art.** Artwork comes from a title + artist lookup, so obscure tracks, remixes, and local files can miss or match the wrong release.

## Notes

Windows only — it reads SMTC, which is available on Windows 10 1809 and later. Written in Rust; the release binary is a single ~2.7 MB exe with no runtime dependencies.
