use eyre::{OptionExt, WrapErr};
use tauri::AppHandle;
use tracing::instrument;

use crate::{
    downloader::{download_progress::DownloadProgress, episode_type::EpisodeType},
    extensions::AppHandleExt,
    types::{
        bangumi_info::BangumiInfo, cheese_info::CheeseInfo,
        get_bangumi_info_params::GetBangumiInfoParams, get_cheese_info_params::GetCheeseInfoParams,
        get_normal_info_params::GetNormalInfoParams, normal_info::NormalInfo,
        youtube::YoutubeVideoInfo,
    },
};

#[derive(Clone)]
pub enum EpisodeInfo {
    Normal(NormalInfo),
    Bangumi(BangumiInfo, i64),
    Cheese(CheeseInfo, i64),
    Youtube(YoutubeVideoInfo),
}

pub trait GetOrInitEpisodeInfo {
    async fn get_or_init<'a>(
        &'a mut self,
        app: &AppHandle,
        progress: &DownloadProgress,
    ) -> eyre::Result<&'a mut EpisodeInfo>;
}

impl GetOrInitEpisodeInfo for Option<EpisodeInfo> {
    #[instrument(level = "error", skip_all)]
    async fn get_or_init<'a>(
        &'a mut self,
        app: &AppHandle,
        progress: &DownloadProgress,
    ) -> eyre::Result<&'a mut EpisodeInfo> {
        if let Some(info) = self {
            return Ok(info);
        }

        let bili_client = app.get_bili_client();
        let (aid, ep_id, episode_type) = (progress.aid, progress.ep_id, progress.episode_type);

        let new_info = match episode_type {
            EpisodeType::Normal => {
                let info = bili_client
                    .get_normal_info(GetNormalInfoParams::Aid(aid))
                    .await
                    .wrap_err("获取普通视频信息失败")?;
                EpisodeInfo::Normal(info)
            }
            EpisodeType::Bangumi => {
                let ep_id = ep_id.ok_or_eyre("ep_id为None")?;
                let info = bili_client
                    .get_bangumi_info(GetBangumiInfoParams::EpId(ep_id))
                    .await
                    .wrap_err("获取番剧信息失败")?;
                EpisodeInfo::Bangumi(info, ep_id)
            }
            EpisodeType::Cheese => {
                let ep_id = ep_id.ok_or_eyre("ep_id为None")?;
                let info = bili_client
                    .get_cheese_info(GetCheeseInfoParams::EpId(ep_id))
                    .await
                    .wrap_err("获取课程信息失败")?;
                EpisodeInfo::Cheese(info, ep_id)
            }
            EpisodeType::Youtube => {
                let video_id = progress
                    .youtube_video_id
                    .clone()
                    .ok_or_eyre("youtube_video_id为None")?;
                let info = app
                    .get_youtube_client()
                    .get_video(&video_id)
                    .await
                    .wrap_err("获取 YouTube 视频信息失败")?;
                EpisodeInfo::Youtube(info)
            }
        };

        Ok(self.insert(new_info))
    }
}
