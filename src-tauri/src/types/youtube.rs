use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct GetYoutubeParams {
    pub input: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct GetYoutubeChannelParams {
    pub channel_id: String,
    pub continuation: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct GetYoutubePlaylistParams {
    pub playlist_id: String,
    pub continuation: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YoutubeVideoItem {
    pub video_id: String,
    pub title: String,
    pub thumbnail: String,
    pub duration: u64,
    pub published_text: String,
    pub view_text: String,
    pub channel_name: String,
    pub channel_id: String,
    pub channel_avatar: String,
    pub order: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YoutubeChapter {
    pub title: String,
    pub start_sec: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YoutubeCaption {
    pub lang: String,
    pub name: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YoutubeVideoInfo {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub channel_id: String,
    pub channel_name: String,
    pub channel_avatar: String,
    pub thumbnail: String,
    pub duration: u64,
    pub published_at: i64,
    pub published_text: String,
    pub view_count: i64,
    pub view_text: String,
    pub tags: Vec<String>,
    pub chapters: Vec<YoutubeChapter>,
    pub captions: Vec<YoutubeCaption>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YoutubeChannelInfo {
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub avatar: String,
    pub videos: Vec<YoutubeVideoItem>,
    pub continuation: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct YoutubePlaylistInfo {
    pub playlist_id: String,
    pub title: String,
    pub author_name: String,
    pub author_id: String,
    pub author_avatar: String,
    pub thumbnail: String,
    pub videos: Vec<YoutubeVideoItem>,
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum YoutubeSearchResult {
    Video(YoutubeVideoInfo),
    Channel(YoutubeChannelInfo),
    Playlist(YoutubePlaylistInfo),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct CreateYoutubeDownloadTaskParams {
    pub collection_title: String,
    pub videos: Vec<YoutubeVideoItem>,
}
