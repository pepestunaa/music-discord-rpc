#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use discord_presence::{models::ActivityType, Client as DiscordClient};
use lru::LruCache;
use serde::Deserialize;
use std::env;
use std::num::NonZeroUsize;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use urlencoding::encode;
use winreg::enums::*;
use winreg::RegKey;

use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

const DEFAULT_CLIENT_ID: u64 = 1511746527125700842;
const APP_REGISTRY_NAME: &str = "music_discord_rpc";
// Upgraders from the old ytm_discord_rpc build would otherwise keep a
// second, dead startup entry forever.
const LEGACY_REGISTRY_NAME: &str = "ytm_discord_rpc";
const DEFAULT_ART_URL: &str = "https://c.tenor.com/1gdoP8gQoqoAAAAC/tenor.gif";

#[derive(Debug)]
struct MediaInfo {
    title: String,
    artist: String,
    source_app_id: String,
    start_timestamp: u64,
    end_timestamp: u64,
    is_playing: bool,
}
fn get_current_media() -> Option<MediaInfo> {
    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .ok()?
        .get()
        .ok()?;
    let session = manager.GetCurrentSession().ok()?;

    let playback = session.GetPlaybackInfo().ok()?;
    let status = playback.PlaybackStatus().ok()?;
    let is_playing = status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;

    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = props.Title().ok()?.to_string();
    let artist = props.Artist().ok()?.to_string();

    if title.is_empty() {
        return None;
    }

    // Identifies which app owns the media session (e.g. Spotify, a browser,
    // VLC), so the presence can brand itself after the real player.
    let source_app_id = session
        .SourceAppUserModelId()
        .map(|s| s.to_string())
        .unwrap_or_default();

    let timeline = session.GetTimelineProperties().ok()?;

    let position_ticks = timeline.Position().ok()?.Duration;
    let duration_ticks = timeline.EndTime().ok()?.Duration;

    let last_updated_filetime = timeline.LastUpdatedTime().ok()?.UniversalTime;

    let last_updated_secs = (last_updated_filetime / 10_000_000) as u64 - 11_644_473_600;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let time_drift = now_secs.saturating_sub(last_updated_secs);
    let position_secs = (position_ticks / 10_000_000) as u64;
    let duration_secs = (duration_ticks / 10_000_000) as u64;
    let real_position_secs = position_secs + time_drift;

    Some(MediaInfo {
        title,
        artist,
        source_app_id,
        start_timestamp: now_secs.saturating_sub(real_position_secs),
        end_timestamp: now_secs.saturating_sub(real_position_secs) + duration_secs,
        is_playing,
    })
}
fn format_media_str(text: &str) -> String {
    let mut s = text.trim().to_string();
    if s.is_empty() {
        return "Unknown text".to_string();
    }
    if s.chars().count() < 2 {
        s.push(' ');
    }
    if s.chars().count() > 128 {
        let mut truncated: String = s.chars().take(125).collect();
        truncated.push_str("...");
        return truncated;
    }
    s
}

struct PlayerInfo {
    name: String,
    use_spotify_search: bool,
}

impl PlayerInfo {
    fn from_aumid(aumid: &str) -> PlayerInfo {
        let lower = aumid.to_lowercase();
        let known: &[(&str, &str, bool)] = &[
            ("spotify", "Spotify", true),
            ("ytmusic", "YouTube Music", false),
            ("youtube", "YouTube Music", false),
            ("microsoftedge", "Edge", false),
            ("msedge", "Edge", false),
            ("chrome", "Chrome", false),
            ("firefox", "Firefox", false),
            ("vlc", "VLC", false),
            ("foobar2000", "foobar2000", false),
            ("winamp", "Winamp", false),
            ("aimp", "AIMP", false),
            ("mediamonkey", "MediaMonkey", false),
        ];
        for (needle, name, spotify) in known {
            if lower.contains(needle) {
                return PlayerInfo {
                    name: name.to_string(),
                    use_spotify_search: *spotify,
                };
            }
        }
        let name = if aumid.is_empty() {
            "Music".to_string()
        } else {
            // AUMIDs look like "Publisher.AppName-suffix!EntryPoint"; the
            // last segment is the closest thing to a readable app name.
            let tail = aumid
                .rsplit(|c| c == '!' || c == '.')
                .find(|s| !s.is_empty())
                .unwrap_or(aumid);
            capitalize_first(&tail.chars().take(32).collect::<String>())
        };
        PlayerInfo {
            name,
            use_spotify_search: false,
        }
    }

    fn search_url(&self, query: &str) -> String {
        if self.use_spotify_search {
            format!("https://open.spotify.com/search/{}", encode(query))
        } else {
            format!("https://music.youtube.com/search?q={}", encode(query))
        }
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => s.to_string(),
    }
}

#[derive(Deserialize)]
struct ItunesResponse {
    results: Vec<ItunesTrack>,
}

#[derive(Deserialize)]
struct ItunesTrack {
    #[serde(rename = "artworkUrl100")]
    artwork_url_100: Option<String>,
}

fn get_album_art_url(title: &str, artist: &str) -> Option<String> {
    let query = format!("{} {}", title, artist);
    let url = format!(
        "https://itunes.apple.com/search?term={}&media=music&limit=1",
        encode(&query)
    );

    let body = ureq::get(&url)
        .timeout(Duration::from_secs(5))
        .call()
        .ok()?
        .into_string()
        .ok()?;

    let result: ItunesResponse = serde_json::from_str(&body).ok()?;

    result
        .results
        .first()?
        .artwork_url_100
        .as_ref()
        .map(|url| url.replace("100x100bb", "512x512bb"))
}

fn add_to_startup() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r#"Software\Microsoft\Windows\CurrentVersion\Run"#;

    if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_SET_VALUE) {
        let _ = key.delete_value(LEGACY_REGISTRY_NAME);
        if let Ok(exe_path) = env::current_exe() {
            let exe_path_str = exe_path.to_string_lossy().into_owned();
            let _ = key.set_value(APP_REGISTRY_NAME, &exe_path_str);
        }
    }
}
fn main() {
    add_to_startup();

    let client_id = env::var("MUSIC_RPC_CLIENT_ID")
        .ok()
        .and_then(|id| id.parse::<u64>().ok())
        .unwrap_or(DEFAULT_CLIENT_ID);

    let mut drpc = DiscordClient::new(client_id);
    drpc.on_ready(|_ctx| {
        println!("Discord Rich Presence Connected!");
    })
    .persist();
    drpc.start();
    println!("Connecting to Discord...");
    let discord_deadline = Instant::now() + Duration::from_secs(30);
    while !DiscordClient::is_ready() {
        if Instant::now() >= discord_deadline {
            println!("Discord is not running; activity updates will retry once it starts.");
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }
    let mut current_track = String::new();
    let mut current_start = None;
    let mut current_source = String::new();
    let mut needs_update = true;
    let mut is_idle = false;
    let mut art_cache: LruCache<String, Option<String>> =
        LruCache::new(NonZeroUsize::new(20).unwrap());
    println!("Monitoring Windows SMTC...");
    loop {
        thread::sleep(Duration::from_millis(500));
        let Some(media) = get_current_media() else {
            // No media session at all — clear activity
            if !is_idle {
                println!("No media session found. Clearing activity.");
                let _ = drpc.clear_activity();
                is_idle = true;
            }
            continue;
        };

        // If music is paused or stopped, clear the activity
        if !media.is_playing {
            if !is_idle {
                println!("Music paused/stopped. Clearing activity.");
                let _ = drpc.clear_activity();
                is_idle = true;
            }
            continue;
        }

        // Music is playing — restore activity if needed
        if is_idle {
            println!("Music resumed. Restoring activity.");
            is_idle = false;
            needs_update = true;
        }

        if current_track != media.title
            || current_start != Some(media.start_timestamp)
            || current_source != media.source_app_id
        {
            println!("Listening: {} - {}", media.title, media.artist);
            current_track = media.title.clone();
            current_start = Some(media.start_timestamp);
            current_source = media.source_app_id.clone();
            needs_update = true;
        }
        if needs_update {
            let title = format_media_str(&media.title);
            let artist = format_media_str(&media.artist);
            let player = PlayerInfo::from_aumid(&media.source_app_id);
            let start = media.start_timestamp;
            let end = media.end_timestamp;

            // Fetch album art (with cache)
            let cache_key = format!("{}|{}", media.title, media.artist);
            let thumbnail_url = if let Some(cached) = art_cache.get(&cache_key) {
                cached.clone().unwrap_or_else(|| DEFAULT_ART_URL.to_string())
            } else {
                let art = get_album_art_url(&media.title, &media.artist);
                art_cache.put(cache_key.clone(), art.clone());
                art.unwrap_or_else(|| DEFAULT_ART_URL.to_string())
            };

            let listen_url = player.search_url(&format!("{} {}", media.title, media.artist));
            let artist_url = player.search_url(&media.artist);

            let res = drpc.set_activity(|act| {
                act.activity_type(ActivityType::Listening)
                    .details(&title)
                    .state(&artist)
                    .timestamps(|t| t.start(start).end(end))
                    .assets(|a| {
                        a.large_image(&thumbnail_url)
                            .large_text(&player.name)
                            .small_image("https://c.tenor.com/I0IYOKVklREAAAAC/tenor.gif")
                            .small_text("Bub")
                    })
                    .append_buttons(|b| b.label("Listen Along").url(listen_url))
                    .append_buttons(|b| b.label("View Artist").url(artist_url))
            });
            match res {
                Ok(_) => {
                    needs_update = false;
                    // Discord rejects activity updates faster than roughly
                    // one per 5 seconds.
                    thread::sleep(Duration::from_secs(5));
                }
                Err(e) => {
                    eprintln!("Failed to set activity: {}", e);
                }
            }
        }
    }
}
