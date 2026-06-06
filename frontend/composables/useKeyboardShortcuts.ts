import { ref, reactive, onMounted, onUnmounted, watch, type Ref } from 'vue'

/** 快捷键组合描述 */
export interface ShortcutBinding {
  /** 主键，如 's', 'Enter', 'Escape' */
  key: string
  /** 是否需要 Ctrl（Mac 上映射为 Meta） */
  ctrl?: boolean
  /** 是否需要 Shift */
  shift?: boolean
  /** 是否需要 Alt */
  alt?: boolean
}

/** 已注册的快捷键条目 */
interface ShortcutEntry {
  id: string
  name: string
  binding: ShortcutBinding
  handler: () => void | Promise<void>
  /** 是否启用 */
  enabled: Ref<boolean>
  /** 防抖间隔 (ms)，0 表示不防抖 */
  debounceMs: number
  /** 是否在输入框/文本域中仍然触发（默认 false） */
  allowInInput: boolean
}

/** 用户自定义快捷键配置（持久化到 localStorage） */
interface ShortcutConfig {
  [shortcutId: string]: ShortcutBinding | null  // null = 禁用
}

const STORAGE_KEY = 'keyboard-shortcuts-config'

// ==================== 内部状态 ====================

const entries = new Map<string, ShortcutEntry>()
let debounceTimers = new Map<string, ReturnType<typeof setTimeout>>()

/** 当前正在保存中的快捷键 ID（用于防重复触发） */
const activeSavingIds = new Set<string>()

/**
 * 将 KeyboardEvent 解析为标准化的 ShortcutBinding
 */
function eventToBinding(e: KeyboardEvent): ShortcutBinding {
  return {
    key: e.key.toLowerCase(),
    ctrl: e.ctrlKey || e.metaKey,
    shift: e.shiftKey,
    alt: e.altKey,
  }
}

/**
 * 判断两个绑定是否匹配
 */
function bindingsMatch(a: ShortcutBinding, b: ShortcutBinding): boolean {
  return (
    a.key === b.key &&
    !!a.ctrl === !!b.ctrl &&
    !!a.shift === !!b.shift &&
    !!a.alt === !!b.alt
  )
}

/**
 * 将绑定转换为可读字符串，如 "Ctrl+S"
 */
export function bindingToString(binding: ShortcutBinding): string {
  const parts: string[] = []
  if (binding.ctrl) parts.push('Ctrl')
  if (binding.alt) parts.push('Alt')
  if (binding.shift) parts.push('Shift')
  // 首字母大写显示
  const displayKey = binding.key.length === 1 ? binding.key.toUpperCase() : binding.key
  parts.push(displayKey)
  return parts.join('+')
}

/**
 * 从用户输入字符串解析绑定，如 "ctrl+shift+s" → { key:'s', ctrl:true, shift:true }
 */
export function parseBindingString(str: string): ShortcutBinding | null {
  const trimmed = str.trim()
  if (!trimmed || trimmed === 'disabled' || trimmed === 'none') return null

  const parts = trimmed.toLowerCase().split('+').map(p => p.trim()).filter(Boolean)
  let key = ''
  let ctrl = false
  let shift = false
  let alt = false

  for (const part of parts) {
    if (part === 'ctrl' || part === 'control' || part === 'meta' || part === 'cmd') {
      ctrl = true
    } else if (part === 'shift') {
      shift = true
    } else if (part === 'alt' || part === 'option') {
      alt = true
    } else {
      key = part
    }
  }

  if (!key) return null
  return { key, ctrl, shift, alt }
}

/**
 * 检查事件目标是否为文本输入元素
 */
function isInputTarget(e: EventTarget | null): boolean {
  if (!e) return false
  const tag = (e as HTMLElement).tagName
  const editable = (e as HTMLElement).isContentEditable
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || editable
}

// ==================== 配置持久化 ====================

function loadConfig(): ShortcutConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch { /* ignore */ }
  return {}
}

function saveConfig(config: ShortcutConfig) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
  } catch { /* ignore */ }
}

let userConfig = loadConfig()

// ==================== 核心：全局键盘监听 ====================

function handleGlobalKeydown(e: KeyboardEvent) {
  // 跳过已处理的事件
  if (e.defaultPrevented) return

  const eventBinding = eventToBinding(e)

  for (const [id, entry] of entries) {
    // 检查是否禁用
    if (!entry.enabled.value) continue

    // 使用用户自定义绑定或默认绑定
    const effectiveBinding = userConfig[id] || entry.binding
    if (!effectiveBinding) continue // 用户显式禁用

    // 匹配检查
    if (!bindingsMatch(eventBinding, effectiveBinding)) continue

    // 输入框检查：如果快捷键不允许在输入框中触发，且当前焦点在输入框中，则跳过
    if (!entry.allowInInput && isInputTarget(e.target)) continue

    // ===== 执行！ =====
    e.preventDefault()
    e.stopPropagation()

    // 防抖 + 并发保护
    if (activeSavingIds.has(id)) return
    if (entry.debounceMs > 0) {
      const existing = debounceTimers.get(id)
      if (existing) return // 防抖窗口内，忽略
    }

    // 设置防抖计时器
    if (entry.debounceMs > 0) {
      const timer = setTimeout(() => {
        debounceTimers.delete(id)
      }, entry.debounceMs)
      debounceTimers.set(id, timer)
    }

    // 标记为活跃（防止并发）
    activeSavingIds.add(id)

    // 异步执行 handler，完成后解除并发标记
    Promise.resolve(entry.handler()).finally(() => {
      activeSavingIds.delete(id)
    })

    break // 一个事件只匹配一个快捷键
  }
}

// ==================== 公共 API ====================

/**
 * 注册一个全局快捷键
 *
 * @param id       唯一标识符，如 'save'
 * @param name     显示名称，如 '保存文档'
 * @param binding  默认按键绑定
 * @param handler  触发时执行的回调
 * @param options  可选配置
 */
export function registerShortcut(
  id: string,
  name: string,
  binding: ShortcutBinding,
  handler: () => void | Promise<void>,
  options?: {
    debounceMs?: number
    allowInInput?: boolean
    defaultEnabled?: boolean
  },
) {
  const entry: ShortcutEntry = {
    id,
    name,
    binding,
    handler,
    enabled: ref(options?.defaultEnabled !== false),
    debounceMs: options?.debounceMs ?? 1000,
    allowInInput: options?.allowInInput ?? true, // 默认在输入框中也触发保存
  }

  entries.set(id, entry)
}

/**
 * 注销快捷键
 */
export function unregisterShortcut(id: string) {
  entries.delete(id)
  const timer = debounceTimers.get(id)
  if (timer) {
    clearTimeout(timer)
    debounceTimers.delete(id)
  }
  activeSavingIds.delete(id)
}

/**
 * 获取用户对指定快捷键的自定义绑定（null 表示用户禁用了）
 */
export function getUserBinding(id: string): ShortcutBinding | null {
  return userConfig[id] ?? null
}

/**
 * 设置用户自定义绑定（传 null 禁用）
 */
export function setUserBinding(id: string, binding: ShortcutBinding | null) {
  if (binding === null) {
    userConfig[id] = null
  } else {
    userConfig[id] = binding
  }
  saveConfig(userConfig)
}

/**
 * 重置指定快捷键为默认绑定
 */
export function resetUserBinding(id: string) {
  delete userConfig[id]
  saveConfig(userConfig)
}

/**
 * 获取所有已注册快捷键的信息列表（用于设置界面展示）
 */
export function getRegisteredShortcuts() {
  return Array.from(entries.values()).map(entry => ({
    id: entry.id,
    name: entry.name,
    defaultBinding: entry.binding,
    userBinding: userConfig[entry.id] ?? null,
    enabled: entry.enabled,
  }))
}

/**
 * 检测快捷键冲突：返回与给定绑定冲突的已注册快捷键 ID 列表
 */
export function detectConflicts(
  binding: ShortcutBinding | null,
  excludeId?: string,
): string[] {
  if (!binding) return []
  const conflicts: string[] = []
  for (const [id, entry] of entries) {
    if (id === excludeId) continue
    const effective = userConfig[id] || entry.binding
    if (effective && bindingsMatch(binding, effective)) {
      conflicts.push(id)
    }
  }
  return conflicts
}

// ==================== Composable ====================

/**
 * 键盘快捷键管理 Composable
 *
 * 在组件中调用 useKeyboardShortcuts() 以激活全局键盘监听。
 * 生命周期内自动管理事件绑定。
 */
export function useKeyboardShortcuts() {
  onMounted(() => {
    document.addEventListener('keydown', handleGlobalKeydown, true) // capture 阶段，优先于组件内处理
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleGlobalKeydown, true)
  })

  return {
    registerShortcut,
    unregisterShortcut,
    getUserBinding,
    setUserBinding,
    resetUserBinding,
    getRegisteredShortcuts,
    detectConflicts,
    bindingToString,
    parseBindingString,
  }
}
