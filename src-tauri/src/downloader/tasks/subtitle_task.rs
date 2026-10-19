use std::sync::Arc;

use eyre::WrapErr;
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::instrument;

use crate::{
    downloader::{
        download_progress::DownloadProgress, download_task::DownloadTask, episode_type::EpisodeType,
    },
    extensions::{AppHandleExt, GetOrInitPlayerInfo},
    types::player_info::PlayerInfo,
    utils,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct SubtitleTask {
    pub selected: bool,
    pub completed: bool,
}

impl SubtitleTask {
    pub fn mark_uncompleted(&mut self) {
        self.completed = false;
    }

    pub fn is_completed(&self) -> bool {
        !self.selected || self.completed
    }

    #[instrument(level = "error", skip_all)]
    pub async fn process(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
        player_info: &mut Option<PlayerInfo>,
    ) -> eyre::Result<()> {
        use std::fmt::Write;

        if progress.episode_type == EpisodeType::Youtube {
            return self.process_youtube(download_task, progress).await;
        }

        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);

        let player_info = player_info
            .get_or_init(&download_task.app, progress)
            .await?;

        let bili_client = download_task.app.get_bili_client();

        for subtitle_detail in &player_info.subtitle.subtitles {
            let url = format!("http:{}", subtitle_detail.subtitle_url);
            let subtitle = bili_client
                .get_subtitle(&url)
                .await
                .wrap_err("获取字幕失败")?;

            let mut srt_content = String::new();
            for (i, b) in subtitle.body.iter().enumerate() {
                let index = i + 1;
                let content = &b.content;
                let start_time = utils::seconds_to_srt_time(b.from);
                let end_time = utils::seconds_to_srt_time(b.to);
                let _ = writeln!(
                    &mut srt_content,
                    "{index}\n{start_time} --> {end_time}\n{content}\n"
                );
            }

            let lan = utils::filename_filter(&subtitle_detail.lan);
            let save_path = episode_dir.join(format!("{filename}.{lan}.srt"));
            std::fs::write(save_path, srt_content)?;
        }

        download_task.update_progress(|p| p.subtitle_task.completed = true);

        Ok(())
    }

    #[instrument(level = "error", skip_all)]
    async fn process_youtube(
        &self,
        download_task: &Arc<DownloadTask>,
        progress: &DownloadProgress,
    ) -> eyre::Result<()> {
        let (episode_dir, filename) = (&progress.episode_dir, &progress.filename);
        let video_id = progress
            .youtube_video_id
            .as_deref()
            .ok_or_else(|| eyre::eyre!("youtube_video_id为None"))?;
        let youtube_client = download_task.app.get_youtube_client();
        let info = youtube_client
            .get_video(video_id)
            .await
            .wrap_err("获取 YouTube 字幕信息失败")?;

        for caption in &info.captions {
            let json = youtube_client
                .get_caption_json(&caption.url)
                .await
                .wrap_err("下载 YouTube 字幕失败")?;
            let srt_content = caption_json_to_srt(&json);
            if srt_content.is_empty() {
                continue;
            }
            let lan = utils::filename_filter(if caption.lang.is_empty() {
                "und"
            } else {
                &caption.lang
            });
            let save_path = episode_dir.join(format!("{filename}.{lan}.srt"));
            std::fs::write(&save_path, srt_content)
                .wrap_err(format!("保存 YouTube 字幕到`{}`失败", save_path.display()))?;
        }

        download_task.update_progress(|p| p.subtitle_task.completed = true);
        Ok(())
    }
}

fn caption_json_to_srt(value: &serde_json::Value) -> String {
    use std::fmt::Write;

    let Some(events) = value.get("events").and_then(serde_json::Value::as_array) else {
        return String::new();
    };
    let mut srt_content = String::new();
    let mut index = 1;
    for event in events {
        let start_ms = event
            .get("tStartMs")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let duration_ms = event
            .get("dDurationMs")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let text = event
            .get("segs")
            .and_then(serde_json::Value::as_array)
            .map(|segs| {
                segs.iter()
                    .filter_map(|seg| seg.get("utf8").and_then(serde_json::Value::as_str))
                    .collect::<String>()
            })
            .unwrap_or_default()
            .replace('\n', " ");
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let start = utils::seconds_to_srt_time(start_ms as f64 / 1000.0);
        let end = utils::seconds_to_srt_time((start_ms + duration_ms.max(1)) as f64 / 1000.0);
        let _ = writeln!(&mut srt_content, "{index}\n{start} --> {end}\n{text}\n");
        index += 1;
    }
    srt_content
}
