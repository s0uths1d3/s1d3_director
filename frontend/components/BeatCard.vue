<template>
  <div
    class="group relative px-4 py-3 transition-colors hover:bg-slate-800/30"
    :class="[typeBorderClass, isActive ? 'bg-indigo-500/5 ring-1 ring-inset ring-indigo-500/20' : '']"
  >
    <!-- 类型标签 + 角色名 -->
    <div class="flex items-center gap-2 mb-1.5">
      <span
        class="text-[10px] font-medium uppercase tracking-wider px-1.5 py-0.5 rounded"
        :class="typeBadgeClass"
      >
        {{ typeLabel }}
      </span>
      <span v-if="beat.speaker" class="text-xs font-medium text-purple-300">
        {{ beat.speaker }}
      </span>
      <span v-if="beat.emotion" class="text-[10px] text-slate-500 bg-slate-800/50 px-1.5 py-0.5 rounded">
        {{ beat.emotion }}
      </span>

      <!-- 删除按钮 -->
      <button
        @click="$emit('delete')"
        class="ml-auto p-1 rounded opacity-0 group-hover:opacity-100 text-slate-600 hover:text-red-400 hover:bg-red-500/10 transition-all"
        title="删除此节拍"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
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

    <!-- 备选项选择 -->
    <div v-if="beat.alternatives && beat.alternatives.length > 0" class="mt-2 pt-2 border-t border-slate-700/20">
      <div class="text-[10px] text-slate-500 mb-1.5">备选方案 ({{ beat.alternatives.length }})</div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="(alt, altIdx) in beat.alternatives"
          :key="altIdx"
          @click="$emit('select-alt', altIdx)"
          class="px-2 py-1 text-[11px] rounded-md border transition-all"
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
import { ref, computed } from 'vue'
import type { Beat } from '~/stores/scriptStore'

interface Props {
  sceneId: number
  beatIndex: number
  beat: Beat
  isActive?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isActive: false,
})

defineEmits<{
  'update-content': [content: string]
  'select-alt': [altIndex: number]
  delete: []
}>()

const editorRef = ref<HTMLElement>()
const localContent = ref(props.beat.content)

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

// 失去焦点时同步内容
function handleBlur(e: FocusEvent) {
  const target = e.target as HTMLElement
  const newContent = target.textContent?.trim() || ''

  if (newContent !== props.beat.content) {
    localContent.value = newContent
    // 使用 emit 通知父组件更新
    emit('update-content', newContent)
  }
}

function handleInput(e: Event) {
  const target = e.target as HTMLElement
  localContent.value = target.textContent || ''
}

// Enter 键处理：Shift+Enter 换行，普通 Enter 不换行（可选行为）
function handleEnterKeydown(e: KeyboardEvent) {
  if (!e.shiftKey) {
    // 默认允许换行，不做拦截
  }
}

// 监听外部 beat 变化以同步本地内容
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
