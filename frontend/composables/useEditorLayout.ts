import { ref, watchEffect } from 'vue'

const STORAGE_KEY = 'editor-panel-layout'

/** 默认布局配置 */
export const DEFAULT_LAYOUT = {
  leftWidth: 280,
  rightWidth: 320,
  emotionHeight: 80,
} as const

export type PanelLayout = {
  leftWidth: number
  rightWidth: number
  emotionHeight: number
}

/** 限制值在 [min, max] 范围内 */
function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

/**
 * 编辑器面板布局管理（可拖拽调整大小 + localStorage 持久化 + 恢复默认）
 *
 * 支持三个可调区域：
 * - 左侧面板宽度 (leftPanel) — 因果图谱 / 关系网络
 * - 右侧面板宽度 (rightPanel) — 播放器 / YAML 源码
 * - 顶部情感曲线高度 (emotionBar) — EmotionCurve 迷你图
 */
export function useEditorLayout() {
  // 从 localStorage 读取，无则使用默认值
  let stored: string | null = null
  try { stored = localStorage.getItem(STORAGE_KEY) } catch {}

  const raw = stored ? (JSON.parse(stored) as Partial<PanelLayout>) : {}
  const layout = ref<PanelLayout>({
    leftWidth: clamp(raw.leftWidth ?? DEFAULT_LAYOUT.leftWidth, 180, 600),
    rightWidth: clamp(raw.rightWidth ?? DEFAULT_LAYOUT.rightWidth, 200, 700),
    emotionHeight: clamp(raw.emotionHeight ?? DEFAULT_LAYOUT.emotionHeight, 0, 250),
  })

  // 布局变化时自动持久化
  watchEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(layout.value))
    } catch { /* storage full / private mode */ }
  })

  /** 恢复默认布局 */
  function resetToDefaults() {
    layout.value = { ...DEFAULT_LAYOUT }
  }

  /** 更新左侧面板宽度 */
  function setLeftWidth(px: number) {
    layout.value.leftWidth = clamp(Math.round(px), 180, 600)
  }

  /** 更新右侧面板宽度 */
  function setRightWidth(px: number) {
    layout.value.rightWidth = clamp(Math.round(px), 200, 700)
  }

  /** 更新情感曲线高度（0 表示隐藏） */
  function setEmotionHeight(px: number) {
    layout.value.emotionHeight = clamp(Math.round(px), 0, 250)
  }

  return {
    layout,        // reactive { leftWidth, rightWidth, emotionHeight }
    resetToDefaults,
    setLeftWidth,
    setRightWidth,
    setEmotionHeight,
  }
}
