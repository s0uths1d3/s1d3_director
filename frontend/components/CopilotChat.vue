<template>
  <!-- 浮动聊天窗口 -->
  <div
    class="fixed bottom-4 right-4 z-40 flex flex-col transition-all duration-300 ease-out origin-bottom-right"
    :class="copilotStore.isOpen ? 'w-[380px] h-[520px]' : 'w-[380px] h-auto'"
  >
    <!-- 折叠状态：悬浮按钮 -->
    <div
      v-if="!copilotStore.isOpen"
      @click="copilotStore.togglePanel()"
      class="cursor-pointer bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 rounded-xl p-3 shadow-lg shadow-indigo-500/20 flex items-center gap-2 transition-all hover:scale-105 active:scale-100"
    >
      <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
      </svg>
      <span class="text-sm font-medium text-white">AI 副编剧</span>
      <span v-if="unreadCount > 0" class="ml-auto w-5 h-5 bg-red-500 rounded-full text-[10px] text-white flex items-center justify-center font-bold">
        {{ unreadCount }}
      </span>
    </div>

    <!-- 展开状态：完整聊天窗口 -->
    <div
      v-else
      class="flex flex-col bg-slate-800/95 backdrop-blur-lg rounded-xl border border-slate-700/60 shadow-2xl overflow-hidden"
    >
      <!-- 头部 -->
      <div class="flex-shrink-0 flex items-center justify-between px-4 py-3 border-b border-slate-700/50 bg-slate-900/50">
        <div class="flex items-center gap-2">
          <div class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></div>
          <span class="text-sm font-medium text-white">AI 副编剧</span>
          <span class="text-[10px] text-slate-500">Copilot</span>
        </div>
        <div class="flex items-center gap-1">
          <button
            @click="clearChat"
            class="p-1.5 rounded text-slate-500 hover:text-slate-300 hover:bg-slate-700/50 transition-colors"
            title="清空对话"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
          <button
            @click="copilotStore.togglePanel()"
            class="p-1.5 rounded text-slate-500 hover:text-slate-300 hover:bg-slate-700/50 transition-colors"
            title="收起"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- 对话消息列表 -->
      <div ref="messageListRef" class="flex-1 overflow-y-auto px-4 py-3 space-y-3 scroll-smooth">
        <!-- 欢迎消息 -->
        <div v-if="copilotStore.messages.length === 0" class="text-center py-8">
          <div class="inline-flex items-center justify-center w-12 h-12 rounded-full bg-indigo-500/10 mb-3">
            <svg class="w-6 h-6 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
          </div>
          <p class="text-sm text-slate-400">你好！我是 AI 副编剧</p>
          <p class="text-xs text-slate-500 mt-1">可以帮你分析角色动机、优化节奏、检查一致性等</p>
        </div>

        <!-- 消息列表 -->
        <div
          v-for="(msg, idx) in copilotStore.messages"
          :key="idx"
          class="flex"
          :class="msg.role === 'user' ? 'justify-end' : 'justify-start'"
        >
          <div
            class="max-w-[85%] rounded-xl px-3.5 py-2.5 text-sm leading-relaxed"
            :class="msg.role === 'user'
              ? 'bg-indigo-600 text-white rounded-br-sm'
              : 'bg-slate-700/70 text-slate-200 rounded-bl-sm'"
          >
            <!-- 简单 Markdown 渲染 -->
            <div v-html="renderMarkdown(msg.content)" class="msg-content"></div>
            <div
              class="text-[10px] mt-1 opacity-50"
              :class="msg.role === 'user' ? 'text-right' : 'text-left'"
            >
              {{ formatTime(msg.timestamp) }}
            </div>
          </div>
        </div>

        <!-- 加载中指示器 -->
        <div v-if="copilotStore.isLoading" class="flex justify-start">
          <div class="bg-slate-700/70 rounded-xl rounded-bl-sm px-4 py-3">
            <div class="flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-bounce" style="animation-delay: 0ms"></span>
              <span class="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-bounce" style="animation-delay: 150ms"></span>
              <span class="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-bounce" style="animation-delay: 300ms"></span>
            </div>
          </div>
        </div>
      </div>

      <!-- 快捷操作按钮 -->
      <div class="flex-shrink-0 px-4 py-2 border-t border-slate-700/30">
        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="action in quickActions"
            :key="action.label"
            @click="sendQuickAction(action)"
            :disabled="copilotStore.isLoading"
            class="px-2.5 py-1 text-[11px] rounded-full border border-slate-600/60 text-slate-400 hover:border-indigo-500/50 hover:text-indigo-300 hover:bg-indigo-500/5 transition-all disabled:opacity-50"
          >
            {{ action.label }}
          </button>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="flex-shrink-0 px-4 py-3 border-t border-slate-700/50 bg-slate-900/30">
        <form @submit.prevent="sendMessage" class="flex items-end gap-2">
          <textarea
            ref="inputRef"
            v-model="inputText"
            placeholder="输入你的问题或指令..."
            rows="1"
            class="flex-1 bg-slate-900/70 border border-slate-700/50 rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 resize-none focus:outline-none focus:ring-1 focus:ring-indigo-500/50 focus:border-transparent max-h-[100px]"
            @keydown.enter.exact.prevent="sendMessage"
            @input="autoResize"
          ></textarea>
          <button
            type="submit"
            :disabled="!inputText.trim() || copilotStore.isLoading"
            class="flex-shrink-0 w-9 h-9 rounded-lg bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white flex items-center justify-center transition-all disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </button>
        </form>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted } from 'vue'
import { useCopilotStore } from '~/stores/copilotStore'

const copilotStore = useCopilotStore()

const inputText = ref('')
const messageListRef = ref<HTMLElement>()
const inputRef = ref<HTMLTextAreaElement>()

// 未读计数（简单实现）
const unreadCount = computed(() => {
  // 当面板关闭时，助手消息数即为未读
  return copilotStore.messages.filter(m => m.role === 'assistant').length
})

// 快捷操作
const quickActions = [
  { label: '分析角色动机',   action: 'analyze_motivation' },
  { label: '优化节奏',       action: 'optimize_pacing' },
  { label: '检查因果一致性', action: 'check_causal_consistency' },
  { label: '生成备选对白',   action: 'generate_dialogue_alt' },
]

// 发送消息
async function sendMessage() {
  const text = inputText.value.trim()
  if (!text || copilotStore.isLoading) return

  inputText.value = ''
  copilotStore.addUserMessage(text)

  await scrollToBottom()
  await callApi(text)
}

// 发送快捷操作
async function sendQuickAction(action: typeof quickActions[number]) {
  if (copilotStore.isLoading) return

  copilotStore.addUserMessage(action.label)
  await scrollToBottom()

  try {
    copilotStore.isLoading = true

    // 获取当前剧本 YAML
    const scriptYaml = typeof window !== 'undefined'
      ? sessionStorage.getItem('script_yaml') || ''
      : ''

    const response = await fetch('/api/co-pilot/suggest', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        script_yaml: scriptYaml,
      }),
    })

    if (!response.ok) throw new Error(`请求失败: ${response.status}`)

    const data = await response.json()

    // 根据动作类型过滤/格式化建议
    let reply = ''
    if (data.suggestions && Array.isArray(data.suggestions)) {
      const relevant = action.action
        ? data.suggestions.filter((s: any) =>
            !s.type || s.type === action.action.replace('check_', '').replace('generate_', '') ||
            s.type === '节奏' || s.type === '角色' || s.type === '对白' || s.type === '结构' ||
            true // 如果类型不匹配就返回全部
          )
        : data.suggestions

      if (relevant.length > 0) {
        reply = `【${action.label}分析结果】\n\n` + relevant.map((s: any) =>
          `- [${s.type || '建议'}] ${s.message}${s.target_scene_id ? ` (场景 ${s.target_scene_id})` : ''}`
        ).join('\n')
      } else {
        reply = '当前剧本整体质量良好，没有发现明显问题。'
      }
    } else {
      reply = data.reply || data.message || `${action.label}处理完成`
    }

    copilotStore.addAssistantMessage(reply)
  } catch (error: any) {
    console.error('快捷操作失败:', error)
    copilotStore.addAssistantMessage(`抱歉，操作失败: ${error.message}`)
  } finally {
    copilotStore.isLoading = false
    await scrollToBottom()
  }
}

// 调用聊天 API
async function callApi(userMessage: string) {
  try {
    copilotStore.isLoading = true

    // 构建消息数组：历史消息 + 当前用户消息
    const apiMessages = [
      ...copilotStore.messages.slice(0, -1).map(m => ({
        role: m.role,
        content: m.content,
      })),
      { role: 'user' as const, content: userMessage },
    ]

    // 获取当前剧本 YAML（从 sessionStorage）
    const scriptYaml = typeof window !== 'undefined'
      ? sessionStorage.getItem('script_yaml') || undefined
      : undefined

    const response = await fetch('/api/co-pilot/chat', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        messages: apiMessages,
        context: getScriptContext(),
        full_script_yaml: scriptYaml,
      }),
    })

    if (!response.ok) throw new Error(`请求失败: ${response.status}`)

    const data = await response.json()
    const reply = data.reply || data.message || data.content || '已收到您的反馈'
    copilotStore.addAssistantMessage(reply)

    // 如果有建议，也显示出来
    if (data.suggestions && Array.isArray(data.suggestions)) {
      for (const s of data.suggestions) {
        if (typeof s === 'string' && s !== reply) {
          copilotStore.addAssistantMessage(`建议: ${s}`)
        }
      }
    }
  } catch (error: any) {
    console.error('API 调用失败:', error)
    copilotStore.addAssistantMessage(`抱歉，服务暂时不可用: ${error.message}`)
  } finally {
    copilotStore.isLoading = false
    await scrollToBottom()
  }
}

// 获取剧本上下文摘要
function getScriptContext(): any {
  // 从 store 获取简要上下文信息
  return {
    scene_count: copilotStore.messages.length > 0 ? undefined : 0,
  }
}

// 清空对话
function clearChat() {
  if (confirm('确定清空所有对话记录？')) {
    copilotStore.clearMessages()
  }
}

// 滚动到底部
async function scrollToBottom() {
  await nextTick()
  if (messageListRef.value) {
    messageListRef.value.scrollTop = messageListRef.value.scrollHeight
  }
}

// 自动调整输入框高度
function autoResize(e: Event) {
  const target = e.target as HTMLTextAreaElement
  target.style.height = 'auto'
  target.style.height = Math.min(target.scrollHeight, 100) + 'px'
}

// 时间格式化
function formatTime(timestamp: number): string {
  const d = new Date(timestamp)
  return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`
}

// 简易 Markdown 渲染（支持加粗、代码块、列表）
function renderMarkdown(text: string): string {
  let html = text
    // 代码块
    .replace(/```(\w*)\n?([\s\S]*?)```/g, '<pre class="bg-slate-900/60 rounded p-2 my-1 text-xs overflow-x-auto"><code>$2</code></pre>')
    // 行内代码
    .replace(/`([^`]+)`/g, '<code class="bg-slate-900/60 px-1 rounded text-xs">$1</code>')
    // 加粗
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    // 斜体
    .replace(/\*([^*]+)\*/g, '<em>$1</em>')
    // 标题
    .replace(/^### (.+)$/gm, '<strong class="text-base">$1</strong>')
    .replace(/^## (.+)$/gm, '<strong class="text-base text-indigo-300">$1</strong>')
    // 无序列表
    .replace(/^- (.+)$/gm, '<li class="ml-2 list-disc">$1</li>')
    // 有序列表
    .replace(/^\d+\. (.+)$/gm, '<li class="ml-2 list-decimal">$1</li>')
    // 换行
    .replace(/\n/g, '<br/>')

  return html
}

onMounted(() => {
  // 默认展开
  if (copilotStore.messages.length === 0) {
    // 保持折叠状态
  }
})
</script>

<style scoped>
.msg-content :deep(pre) {
  white-space: pre-wrap;
  word-break: break-word;
}
.msg-content :deep(li) {
  display: list-item;
}
</style>
