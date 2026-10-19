<script setup lang="ts">
import { computed, inject, ref, useTemplateRef, watch } from 'vue'
import { SelectionArea, PartialSelectionOptions } from '@viselect/vue'
import { NButton, NDropdown, useMessage } from 'naive-ui'
import { commands, YoutubeSearchResult, YoutubeVideoInfo, YoutubeVideoItem } from '../../../bindings.ts'
import { searchPaneRefKey } from '../../../injection_keys.ts'
import { mediaImage, useEpisodeDropdown, useKeyedSelection } from '../../../utils.tsx'
import YoutubeVideoCard from './YoutubeVideoCard.vue'

const result = defineModel<YoutubeSearchResult>('result', { required: true })

const searchPaneRef = inject(searchPaneRefKey)
const message = useMessage()
const loadingMore = ref(false)

const selectionOptions: PartialSelectionOptions = {
  selectables: '.selectable',
  features: { deselectOnBlur: true },
  boundaries: '.youtube-panel-selection-container',
}
const selectionAreaRef = useTemplateRef('selectionAreaRef')
const { selectedIds, updateSelectedIds, unselectAll } = useKeyedSelection()
const checkedIds = ref<Set<string>>(new Set())
const cardRefs = useTemplateRef('cardRefs')

const isList = computed(() => 'Channel' in result.value || 'Playlist' in result.value)

const videos = computed<YoutubeVideoItem[]>(() => {
  if ('Channel' in result.value) {
    return result.value.Channel.videos
  }
  if ('Playlist' in result.value) {
    return result.value.Playlist.videos
  }
  return [videoInfoToItem(result.value.Video)]
})

const continuation = computed<string | null>(() => {
  if ('Channel' in result.value) {
    return result.value.Channel.continuation
  }
  if ('Playlist' in result.value) {
    return result.value.Playlist.continuation
  }
  return null
})

const collectionTitle = computed(() => {
  if ('Channel' in result.value) {
    return result.value.Channel.title || result.value.Channel.videos[0]?.channel_name || ''
  }
  if ('Playlist' in result.value) {
    return result.value.Playlist.title || result.value.Playlist.videos[0]?.title || ''
  }
  return result.value.Video.channel_name || result.value.Video.title
})

const header = computed(() => {
  if ('Channel' in result.value) {
    const channel = result.value.Channel
    return {
      title: channel.title,
      description: channel.description,
      cover: channel.avatar || channel.videos[0]?.thumbnail || '',
      authorName: channel.title,
      authorAvatar: channel.avatar,
      authorId: channel.channel_id,
    }
  }
  if ('Playlist' in result.value) {
    const playlist = result.value.Playlist
    return {
      title: playlist.title,
      description: '',
      cover: playlist.thumbnail,
      authorName: playlist.author_name,
      authorAvatar: playlist.author_avatar,
      authorId: playlist.author_id,
    }
  }
  const video = result.value.Video
  return {
    title: video.title,
    description: video.description,
    cover: video.thumbnail,
    authorName: video.channel_name,
    authorAvatar: video.channel_avatar,
    authorId: video.channel_id,
  }
})

const sourceKey = computed(() => {
  if ('Channel' in result.value) {
    return `channel:${result.value.Channel.channel_id}`
  }
  if ('Playlist' in result.value) {
    return `playlist:${result.value.Playlist.playlist_id}`
  }
  return `video:${result.value.Video.video_id}`
})

watch(sourceKey, () => {
  selectedIds.value.clear()
  checkedIds.value.clear()
  selectionAreaRef.value?.selection?.clearSelection()
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
})

const cardRefsMap = computed(() => {
  const map = new Map<string, InstanceType<typeof YoutubeVideoCard>>()
  cardRefs.value?.forEach((card, index) => {
    if (card !== null) {
      map.set(String(index), card)
    }
  })
  return map
})

const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useEpisodeDropdown(
  () => {
    selectedIds.value.forEach((id) => checkedIds.value.add(id))
    dropdownShowing.value = false
  },
  () => {
    selectedIds.value.forEach((id) => checkedIds.value.delete(id))
    dropdownShowing.value = false
  },
  () => {
    videos.value.forEach((_, index) => selectedIds.value.add(String(index)))
    dropdownShowing.value = false
  },
)

function videoInfoToItem(info: YoutubeVideoInfo): YoutubeVideoItem {
  return {
    video_id: info.video_id,
    title: info.title,
    thumbnail: info.thumbnail,
    duration: info.duration,
    published_text: info.published_text,
    view_text: info.view_text,
    channel_name: info.channel_name,
    channel_id: info.channel_id,
    channel_avatar: info.channel_avatar,
    order: 1,
  }
}

function orderedItem(video: YoutubeVideoItem, index: number): YoutubeVideoItem {
  return { ...video, order: index + 1 }
}

async function downloadIndexes(indexes: number[]) {
  const items = indexes
    .filter((index) => videos.value[index] !== undefined)
    .map((index) => orderedItem(videos.value[index], index))
  if (items.length === 0) {
    return
  }
  await commands.createDownloadTasks({
    Youtube: { collection_title: collectionTitle.value, videos: items },
  })
  for (const index of indexes) {
    cardRefsMap.value.get(String(index))?.playDownloadAnimation()
    await new Promise((resolve) => setTimeout(resolve, 200))
  }
}

async function downloadChecked() {
  const indexes = [...checkedIds.value]
    .map((id) => Number(id))
    .filter((index) => !Number.isNaN(index))
    .sort((a, b) => a - b)
  if (indexes.length === 0) {
    message.warning('请先勾选要下载的视频')
    return
  }
  await downloadIndexes(indexes)
}

function toggleCheck(index: number) {
  const key = String(index)
  if (checkedIds.value.has(key)) {
    checkedIds.value.delete(key)
  } else {
    checkedIds.value.add(key)
  }
}

function handleContextMenu(index: number) {
  const key = String(index)
  if (selectedIds.value.has(key)) {
    return
  }
  selectedIds.value.clear()
  selectedIds.value.add(key)
  const selection = selectionAreaRef.value?.selection
  if (selection) {
    selection.clearSelection()
    selection.select(`[data-key="${key}"]`)
  }
}

function searchChannel(channelId: string, channelName: string) {
  if (channelId !== '') {
    searchPaneRef?.value?.search(channelId, 'Youtube')
    return
  }
  if (channelName.startsWith('@')) {
    searchPaneRef?.value?.search(channelName, 'Youtube')
  }
}

async function loadMore() {
  const token = continuation.value
  if (token === null || loadingMore.value) {
    return
  }
  loadingMore.value = true
  if ('Channel' in result.value) {
    const current = result.value.Channel
    const response = await commands.getYoutubeChannel({
      channel_id: current.channel_id,
      continuation: token,
    })
    loadingMore.value = false
    if (response.status === 'error') {
      console.error(response.error)
      message.error(response.error.err_title)
      return
    }
    const page = response.data
    result.value = {
      Channel: {
        ...current,
        title: current.title || page.title,
        description: current.description || page.description,
        avatar: current.avatar || page.avatar,
        channel_id: current.channel_id || page.channel_id,
        videos: [...current.videos, ...page.videos],
        continuation: page.continuation,
      },
    }
    return
  }
  if ('Playlist' in result.value) {
    const current = result.value.Playlist
    const response = await commands.getYoutubePlaylist({
      playlist_id: current.playlist_id,
      continuation: token,
    })
    loadingMore.value = false
    if (response.status === 'error') {
      console.error(response.error)
      message.error(response.error.err_title)
      return
    }
    const page = response.data
    result.value = {
      Playlist: {
        ...current,
        title: current.title || page.title,
        author_name: current.author_name || page.author_name,
        author_id: current.author_id || page.author_id,
        author_avatar: current.author_avatar || page.author_avatar,
        thumbnail: current.thumbnail || page.thumbnail,
        videos: [...current.videos, ...page.videos],
        continuation: page.continuation,
      },
    }
  }
}
</script>

<template>
  <div class="flex flex-col h-full select-none overflow-auto">
    <div class="flex m-2 rounded-lg">
      <img
        v-if="header.cover !== ''"
        class="object-contain mr-2 rounded-lg h-25"
        :src="mediaImage(header.cover)"
        alt=""
        draggable="false" />
      <div class="flex flex-col w-full break-words overflow-auto">
        <span class="font-bold text-lg line-clamp-1">{{ header.title }}</span>
        <span class="line-clamp-2 whitespace-pre-wrap">{{ header.description }}</span>
        <button
          v-if="header.authorId !== '' || header.authorName.startsWith('@')"
          type="button"
          class="mt-auto ml-auto flex items-center color-inherit bg-transparent border-0 p-0 cursor-pointer"
          @click="searchChannel(header.authorId, header.authorName)">
          <img
            v-if="header.authorAvatar !== ''"
            class="w-8 h-8 rounded-full"
            :src="mediaImage(header.authorAvatar)"
            alt=""
            draggable="false" />
          <div class="line-clamp-1" :title="header.authorName">{{ header.authorName }}</div>
        </button>
        <div v-else-if="header.authorName !== ''" class="mt-auto ml-auto flex items-center">
          <img
            v-if="header.authorAvatar !== ''"
            class="w-8 h-8 rounded-full"
            :src="mediaImage(header.authorAvatar)"
            alt=""
            draggable="false" />
          <div class="line-clamp-1" :title="header.authorName">{{ header.authorName }}</div>
        </div>
      </div>
    </div>

    <SelectionArea
      v-if="isList"
      ref="selectionAreaRef"
      :options="selectionOptions"
      @move="updateSelectedIds"
      @start="unselectAll" />
    <div
      class="youtube-panel-selection-container flex flex-col flex-1 px-2 overflow-auto"
      @contextmenu="isList ? showDropdown($event) : undefined">
      <div v-if="isList" class="animate-pulse text-violet">左键拖动进行框选，右键打开菜单</div>
      <div class="flex flex-wrap gap-2">
        <YoutubeVideoCard
          ref="cardRefs"
          v-for="(video, index) in videos"
          :key="`${sourceKey}-${index}-${video.video_id}`"
          :data-key="String(index)"
          :class="[
            isList ? 'selectable' : 'border border-solid border-gray-2',
            isList && selectedIds.has(String(index)) ? 'selected shadow-md' : 'hover:bg-gray-1',
          ]"
          :video="video"
          :checked="checkedIds.has(String(index))"
          :show-checkbox="isList"
          :download-video="() => downloadIndexes([index])"
          :toggle-check="isList ? () => toggleCheck(index) : undefined"
          :handle-context-menu="isList ? () => handleContextMenu(index) : undefined"
          :search-channel="
            video.channel_id !== '' || video.channel_name.startsWith('@')
              ? () => searchChannel(video.channel_id, video.channel_name)
              : undefined
          " />
      </div>
    </div>

    <div v-if="isList || continuation !== null" class="flex gap-2 m-2 box-border">
      <n-button v-if="continuation !== null" size="small" :loading="loadingMore" @click="loadMore">加载更多</n-button>
      <n-button v-if="isList" class="ml-auto" size="small" type="primary" @click="downloadChecked">
        下载勾选视频
      </n-button>
    </div>

    <n-dropdown
      v-if="isList"
      placement="bottom-start"
      trigger="manual"
      :x="dropdownX"
      :y="dropdownY"
      :options="dropdownOptions"
      :show="dropdownShowing"
      :on-clickoutside="() => (dropdownShowing = false)" />
  </div>
</template>

<style scoped>
.youtube-panel-selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}
</style>
