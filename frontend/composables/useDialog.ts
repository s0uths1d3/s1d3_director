import { reactive, readonly } from 'vue'

/**
 * 全局自定义弹窗系统 — 替代原生 alert / confirm / prompt
 *
 * 使用方式：
 *   const dialog = useDialog()
 *   await dialog.alert('操作成功')           // 信息提示弹窗
 *   const ok = await dialog.confirm('确定删除？') // 确认弹窗，返回 boolean
 *   const name = await dialog.prompt('请输入名称', { defaultValue: '未命名' }) // 输入弹窗，返回 string | null
 */

export type DialogType = 'alert' | 'confirm' | 'prompt'

export interface DialogOptions {
  title?: string
  message: string
  type?: DialogType
  confirmText?: string
  cancelText?: string
  /** prompt 模式下的默认值 */
  defaultValue?: string
  /** prompt 模式下的 placeholder */
  placeholder?: string
  /** 弹窗类型样式：info(默认) / success / warning / danger */
  variant?: 'info' | 'success' | 'warning' | 'danger'
}

interface DialogState {
  visible: boolean
  options: DialogOptions
  resolve: ((value: any) => void) | null
}

const state = reactive<DialogState>({
  visible: false,
  options: {
    message: '',
    type: 'alert',
    variant: 'info',
  },
  resolve: null,
})

let idCounter = 0

function show(options: DialogOptions): Promise<any> {
  return new Promise((resolve) => {
    idCounter++
    state.options = {
      ...options,
      type: options.type || 'alert',
      variant: options.variant || 'info',
      confirmText: options.confirmText || (options.type === 'confirm' ? '确定' : '知道了'),
      cancelText: options.cancelText || '取消',
      defaultValue: options.defaultValue || '',
      placeholder: options.placeholder || '',
    }
    state.resolve = resolve
    state.visible = true
  })
}

/** 显示信息提示弹窗（替代 alert） */
async function alert(message: string, options?: Omit<DialogOptions, 'message' | 'type'>): Promise<void> {
  await show({ ...options, message, type: 'alert' })
}

/** 显示确认弹窗（替代 confirm），返回 true（确认）/ false（取消） */
async function confirm(message: string, options?: Omit<DialogOptions, 'message' | 'type'>): Promise<boolean> {
  return show({ ...options, message, type: 'confirm' })
}

/** 显示输入弹窗（替代 prompt），返回输入的字符串或 null（取消） */
async function prompt(message: string, options?: Omit<DialogOptions, 'message' | 'type'>): Promise<string | null> {
  return show({ ...options, message, type: 'prompt' })
}

/** 内部方法：用户点击确认 */
function handleConfirm(value?: any) {
  state.visible = false
  if (state.resolve) {
    if (state.options.type === 'prompt') {
      state.resolve(value ?? '')
    } else {
      state.resolve(true)
    }
    state.resolve = null
  }
}

/** 内部方法：用户点击取消/关闭 */
function handleCancel() {
  state.visible = false
  if (state.resolve) {
    if (state.options.type === 'prompt') {
      state.resolve(null)
    } else {
      state.resolve(false)
    }
    state.resolve = null
  }
}

export function useDialog() {
  return {
    alert,
    confirm,
    prompt,
    /** 当前弹窗状态（供 AppDialog 组件读取） */
    state: readonly(state),
    handleConfirm,
    handleCancel,
  }
}
