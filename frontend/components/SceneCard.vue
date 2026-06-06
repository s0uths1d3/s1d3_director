<template>
  <div
    class="group-scene bg-slate-800/50 backdrop-blur-sm rounded-xl border border-slate-700/50 overflow-hidden transition-all duration-200"
    :class="isExpanded ? '' : 'hover:border-slate-600/50'"
  >
    <!-- ===== 场景标题栏（始终可见，点击切换折叠） ===== -->
    <div
      @click="toggleExpand"
      class="px-4 py-3 border-b border-slate-700/30 cursor-pointer select-none hover:bg-slate-700/20 transition-colors"
    >
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-3">
          <!-- 折叠箭头指示器 -->
          <svg
            class="w-3.5 h-3.5 text-indigo-400 transition-transform duration-300 ease-out flex-shrink-0"
            :class="{ 'rotate-90': isExpanded, '-rotate-90': !isExpanded }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 5l7 7-7 7" />
          </svg>

          <span class="text-xs font-mono text-indigo-400">SCENE {{ scene.id }}</span>
          <!-- 场景地点（可编辑） -->
          <input
            :value="scene.location"
            @input.stop="(e: Event) => emit('update-field', 'location', (e.target as HTMLInputElement).value)"
            class="text-sm font-medium bg-transparent text-white border-b border-slate-600/50 focus:border-indigo-400 outline-none px-1 min-w-[120px] placeholder:text-slate-600"
            placeholder="场景地点"
            title="点击编辑场景地点"
          />
          <span v-if="scene.time" class="text-xs text-slate-500">·</span>
          <!-- 场景时间（可编辑） -->
          <input
            v-if="scene.time || editingTime"
            :value="scene.time ?? ''"
            @blur="editingTime = false; if (!scene.time) emit('update-field', 'time', '')"
            @input.stop="(e: Event) => emit('update-field', 'time', (e.target as HTMLInputElement).value)"
            class="text-xs bg-transparent text-slate-400 border-b border-slate-600/50 focus:border-indigo-400 outline-none px-1 w-24 min-w-0 placeholder:text-slate-600"
            placeholder="时间"
            title="点击编辑时间"
          />
          <button
            v-else-if="!scene.time"
            @click.stop="editingTime = true"
            class="text-xs text-slate-600 hover:text-slate-400 px-1 rounded hover:bg-slate-700/30 transition-colors"
          >+ 时间</button>
        </div>

        <!-- 右侧操作按钮 -->
        <div class="flex items-center gap-2">
          <!-- 节拍计数（折叠时显示） -->
          <span v-if="!isExpanded" class="text-[10px] text-slate-500 mr-1">
            {{ scene.beats.length }} 个节拍
          </span>

          <!-- AI 重生成整场按钮 -->
          <button
            v-if="!isRegenerating"
            @click.stop="emit('ai-regenerate-scene')"
            class="flex items-center gap-1 px-2 py-1 text-[10px] rounded-md text-emerald-400/70 hover:text-emerald-300 hover:bg-emerald-500/10 transition-all"
            title="AI 重生成整场内容"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            AI 重生
          </button>
          <div v-else class="flex items-center gap-1 px-2 py-1 text-[10px] rounded-md bg-emerald-500/10 text-emerald-400">
            <svg class="animate-spin w-3 h-3" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            AI 生成中...
          </div>

          <!-- 删除场景 -->
          <button
            @click.stop="emit('delete-scene')"
            class="p-1 rounded text-slate-600 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-scene:hover:opacity-100"
            title="删除此场景"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
          </button>
        </div>
      </div>

      <!-- 第二行：情绪滑块 + media_hints（展开时显示完整，折叠时精简） -->
      <transition name="slide-fade">
        <div v-show="isExpanded" class="flex items-center gap-4">
          <!-- 情绪强度滑块 -->
          <div class="flex items-center gap-2 flex-1 max-w-xs">
            <span class="text-[10px] text-slate-500 whitespace-nowrap">情绪强度</span>
            <input
              type="range"
              min="0"
              max="10"
              step="0.1"
              :value="scene.emotion_intensity"
              @input.stop="(e: Event) => emit('update-field', 'emotion_intensity', parseFloat((e.target as HTMLInputElement).value))"
              class="flex-1 h-1 appearance-none bg-slate-700 rounded-full cursor-pointer accent-indigo-500 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-indigo-400 [&::-webkit-slider-thumb]:shadow-[0_0_6px_rgba(129,140,248,0.5)]"
            />
            <span class="text-[10px] font-mono text-slate-400 w-6 text-right">{{ scene.emotion_intensity.toFixed(1) }}</span>
          </div>

          <!-- 媒体提示 -->
          <div v-if="scene.media_hints" class="flex items-center gap-2 text-xs text-slate-500 ml-auto">
            <span v-if="scene.media_hints.camera" title="镜头提示">📷 {{ scene.media_hints.camera }}</span>
            <span v-if="scene.media_hints.music" title="音乐提示">🎵 {{ scene.media_hints.music }}</span>
          </div>
        </div>
      </transition>
    </div>

    <!-- ===== 可折叠内容区：Beat 卡片列表 ===== -->
    <transition
      name="expand-collapse"
      @enter="onEnter"
      @after-enter="onAfterEnter"
      @leave="onLeave"
      @after-leave="onAfterLeave"
    >
      <div v-show="isExpanded" ref="contentRef" class="overflow-hidden">
        <div class="divide-y divide-slate-700/20">
          <BeatCard
            v-for="(beat, idx) in scene.beats"
            :key="idx"
            :scene-id="scene.id"
            :beat-index="idx"
            :beat="beat"
            :is-active="isActive && activeBeatIndex === idx"
            :is-regenerating="regeneratingBeatIdx === `${scene.id}:${idx}`"
            @update-content="(content) => emit('update-beat', idx, content)"
            @update-type="(newType) => emit('update-beat-type', idx, newType)"
            @update-speaker="(speaker) => emit('update-speaker', idx, speaker)"
            @update-emotion="(emotion) => emit('update-emotion', idx, emotion)"
            @select-alt="(altIdx) => emit('select-alt', idx, altIdx)"
            @remove-alt="(altIdx) => emit('remove-alt', idx, altIdx)"
            @add-alt="(content, opts) => emit('add-alt', idx, content, opts)"
            @delete="emit('delete-beat', idx)"
            @ai-regenerate="() => emit('ai-regenerate-beat', idx)"
            @move-up="() => emit('move-beat', idx, 'up')"
            @move-down="() => emit('move-beat', idx, 'down')"
            @toggle-history="() => emit('toggle-history', idx)"
          />

          <!-- 添加节拍按钮 -->
          <button
            @click="emit('add-beat')"
            class="w-full px-4 py-2.5 text-xs text-slate-500 hover:text-indigo-400 hover:bg-slate-700/20 transition-colors flex items-center justify-center gap-1"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            添加节拍
          </button>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import BeatCard from '~/components/BeatCard.vue'
import type { Scene } from '~/stores/scriptStore'

interface Props {
  scene: Scene
  /** 当前场景中是否有活跃的 beat */
  isActive?: boolean
  /** 活跃的 beat 索引 */
  activeBeatIndex?: number
  /** 整个场景是否正在 AI 重新生成 */
  isRegenerating?: boolean
  /** 正在生成的 beat key (sceneId:beatIndex) */
  regeneratingBeatIdx?: string
  /** 默认是否展开 */
  defaultExpanded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
  activeBeatIndex: -1,
  isRegenerating: false,
  regeneratingBeatIdx: '',
  defaultExpanded: true,
})

const emit = defineEmits<{
  'update-field': [field: string, value: any]
  'update-beat': [beatIndex: number, content: string]
  'update-beat-type': [beatIndex: number, newType: string]
  'update-speaker': [beatIndex: number, speaker: string]
  'update-emotion': [beatIndex: number, emotion: string]
  'select-alt': [beatIndex: number, altIndex: number]
  'remove-alt': [beatIndex: number, altIndex: number]
  'add-alt': [beatIndex: number, content: string, options?: { name?: string; emotion?: string }]
  'add-beat': []
  'delete-beat': [beatIndex: number]
  'delete-scene': []
  'ai-regenerate-scene': []
  'ai-regenerate-beat': [beatIndex: number]
  'move-beat': [beatIndex: number, direction: 'up' | 'down']
  'toggle-history': [beatIndex: number]
}>()

// 折叠状态
const isExpanded = ref(props.defaultExpanded)
const editingTime = ref(false)
const contentRef = ref<HTMLElement>()

function toggleExpand() {
  isExpanded.value = !isExpanded.value
}

// ---- Vue Transition 动画钩子：实现平滑高度过渡 ----

/** 进入动画开始：先设 height=0 再触发重绘 */
function onEnter(el: Element) {
  const htmlEl = el as HTMLElement
  htmlEl.style.height = '0'
  htmlEl.style.overflow = 'hidden'
  // 下一帧设置实际高度 → CSS transition 生效
  requestAnimationFrame(() => {
    htmlEl.style.height = htmlEl.scrollHeight + 'px'
  })
}

/** 进入动画结束：清除内联样式让内容自适应 */
function onAfterEnter(el: Element) {
  const htmlEl = el as HTMLElement
  htmlEl.style.height = ''
  htmlEl.style.overflow = ''
}

/** 离开动画开始：记录当前高度 */
function onLeave(el: Element) {
  const htmlEl = el as HTMLElement
  htmlEl.style.height = htmlEl.scrollHeight + 'px'
  htmlEl.style.overflow = 'hidden'
  // 下一帧设为 0 → 触发收缩
  requestAnimationFrame(() => {
    htmlEl.style.height = '0'
  })
}

/** 离开动画结束 */
function onAfterLeave(el: Element) {
  const htmlEl = el as HTMLElement
  htmlEl.style.height = ''
  htmlEl.style.overflow = ''
}
</script>

<style scoped>
/* 内容区域高度过渡 */
.expand-collapse-active {
  transition: height 0.3s ease-out, opacity 0.25s ease;
}

/* 头部第二行（情绪滑块等）淡入淡出 */
.slide-fade-enter-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
  transition-origin: top;
}
.slide-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
  transition-origin: top;
}
.slide-fade-enter-from {
  opacity: 0;
  transform: translateY(-4px);
}
.slide-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
