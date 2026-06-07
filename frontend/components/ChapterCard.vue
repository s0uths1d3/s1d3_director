<template>
  <div
    class="chapter-card bg-slate-800/30 rounded-xl border border-slate-700/40 overflow-hidden transition-colors duration-200"
    :class="{ 'border-indigo-500/30': isExpanded }"
  >
    <!-- ===== 章节头部（可点击切换收起/展开） ===== -->
    <div
      ref="headerRef"
      role="button"
      :aria-expanded="isExpanded"
      :aria-controls="`chapter-content-${chapterId}`"
      tabindex="0"
      @click="toggleExpand"
      @keydown.enter.prevent="toggleExpand"
      @keydown.space.prevent="toggleExpand"
      class="px-4 py-3 cursor-pointer select-none hover:bg-slate-700/20 transition-colors group/header"
    >
      <div class="flex items-center justify-between">
        <!-- 左侧：章节标识 + 标题 + 摘要 -->
        <div class="flex items-center gap-3 min-w-0">
          <!-- 折叠箭头 -->
          <svg
            class="w-4 h-4 text-indigo-400 transition-transform duration-300 ease-out flex-shrink-0"
            :class="{ 'rotate-90': isExpanded }"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M9 5l7 7-7 7" />
          </svg>

          <!-- 章节序号标签 -->
          <span class="text-[11px] font-bold text-indigo-400 bg-indigo-500/15 px-2 py-0.5 rounded shrink-0">
            第{{ chapterIndex }}章
          </span>

          <!-- 标题 -->
          <h3 class="text-sm font-semibold text-white truncate">{{ title }}</h3>
        </div>

        <!-- 右侧：元信息 + 操作 -->
        <div class="flex items-center gap-2 shrink-0">
          <!-- 情节线标签 -->
          <span v-if="plotLine" class="text-[10px] px-2 py-0.5 rounded-full bg-indigo-500/15 text-indigo-300">
            {{ plotLine }}
          </span>

          <!-- 统计信息 -->
          <span class="text-[10px] text-slate-500 tabular-nums">
            {{ sceneCount }} 场景 · {{ totalBeats }} 节拍
          </span>

          <!-- 收起/展开文字提示 -->
          <span class="text-[10px] text-slate-600 group-hover/header:text-slate-400 transition-colors">
            {{ isExpanded ? '收起' : '展开' }}
          </span>
        </div>
      </div>

      <!-- 摘要（始终显示，收起时截断） -->
      <p
        v-if="summary"
        class="text-[11px] text-slate-500 mt-1.5 line-clamp-2 pl-8 transition-all duration-300"
        :class="{ 'line-clamp-1': !isExpanded }"
      >
        {{ summary }}
      </p>
    </div>

    <!-- ===== 内容区（收起时完全隐藏，展开时平滑显示） ===== -->
    <div
      :id="`chapter-content-${chapterId}`"
      role="region"
      :aria-label="`${title} 的场景内容`"
      :aria-hidden="!isExpanded"
      ref="contentRef"
      class="overflow-hidden transition-all duration-300 ease-out"
      :style="{ height: isExpanded ? contentHeightPx : '0px', opacity: isExpanded ? 1 : 0 }"
    >
      <div ref="innerContentRef" class="pt-3 space-y-4">
        <!-- 仅展开时渲染内容 -->
        <slot v-if="isExpanded" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'

interface Props {
  /** 章节唯一 ID */
  chapterId: string
  /** 章节标题 */
  title: string
  /** 章节序号（用于显示 "第N章"） */
  chapterIndex: number
  /** 章节摘要 */
  summary?: string
  /** 情节线名称 */
  plotLine?: string
  /** 该章包含的场景数量 */
  sceneCount: number
  /** 该章的总节拍数 */
  totalBeats: number
  /** 默认是否展开（默认 true） */
  defaultExpanded?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  summary: '',
  plotLine: '',
  defaultExpanded: true,
})

const emit = defineEmits<{
  'toggle': [expanded: boolean]
}>()

// ==================== 状态 ====================

const isExpanded = ref(props.defaultExpanded)
const contentRef = ref<HTMLElement>()
const innerContentRef = ref<HTMLElement>()
const contentHeightPx = ref('0px')
let resizeObserver: ResizeObserver | null = null

// ==================== 方法 ====================

/** 切换展开/收起 */
function toggleExpand() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value) {
    nextTick(measureContentHeight)
  }
  emit('toggle', isExpanded.value)
}

/** 测量内容实际高度 */
function measureContentHeight() {
  nextTick(() => {
    if (innerContentRef.value) {
      contentHeightPx.value = innerContentRef.value.scrollHeight + 'px'
    }
  })
}

// ==================== 生命周期 ====================

onMounted(() => {
  if (isExpanded.value) measureContentHeight()
  window.addEventListener('resize', onResize)

  // 用 ResizeObserver 监听内容区域尺寸变化（SceneCard 内容动态改变时自动更新高度）
  if (innerContentRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (isExpanded.value) measureContentHeight()
    })
    resizeObserver.observe(innerContentRef.value)
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
})

let resizeTimer: ReturnType<typeof setTimeout> | null = null
function onResize() {
  if (resizeTimer) clearTimeout(resizeTimer)
  resizeTimer = setTimeout(() => {
    if (isExpanded.value) measureContentHeight()
  }, 150)
}

watch(
  () => [props.sceneCount, props.totalBeats],
  () => { if (isExpanded.value) measureContentHeight() }
)

watch(isExpanded, (val) => {
  if (val) nextTick(measureContentHeight)
})

// innerContentRef 可能在 mount 时为空（异步 slot），watch 补充设置
watch(innerContentRef, (el) => {
  if (el && !resizeObserver) {
    resizeObserver = new ResizeObserver(() => {
      if (isExpanded.value) measureContentHeight()
    })
    resizeObserver.observe(el)
    if (isExpanded.value) measureContentHeight()
  }
}, { immediate: true })
</script>

<style scoped>
/* 键盘焦点样式 */
[role="button"]:focus-visible {
  outline: 2px solid rgba(99, 102, 241, 0.5);
  outline-offset: 2px;
  border-radius: 4px;
}
</style>
