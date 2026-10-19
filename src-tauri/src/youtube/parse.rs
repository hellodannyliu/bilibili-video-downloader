use serde_json::Value;

use crate::types::youtube::{YoutubeCaption, YoutubeChapter, YoutubeVideoInfo, YoutubeVideoItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YoutubeTarget {
    Video(String),
    Channel(String),
    Playlist(String),
}

pub fn classify_youtube_input(input: &str) -> eyre::Result<YoutubeTarget> {
    let input = input.trim();
    if input.is_empty() {
        return Err(eyre::eyre!("请输入 YouTube 视频、频道或播放列表链接"));
    }

    if looks_like_url(input) {
        return classify_url(input);
    }

    if let Some(id) = bare_video_id(input) {
        return Ok(YoutubeTarget::Video(id));
    }

    if is_playlist_id(input) {
        return Ok(YoutubeTarget::Playlist(input.to_string()));
    }

    if let Some(id) = bare_channel_id(input) {
        return Ok(YoutubeTarget::Channel(id));
    }

    if let Some(handle) = input.strip_prefix('@')
        && !handle.is_empty()
        && !handle.contains('/')
    {
        return Ok(YoutubeTarget::Channel(format!(
            "https://www.youtube.com/@{handle}"
        )));
    }

    Err(eyre::eyre!(
        "无法识别这个 YouTube 输入，请使用视频链接、频道链接或播放列表链接"
    ))
}

fn looks_like_url(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.contains("youtube.com")
        || lower.contains("youtu.be")
}

fn classify_url(input: &str) -> eyre::Result<YoutubeTarget> {
    let (host, path, query) = split_url(input);
    let host = host.to_ascii_lowercase();
    if !is_youtube_host(&host) {
        return Err(eyre::eyre!("不是 YouTube 链接"));
    }

    let path = path.trim_matches('/');
    let video_id = query_param(&query, "v").or_else(|| video_id_from_path(path));
    let playlist_id = query_param(&query, "list");

    if path == "playlist" || path.starts_with("playlist/") {
        let id = playlist_id.ok_or_eyre_msg("播放列表链接里没有 list 参数")?;
        return Ok(YoutubeTarget::Playlist(id));
    }

    if let Some(id) = video_id {
        return Ok(YoutubeTarget::Video(id));
    }

    if let Some(id) = playlist_id {
        return Ok(YoutubeTarget::Playlist(id));
    }

    if let Some(id) = channel_id_from_path(path) {
        return Ok(YoutubeTarget::Channel(id));
    }

    if path.starts_with('@') || path.starts_with("c/") || path.starts_with("user/") {
        return Ok(YoutubeTarget::Channel(normalize_channel_url(&host, path)));
    }

    Err(eyre::eyre!("无法从这个链接里解析出视频、频道或播放列表"))
}

fn ok_or_eyre_msg<T>(value: Option<T>, message: &str) -> eyre::Result<T> {
    value.ok_or_else(|| eyre::eyre!(message.to_string()))
}

trait OptionMsg<T> {
    fn ok_or_eyre_msg(self, message: &str) -> eyre::Result<T>;
}

impl<T> OptionMsg<T> for Option<T> {
    fn ok_or_eyre_msg(self, message: &str) -> eyre::Result<T> {
        ok_or_eyre_msg(self, message)
    }
}

fn is_youtube_host(host: &str) -> bool {
    host == "youtu.be"
        || host.ends_with(".youtu.be")
        || host == "youtube.com"
        || host.ends_with(".youtube.com")
        || host == "youtube-nocookie.com"
        || host.ends_with(".youtube-nocookie.com")
}

fn split_url(input: &str) -> (String, String, String) {
    let without_scheme = input.split_once("://").map_or(input, |(_, rest)| rest);
    let without_scheme = without_scheme.strip_prefix("//").unwrap_or(without_scheme);
    let (host_path, query) = without_scheme
        .split_once('?')
        .map_or((without_scheme, ""), |(host_path, query)| {
            (host_path, query.split('#').next().unwrap_or(query))
        });
    let host_path = host_path.split('#').next().unwrap_or(host_path);
    let (host, path) = host_path
        .split_once('/')
        .map_or((host_path, ""), |(host, path)| (host, path));
    let host = host.split('@').next_back().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    (host.to_string(), path.to_string(), query.to_string())
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if k == key && !v.is_empty() {
            return Some(percent_decode(v));
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                out.push(value);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn bare_video_id(input: &str) -> Option<String> {
    if is_video_id(input) {
        Some(input.to_string())
    } else {
        None
    }
}

fn is_video_id(input: &str) -> bool {
    input.len() == 11
        && input
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn video_id_from_path(path: &str) -> Option<String> {
    let segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let candidate = match segments.as_slice() {
        ["shorts", id] | ["embed", id] | ["live", id] | ["v", id] | ["watch", id] | [id] => *id,
        _ => return None,
    };
    is_video_id(candidate).then(|| candidate.to_string())
}

fn bare_channel_id(input: &str) -> Option<String> {
    if input.starts_with("UC") && input.len() == 24 && is_youtube_token(input) {
        Some(input.to_string())
    } else {
        None
    }
}

fn channel_id_from_path(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if let ["channel", id, ..] = segments.as_slice()
        && id.starts_with("UC")
    {
        return Some((*id).to_string());
    }
    None
}

fn normalize_channel_url(host: &str, path: &str) -> String {
    let host = if host == "youtu.be" {
        "www.youtube.com"
    } else {
        host
    };
    format!("https://{host}/{path}")
}

fn is_playlist_id(input: &str) -> bool {
    const PREFIXES: &[&str] = &["PL", "UU", "LL", "FL", "OL", "RD", "TL", "WL", "OLAK", "PU"];
    if input == "WL" || input == "LL" {
        return true;
    }
    input.len() > 11
        && is_youtube_token(input)
        && PREFIXES.iter().any(|prefix| input.starts_with(prefix))
}

fn is_youtube_token(input: &str) -> bool {
    input
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn extract_assigned_json(text: &str, marker: &str) -> Option<Value> {
    let idx = text.find(marker)?;
    let relative = text[idx..].find('{')?;
    let start = idx + relative;
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in text[start..].char_indices() {
        let index = start + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return serde_json::from_str(&text[start..=index]).ok();
                }
            }
            _ => {}
        }
    }
    None
}

pub fn text_of(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    if let Some(text) = value.get("simpleText").and_then(Value::as_str) {
        return text.to_string();
    }
    if let Some(text) = value.get("content").and_then(Value::as_str) {
        return text.to_string();
    }
    if let Some(runs) = value.get("runs").and_then(Value::as_array) {
        return runs
            .iter()
            .filter_map(|run| run.get("text").and_then(Value::as_str))
            .collect();
    }
    if let Some(text) = value.pointer("/text/content").and_then(Value::as_str) {
        return text.to_string();
    }
    String::new()
}

pub fn walk<'a>(value: &'a Value, visit: &mut dyn FnMut(&'a Value)) {
    visit(value);
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, visit);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                walk(item, visit);
            }
        }
        _ => {}
    }
}

pub fn find_all<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    let mut found = Vec::new();
    let mut visit = |current: &'a Value| {
        if let Some(child) = current.get(key) {
            found.push(child);
        }
    };
    walk(value, &mut visit);
    found
}

pub fn continuation_token(value: &Value) -> Option<String> {
    let renderers = find_all(value, "continuationItemRenderer");
    for renderer in renderers {
        if let Some(token) = renderer
            .pointer("/continuationEndpoint/continuationCommand/token")
            .and_then(Value::as_str)
        {
            return Some(token.to_string());
        }
    }
    None
}

pub fn parse_lockup_videos(value: &Value) -> Vec<YoutubeVideoItem> {
    let mut videos = Vec::new();
    for lockup in find_all(value, "lockupViewModel") {
        if let Some(item) = parse_lockup(lockup) {
            videos.push(item);
        }
    }
    dedupe_videos(videos)
}

fn parse_lockup(lockup: &Value) -> Option<YoutubeVideoItem> {
    let content_type = lockup.get("contentType").and_then(Value::as_str);
    if content_type.is_some_and(|kind| kind != "LOCKUP_CONTENT_TYPE_VIDEO") {
        return None;
    }
    let video_id = lockup.get("contentId").and_then(Value::as_str)?;
    if !is_video_id(video_id) {
        return None;
    }
    let meta = lockup.pointer("/metadata/lockupMetadataViewModel")?;
    let title = text_of(meta.get("title").unwrap_or(&Value::Null));
    if title.is_empty() {
        return None;
    }

    let mut channel_name = String::new();
    let mut view_text = String::new();
    let mut published_text = String::new();
    if let Some(rows) = meta
        .pointer("/metadata/contentMetadataViewModel/metadataRows")
        .and_then(Value::as_array)
    {
        if let Some(parts) = rows
            .first()
            .and_then(|row| row.get("metadataParts"))
            .and_then(Value::as_array)
            && let Some(part) = parts.first()
        {
            channel_name = text_of(part.get("text").unwrap_or(&Value::Null));
        }
        if let Some(parts) = rows
            .get(1)
            .and_then(|row| row.get("metadataParts"))
            .and_then(Value::as_array)
        {
            if let Some(part) = parts.first() {
                view_text = text_of(part.get("text").unwrap_or(&Value::Null));
            }
            if let Some(part) = parts.get(1) {
                published_text = text_of(part.get("text").unwrap_or(&Value::Null));
            }
        }
    }

    Some(YoutubeVideoItem {
        video_id: video_id.to_string(),
        title,
        thumbnail: thumbnail_from(lockup).unwrap_or_else(|| default_thumbnail(video_id)),
        duration: duration_from_lockup(lockup),
        published_text,
        view_text,
        channel_name,
        channel_id: String::new(),
        channel_avatar: String::new(),
        order: 0,
    })
}

fn duration_from_lockup(lockup: &Value) -> u64 {
    for badge in find_all(lockup, "thumbnailBadgeViewModel") {
        let text = text_of(badge.get("text").unwrap_or(&Value::Null));
        if let Some(duration) = duration_from_clock(&text) {
            return duration;
        }
    }
    0
}

fn thumbnail_from(value: &Value) -> Option<String> {
    let mut best: Option<(i64, String)> = None;
    for source in find_all(value, "sources") {
        let Some(items) = source.as_array() else {
            continue;
        };
        for item in items {
            let Some(url) = item.get("url").and_then(Value::as_str) else {
                continue;
            };
            if url.is_empty() {
                continue;
            }
            let width = item.get("width").and_then(Value::as_i64).unwrap_or(0);
            if best
                .as_ref()
                .is_none_or(|(best_width, _)| width >= *best_width)
            {
                best = Some((width, url.to_string()));
            }
        }
    }
    best.map(|(_, url)| url)
}

pub fn default_thumbnail(video_id: &str) -> String {
    format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg")
}

pub fn duration_from_clock(text: &str) -> Option<u64> {
    let text = text.trim();
    if text.is_empty() || text.contains(|c: char| c != ':' && !c.is_ascii_digit()) {
        return None;
    }
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() != 2 && parts.len() != 3 {
        return None;
    }
    let mut numbers = Vec::new();
    for part in parts {
        if part.is_empty() {
            return None;
        }
        numbers.push(part.parse::<u64>().ok()?);
    }
    if numbers.len() == 3 {
        Some(numbers[0] * 3600 + numbers[1] * 60 + numbers[2])
    } else {
        Some(numbers[0] * 60 + numbers[1])
    }
}

pub fn parse_playlist_videos(value: &Value) -> Vec<YoutubeVideoItem> {
    let mut videos = Vec::new();
    for renderer in find_all(value, "playlistPanelVideoRenderer") {
        if let Some(item) = parse_playlist_video(renderer) {
            videos.push(item);
        }
    }
    dedupe_videos(videos)
}

fn parse_playlist_video(value: &Value) -> Option<YoutubeVideoItem> {
    let video_id = value.get("videoId").and_then(Value::as_str)?;
    if !is_video_id(video_id) {
        return None;
    }
    let title = text_of(value.get("title").unwrap_or(&Value::Null));
    if title.is_empty() {
        return None;
    }
    let length = text_of(value.get("lengthText").unwrap_or(&Value::Null));
    let channel_name = text_of(value.get("shortBylineText").unwrap_or(&Value::Null));
    let channel_id = value
        .pointer("/shortBylineText/runs/0/navigationEndpoint/browseEndpoint/browseId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let thumbnail = value
        .pointer("/thumbnail/thumbnails")
        .and_then(Value::as_array)
        .and_then(|items| items.last())
        .and_then(|item| item.get("url"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| default_thumbnail(video_id));
    let order = text_of(value.get("indexText").unwrap_or(&Value::Null))
        .parse::<i64>()
        .unwrap_or(0);

    Some(YoutubeVideoItem {
        video_id: video_id.to_string(),
        title,
        thumbnail,
        duration: duration_from_clock(&length).unwrap_or(0),
        published_text: String::new(),
        view_text: String::new(),
        channel_name,
        channel_id,
        channel_avatar: String::new(),
        order,
    })
}

fn dedupe_videos(videos: Vec<YoutubeVideoItem>) -> Vec<YoutubeVideoItem> {
    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::new();
    for video in videos {
        if seen.insert(video.video_id.clone()) {
            unique.push(video);
        }
    }
    unique
}

pub fn channel_metadata(value: &Value) -> (String, String, String, String) {
    let Some(meta) = find_all(value, "channelMetadataRenderer")
        .into_iter()
        .next()
    else {
        return (String::new(), String::new(), String::new(), String::new());
    };
    let title = meta.get("title").and_then(Value::as_str).unwrap_or("");
    let channel_id = meta.get("externalId").and_then(Value::as_str).unwrap_or("");
    let description = meta
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    let avatar = meta
        .pointer("/avatar/thumbnails")
        .and_then(Value::as_array)
        .and_then(|items| items.last())
        .and_then(|item| item.get("url"))
        .and_then(Value::as_str)
        .unwrap_or("");
    (
        channel_id.to_string(),
        title.to_string(),
        description.to_string(),
        avatar.to_string(),
    )
}

pub fn playlist_header(value: &Value) -> (String, String, String, String) {
    let title = find_all(value, "playlistMetadataRenderer")
        .into_iter()
        .find_map(|meta| meta.get("title").and_then(Value::as_str))
        .unwrap_or("")
        .to_string();

    let header = find_all(value, "pageHeaderViewModel").into_iter().next();
    let mut author_name = String::new();
    let mut author_id = String::new();
    let mut author_avatar = String::new();
    if let Some(header) = header {
        for content in find_all(header, "content") {
            let text = content.as_str().unwrap_or("");
            if let Some(name) = text.strip_prefix("by ")
                && !name.is_empty()
                && author_name.is_empty()
            {
                author_name = name.to_string();
            }
        }
        author_id = find_all(header, "browseEndpoint")
            .into_iter()
            .find_map(|endpoint| endpoint.get("browseId").and_then(Value::as_str))
            .filter(|id| id.starts_with("UC"))
            .unwrap_or("")
            .to_string();
        author_avatar = thumbnail_from(header).unwrap_or_default();
    }
    (title, author_name, author_id, author_avatar)
}

pub fn parse_video_info(
    video_id: &str,
    next: &Value,
    player: Option<&Value>,
) -> Option<YoutubeVideoInfo> {
    let primary = find_all(next, "videoPrimaryInfoRenderer")
        .into_iter()
        .next()?;
    let secondary = find_all(next, "videoSecondaryInfoRenderer")
        .into_iter()
        .next();
    let title = text_of(primary.get("title").unwrap_or(&Value::Null));
    if title.is_empty() {
        return None;
    }
    let published_text = text_of(primary.get("dateText").unwrap_or(&Value::Null));
    let view_text = text_of(
        primary
            .pointer("/viewCount/videoViewCountRenderer/viewCount")
            .unwrap_or(&Value::Null),
    );

    let mut description = String::new();
    let mut channel_name = String::new();
    let mut channel_id = String::new();
    let mut channel_avatar = String::new();
    if let Some(secondary) = secondary {
        description = secondary
            .pointer("/attributedDescription/content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(owner) = secondary.pointer("/owner/videoOwnerRenderer") {
            channel_name = text_of(owner.get("title").unwrap_or(&Value::Null));
            channel_id = owner
                .pointer("/title/runs/0/navigationEndpoint/browseEndpoint/browseId")
                .or_else(|| owner.pointer("/navigationEndpoint/browseEndpoint/browseId"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            channel_avatar = owner
                .pointer("/thumbnail/thumbnails")
                .and_then(Value::as_array)
                .and_then(|items| items.last())
                .and_then(|item| item.get("url"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
        }
    }

    let details = player.and_then(|player| player.get("videoDetails"));
    let micro = player.and_then(|player| player.pointer("/microformat/playerMicroformatRenderer"));
    if description.is_empty() {
        description = details
            .and_then(|details| details.get("shortDescription"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }
    if channel_name.is_empty() {
        channel_name = details
            .and_then(|details| details.get("author"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }
    if channel_id.is_empty() {
        channel_id = details
            .and_then(|details| details.get("channelId"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
    }

    let duration = details
        .and_then(|details| details.get("lengthSeconds"))
        .and_then(Value::as_str)
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(0);
    let published_at = micro
        .and_then(|micro| micro.get("publishDate"))
        .and_then(Value::as_str)
        .and_then(parse_published_at)
        .or_else(|| parse_published_at(&published_text))
        .unwrap_or(0);
    let tags = details
        .and_then(|details| details.get("keywords"))
        .and_then(Value::as_array)
        .map(|keywords| {
            keywords
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    let thumbnail = micro
        .and_then(|micro| micro.pointer("/thumbnail/thumbnails"))
        .and_then(Value::as_array)
        .and_then(|items| items.last())
        .and_then(|item| item.get("url"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| default_thumbnail(video_id));

    Some(YoutubeVideoInfo {
        video_id: video_id.to_string(),
        title,
        description,
        channel_id,
        channel_name,
        channel_avatar,
        thumbnail,
        duration,
        published_at,
        published_text,
        view_count: parse_view_count(&view_text),
        view_text,
        tags,
        chapters: parse_chapters(next),
        captions: player.map(parse_captions).unwrap_or_default(),
    })
}

pub fn parse_chapters(value: &Value) -> Vec<YoutubeChapter> {
    let mut chapters = Vec::new();
    for renderer in find_all(value, "chapterRenderer") {
        let title = text_of(renderer.get("title").unwrap_or(&Value::Null));
        let start_ms = renderer
            .get("timeRangeStartMillis")
            .and_then(Value::as_f64)
            .or_else(|| {
                renderer
                    .get("timeRangeStartMillis")
                    .and_then(Value::as_i64)
                    .map(|ms| ms as f64)
            });
        let Some(start_ms) = start_ms else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        chapters.push(YoutubeChapter {
            title,
            start_sec: start_ms / 1000.0,
        });
    }
    chapters.sort_by(|a, b| {
        a.start_sec
            .partial_cmp(&b.start_sec)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    chapters.dedup_by(|a, b| (a.start_sec - b.start_sec).abs() < 0.01 && a.title == b.title);
    chapters
}

pub fn parse_captions(player: &Value) -> Vec<YoutubeCaption> {
    let mut captions = Vec::new();
    for track in find_all(player, "captionTracks")
        .into_iter()
        .filter_map(Value::as_array)
        .flatten()
    {
        let Some(url) = track.get("baseUrl").and_then(Value::as_str) else {
            continue;
        };
        if url.is_empty() {
            continue;
        }
        let lang = track
            .get("languageCode")
            .and_then(Value::as_str)
            .unwrap_or("und")
            .to_string();
        let name = text_of(track.get("name").unwrap_or(&Value::Null));
        captions.push(YoutubeCaption {
            lang,
            name,
            url: url.to_string(),
        });
    }
    captions
}

pub fn parse_view_count(text: &str) -> i64 {
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    digits.parse().unwrap_or(0)
}

pub fn parse_published_at(text: &str) -> Option<i64> {
    let text = text.trim();
    if let Ok(date) = chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d") {
        return Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp());
    }
    let parts: Vec<&str> = text
        .split_whitespace()
        .map(|part| part.trim_matches(','))
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() == 3 {
        let month = month_number(parts[0])?;
        let day: u32 = parts[1].parse().ok()?;
        let year: i32 = parts[2].parse().ok()?;
        let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
        return Some(date.and_hms_opt(0, 0, 0)?.and_utc().timestamp());
    }
    None
}

fn month_number(name: &str) -> Option<u32> {
    Some(match name {
        "Jan" | "January" => 1,
        "Feb" | "February" => 2,
        "Mar" | "March" => 3,
        "Apr" | "April" => 4,
        "May" => 5,
        "Jun" | "June" => 6,
        "Jul" | "July" => 7,
        "Aug" | "August" => 8,
        "Sep" | "Sept" | "September" => 9,
        "Oct" | "October" => 10,
        "Nov" | "November" => 11,
        "Dec" | "December" => 12,
        _ => return None,
    })
}

pub fn fill_channel_fields(
    videos: &mut [YoutubeVideoItem],
    channel_id: &str,
    channel_name: &str,
    avatar: &str,
) {
    for (index, video) in videos.iter_mut().enumerate() {
        if video.channel_id.is_empty() {
            video.channel_id = channel_id.to_string();
        }
        if video.channel_name.is_empty() {
            video.channel_name = channel_name.to_string();
        }
        if video.channel_avatar.is_empty() {
            video.channel_avatar = avatar.to_string();
        }
        if video.order <= 0 {
            video.order = i64::try_from(index + 1).unwrap_or(i64::MAX);
        }
    }
}

pub fn playlist_thumbnail(videos: &[YoutubeVideoItem]) -> String {
    videos
        .iter()
        .find_map(|video| {
            if video.thumbnail.is_empty() {
                None
            } else {
                Some(video.thumbnail.clone())
            }
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        YoutubeTarget, classify_youtube_input, duration_from_clock, parse_published_at,
        parse_view_count,
    };

    #[test]
    fn classifies_video_channel_and_playlist_urls() {
        assert_eq!(
            classify_youtube_input("https://www.youtube.com/watch?v=jNQXAC9IVRw").unwrap(),
            YoutubeTarget::Video("jNQXAC9IVRw".to_string())
        );
        assert_eq!(
            classify_youtube_input("https://youtu.be/jNQXAC9IVRw?t=3").unwrap(),
            YoutubeTarget::Video("jNQXAC9IVRw".to_string())
        );
        assert_eq!(
            classify_youtube_input("https://www.youtube.com/shorts/jNQXAC9IVRw").unwrap(),
            YoutubeTarget::Video("jNQXAC9IVRw".to_string())
        );
        assert_eq!(
            classify_youtube_input(
                "https://www.youtube.com/watch?v=jNQXAC9IVRw&list=PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI"
            )
            .unwrap(),
            YoutubeTarget::Video("jNQXAC9IVRw".to_string())
        );
        assert_eq!(
            classify_youtube_input(
                "https://www.youtube.com/playlist?list=PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI"
            )
            .unwrap(),
            YoutubeTarget::Playlist("PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI".to_string())
        );
        assert_eq!(
            classify_youtube_input("https://www.youtube.com/@jawed/videos").unwrap(),
            YoutubeTarget::Channel("https://www.youtube.com/@jawed/videos".to_string())
        );
        assert_eq!(
            classify_youtube_input("https://www.youtube.com/channel/UC4QobU6STFB0P71PMvOGN5A")
                .unwrap(),
            YoutubeTarget::Channel("UC4QobU6STFB0P71PMvOGN5A".to_string())
        );
        assert_eq!(
            classify_youtube_input("jNQXAC9IVRw").unwrap(),
            YoutubeTarget::Video("jNQXAC9IVRw".to_string())
        );
    }

    #[test]
    fn parses_clock_duration_and_dates() {
        assert_eq!(duration_from_clock("0:19"), Some(19));
        assert_eq!(duration_from_clock("3:55"), Some(235));
        assert_eq!(duration_from_clock("1:02:03"), Some(3723));
        assert_eq!(parse_view_count("445,171,614 views"), 445_171_614);
        assert_eq!(parse_published_at("2005-04-23"), Some(1_114_214_400));
        assert_eq!(parse_published_at("Apr 23, 2005"), Some(1_114_214_400));
        assert_eq!(parse_published_at("2 days ago"), None);
    }
}
