mod parse;

use std::time::{Duration, Instant};

use eyre::{WrapErr, eyre};
use parking_lot::RwLock;
use reqwest::Client;
use serde_json::{Value, json};
use tauri::AppHandle;
use tracing::instrument;

use crate::{
    config::ProxyMode,
    extensions::{AppHandleExt, EyreReportToMessage},
    types::{
        audio_quality::AudioQuality,
        codec_type::CodecType,
        video_quality::VideoQuality,
        youtube::{YoutubeChannelInfo, YoutubePlaylistInfo, YoutubeSearchResult, YoutubeVideoInfo},
    },
};

use parse::{
    YoutubeTarget, channel_metadata, classify_youtube_input, continuation_token,
    extract_assigned_json, fill_channel_fields, parse_lockup_videos, parse_playlist_videos,
    parse_video_info, playlist_header, playlist_thumbnail,
};

const WEB_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
const VIDEOS_TAB_PARAMS: &str = "EgZ2aWRlb3PyBgQKAjoA";
const FALLBACK_CLIENT_VERSION: &str = "2.20261016.01.00";
const FALLBACK_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

#[derive(Clone)]
struct Session {
    visitor_data: String,
    client_version: String,
    api_key: String,
}

struct CachedVideo {
    fetched_at: Instant,
    info: YoutubeVideoInfo,
}

pub struct YoutubeClient {
    app: AppHandle,
    api_client: RwLock<Client>,
    session: RwLock<Option<Session>>,
    video_cache: RwLock<Vec<CachedVideo>>,
}

struct PlayerAttempt {
    name: &'static str,
    version: &'static str,
    client_id: &'static str,
    user_agent: &'static str,
    client: Value,
}

#[derive(Clone)]
struct RawFormat {
    url: String,
    mime: String,
    height: u64,
    fps: u64,
    bitrate: u64,
    content_length: u64,
    includes_audio: bool,
    video_quality: VideoQuality,
    codec_type: CodecType,
    audio_quality: AudioQuality,
}

pub struct YoutubeStreams {
    pub video: Option<YoutubeMedia>,
    pub audio: Option<YoutubeMedia>,
    pub duration: u64,
}

pub struct YoutubeMedia {
    pub url: String,
    pub content_length: u64,
    pub video_quality: VideoQuality,
    pub codec_type: CodecType,
    pub audio_quality: AudioQuality,
    pub includes_audio: bool,
}

impl YoutubeClient {
    pub fn new(app: AppHandle) -> Self {
        let api_client = RwLock::new(build_client(&app));
        Self {
            app,
            api_client,
            session: RwLock::new(None),
            video_cache: RwLock::new(Vec::new()),
        }
    }

    pub fn reload_client(&self) {
        *self.api_client.write() = build_client(&self.app);
        *self.session.write() = None;
        self.video_cache.write().clear();
    }

    #[instrument(level = "error", skip(self))]
    pub async fn search(&self, input: &str) -> eyre::Result<YoutubeSearchResult> {
        match classify_youtube_input(input)? {
            YoutubeTarget::Video(video_id) => {
                Ok(YoutubeSearchResult::Video(self.get_video(&video_id).await?))
            }
            YoutubeTarget::Channel(channel) => Ok(YoutubeSearchResult::Channel(
                self.get_channel(&channel, None).await?,
            )),
            YoutubeTarget::Playlist(playlist_id) => Ok(YoutubeSearchResult::Playlist(
                self.get_playlist(&playlist_id, None).await?,
            )),
        }
    }

    #[instrument(level = "error", skip(self))]
    pub async fn get_video(&self, video_id: &str) -> eyre::Result<YoutubeVideoInfo> {
        if let Some(info) = self.cached_video(video_id) {
            return Ok(info);
        }

        let next = self
            .post_json(
                "next",
                json!({
                    "videoId": video_id,
                }),
            )
            .await
            .wrap_err("获取 YouTube 视频详情失败")?;
        let player = self.watch_player(video_id).await.ok();
        let info = parse_video_info(video_id, &next, player.as_ref())
            .ok_or_else(|| eyre!("没有解析到 YouTube 视频信息，请确认链接可以公开播放"))?;
        self.store_video(info.clone());
        Ok(info)
    }

    #[instrument(level = "error", skip(self, continuation))]
    pub async fn get_channel(
        &self,
        channel: &str,
        continuation: Option<&str>,
    ) -> eyre::Result<YoutubeChannelInfo> {
        let response = if let Some(token) = continuation {
            self.post_json("browse", json!({ "continuation": token }))
                .await
                .wrap_err("获取频道的下一页视频失败")?
        } else {
            let channel_id = self.resolve_channel_id(channel).await?;
            self.post_json(
                "browse",
                json!({
                    "browseId": channel_id,
                    "params": VIDEOS_TAB_PARAMS,
                }),
            )
            .await
            .wrap_err("获取频道视频失败")?
        };

        let (mut channel_id, title, description, avatar) = channel_metadata(&response);
        if channel_id.is_empty() && channel.starts_with("UC") {
            channel_id = channel.to_string();
        }
        let mut videos = parse_lockup_videos(&response);
        fill_channel_fields(&mut videos, &channel_id, &title, &avatar);
        if videos.is_empty() && continuation.is_none() {
            return Err(eyre!("这个频道主页里没有找到可下载的视频"));
        }

        Ok(YoutubeChannelInfo {
            channel_id,
            title,
            description,
            avatar,
            videos,
            continuation: continuation_token(&response),
        })
    }

    #[instrument(level = "error", skip(self, continuation))]
    pub async fn get_playlist(
        &self,
        playlist_id: &str,
        continuation: Option<&str>,
    ) -> eyre::Result<YoutubePlaylistInfo> {
        let videos_response = if let Some(token) = continuation {
            self.post_json("next", json!({ "continuation": token }))
                .await
                .wrap_err("获取播放列表的下一页失败")?
        } else {
            self.post_json("next", json!({ "playlistId": playlist_id }))
                .await
                .wrap_err("获取播放列表视频失败")?
        };
        let mut videos = parse_playlist_videos(&videos_response);
        if videos.is_empty() && continuation.is_none() {
            return Err(eyre!("这个播放列表里没有找到可下载的视频"));
        }

        let (title, author_name, author_id, author_avatar) = if continuation.is_none() {
            let header = self
                .post_json("browse", json!({ "browseId": format!("VL{playlist_id}") }))
                .await
                .unwrap_or(Value::Null);
            playlist_header(&header)
        } else {
            (String::new(), String::new(), String::new(), String::new())
        };
        for video in &mut videos {
            if video.channel_name.is_empty() {
                video.channel_name.clone_from(&author_name);
            }
            if video.channel_id.is_empty() {
                video.channel_id.clone_from(&author_id);
            }
            if video.channel_avatar.is_empty() {
                video.channel_avatar.clone_from(&author_avatar);
            }
        }

        Ok(YoutubePlaylistInfo {
            playlist_id: playlist_id.to_string(),
            thumbnail: playlist_thumbnail(&videos),
            title,
            author_name,
            author_id,
            author_avatar,
            videos,
            continuation: continuation_token(&videos_response),
        })
    }

    #[instrument(level = "error", skip(self))]
    pub async fn get_streams(&self, video_id: &str) -> eyre::Result<YoutubeStreams> {
        let config = self.app.get_config().read().clone();
        let mut fallback: Option<YoutubeStreams> = None;
        let mut last_reason = String::new();

        for attempt in player_attempts() {
            let player = match self.player(video_id, &attempt).await {
                Ok(player) => player,
                Err(err) => {
                    tracing::debug!("YouTube {} 播放器请求失败: {err}", attempt.name);
                    continue;
                }
            };
            let status = player
                .pointer("/playabilityStatus/status")
                .and_then(Value::as_str)
                .unwrap_or("");
            let reason = player
                .pointer("/playabilityStatus/reason")
                .and_then(Value::as_str)
                .unwrap_or("");
            if !reason.is_empty() {
                last_reason = reason.to_string();
            }

            let duration = player
                .pointer("/videoDetails/lengthSeconds")
                .and_then(Value::as_str)
                .and_then(|text| text.parse::<u64>().ok())
                .unwrap_or(0);
            let formats = raw_formats(&player);
            if formats.is_empty() {
                tracing::debug!("YouTube {} 没有返回视频流: {status} {reason}", attempt.name);
                continue;
            }

            let video = select_video(
                &formats,
                &config.video_quality_priority,
                &config.codec_type_priority,
            );
            let audio = select_audio(&formats, &config.audio_quality_priority);
            let streams = YoutubeStreams {
                video: video.map(raw_to_media),
                audio: audio.map(raw_to_media),
                duration,
            };
            if streams.video.is_some() && streams.audio.is_some() {
                return Ok(streams);
            }
            if fallback.is_none() && (streams.video.is_some() || streams.audio.is_some()) {
                fallback = Some(streams);
            }
        }

        if let Some(streams) = fallback {
            return Ok(streams);
        }

        if last_reason.contains("not a bot") || last_reason.contains("Sign in") {
            return Err(eyre!(
                "YouTube 拒绝提供视频流，需要确认当前网络不是机器人。请在设置中配置可用代理，或更换网络后重试"
            ));
        }
        if last_reason.is_empty() {
            Err(eyre!(
                "没有找到可合并的 MP4 视频流或 M4A 音频流。内置 FFmpeg 只能把这两种格式合并成 MP4"
            ))
        } else {
            Err(eyre!("YouTube 没有返回可下载的视频流: {last_reason}"))
        }
    }

    pub async fn get_caption_json(&self, url: &str) -> eyre::Result<Value> {
        let url = if url.contains("fmt=") {
            url.to_string()
        } else if url.contains('?') {
            format!("{url}&fmt=json3")
        } else {
            format!("{url}?fmt=json3")
        };
        let body = self
            .get_text(&url)
            .await
            .wrap_err("下载 YouTube 字幕失败")?;
        serde_json::from_str(&body).wrap_err("解析 YouTube 字幕失败")
    }

    fn cached_video(&self, video_id: &str) -> Option<YoutubeVideoInfo> {
        let cache = self.video_cache.read();
        cache.iter().find_map(|item| {
            if item.info.video_id == video_id
                && item.fetched_at.elapsed() < Duration::from_secs(600)
            {
                Some(item.info.clone())
            } else {
                None
            }
        })
    }

    fn store_video(&self, info: YoutubeVideoInfo) {
        let mut cache = self.video_cache.write();
        cache.retain(|item| {
            item.info.video_id != info.video_id
                && item.fetched_at.elapsed() < Duration::from_secs(600)
        });
        cache.push(CachedVideo {
            fetched_at: Instant::now(),
            info,
        });
    }

    async fn resolve_channel_id(&self, channel: &str) -> eyre::Result<String> {
        if channel.starts_with("UC") && !channel.contains('/') {
            return Ok(channel.to_string());
        }
        let url = if channel.starts_with("http://") || channel.starts_with("https://") {
            channel.to_string()
        } else if let Some(handle) = channel.strip_prefix('@') {
            format!("https://www.youtube.com/@{handle}")
        } else {
            format!("https://www.youtube.com/{channel}")
        };
        let response = self
            .post_json("navigation/resolve_url", json!({ "url": url }))
            .await
            .wrap_err("解析 YouTube 频道链接失败")?;
        response
            .pointer("/endpoint/browseEndpoint/browseId")
            .and_then(Value::as_str)
            .filter(|id| id.starts_with("UC"))
            .map(ToString::to_string)
            .ok_or_else(|| eyre!("没有从这个频道链接解析出频道 ID"))
    }

    async fn watch_player(&self, video_id: &str) -> eyre::Result<Value> {
        let html = self
            .get_text(&format!("https://www.youtube.com/watch?v={video_id}&hl=en"))
            .await
            .wrap_err("打开 YouTube 视频页面失败")?;
        extract_assigned_json(&html, "ytInitialPlayerResponse")
            .ok_or_else(|| eyre!("视频页面里没有播放器信息"))
    }

    async fn player(&self, video_id: &str, attempt: &PlayerAttempt) -> eyre::Result<Value> {
        let session = self.session().await?;
        let mut client = attempt.client.clone();
        if attempt.name == "WEB"
            && let Some(object) = client.as_object_mut()
        {
            object.insert(
                "clientVersion".to_string(),
                Value::String(session.client_version.clone()),
            );
            object.insert(
                "visitorData".to_string(),
                Value::String(session.visitor_data.clone()),
            );
        }
        let body = json!({
            "videoId": video_id,
            "context": { "client": client },
            "contentCheckOk": true,
            "racyCheckOk": true,
        });
        let url = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
        self.post_raw(
            url,
            body,
            attempt.client_id,
            attempt.version,
            attempt.user_agent,
        )
        .await
    }

    async fn post_json(&self, path: &str, mut body: Value) -> eyre::Result<Value> {
        let session = self.session().await?;
        let context = json!({
            "client": {
                "clientName": "WEB",
                "clientVersion": session.client_version,
                "hl": "en",
                "gl": "US",
                "visitorData": session.visitor_data,
            }
        });
        if let Some(object) = body.as_object_mut() {
            object.insert("context".to_string(), context);
        }
        let url = format!(
            "https://www.youtube.com/youtubei/v1/{path}?prettyPrint=false&key={}",
            session.api_key
        );
        self.post_raw(&url, body, "1", &session.client_version, WEB_USER_AGENT)
            .await
    }

    async fn post_raw(
        &self,
        url: &str,
        body: Value,
        client_id: &str,
        client_version: &str,
        user_agent: &str,
    ) -> eyre::Result<Value> {
        let client = self.api_client.read().clone();
        let response = client
            .post(url)
            .header("content-type", "application/json")
            .header("user-agent", user_agent)
            .header("x-youtube-client-name", client_id)
            .header("x-youtube-client-version", client_version)
            .header("origin", "https://www.youtube.com")
            .header("referer", "https://www.youtube.com/")
            .body(body.to_string())
            .send()
            .await
            .wrap_err("请求 YouTube 接口失败")?;
        let status = response.status();
        let text = response.text().await.wrap_err("读取 YouTube 响应失败")?;
        if !status.is_success() {
            return Err(eyre!("YouTube 接口返回 {status}: {}", truncate(&text)));
        }
        serde_json::from_str(&text).wrap_err("解析 YouTube 响应失败")
    }

    async fn get_text(&self, url: &str) -> eyre::Result<String> {
        let client = self.api_client.read().clone();
        let response = client
            .get(url)
            .header("user-agent", WEB_USER_AGENT)
            .header("accept-language", "en-US,en;q=0.9")
            .send()
            .await
            .wrap_err("请求 YouTube 页面失败")?;
        let status = response.status();
        let text = response.text().await.wrap_err("读取 YouTube 页面失败")?;
        if !status.is_success() {
            return Err(eyre!("YouTube 页面返回 {status}"));
        }
        Ok(text)
    }

    async fn session(&self) -> eyre::Result<Session> {
        if let Some(session) = self.session.read().clone() {
            return Ok(session);
        }
        let html = self
            .get_text("https://www.youtube.com")
            .await
            .wrap_err("连接 YouTube 失败")?;
        let visitor_data = capture(&html, "\"VISITOR_DATA\":\"", "\"").unwrap_or_default();
        let client_version = capture(&html, "\"INNERTUBE_CLIENT_VERSION\":\"", "\"")
            .unwrap_or(FALLBACK_CLIENT_VERSION);
        let api_key = capture(&html, "\"INNERTUBE_API_KEY\":\"", "\"").unwrap_or(FALLBACK_API_KEY);
        let session = Session {
            visitor_data: visitor_data.to_string(),
            client_version: client_version.to_string(),
            api_key: api_key.to_string(),
        };
        *self.session.write() = Some(session.clone());
        Ok(session)
    }
}

fn capture<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let index = text.find(start)?;
    let rest = &text[index + start.len()..];
    let end_index = rest.find(end)?;
    Some(&rest[..end_index])
}

fn truncate(text: &str) -> String {
    let mut chars = text.chars();
    let snippet: String = chars.by_ref().take(180).collect();
    if chars.next().is_some() {
        format!("{snippet}...")
    } else {
        snippet
    }
}

fn player_attempts() -> Vec<PlayerAttempt> {
    vec![
        PlayerAttempt {
            name: "ANDROID",
            version: "20.10.38",
            client_id: "3",
            user_agent: "com.google.android.youtube/20.10.38 (Linux; U; Android 11) gzip",
            client: json!({
                "clientName": "ANDROID",
                "clientVersion": "20.10.38",
                "androidSdkVersion": 30,
                "hl": "en",
                "gl": "US",
            }),
        },
        PlayerAttempt {
            name: "IOS",
            version: "20.10.4",
            client_id: "5",
            user_agent: "com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X)",
            client: json!({
                "clientName": "IOS",
                "clientVersion": "20.10.4",
                "deviceMake": "Apple",
                "deviceModel": "iPhone16,2",
                "osName": "iOS",
                "osVersion": "18.3.2.22D82",
                "hl": "en",
                "gl": "US",
            }),
        },
        PlayerAttempt {
            name: "ANDROID_VR",
            version: "1.62.27",
            client_id: "28",
            user_agent: "com.google.android.apps.youtube.vr.oculus/1.62.27 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip",
            client: json!({
                "clientName": "ANDROID_VR",
                "clientVersion": "1.62.27",
                "deviceMake": "Oculus",
                "deviceModel": "Quest 3",
                "androidSdkVersion": 32,
                "hl": "en",
                "gl": "US",
            }),
        },
        PlayerAttempt {
            name: "WEB",
            version: FALLBACK_CLIENT_VERSION,
            client_id: "1",
            user_agent: WEB_USER_AGENT,
            client: json!({
                "clientName": "WEB",
                "clientVersion": FALLBACK_CLIENT_VERSION,
                "hl": "en",
                "gl": "US",
            }),
        },
    ]
}

fn raw_formats(player: &Value) -> Vec<RawFormat> {
    let mut formats = Vec::new();
    for key in ["formats", "adaptiveFormats"] {
        let Some(items) = player
            .pointer(&format!("/streamingData/{key}"))
            .and_then(Value::as_array)
        else {
            continue;
        };
        for item in items {
            if let Some(format) = raw_format(item) {
                formats.push(format);
            }
        }
    }
    formats
}

fn raw_format(value: &Value) -> Option<RawFormat> {
    let url = value.get("url").and_then(Value::as_str)?;
    if !url.starts_with("https://") {
        return None;
    }
    let mime = value
        .get("mimeType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let base_mime = mime.split(';').next().unwrap_or("").trim();
    if base_mime != "video/mp4" && base_mime != "audio/mp4" {
        return None;
    }
    let height = value.get("height").and_then(Value::as_u64).unwrap_or(0);
    let fps = value.get("fps").and_then(Value::as_u64).unwrap_or(0);
    let bitrate = value
        .get("bitrate")
        .or_else(|| value.get("averageBitrate"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let content_length = value
        .get("contentLength")
        .and_then(Value::as_str)
        .and_then(|text| text.parse().ok())
        .unwrap_or(0);
    let includes_audio = base_mime == "audio/mp4"
        || value.get("audioQuality").is_some()
        || value.get("audioSampleRate").is_some();
    let is_video = base_mime == "video/mp4" && height > 0;
    if !is_video && base_mime != "audio/mp4" {
        return None;
    }
    Some(RawFormat {
        url: url.to_string(),
        mime: mime.clone(),
        height,
        fps,
        bitrate,
        content_length,
        includes_audio: is_video && includes_audio,
        video_quality: video_quality_of(height, fps),
        codec_type: codec_of(&mime),
        audio_quality: audio_quality_of(bitrate),
    })
}

fn video_quality_of(height: u64, fps: u64) -> VideoQuality {
    if height >= 4320 {
        VideoQuality::Video8K
    } else if height >= 2160 {
        VideoQuality::Video4K
    } else if height >= 1440 {
        VideoQuality::Video1080PPlus
    } else if height >= 1080 {
        if fps >= 50 {
            VideoQuality::Video1080P60
        } else {
            VideoQuality::Video1080P
        }
    } else if height >= 720 {
        if fps >= 50 {
            VideoQuality::Video720P60
        } else {
            VideoQuality::Video720P
        }
    } else if height >= 480 {
        VideoQuality::Video480P
    } else if height >= 360 {
        VideoQuality::Video360P
    } else if height > 0 {
        VideoQuality::Video240P
    } else {
        VideoQuality::Unknown
    }
}

fn codec_of(mime: &str) -> CodecType {
    let lower = mime.to_ascii_lowercase();
    if lower.contains("avc1") {
        CodecType::AVC
    } else if lower.contains("hev1") || lower.contains("hvc1") {
        CodecType::HEVC
    } else if lower.contains("av01") {
        CodecType::AV1
    } else if lower.contains("mp4a") {
        CodecType::Audio
    } else {
        CodecType::Unknown
    }
}

fn audio_quality_of(bitrate: u64) -> AudioQuality {
    if bitrate >= 190_000 {
        AudioQuality::Audio192K
    } else if bitrate >= 100_000 {
        AudioQuality::Audio132K
    } else if bitrate > 0 {
        AudioQuality::Audio64K
    } else {
        AudioQuality::Unknown
    }
}

fn select_video(
    formats: &[RawFormat],
    quality_priority: &[VideoQuality],
    codec_priority: &[CodecType],
) -> Option<RawFormat> {
    let videos: Vec<&RawFormat> = formats
        .iter()
        .filter(|format| format.video_quality != VideoQuality::Unknown)
        .collect();
    let adaptive: Vec<&RawFormat> = videos
        .iter()
        .copied()
        .filter(|format| !format.includes_audio)
        .collect();
    let pool: Vec<&RawFormat> = if adaptive.is_empty() {
        videos
    } else {
        adaptive
    };

    for quality in quality_priority {
        for codec in codec_priority {
            if let Some(format) = pool
                .iter()
                .copied()
                .find(|format| format.video_quality == *quality && format.codec_type == *codec)
            {
                return Some(format.clone());
            }
        }
    }
    pool.into_iter()
        .max_by_key(|format| (format.height, format.fps, format.bitrate))
        .cloned()
}

fn select_audio(formats: &[RawFormat], quality_priority: &[AudioQuality]) -> Option<RawFormat> {
    let audios: Vec<&RawFormat> = formats
        .iter()
        .filter(|format| format.mime.starts_with("audio/mp4"))
        .collect();
    for quality in quality_priority {
        if let Some(format) = audios
            .iter()
            .copied()
            .find(|format| format.audio_quality == *quality)
        {
            return Some(format.clone());
        }
    }
    audios
        .into_iter()
        .max_by_key(|format| format.bitrate)
        .cloned()
}

fn raw_to_media(format: RawFormat) -> YoutubeMedia {
    YoutubeMedia {
        url: format.url,
        content_length: format.content_length,
        video_quality: format.video_quality,
        codec_type: format.codec_type,
        audio_quality: format.audio_quality,
        includes_audio: format.includes_audio,
    }
}

pub fn parse_published_text(text: &str) -> i64 {
    parse::parse_published_at(text).unwrap_or(0)
}

pub fn youtube_media_headers(url: &str) -> Option<(&'static str, &'static str)> {
    if !(url.contains("googlevideo.com") || url.contains("youtube.com")) {
        return None;
    }
    let user_agent = if url.contains("c=ANDROID") {
        "com.google.android.youtube/20.10.38 (Linux; U; Android 11) gzip"
    } else if url.contains("c=IOS") {
        "com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X)"
    } else {
        WEB_USER_AGENT
    };
    Some(("https://www.youtube.com/", user_agent))
}

fn build_client(app: &AppHandle) -> Client {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(WEB_USER_AGENT);
    let proxy_mode = app.get_config().read().proxy_mode;
    builder = match proxy_mode {
        ProxyMode::NoProxy => builder.no_proxy(),
        ProxyMode::System => builder,
        ProxyMode::Custom => {
            let config = app.get_config().read().clone();
            let proxy_url = format!("http://{}:{}", config.proxy_host, config.proxy_port);
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => builder.proxy(proxy),
                Err(err) => {
                    let err_title = format!("YouTube 客户端将`{proxy_url}`设为代理失败，将直连");
                    let message = eyre!(err).to_message();
                    tracing::error!(err_title, message);
                    builder.no_proxy()
                }
            }
        }
    };
    builder.build().expect("创建 YouTube HTTP 客户端失败")
}

#[cfg(test)]
mod tests {
    use super::{
        RawFormat, audio_quality_of, codec_of, select_audio, select_video, video_quality_of,
    };
    use crate::types::{
        audio_quality::AudioQuality, codec_type::CodecType, video_quality::VideoQuality,
    };

    fn format(mime: &str, height: u64, fps: u64, bitrate: u64, includes_audio: bool) -> RawFormat {
        RawFormat {
            url: format!("https://example.test/{height}-{bitrate}"),
            mime: mime.to_string(),
            height,
            fps,
            bitrate,
            content_length: 10,
            includes_audio,
            video_quality: video_quality_of(height, fps),
            codec_type: codec_of(mime),
            audio_quality: audio_quality_of(bitrate),
        }
    }

    #[test]
    fn prefers_priority_mp4_and_m4a() {
        let formats = vec![
            format(
                "video/mp4; codecs=\"avc1.640028\"",
                1080,
                30,
                4_000_000,
                false,
            ),
            format(
                "video/mp4; codecs=\"av01.0.08M.08\"",
                1080,
                30,
                2_000_000,
                false,
            ),
            format(
                "video/mp4; codecs=\"avc1.4d401f, mp4a.40.2\"",
                720,
                30,
                1_000_000,
                true,
            ),
            format("audio/mp4; codecs=\"mp4a.40.2\"", 0, 0, 128_000, false),
            format("audio/mp4; codecs=\"mp4a.40.2\"", 0, 0, 48_000, false),
        ];
        let video = select_video(
            &formats,
            &[VideoQuality::Video1080P, VideoQuality::Video720P],
            &[CodecType::AVC, CodecType::AV1],
        )
        .unwrap();
        assert_eq!(video.height, 1080);
        assert_eq!(video.codec_type, CodecType::AVC);
        assert!(!video.includes_audio);

        let audio =
            select_audio(&formats, &[AudioQuality::Audio132K, AudioQuality::Audio64K]).unwrap();
        assert_eq!(audio.bitrate, 128_000);
    }

    #[tokio::test]
    #[ignore = "requests the live YouTube website"]
    async fn parses_live_channel_playlist_and_video() {
        use super::{
            FALLBACK_API_KEY, FALLBACK_CLIENT_VERSION, VIDEOS_TAB_PARAMS, WEB_USER_AGENT, capture,
        };
        use crate::youtube::parse::{parse_lockup_videos, parse_playlist_videos, parse_video_info};
        use serde_json::{Value, json};

        let client = reqwest::Client::builder()
            .user_agent(WEB_USER_AGENT)
            .build()
            .expect("client");
        let html = client
            .get("https://www.youtube.com")
            .send()
            .await
            .expect("homepage")
            .text()
            .await
            .expect("homepage text");
        let api_key = capture(&html, "\"INNERTUBE_API_KEY\":\"", "\"").unwrap_or(FALLBACK_API_KEY);
        let client_version = capture(&html, "\"INNERTUBE_CLIENT_VERSION\":\"", "\"")
            .unwrap_or(FALLBACK_CLIENT_VERSION);
        let visitor = capture(&html, "\"VISITOR_DATA\":\"", "\"").unwrap_or("");

        let post = |path: &'static str, mut body: Value| {
            let client = client.clone();
            let api_key = api_key.to_string();
            let client_version = client_version.to_string();
            let visitor = visitor.to_string();
            async move {
                let context = json!({
                    "client": {
                        "clientName": "WEB",
                        "clientVersion": client_version,
                        "hl": "en",
                        "gl": "US",
                        "visitorData": visitor,
                    }
                });
                body.as_object_mut()
                    .expect("object")
                    .insert("context".to_string(), context);
                let url = format!(
                    "https://www.youtube.com/youtubei/v1/{path}?prettyPrint=false&key={api_key}"
                );
                let text = client
                    .post(url)
                    .header("content-type", "application/json")
                    .header("user-agent", WEB_USER_AGENT)
                    .body(body.to_string())
                    .send()
                    .await
                    .expect("post")
                    .text()
                    .await
                    .expect("body");
                serde_json::from_str::<Value>(&text).expect("json")
            }
        };

        let channel = post(
            "browse",
            json!({
                "browseId": "UC4QobU6STFB0P71PMvOGN5A",
                "params": VIDEOS_TAB_PARAMS,
            }),
        )
        .await;
        let videos = parse_lockup_videos(&channel);
        assert!(
            videos.iter().any(|video| video.video_id == "jNQXAC9IVRw"),
            "channel videos: {videos:?}"
        );

        let playlist = post(
            "next",
            json!({ "playlistId": "PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI" }),
        )
        .await;
        let playlist_videos = parse_playlist_videos(&playlist);
        assert!(
            playlist_videos.len() > 1,
            "playlist videos: {playlist_videos:?}"
        );

        let next = post("next", json!({ "videoId": "jNQXAC9IVRw" })).await;
        let info = parse_video_info("jNQXAC9IVRw", &next, None).expect("video info");
        assert!(!info.title.is_empty(), "video info: {info:?}");
        assert_eq!(info.video_id, "jNQXAC9IVRw");
    }
}
