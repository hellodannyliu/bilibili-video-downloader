<script setup lang="ts">
import { computed } from 'vue'
import { mediaImage } from '../utils.tsx'

const props = defineProps<{
  upName: string
  upAvatar: string
  upUid?: number
  href?: string
}>()

const link = computed(() => {
  if (props.href !== undefined && props.href !== '') {
    return props.href
  }
  if (props.upUid !== undefined) {
    return `https://space.bilibili.com/${props.upUid}`
  }
  return undefined
})
</script>

<template>
  <a
    v-if="link !== undefined"
    class="flex items-center color-inherit no-underline"
    :href="link"
    target="_blank"
    draggable="false">
    <img class="w-8 h-8 rounded-full" :src="mediaImage(upAvatar, '@128w_128h')" alt="" draggable="false" />
    <div class="line-clamp-1" :title="upName">{{ upName }}</div>
  </a>
  <div v-else class="flex items-center">
    <img class="w-8 h-8 rounded-full" :src="mediaImage(upAvatar, '@128w_128h')" alt="" draggable="false" />
    <div class="line-clamp-1" :title="upName">{{ upName }}</div>
  </div>
</template>
