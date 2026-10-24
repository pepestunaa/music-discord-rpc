# Music Discord RPC

![Alt Text](example.png?v=2)

Shows whatever you're listening to as Discord Rich Presence — works with **any** player Windows knows about (Spotify, YouTube Music in your browser, VLC, foobar2000, Winamp, and anything else that hooks into Windows media controls).

The presence adapts to the real player: the app name shown on the artwork comes from the detected player, and the **Listen Along** / **View Artist** buttons point at the matching search (Spotify search for Spotify, YouTube Music search for everything else).

## Install

Grab the latest `music_discord_rpc.exe` from the [releases tab](https://github.com/pepestunaa/music-discord-rpc/releases), run it, enjoy.

Heads-up: on every run it adds itself to Windows startup (registry key `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` under the name `music_discord_rpc`). To remove it: Task Manager → Startup apps → disable `music_discord_rpc`, or delete that value with `regedit`. If something goes wrong you can also just end task `music_discord_rpc.exe` in Task Manager.

## Customize your own Discord application (optional)

By default this uses the bundled Discord application ID. To use your own:

- go to [discord.com/developers/applications](https://discord.com/developers/applications)
- create a new application
- copy the Client ID
- run the exe with the environment variable `MUSIC_RPC_CLIENT_ID` set to your ID (or edit `DEFAULT_CLIENT_ID` in `src/main.rs` and rebuild)
- enjoy

## Build from source

- install [Rust](https://rustup.rs)
- `cargo build --release`
- run `target\release\music_discord_rpc.exe`

Windows only (it reads the Windows System Media Transport Controls session).
