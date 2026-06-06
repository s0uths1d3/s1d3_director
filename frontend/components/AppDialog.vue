<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div
        v-if="dialogState.visible"
        class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50 backdrop-blur-sm"
        @click.self="dialog.handleCancel()"
        @keydown.esc="dialog.handleCancel()"
      >
        <!-- 弹窗主体 -->
        <div
          class="bg-slate-800 rounded-xl border border-slate-700/50 w-full max-w-md p-6 shadow-xl transform transition-all"
          @click.stop
        >
          <!-- 图标 + 标题区域 -->
          <div class="flex items-start gap-3 mb-4">
            <!-- 图标 -->
            <div
              class="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center"
              :class="iconBgClass"
            >
              <!-- info / success / warning / danger 图标 -->
              <svg v-if="variant === 'success'" class="w-5 h-5" :class="iconTextClass" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <svg v-else-if="variant === 'danger'" class="w-5 h-5" :class="iconTextClass" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <svg v-else-if="variant === 'warning'" class="w-5 h-5" :class="iconTextClass" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <svg v-else class="w-5 h-5" :class="iconTextClass" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>

            <!-- 标题 + 消息 -->
            <div class="flex-1 min-w-0">
              <h3 v-if="dialogState.options.title" class="text-base font-semibold text-white leading-snug">
                {{ dialogState.options.title }}
              </h3>
              <p
                class="text-sm leading-relaxed break-words whitespace-pre-wrap"
                :class="dialogState.options.title ? 'text-slate-300 mt-1' : 'text-slate-200'"
              >{{ dialogState.options.message }}</p>
            </div>
          </div>

          <!-- prompt 输入框 -->
          <div v-if="dialogState.options.type === 'prompt'" class="mb-4">
            <input
              ref="inputRef"
              v-model="promptValue"
              type="text"
              :placeholder="dialogState.options.placeholder || '请输入...'"
              maxlength="200"
              class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
              @keyup.enter="onConfirm"
            />
          </div>

          <!-- 按钮组 -->
          <div class="flex justify-end gap-2 mt-2">
            <!-- 取消按钮（confirm 和 prompt 模式显示） -->
            <button
              v-if="dialogState.options.type !== 'alert'"
              @click="dialog.handleCancel()"
              class="px-4 py-2 text-sm text-slate-400 hover:text-white hover:bg-slate-700/50 rounded-lg transition-colors"
            >
              {{ dialogState.options.cancelText }}
            </button>

            <!-- 确认按钮 -->
            <button
              ref="confirmBtnRef"
              @click="onConfirm"
              class="px-4 py-2 text-sm rounded-lg font-medium transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-800"
              :class="confirmButtonClass"
            >
              {{ dialogState.options.confirmText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useDialog } from '~/composables/useDialog'

const dialog = useDialog()
const dialogState = dialog.state

const inputRef = ref<HTMLInputElement>()
const confirmBtnRef = ref<HTMLButtonElement>()
const promptValue = ref('')

// 当前变体样式
const variant = computed(() => dialogState.options.variant || 'info')

const iconBgClass = computed(() => ({
  'bg-indigo-500/20': variant.value === 'info',
  'bg-emerald-500/20': variant.value === 'success',
  'bg-amber-500/20': variant.value === 'warning',
  'bg-red-500/20': variant.value === 'danger',
}))

const iconTextClass = computed(() => ({
  'text-indigo-400': variant.value === 'info',
  'text-emerald-400': variant.value === 'success',
  'text-amber-400': variant.value === 'warning',
  'text-red-400': variant.value === 'danger',
}))

const confirmButtonClass = computed(() => {
  const base = 'text-white'
  const variants: Record<string, string> = {
    info: 'bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 focus:ring-indigo-500/50',
    success: 'bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 focus:ring-emerald-500/50',
    warning: 'bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-500 hover:to-orange-500 focus:ring-amber-500/50',
    danger: 'bg-gradient-to-r from-red-600 to-rose-600 hover:from-red-500 hover:to-rose-500 focus:ring-red-500/50',
  }
  return `${base} ${variants[variant.value] || variants.info}`
})

// 弹窗打开时自动聚焦
watch(() => dialogState.visible, async (visible) => {
  if (visible) {
    // 初始化 prompt 值
    if (dialogState.options.type === 'prompt') {
      promptValue.value = dialogState.options.defaultValue || ''
      await nextTick()
      inputRef.value?.focus()
      inputRef.value?.select()
    } else {
      await nextTick()
      confirmBtnRef.value?.focus()
    }
  }
})

function onConfirm() {
  dialog.handleConfirm(promptValue.value)
}
</script>

<style scoped>
/* 弹窗淡入淡出动画 */
.dialog-fade-enter-active {
  transition: opacity 0.2s ease;
}
.dialog-fade-enter-active > div {
  transition: transform 0.2s ease, opacity 0.2s ease;
}
.dialog-fade-leave-active {
  transition: opacity 0.15s ease;
}
.dialog-fade-leave-active > div {
  transition: transform 0.15s ease, opacity 0.15s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}
.dialog-fade-enter-from > div,
.dialog-fade-leave-to > div {
  transform: scale(0.95);
  opacity: 0;
}
</style>
