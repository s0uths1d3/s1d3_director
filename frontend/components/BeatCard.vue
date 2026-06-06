<template>
  <div
    class="group relative px-4 py-3 transition-colors hover:bg-slate-800/30"
    :class="[typeBorderClass, isActive ? 'bg-indigo-500/5 ring-1 ring-inset ring-indigo-500/20' : '']"
  >
    <!-- 第一行：类型标签 + 说话人 + 情绪 + 操作按钮 -->
    <div class="flex items-center gap-2 mb-1.5 flex-wrap">
      <!-- 类型选择器（点击切换） -->
      <button
        @click="showTypeMenu = !showTypeMenu"
        class="text-[10px] font-medium uppercase tracking-wider px-1.5 py-0.5 rounded cursor-pointer hover:opacity-80 transition-opacity"
        :class="typeBadgeClass"
        title="点击切换节拍类型"
      >
        {{ typeLabel }}
        <svg class="w-2.5 h-2.5 inline ml-0.5 opacity-60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      <!-- 类型下拉菜单 -->
      <div
        v-if="showTypeMenu"
        class="absolute z-50 mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-xl p-1 min-w-[100px]"
        :style="{ top: '28px', left: `${typeLabel.length * 8 + 16}px` }"
      >
        <button
          v-for="bt in beatTypes"
          :key="bt.value"
          @click="changeType(bt.value); showTypeMenu = false"
          class="block w-full text-left text-xs px-2.5 py-1.5 rounded-md transition-colors"
          :class="beat.type === bt.value ? 'bg-indigo-500/20 text-indigo-300' : 'text-slate-300 hover:bg-slate-700/50'"
        >
          {{ bt.label }}
        </button>
      </div>

      <!-- 说话人输入（对白/独白时显示） -->
      <template v-if="beat.type === 'dialogue' || beat.type === 'monologue' || beat.speaker">
        <input
          v-model="speakerInput"
          @blur="commitSpeaker"
          @keydown.enter="commitSpeaker"
          placeholder="角色名"
          class="text-xs font-medium bg-transparent text-purple-300 border-b border-purple-500/30 focus:border-purple-400 outline-none px-1 py-0 w-20 min-w-0 placeholder:text-purple-600/50"
          title="说话人（可编辑）"
        />
      </template>

      <!-- 情绪标签/输入 -->
      <input
        v-if="beat.emotion !== undefined || showEmotionInput"
        v-model="emotionInput"
        @blur="commitEmotion; if (!emotionInput) showEmotionInput = false"
        @keydown.enter="commitEmotion"
        placeholder="情绪"
        class="text-[10px] bg-transparent text-slate-400 border-b border-slate-600/50 focus:border-slate-400 outline-none px-1 py-0 w-14 min-w-0 placeholder:text-slate-600"
        title="情绪标注（可编辑）"
      />
      <button
        v-else
        @click="showEmotionInput = true; $nextTick(() => emotionEl?.focus())"
        class="text-[10px] text-slate-600 hover:text-slate-400 px-1 rounded hover:bg-slate-700/30 transition-colors"
        title="添加情绪标注"
      >
        + 情绪
      </button>

      <!-- 右侧操作区 -->
      <div class="ml-auto flex items-center gap-1">
        <!-- AI 重生成按钮 -->
        <button
          v-if="!isRegenLoading"
          @click="handleAIRegenerate"
          class="p-1 rounded opacity-0 group-hover:opacity-100 text-slate-500 hover:text-emerald-400 hover:bg-emerald-500/10 transition-all"
          title="AI 重生成此节拍"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
        <!-- 加载中状态 -->
        <div v-else class="flex items-center gap-1 px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 text-[10px]">
          <svg class="animate-spin w-3 h-3" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          AI 生成中...
        </div>

        <!-- 上移 / 下移 -->
        <button
          v-if="beatIndex > 0"
          @click="$emit('move-up')"
          class="p-1 rounded opacity-0 group-hover:opacity-100 text-slate-600 hover:text-slate-300 hover:bg-slate-700/50 transition-all"
          title="上移"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>
        </button>

        <!-- 删除按钮 -->
        <button
          @click="$emit('delete')"
          class="p-1 rounded opacity-0 group-hover:opacity-100 text-slate-600 hover:text-red-400 hover:bg-red-500/10 transition-all"
          title="删除此节拍"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 内容编辑区 -->
    <div
      contenteditable="true"
      :data-placeholder="'输入' + typeLabel + '内容...'"
      @blur="handleBlur"
      @input="handleInput"
      @keydown.enter="handleEnterKeydown"
      class="text-sm text-slate-200 leading-relaxed outline-none min-h-[20px] empty:before:content-[attr(data-placeholder)] empty:before:text-slate-600 empty:before:text-xs focus:empty:before:text-slate-500 break-words"
      :textContent="localContent"
      ref="editorRef"
    ></div>

    <!-- 备选项选择区域 -->
    <div v-if="beat.alternatives && beat.alternatives.length > 0" class="mt-2 pt-2 border-t border-slate-700/20">
      <div class="flex items-center justify-between mb-1.5">
        <span class="text-[10px] text-slate-500">备选方案 ({{ beat.alternatives.length }})</span>
        <button
          v-if="hasHistory"
          @click="$emit('toggle-history')"
          class="text-[10px] text-indigo-400/70 hover:text-indigo-300 transition-colors"
          title="查看修改历史"
        >
          修改记录
        </button>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="(alt, altIdx) in beat.alternatives"
          :key="altIdx"
          @click="$emit('select-alt', altIdx)"
          class="px-2 py-1 text-[11px] rounded-md border transition-all group-alt"
          :class="beat.selected === altIdx
            ? 'border-indigo-500 bg-indigo-500/15 text-indigo-300'
            : 'border-slate-700/50 text-slate-400 hover:border-slate-600 hover:text-slate-300'"
        >
          <span class="font-medium">方案{{ altIdx + 1 }}</span>
          <span v-if="alt.tone" class="ml-1 opacity-60">({{ alt.tone }})</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue'
import type { Beat } from '~/stores/scriptStore'
import { BEAT_TYPES } from '~/stores/scriptStore'

interface Props {
  sceneId: number
  beatIndex: number
  beat: Beat
  isActive?: boolean
  isRegenerating?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
  isRegenerating: false,
})

const emit = defineEmits<{
  'update-content': [content: string]
  'update-type': [newType: string]
  'update-speaker': [speaker: string]
  'update-emotion': [emotion: string]
  'select-alt': [altIndex: number]
  delete: []
  'ai-regenerate': []
  'move-up': []
  'move-down': []
  'toggle-history': []
}>()

const editorRef = ref<HTMLElement>()
const localContent = ref(props.beat.content)
const speakerInput = ref(props.beat.speaker || '')
const emotionInput = ref(props.beat.emotion || '')
const showTypeMenu = ref(false)
const showEmotionInput = ref(false)
const emotionEl = ref<HTMLInputElement | null>(null)

// 类型列表
const beatTypes = BEAT_TYPES

// 是否正在 AI 重生成
const isRegenLoading = computed(() => props.isRegenerating)

// 是否有修改历史（通过备选数量判断，原始版本会被保存为备选）
const hasHistory = computed(() =>
  props.beat.alternatives.some(a => a.tone === '原始版本')
)

// 类型标签映射
const typeConfig: Record<string, { label: string; badgeClass: string; borderClass: string }> = {
  action: {
    label: '动作',
    badgeClass: 'bg-slate-700/60 text-slate-300',
    borderClass: 'border-l-2 border-l-slate-500',
  },
  dialogue: {
    label: '对白',
    badgeClass: 'bg-blue-500/15 text-blue-300',
    borderClass: 'border-l-2 border-l-blue-500',
  },
  monologue: {
    label: '独白',
    badgeClass: 'bg-purple-500/15 text-purple-300',
    borderClass: 'border-l-2 border-l-purple-500',
  },
  parenthetical: {
    label: '括号说明',
    badgeClass: 'bg-yellow-500/15 text-yellow-300',
    borderClass: 'border-l-2 border-l-dashed border-l-yellow-500/60',
  },
}

const typeLabel = computed(() =>
  typeConfig[props.beat.type]?.label || props.beat.type || '未知'
)

const typeBadgeClass = computed(() =>
  typeConfig[props.beat.type]?.badgeClass || 'bg-slate-700/60 text-slate-300'
)

const typeBorderClass = computed(() =>
  typeConfig[props.beat.type]?.borderClass || 'border-l-2 border-l-slate-600'
)

// ==================== 类型切换 ====================
function changeType(newType: string) {
  emit('update-type', newType)
}

// ==================== 说话人 ====================
function commitSpeaker() {
  emit('update-speaker', speakerInput.value.trim())
}

// ==================== 情绪 ====================
function commitEmotion() {
  const val = emotionInput.value.trim()
  emit('update-emotion', val)
  if (!val) showEmotionInput.value = false
}

// ==================== 内容编辑 ====================
function handleBlur(e: FocusEvent) {
  const target = e.target as HTMLElement
  const newContent = target.textContent?.trim() || ''

  if (newContent !== props.beat.content) {
    localContent.value = newContent
    emit('update-content', newContent)
  }
}

function handleInput(e: Event) {
  const target = e.target as HTMLElement
  localContent.value = target.textContent || ''
}

function handleEnterKeydown(e: KeyboardEvent) {
  if (!e.shiftKey) {
    // 默认允许换行
  }
}

// ==================== AI 重生成 ====================
async function handleAIRegenerate() {
  emit('ai-regenerate')
}

// ==================== 外部同步 ====================
import { watch } from 'vue'

watch(
  () => props.beat.content,
  (newVal) => {
    if (newVal !== localContent.value) {
      localContent.value = newVal
      if (editorRef.value && document.activeElement !== editorRef.value) {
        editorRef.value.textContent = newVal
      }
    }
  }
)

watch(
  () => props.beat.speaker,
  (newVal) => {
    if ((newVal || '') !== speakerInput.value) {
      speakerInput.value = newVal || ''
    }
  }
)

watch(
  () => props.beat.emotion,
  (newVal) => {
    if ((newVal || '') !== emotionInput.value) {
      emotionInput.value = newVal || ''
    }
  }
)

// 点击外部关闭类型菜单
if (typeof document !== 'undefined') {
  // 在 onMounted 中处理更好，但这里用简单方式
}
</script>

<style scoped>
[contenteditable]:focus {
  outline: none;
}

[contenteditable]:empty::before {
  content: attr(data-placeholder);
  pointer-events: none;
}
</style>
