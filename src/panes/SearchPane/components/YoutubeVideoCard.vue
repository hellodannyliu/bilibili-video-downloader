<script setup lang="ts">
import { computed, inject, useTemplateRef } from 'vue'
import { YoutubeVideoItem } from '../../../bindings.ts'
import SimpleCheckbox from '../../../components/SimpleCheckbox.vue'
import IconButton from '../../../components/IconButton.vue'
import { PhDownloadSimple, PhGoogleChromeLogo, PhMagnifyingGlass } from '@phosphor-icons/vue'
import { isElementInViewport, mediaImage, playTaskToQueueAnimation } from '../../../utils.tsx'
import { navDownloadButtonRefKey } from '../../../injection_keys.ts'

const props = defineProps<{
  video: YoutubeVideoItem
  checked?: boolean
  showCheckbox?: boolean
  downloadVideo?: () => Promise<void>
  toggleCheck?: () => void
  handleContextMenu?: () => void
  searchChannel?: () => void
}>()

const navDownloadButtonRef = inject(navDownloadButtonRefKey)
const rootDivRef = useTemplateRef('rootDivRef')
const downloadButtonRef = useTemplateRef('downloadButtonRef')

const href = computed(() => `https://www.youtube.com/watch?v=${props.video.video_id}`)

const durationText = computed(() => formatDuration(props.video.duration))

const metaText = computed(() => {
  return [props.video.view_text, props.video.published_text].filter((text) => text !== '').join(' · ')
})

function formatDuration(seconds: number): string {
  if (seconds <= 0) {
    return ''
  }
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const remain = Math.floor(seconds % 60)
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(remain).padStart(2, '0')}`
  }
  return `${minutes}:${String(remain).padStart(2, '0')}`
}

async function handleDownloadClick() {
  if (props.downloadVideo === undefined) {
    return
  }
  await props.downloadVideo()
  playDownloadAnimation()
}

function playDownloadAnimation() {
  if (rootDivRef.value === null) {
    return
  }
  const from = downloadButtonRef.value?.$el
  const to = navDownloadButtonRef?.value
  if (from instanceof Element && to !== undefined && to !== null && isElementInViewport(rootDivRef.value)) {
    playTaskToQueueAnimation(from, to)
  }
}

defineExpose({ playDownloadAnimation })
</script>

<template>
  <div class="flex flex-col w-200px relative p-3 rounded-lg" @contextmenu="handleContextMenu?.()" ref="rootDivRef">
    <SimpleCheckbox
      v-if="showCheckbox && toggleCheck !== undefined"
      class="absolute top-6 left-6 z-1 backdrop-blur-2"
      :checked="checked === true"
      :on-click="toggleCheck" />
    <div class="relative">
      <img
        class="w-200px h-125px rounded-lg object-cover lazyload"
        :data-src="mediaImage(video.thumbnail)"
        :key="video.thumbnail"
        alt=""
        draggable="false" />
      <div v-if="durationText !== ''" class="absolute bottom-1 right-1 bg-black/75 text-white text-12px px-1 rounded">
        {{ durationText }}
      </div>
    </div>

    <div class="w-full flex flex-col h-45px mt-2">
      <span class="line-clamp-2" :title="video.title">{{ video.title }}</span>
    </div>

    <div class="flex items-center whitespace-nowrap text-gray text-12px w-full overflow-hidden">
      <button
        v-if="searchChannel !== undefined && video.channel_name !== ''"
        type="button"
        class="min-w-0 color-inherit bg-transparent border-0 p-0 hover:text-sky-5 mr-1 cursor-pointer"
        :title="`查看 ${video.channel_name} 的频道`"
        @click="searchChannel">
        <div class="truncate text-ellipsis" :title="video.channel_name">{{ video.channel_name }}</div>
      </button>
      <span v-else class="min-w-0 truncate text-ellipsis mr-1" :title="video.channel_name">
        {{ video.channel_name }}
      </span>
      <span v-if="metaText !== ''" class="ml-auto flex-shrink-0 truncate max-w-90px" :title="metaText">
        {{ video.published_text || video.view_text }}
      </span>
    </div>

    <div class="flex gap-1 items-center">
      <IconButton title="在浏览器中打开" :href="href">
        <PhGoogleChromeLogo :size="24" />
      </IconButton>
      <IconButton
        v-if="searchChannel !== undefined && (video.channel_id !== '' || video.channel_name !== '')"
        title="查看该作者主页的所有视频"
        @click="searchChannel">
        <PhMagnifyingGlass :size="24" />
      </IconButton>
      <IconButton
        v-if="downloadVideo !== undefined"
        ref="downloadButtonRef"
        title="一键下载"
        class="ml-auto"
        @click="handleDownloadClick">
        <PhDownloadSimple :size="24" />
      </IconButton>
    </div>
  </div>
</template>
