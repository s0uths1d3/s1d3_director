<template>
  <div class="flex flex-col h-full p-3 space-y-3">
    <!-- 当前播放信息 -->
    <div v-if="currentBeat" class="bg-slate-800/60 rounded-lg p-3 border border-slate-700/40">
      <div class="flex items-center gap-2 mb-2">
        <span class="text-[10px] font-mono text-indigo-400 bg-indigo-500/10 px-1.5 py-0.5 rounded">
          SCENE {{ currentScene?.id }}
        </span>
        <span class="text-xs text-slate-400 truncate">{{ currentScene?.location }}</span>
      </div>

      <!-- 当前节拍内容 -->
      <div
        ref="contentRef"
        class="min-h-[80px] max-h-[160px] overflow-auto rounded-md p-2.5 text-sm leading-relaxed"
        :class="beatTypeClass"
      >
        <div v-if="currentBeat.speaker" class="text-xs font-semibold text-purple-400 mb-1">
          {{ currentBeat.speaker }}
          <span v-if="currentBeat.emotion" class="ml-1.5 text-[10px] text-slate-500 font-normal">
            [{{ currentBeat.emotion }}]
          </span>
        </div>
        <p class="text-slate-200 whitespace-pre-wrap break-words">{{ currentBeat.content }}</p>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else class="flex flex-col items-center justify-center h-40 text-slate-600">
      <svg class="w-8 h-8 mb-2 opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
          d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
          d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <p class="text-xs">点击播放开始</p>
    </div>

    <!-- 播放控制区 -->
    <div class="space-y-3 pt-1">
      <!-- 进度条 -->
      <div class="space-y-1">
        <input
          type="range"
          min="0"
          :max="Math.max(0, scriptStore.totalBeats - 1)"
          :value="scriptStore.playPosition"
          @input="handleSeek"
          class="w-full h-1.5 bg-slate-700 rounded-full appearance-none cursor-pointer accent-indigo-500"
        />
        <div class="flex justify-between text-[10px] text-slate-600">
          <span>{{ progressLabel }}</span>
          <span>{{ scriptStore.totalBeats }} 节拍</span>
        </div>
      </div>

      <!-- 控制按钮行 -->
      <div class="flex items-center justify-center gap-3">
        <!-- 上一个节拍 -->
        <button
          @click="prevBeat"
          :disabled="scriptStore.playPosition <= 0"
          class="p-2 rounded-lg text-slate-400 hover:text-white hover:bg-slate-700/50 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="上一拍"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
            <path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" />
          </svg>
        </button>

        <!-- 播放/暂停 -->
        <button
          @click="togglePlay"
          class="p-3 rounded-full transition-all duration-200"
          :class="scriptStore.isPlaying
            ? 'bg-red-500/20 hover:bg-red-500/30 text-red-400'
            : 'bg-gradient-to-br from-indigo-500 to-purple-600 hover:from-indigo-400 hover:to-purple-500 text-white shadow-lg shadow-indigo-500/20'"
          :title="scriptStore.isPlaying ? '暂停' : '播放'"
        >
          <svg v-if="!scriptStore.isPlaying" class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
            <path d="M8 5v14l11-7z" />
          </svg>
          <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
            <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
          </svg>
        </button>

        <!-- 下一个节拍 -->
        <button
          @click="nextBeat"
          :disabled="scriptStore.playPosition >= scriptStore.totalBeats - 1"
          class="p-2 rounded-lg text-slate-400 hover:text-white hover:bg-slate-700/50 transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
          title="下一拍"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
            <path d="M16 18h2V6h-2zm-11.5 6l8.5-6V6l-8.5 6z" />
          </svg>
        </button>
      </div>

      <!-- TTS 开关 + 停止按钮 -->
      <div class="flex items-center justify-between">
        <label class="flex items-center gap-2 cursor-pointer group">
          <span class="text-xs text-slate-500 group-hover:text-slate-300 transition-colors">TTS 朗读</span>
          <button
            @click="ttsEnabled = !ttsEnabled"
            class="relative w-9 h-5 rounded-full transition-colors duration-200"
            :class="ttsEnabled ? 'bg-indigo-500' : 'bg-slate-700'"
          >
            <span
              class="absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform duration-200"
              :class="ttsEnabled ? 'translate-x-4' : ''"
            ></span>
          </button>
        </label>

        <button
          @click="stopPlay"
          class="px-3 py-1 text-xs text-slate-500 hover:text-red-400 border border-slate-700/50 hover:border-red-500/30 rounded-lg transition-colors"
        >
          停止
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useScriptStore } from '~/stores/scriptStore'

const scriptStore = useScriptStore()
const contentRef = ref<HTMLElement>()
const ttsEnabled = ref(false)

// 播放定时器
let playTimer: ReturnType<typeof setInterval> | null = null

// 当前播放中的 beat 信息
const currentBeat = computed(() => {
  const flat = scriptStore.flatBeats
  if (scriptStore.playPosition >= 0 && scriptStore.playPosition < flat.length) {
    return flat[scriptStore.playPosition].beat
  }
  return null
})

const currentScene = computed(() => {
  const flat = scriptStore.flatBeats
  if (scriptStore.playPosition >= 0 && scriptStore.playPosition < flat.length) {
    const item = flat[scriptStore.playPosition]
    return scriptStore.scenes.find(s => s.id === item.sceneId)
  }
  return null
})

const beatTypeClass = computed(() => {
  if (!currentBeat.value) return ''
  switch (currentBeat.value.type) {
    case 'action':         return 'border-l-2 border-slate-500 bg-slate-900/40'
    case 'dialogue':       return 'border-l-2 border-blue-500 bg-blue-950/20'
    case 'monologue':      return 'border-l-2 border-purple-500 bg-purple-950/20'
    case 'parenthetical':  return 'border-l-2 border-dashed border-yellow-500/60 bg-yellow-950/10'
    default:               return 'border-l-2 border-slate-600 bg-slate-900/30'
  }
})

const progressLabel = computed(() => {
  const pos = scriptStore.playPosition
  const total = scriptStore.totalBeats
  if (total === 0) return '0 / 0'
  return `${pos + 1} / ${total}`
})

// 播放控制
function togglePlay() {
  scriptStore.togglePlay()

  if (scriptStore.isPlaying) {
    startAutoPlay()
  } else {
    stopAutoPlay()
  }
}

function startAutoPlay() {
  stopAutoPlay()
  playTimer = setInterval(() => {
    if (scriptStore.playPosition >= scriptStore.totalBeats - 1) {
      scriptStore.stopPlay()
      stopAutoPlay()
      return
    }

    // 移动到下一个 beat
    scriptStore.nextBeat()

    // TTS 朗读
    if (ttsEnabled.value && currentBeat.value?.content) {
      speakContent(currentBeat.value.content)
    }

    // 自动滚动到当前内容
    scrollToCurrent()
  }, 2500) // 每 2.5 秒切换一个节拍
}

function stopAutoPlay() {
  if (playTimer) {
    clearInterval(playTimer)
    playTimer = null
  }
}

function stopPlay() {
  scriptStore.stopPlay()
  stopAutoPlay()
}

function prevBeat() {
  if (scriptStore.playPosition > 0) {
    scriptStore.playPosition--
    const flat = scriptStore.flatBeats
    if (flat[scriptStore.playPosition]) {
      const item = flat[scriptStore.playPosition]
      scriptStore.currentSceneId = item.sceneId
      scriptStore.currentBeatIndex = item.beatIndex
    }
  }
}

function nextBeat() {
  if (scriptStore.playPosition < scriptStore.totalBeats - 1) {
    scriptStore.nextBeat()
  }
}

function handleSeek(e: Event) {
  const target = e.target as HTMLInputElement
  const newPos = parseInt(target.value)
  scriptStore.playPosition = newPos

  const flat = scriptStore.flatBeats
  if (flat[newPos]) {
    scriptStore.currentSceneId = flat[newPos].sceneId
    scriptStore.currentBeatIndex = flat[newPos].beatIndex
  }
}

// 自动滚动当前内容区域
function scrollToCurrent() {
  nextTick(() => {
    if (contentRef.value) {
      contentRef.value.scrollTop = 0
    }
  })
}

// TTS 朗读（使用 Web Speech API）
let speechSynthesisUtterance: SpeechSynthesisUtterance | null = null

function speakContent(text: string) {
  if (!('speechSynthesis' in window)) return

  // 取消之前的朗读
  window.speechSynthesis.cancel()

  speechSynthesisUtterance = new SpeechSynthesisUtterance(text)
  speechSynthesisUtterance.lang = 'zh-CN'
  speechSynthesisUtterance.rate = 1.0
  speechSynthesisUtterance.pitch = 1.0

  window.speechSynthesis.speak(speechSynthesisUtterance)
}

// 监听播放状态变化，自动滚动
watch(
  () => scriptStore.playPosition,
  () => {
    scrollToCurrent()
  }
)

// 组件卸载时清理
import { onUnmounted } from 'vue'

onUnmounted(() => {
  stopAutoPlay()
  window.speechSynthesis?.cancel()
})
</script>
