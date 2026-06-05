import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
  timestamp: number
}

export interface Suggestion {
  type: string
  message: string
  target_scene_id?: number
}

export const useCopilotStore = defineStore('copilot', () => {
  // 状态
  const messages = ref<ChatMessage[]>([])
  const isLoading = ref(false)
  const suggestions = ref<Suggestion[]>([])
  const isOpen = ref(false)

  // 方法：添加用户消息
  function addUserMessage(content: string) {
    messages.value.push({
      role: 'user',
      content,
      timestamp: Date.now(),
    })
  }

  // 方法：添加助手回复
  function addAssistantMessage(content: string) {
    messages.value.push({
      role: 'assistant',
      content,
      timestamp: Date.now(),
    })
  }

  // 方法：设置建议列表
  function setSuggestions(list: Suggestion[]) {
    suggestions.value = list
  }

  // 方法：清空聊天记录
  function clearMessages() {
    messages.value = []
  }

  // 方法：切换面板开关
  function togglePanel() {
    isOpen.value = !isOpen.value
  }

  return {
    messages,
    isLoading,
    suggestions,
    isOpen,
    addUserMessage,
    addAssistantMessage,
    setSuggestions,
    clearMessages,
    togglePanel,
  }
})
