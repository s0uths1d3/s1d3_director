<template>
  <div
    class="group relative px-4 py-3 transition-colors hover:bg-slate-800/30"
    :class="[typeBorderClass, isActive ? 'bg-indigo-500/5 ring-1 ring-inset ring-indigo-500/20' : '']"
  >
    <!-- ===== 第一行：类型 + 角色 + 情绪 + 操作按钮 ===== -->
    <div class="flex items-center gap-2 mb-1.5">
      <!-- 类型标签（点击切换） -->
      <div class="relative" ref="typeDropdownRef">
        <button
          @click="showTypeMenu = !showTypeMenu"
          class="text-[10px] font-medium uppercase tracking-wider px-1.5 py-0.5 rounded hover:opacity-80 transition-opacity cursor-pointer"
          :class="typeBadgeClass"
        >
          {{ typeLabel }} ▾
        </button>
        <!-- 类型切换下拉菜单 -->
        <div
          v-if="showTypeMenu"
          class="absolute z-20 mt-1 min-w-[100px] bg-slate-800 border border-slate-600/60 rounded-lg shadow-xl py-1"
        >
          <button
            v-for="(cfg, key) in typeConfig"
            :key="key"
            @click="selectType(key)"
            class="w-full px-3 py-1.5 text-left text-[11px] transition-colors flex items-center gap-2"
            :class="beat.type === key ? 'text-indigo-300 bg-indigo-500/10' : 'text-slate-300 hover:bg-slate-700'"
          >
            <span class="w-2 h-2 rounded-sm" :class="cfg.badgeClass.split(' ')[0]"></span>
            {{ cfg.label }}
          </button>
        </div>
      </div>

      <!-- 角色名（可编辑） -->
      <input
        v-if="beat.type === 'dialogue' || beat.type === 'monologue'"
        :value="localSpeaker"
        @input="onSpeakerInput"
        @blur="confirmSpeaker"
        @keydown.enter="confirmSpeaker"
        placeholder="角色名"
        class="text-xs font-medium text-purple-300 bg-transparent border-b border-transparent hover:border-slate-600 focus:border-purple-500 outline-none px-0.5 w-16 transition-colors"
      />

      <!-- 情绪（可编辑） -->
      <input
        :value="localEmotion"
        @input="onEmotionInput"
        @blur="confirmEmotion"
        @keydown.enter="confirmEmotion"
        placeholder="情绪"
        class="text-[10px] text-slate-400 bg-transparent border-b border-transparent hover:border-slate-600 focus:border-emerald-500 outline-none px-1 w-14 transition-colors"
      />

      <!-- 右侧操作按钮组 -->
      <div class="ml-auto flex items-center gap-0.5">
        <!-- AI 重生（始终可见） -->
        <button
          @click="$emit('ai-regenerate')"
          class="flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium border transition-all"
          :class="isRegenerating
            ? 'border-indigo-500/40 bg-indigo-500/10 text-indigo-400 animate-pulse'
            : 'border-slate-700/40 bg-slate-800/50 text-slate-400 hover:border-indigo-500/50 hover:text-indigo-400 hover:bg-indigo-500/5'"
          :title="isRegenerating ? '生成中...' : 'AI 重新生成'"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ isRegenerating ? '生成中' : 'AI 重新生成' }}
        </button>
        <!-- 更多操作（hover显示） -->
        <div class="flex items-center gap-0.5 opacity-30 group-hover:opacity-100 transition-opacity">
        <!-- 上移 -->
        <button
          @click="$emit('move-up')"
          class="p-1 rounded text-slate-600 hover:text-slate-300 hover:bg-slate-700/50 transition-all"
          title="上移"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>
        </button>
        <!-- 下移 -->
        <button
          @click="$emit('move-down')"
          class="p-1 rounded text-slate-600 hover:text-slate-300 hover:bg-slate-700/50 transition-all"
          title="下移"
        >
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
        </button>
        <!-- AI 重生 -->
        <button
          @click="$emit('ai-regenerate')"
          class="p-1 rounded text-slate-600 hover:text-indigo-400 hover:bg-indigo-500/10 transition-all"
          :class="{ 'animate-pulse text-indigo-400': isRegenerating }"
          :title="isRegenerating ? '生成中...' : 'AI 重新生成'"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
        <!-- 历史记录 -->
        <button
          @click="$emit('toggle-history')"
          class="p-1 rounded text-slate-600 hover:text-amber-400 hover:bg-amber-500/10 transition-all"
          title="修改历史"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>
        <!-- 删除节拍 -->
        <button
          @click="$emit('delete')"
          class="p-1 rounded text-slate-600 hover:text-red-400 hover:bg-red-500/10 transition-all"
          title="删除此节拍"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      </div>
    </div>

    <!-- ===== 内容编辑区 ===== -->
    <div
      contenteditable="true"
      :data-placeholder="'输入' + typeLabel + '内容...'"
      @blur="handleBlur"
      @input="handleInput"
      @keydown.enter="handleEnterKeydown"
      class="text-sm text-slate-200 leading-relaxed outline-none min-h-[20px] empty:before:content-[attr(data-placeholder)] empty:before:text-slate-600 empty:before:text-xs focus:empty:before:text-slate-500 break-words"
      :textContent="displayedContent"
      ref="editorRef"
    ></div>

    <!-- ===== 备选项选择区（始终显示，方案1=当前内容） ===== -->
    <div class="mt-2 pt-2 border-t border-slate-700/20">
      <div class="flex items-center justify-between mb-1.5">
        <span class="text-[10px] text-slate-500">备选方案 ({{ (beat.alternatives?.length || 0) + 1 }})</span>
        <button
          @click="showAddAltForm = !showAddAltForm"
          class="text-[10px] text-slate-500 hover:text-indigo-300 transition-colors"
        >+ 自定义</button>
      </div>

      <!-- 方案标签列表：方案1(当前) + alternatives -->
      <div class="flex flex-wrap gap-1.5">
        <!-- 方案1 = 当前选中内容 -->
        <button
          @click="$emit('select-alt', -1)"
          class="px-2 py-1 text-[11px] rounded-md border transition-all flex items-center gap-1"
          :class="(beat.selected ?? 0) === 0 || beat.selected === -1 || !beat.selected
            ? 'border-indigo-500 bg-indigo-500/15 text-indigo-300'
            : 'border-slate-700/50 text-slate-400 hover:border-slate-600 hover:text-slate-300'"
        >
          <span class="font-medium">方案1</span>
          <span class="opacity-50 text-[10px]">(当前)</span>
        </button>

        <!-- 来自 alternatives 的方案2、方案3... -->
        <template v-if="beat.alternatives && beat.alternatives.length > 0">
          <button
            v-for="(alt, altIdx) in beat.alternatives"
            :key="altIdx"
            @click="$emit('select-alt', altIdx)"
            class="group/tag px-2 py-1 text-[11px] rounded-md border transition-all flex items-center gap-1"
            :class="beat.selected === altIdx + 1
              ? 'border-indigo-500 bg-indigo-500/15 text-indigo-300'
              : 'border-slate-700/50 text-slate-400 hover:border-slate-600 hover:text-slate-300'"
          >
            <span class="font-medium">{{ alt.name || ('方案' + (altIdx + 2)) }}</span>
            <span v-if="alt.tone" class="opacity-60 text-[10px]">({{ alt.tone }})</span>
            <span v-else-if="alt.emotion" class="opacity-60 text-[10px]">({{ alt.emotion }})</span>
            <span
              @click.stop="$emit('remove-alt', altIdx)"
              class="ml-0.5 opacity-0 group-hover/tag:opacity-100 text-slate-600 hover:text-red-400 transition-opacity cursor-pointer text-[10px] leading-none"
              title="删除此方案"
            >&times;</span>
          </button>
        </template>
      </div>

      <!-- 自定义方案添加表单 -->
      <div v-if="showAddAltForm" class="mt-2 p-2.5 rounded-lg bg-slate-900/60 border border-slate-700/40 space-y-2">
        <input
          v-model="newAltName"
          placeholder="方案名称（可选）"
          class="w-full px-2 py-1 text-xs bg-slate-800 border border-slate-600/50 rounded text-slate-200 placeholder-slate-600 outline-none focus:border-indigo-500 transition-colors"
          @keydown.enter="confirmAddAlt"
        />
        <input
          v-model="newAltEmotion"
          placeholder="关联情绪（可选）"
          class="w-full px-2 py-1 text-xs bg-slate-800 border border-slate-600/50 rounded text-slate-200 placeholder-slate-600 outline-none focus:border-indigo-500 transition-colors"
          @keydown.enter="confirmAddAlt"
        />
        <textarea
          v-model="newAltContent"
          placeholder="方案内容..."
          rows="2"
          class="w-full px-2 py-1.5 text-xs bg-slate-800 border border-slate-600/50 rounded text-slate-200 placeholder-slate-600 outline-none focus:border-indigo-500 transition-colors resize-none"
        ></textarea>
        <div class="flex justify-end gap-2">
          <button
            @click="showAddAltForm = false; resetAltForm()"
            class="px-2.5 py-1 text-[10px] text-slate-400 hover:text-slate-200 transition-colors"
          >取消</button>
          <button
            @click="confirmAddAlt"
            :disabled="!newAltContent.trim()"
            class="px-2.5 py-1 text-[10px] font-medium rounded bg-indigo-600 text-white hover:bg-indigo-500 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >确认</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { Beat } from '~/stores/scriptStore'
import { useScriptStore } from '~/stores/scriptStore'

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
  'remove-alt': [altIndex: number]
  'add-alt': [content: string, options?: { name?: string; emotion?: string }]
  delete: []
  'ai-regenerate': []
  'move-up': []
  'move-down': []
  'toggle-history': []
}>()

const scriptStore = useScriptStore()
const editorRef = ref<HTMLElement>()
const typeDropdownRef = ref<HTMLElement>()
const localContent = ref(props.beat.content)
const localSpeaker = ref(props.beat.speaker || '')
const localEmotion = ref(props.beat.emotion || '')

// 下拉菜单状态
const showTypeMenu = ref(false)

// 自定义方案表单状态
const showAddAltForm = ref(false)
const newAltName = ref('')
const newAltEmotion = ref('')
const newAltContent = ref('')

// 点击外部关闭下拉菜单
function handleClickOutside(e: MouseEvent) {
  if (showTypeMenu.value && typeDropdownRef.value && !typeDropdownRef.value.contains(e.target as Node)) {
    showTypeMenu.value = false
  }
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onUnmounted(() => document.removeEventListener('click', handleClickOutside))

// 类型配置
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

// ---- 类型切换 ----
function selectType(key: string) {
  emit('update-type', key)
  showTypeMenu.value = false
}

// ---- 角色名编辑 ----
function onSpeakerInput(e: Event) {
  localSpeaker.value = (e.target as HTMLInputElement).value
}
function confirmSpeaker() {
  const val = localSpeaker.value.trim()
  if (val !== (props.beat.speaker || '')) {
    emit('update-speaker', val || '')
  }
}

// ---- 情绪编辑 ----
function onEmotionInput(e: Event) {
  localEmotion.value = (e.target as HTMLInputElement).value
}
function confirmEmotion() {
  const val = localEmotion.value.trim()
  if (val !== (props.beat.emotion || '')) {
    emit('update-emotion', val || '')
  }
}

// ---- 内容编辑 ----
function handleBlur(e: FocusEvent) {
  const target = e.target as HTMLElement
  const newContent = target.textContent?.trim() || ''
  if (newContent !== props.beat.content) {
    localContent.value = newContent
    emit('update-content', newContent)
  }
}
function handleInput(e: Event) {
  localContent.value = (e.target as HTMLElement).textContent || ''
}
function handleEnterKeydown(_e: KeyboardEvent) {
  // 允许换行，不做拦截
}

// ---- 备选方案 ----
function confirmAddAlt() {
  const content = newAltContent.value.trim()
  if (!content) return
  emit('add-alt', content, {
    name: newAltName.value.trim() || undefined,
    emotion: newAltEmotion.value.trim() || undefined,
  })
  resetAltForm()
  showAddAltForm.value = false
}
function resetAltForm() {
  newAltName.value = ''
  newAltEmotion.value = ''
  newAltContent.value = ''
}

// ---- 外部变化同步 ----

/** 替换内容中的 char_XXX 占位符为实际角色名 */
function resolvePlaceholders(text: string): string {
  if (!text) return text
  return text.replace(/char_(\d{3})/g, (_match, num) => {
    const charId = `char_${num.padStart(3, '0')}`
    return scriptStore.getCharacterName(charId)
  })
}

/** 显示内容（占位符已替换为实际角色名） */
const displayedContent = computed(() => resolvePlaceholders(localContent.value))

watch(() => props.beat.content, (newVal) => {
  if (newVal !== localContent.value) {
    localContent.value = newVal
    if (editorRef.value && document.activeElement !== editorRef.value) {
      editorRef.value.textContent = newVal
    }
  }
})
watch(() => props.beat.speaker, (newVal) => {
  if ((newVal || '') !== localSpeaker.value) {
    localSpeaker.value = newVal || ''
  }
})
watch(() => props.beat.emotion, (newVal) => {
  if ((newVal || '') !== localEmotion.value) {
    localEmotion.value = newVal || ''
  }
})
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
