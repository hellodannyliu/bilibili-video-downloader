<p align="center">
    <img src="https://github.com/user-attachments/assets/a66896c7-33a6-463e-81fe-bacca3223191" style="align-self: center"/>
</p>

# 📺 哔哩哔哩 & YouTube 视频下载器 | Bilibili & YouTube Video Downloader

[中文](#中文) · [English](#english)

<a id="中文"></a>

## 中文

哔哩哔哩 bilibili B站 视频下载器，也是 YouTube 油管下载器。普通视频、充电视频、番剧、电视剧、电影、课程全都支持下载；YouTube 单视频、作者频道主页和播放列表也同样支持。图形界面 + MP4 合并 + nfo 刮削 + 广告标记 + 字幕下载 + 弹幕下载，轻松将视频加入 emby 等媒体库。

想找 YouTube 下载器、YouTube 视频下载、YouTube 频道下载、YouTube 播放列表下载，或把油管视频保存成 MP4，可以用这个桌面程序完成。

## 📥 快速下载

[v0.2.2 Release](https://github.com/hellodannyliu/bilibili-video-downloader/releases/tag/v0.2.2) 提供已经编译好的 macOS 与 Linux 安装包，直接下载即可使用。macOS 包含 Apple 芯片和 Intel 两个版本。这一版包含哔哩哔哩下载，以及 YouTube 视频、频道和播放列表下载。

| 系统                          | 下载                                                                                                                                                                                                           |
| :---------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS .dmg（Apple 芯片）      | [bilibili-video-downloader_0.2.2_macos_aarch64.dmg](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_macos_aarch64.dmg)                     |
| macOS .dmg（Intel）           | [bilibili-video-downloader_0.2.2_macos_x86_64.dmg](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_macos_x86_64.dmg)                       |
| Linux .deb（Debian / Ubuntu） | [bilibili-video-downloader_0.2.2_linux_amd64.deb](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_linux_amd64.deb)                         |
| Linux .rpm                    | [bilibili-video-downloader_0.2.2_linux_amd64.rpm](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_linux_amd64.rpm)                         |
| Linux 便携版                  | [bilibili-video-downloader_0.2.2_linux_amd64_portable.tar.gz](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_linux_amd64_portable.tar.gz) |

安装包没有经过 Apple 公证。第一次打开时如果系统提示无法验证开发者，请到「系统设置 → 隐私与安全性」里选择仍要打开。

**如果本项目对你有帮助，欢迎点个 Star ⭐ 支持！你的支持是我持续更新维护的动力 🙏**

## 🖥️图形界面

![](https://github.com/user-attachments/assets/73d4a9d7-644b-43f2-9b25-66212e8fd9a8)

## ✨ 主要特性

| 特性         | 说明                                                                                                                                                                         |
| :----------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🖥️图形界面   | 基于 [Tauri v2](https://v2.tauri.app/start/)，跨平台、现代、轻量、简洁、易用                                                                                                 |
| ⚡分片下载   | 最大化下载速度，轻松榨干带宽(如果你想的话)，当然也支持限速                                                                                                                   |
| 📁自定义命名 | `bv号` `标题` `分P` `发布时间` `UP昵称`等...自由组合成你喜欢的目录结构与文件命名规则                                                                                         |
| 🔍视频搜索   | `视频(av/bv)` `番剧(ep/ss)` `课程(ep/ss)` `UP投稿/个人空间(uid)` `收藏夹(fid)` `YouTube 视频/频道/播放列表`                                                                  |
| 👤账号相关   | **登录** `Cookie登录` `二维码登录`<br />**内容** `收藏夹` `历史记录` `稍后再看` `追番追剧`                                                                                   |
| 🎬视频下载   | **类型** `视频(合集、分P、充电视频)` `番剧(正片、PV、相关视频)` `课程` `YouTube`<br />**编码** `AVC` `HEVC` `AV1`<br />**画质** `8K` `杜比视界` `HDR` `4K` `AI智能修复`等... |
| 🎵音频下载   | `无损` `杜比全景声` `192K` `132K` `64K`                                                                                                                                      |
| 📝字幕下载   | 获取视频所有的 CC 字幕，以 `srt` 格式保存。YouTube 字幕同样保存为 `srt`                                                                                                      |
| 🖼️封面下载   | 最高清、无压缩的原始封面图                                                                                                                                                   |
| 💬弹幕下载   | 不仅能下载 `xml` `json` 格式的原始弹幕，还支持将其转为样式可定制的 `ass字幕`。弹幕只来自哔哩哔哩                                                                             |
| 📺NFO刮削    | 还会顺便下载 poster 和 fanart，轻松将视频加入 emby 等媒体库。YouTube 视频也会写 NFO                                                                                          |
| 🎞️章节标记   | 将原视频的章节信息嵌入视频文件，使视频在各类播放器中支持章节导航。YouTube 章节同样可以嵌入                                                                                   |
| 🚫广告标记   | 将广告片段以章节的形式嵌入视频文件，配合兼容的播放器可自动跳过广告                                                                                                           |
| ⚙️任务管理   | `断点续传` `批量操作` `继续` `暂停` `重来` `删除`                                                                                                                            |

## ▶️ YouTube 视频、频道和播放列表

搜索类型选择 **YouTube**，或在 **自动** 里直接粘贴链接：

| 想下载什么         | 可以粘贴                                                                                                    |
| :----------------- | :---------------------------------------------------------------------------------------------------------- |
| 单个视频           | `https://www.youtube.com/watch?v=...`、`https://youtu.be/...`、`https://www.youtube.com/shorts/...`         |
| 作者频道主页的视频 | `https://www.youtube.com/@频道名`、`https://www.youtube.com/channel/UC...`、`https://www.youtube.com/c/...` |
| 播放列表           | `https://www.youtube.com/playlist?list=...`                                                                 |

从一个 YouTube 视频进入作者后，可以查看该频道主页的视频；频道和播放列表都可以继续加载下一页，再勾选下载。下载使用和哔哩哔哩相同的分块流程：按画质、编码和音质优先级选择 MP4 视频与 M4A 音频，内置 FFmpeg 再把它们合并成一个 MP4，并保存封面、SRT 字幕、章节和 NFO。

内置 FFmpeg 只做 MP4 混流，不重新编码，因此只接收 `video/mp4` 和 `audio/mp4`。如果 YouTube 提示 “Sign in to confirm you’re not a bot”，在设置里配置可用代理后再下载。

## 📖 使用方法

这个视频是主要功能的演示

https://github.com/user-attachments/assets/adf84b93-684f-43f3-9948-6ba527213812

## 🔌插件系统（实验性）

- 后端提供进程内动态库插件系统，但非常不成熟
- 有特殊需求建议直接改源码，而不是开发插件
- 这个插件系统**没有做任何安全限制**，这是为了给插件最大的功能性与自由度
- 也正因如此，**任何第三方插件的安全性都无法保证**
- 强烈建议：只使用开源插件，并且自行审查代码后再编译使用
- 不要使用他人发的二进制插件(`dll` / `so` / `dylib`)
- 插件开发文档与示例请看：[src-plugin/examples](src-plugin/examples)

## ⚠️关于被杀毒软件误判为病毒

对于个人开发的项目来说，这个问题几乎是无解的(~~需要购买数字证书给软件签名，甚至给杀毒软件交保护费~~)  
我能想到的解决办法只有：

1. 根据下面的**如何构建(build)**，自行编译
2. 希望你相信我的承诺，我承诺你在[Release页面](https://github.com/hellodannyliu/bilibili-video-downloader/releases)下载到的所有东西都是安全的。切勿轻信他人分享的文件，请**仅**在[Release页面](https://github.com/hellodannyliu/bilibili-video-downloader/releases)下载。任何不是从该页面下载的版本均可能**已被篡改**并**真的包含病毒**(而非误报)，包括但不限于`网盘`、`通过邮箱或社交软件分享`、`issue或discussion里的文件`、`其他fork(仓库)`、`其他网站`

## 🛠️如何构建(build)

构建非常简单，一共就3条命令  
~~前提是你已经安装了Rust、Node、pnpm~~

#### 📋前提

- [Rust](https://www.rust-lang.org/tools/install)
- [Node](https://nodejs.org/en)
- [pnpm](https://pnpm.io/installation)

#### 📝步骤

#### 1. 克隆本仓库

```
git clone https://github.com/lanyeeee/bilibili-video-downloader.git
```

#### 2.安装依赖

```
cd bilibili-video-downloader
pnpm install
```

#### 3.构建(build)

```
pnpm tauri build
```

## 🤝提交PR

**PR请基于`develop`分支开发，并提交至`develop`分支**

**提交前请先：**

1. 运行`pnpm format`以保证代码格式正确
2. 运行`pnpm check`并确认无报错

**如果想新加一个功能，请先开个`issue`或`discussion`讨论一下，避免无效工作**

其他情况的PR欢迎直接提交，比如：

1. 🔧 对原有功能的改进
2. 🐛 修复BUG
3. ⚡ 使用更轻量的库实现原有功能
4. 📝 修订文档
5. ⬆️ 升级、更新依赖的PR也会被接受

## ⚠️免责声明

- 本工具仅作学习、研究、交流使用，使用本工具的用户应自行承担风险
- 作者不对使用本工具导致的任何损失、法律纠纷或其他后果负责
- 作者不对用户使用本工具的行为负责，包括但不限于用户违反法律或任何第三方权益的行为

## 🙏感谢

[bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect)

[ffmpeg](https://github.com/FFmpeg/FFmpeg)

[danmu2ass](https://github.com/gwy15/danmu2ass)

[BilibiliSponsorBlock](https://github.com/hanydd/BilibiliSponsorBlock)

## 💬其他

任何使用中遇到的问题、任何希望添加的功能，都欢迎提交`issue`或开`discussion`交流，我会尽力解决

---

<a id="english"></a>

## English

Bilibili video downloader and YouTube video downloader. Download normal videos, paid videos, bangumi, dramas, movies, and courses from Bilibili. Download a YouTube video, every public video on a YouTube channel, or a YouTube playlist. The desktop app merges video and audio into one MP4 and can also save NFO metadata, subtitles, danmaku, covers, and chapters for media servers such as Emby.

Search for a YouTube downloader, YouTube to MP4, YouTube channel downloader, or YouTube playlist downloader: paste the link, pick the videos, and download them with the same task list used for Bilibili.

## 📥 Download

[v0.2.2](https://github.com/hellodannyliu/bilibili-video-downloader/releases/tag/v0.2.2) has compiled macOS and Linux packages. macOS builds are provided for Apple silicon and Intel. This build downloads Bilibili videos and YouTube videos, channels, and playlists.

| System                       | Download                                                                                                                                                                                                       |
| :--------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS .dmg (Apple silicon)   | [bilibili-video-downloader_0.2.2_macos_aarch64.dmg](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_macos_aarch64.dmg)                     |
| macOS .dmg (Intel)           | [bilibili-video-downloader_0.2.2_macos_x86_64.dmg](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_macos_x86_64.dmg)                       |
| Linux .deb (Debian / Ubuntu) | [bilibili-video-downloader_0.2.2_linux_amd64.deb](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_linux_amd64.deb)                         |
| Linux .rpm                   | [bilibili-video-downloader_0.2.2_linux_amd64.rpm](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_linux_amd64.rpm)                         |
| Linux portable               | [bilibili-video-downloader_0.2.2_linux_amd64_portable.tar.gz](https://github.com/hellodannyliu/bilibili-video-downloader/releases/download/v0.2.2/bilibili-video-downloader_0.2.2_linux_amd64_portable.tar.gz) |

The packages are not notarized by Apple. If macOS says the developer cannot be verified, open System Settings → Privacy & Security and choose Open Anyway.

**If this project helps you, a Star ⭐ really helps it stay maintained.**

## 🖥️ Interface

![](https://github.com/user-attachments/assets/73d4a9d7-644b-43f2-9b25-66212e8fd9a8)

## ✨ Features

| Feature              | What it does                                                                                                                                                                                   |
| :------------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 🖥️ Desktop app       | Built with [Tauri v2](https://v2.tauri.app/start/). Cross-platform, small, and straightforward                                                                                                 |
| ⚡ Chunked download  | Uses range requests so a task can fill the connection. Speed limits are also available                                                                                                         |
| 📁 Names and folders | Combine `bvid`, title, part, publish time, uploader name, and more into your own path template                                                                                                 |
| 🔍 Search            | `video (av/bv)` `bangumi (ep/ss)` `course (ep/ss)` `uploader space (uid)` `favorites (fid)` `YouTube video / channel / playlist`                                                               |
| 👤 Account           | **Sign in** with a cookie or QR code. **Library** favorites, history, watch later, and followed bangumi                                                                                        |
| 🎬 Video             | **Sources** Bilibili videos (seasons, parts, paid), bangumi, courses, and YouTube. **Codecs** `AVC` `HEVC` `AV1`. **Quality** `8K`, Dolby Vision, `HDR`, `4K`, AI repair, and the usual ladder |
| 🎵 Audio             | Lossless, Dolby Atmos, `192K`, `132K`, `64K`                                                                                                                                                   |
| 📝 Subtitles         | Saves every CC subtitle as `srt`, including YouTube captions                                                                                                                                   |
| 🖼️ Cover             | Downloads the original cover                                                                                                                                                                   |
| 💬 Danmaku           | Saves raw `xml` and `json`, and can convert comments to styled `ass` subtitles. Danmaku comes from Bilibili                                                                                    |
| 📺 NFO               | Writes NFO plus poster and fanart for Emby and similar servers. YouTube videos get an NFO file too                                                                                             |
| 🎞️ Chapters          | Embeds chapter markers, including YouTube chapters, so players can jump between them                                                                                                           |
| 🚫 Sponsor segments  | Embeds sponsor ranges as chapters so a compatible player can skip them                                                                                                                         |
| ⚙️ Tasks             | Resume, batch actions, continue, pause, restart, and delete                                                                                                                                    |

## ▶️ YouTube videos, channels, and playlists

Choose **YouTube** in the search box, or paste a link while **Auto** is selected:

| What you want       | Links                                                                                                       |
| :------------------ | :---------------------------------------------------------------------------------------------------------- |
| One video           | `https://www.youtube.com/watch?v=...`, `https://youtu.be/...`, `https://www.youtube.com/shorts/...`         |
| Videos on a channel | `https://www.youtube.com/@handle`, `https://www.youtube.com/channel/UC...`, `https://www.youtube.com/c/...` |
| A playlist          | `https://www.youtube.com/playlist?list=...`                                                                 |

Open a video and jump to that creator’s channel to browse the channel homepage. Channel and playlist pages load more results, and the checked videos are queued together. YouTube uses the same chunked downloader as Bilibili. The app picks an MP4 video and an M4A audio stream from your quality order, then the bundled FFmpeg merges them into one MP4. Cover art, SRT subtitles, chapters, and NFO are saved beside it.

The bundled FFmpeg only muxes MP4. It does not re-encode, so the downloader keeps `video/mp4` and `audio/mp4`. If YouTube answers “Sign in to confirm you’re not a bot”, set a working proxy in Settings and try again.

## 📖 How to use

This clip walks through the main Bilibili workflow:

https://github.com/user-attachments/assets/adf84b93-684f-43f3-9948-6ba527213812

## 🔌 Plugins (experimental)

- The backend can load in-process native plugins. The system is still immature.
- For anything special, change the source instead of writing a plugin.
- Plugins are **not sandboxed**. That keeps them flexible.
- Because of that, **a third-party plugin is only as safe as its code**.
- Use open-source plugins, read them, and compile them yourself.
- Do not run binary plugins (`dll` / `so` / `dylib`) that someone else sends you.
- The sample and notes live in [src-plugin/examples](src-plugin/examples).

## ⚠️ Antivirus false positives

Unsigned personal apps are often flagged. A certificate, or paying a vendor, is the usual way out, and neither is realistic here.

1. Build it yourself with the steps below.
2. The files on the [releases page](https://github.com/hellodannyliu/bilibili-video-downloader/releases) are the ones published by this project. Download **only** from that page. Copies from cloud drives, email, chat apps, issues, discussions, other forks, or other websites may have been modified and may actually contain malware.

## 🛠️ Build

Three commands, after Rust, Node, and pnpm are installed.

#### 📋 Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Node](https://nodejs.org/en)
- [pnpm](https://pnpm.io/installation)

#### 📝 Steps

#### 1. Clone

```
git clone https://github.com/lanyeeee/bilibili-video-downloader.git
```

#### 2. Install dependencies

```
cd bilibili-video-downloader
pnpm install
```

#### 3. Build

```
pnpm tauri build
```

## 🤝 Pull requests

**Branch from `develop` and open the pull request against `develop`.**

**Before submitting:**

1. Run `pnpm format`.
2. Run `pnpm check` and fix anything it reports.

**Please open an issue or discussion before adding a new feature.**

These pull requests are welcome as they are:

1. 🔧 Improvements to existing behavior
2. 🐛 Bug fixes
3. ⚡ Replacing a dependency with a lighter one
4. 📝 Documentation
5. ⬆️ Dependency updates

## ⚠️ Disclaimer

- This tool is for study, research, and discussion. You use it at your own risk.
- The author is not responsible for loss, legal disputes, or other consequences of using it.
- The author is not responsible for how you use it, including any violation of law or of someone else’s rights.

## 🙏 Thanks

[bilibili-API-collect](https://github.com/SocialSisterYi/bilibili-API-collect)

[ffmpeg](https://github.com/FFmpeg/FFmpeg)

[danmu2ass](https://github.com/gwy15/danmu2ass)

[BilibiliSponsorBlock](https://github.com/hanydd/BilibiliSponsorBlock)

## 💬 Contact

Questions and feature requests are welcome in an `issue` or a `discussion`.
