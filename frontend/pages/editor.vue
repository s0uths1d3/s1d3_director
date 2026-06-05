<template>
  <div class="h-screen flex flex-col bg-[var(--bg-dark)] overflow-hidden">
    <!-- 顶部工具栏 -->
    <header class="flex-shrink-0 border-b border-slate-700/50 bg-slate-900/80 backdrop-blur-sm px-4 py-2">
      <div class="flex items-center justify-between">
        <!-- 左侧：标题 + 操作按钮 -->
        <div class="flex items-center gap-3">
          <NuxtLink to="/" class="flex items-center gap-2 text-slate-400 hover:text-white transition-colors">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </NuxtLink>
          <h1 class="text-lg font-semibold text-white truncate max-w-xs">{{ scriptData?.metadata?.title || '剧本编辑器' }}</h1>
          <span v-if="scriptData?.metadata?.style" class="px-2 py-0.5 text-xs font-medium bg-indigo-500/20 text-indigo-300 rounded-full">
            {{ styleLabel }}
          </span>
        </div>

        <!-- 中间：操作按钮组 -->
        <div class="flex items-center gap-2">
          <button
            @click="handleSave"
            :disabled="isSaving"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-700/60 hover:bg-slate-600/60 text-slate-200 text-sm transition-colors disabled:opacity-50"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
            </svg>
            {{ isSaving ? '保存中...' : '保存' }}
          </button>

          <label class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-700/60 hover:bg-slate-600/60 text-slate-200 text-sm cursor-pointer transition-colors">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            导入
            <input type="file" accept=".yaml,.yml,.txt" class="hidden" @change="handleImport" />
          </label>

          <button
            @click="showSettings = !showSettings"
            class="p-1.5 rounded-lg bg-slate-700/60 hover:bg-slate-600/60 text-slate-300 transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>

          <button
            @click="scriptStore.togglePlay()"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all"
            :class="scriptStore.isPlaying
              ? 'bg-red-500/20 text-red-400 hover:bg-red-500/30'
              : 'bg-gradient-to-r from-indigo-600 to-purple-600 text-white hover:from-indigo-500 hover:to-purple-500'"
          >
            <svg v-if="!scriptStore.isPlaying" class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z" />
            </svg>
            <svg v-else class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
            </svg>
            {{ scriptStore.isPlaying ? '停止播放' : '播放模式' }}
          </button>
        </div>

        <!-- 右侧：状态指示 -->
        <div class="flex items-center gap-3 text-xs text-slate-500">
          <span>{{ scriptStore.scenes.length }} 场景</span>
          <span>{{ scriptStore.totalBeats }} 节拍</span>
        </div>
      </div>
    </header>

    <!-- 情感曲线迷你图 -->
    <div v-if="emotionalCurveData" class="flex-shrink-0 h-20 border-b border-slate-700/30 bg-slate-900/40 px-4">
      <EmotionCurve :data="emotionalCurveData" compact />
    </div>

    <!-- 三栏主体布局 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 左侧面板：因果图谱 / 关系网络 (280px) -->
      <aside class="w-[280px] flex-shrink-0 border-r border-slate-700/50 bg-slate-900/30 overflow-hidden flex flex-col">
        <!-- 选项卡切换 -->
        <div class="flex-shrink-0 flex border-b border-slate-700/50">
          <button
            @click="leftTab = 'causal'"
            class="flex-1 px-3 py-2 text-xs font-medium transition-colors"
            :class="leftTab === 'causal'
              ? 'text-indigo-400 border-b-2 border-indigo-400 bg-indigo-500/5'
              : 'text-slate-400 hover:text-slate-200'"
          >
            因果图谱
          </button>
          <button
            @click="leftTab = 'relation'"
            class="flex-1 px-3 py-2 text-xs font-medium transition-colors"
            :class="leftTab === 'relation'
              ? 'text-purple-400 border-b-2 border-purple-400 bg-purple-500/5'
              : 'text-slate-400 hover:text-slate-200'"
          >
            关系网络
          </button>
        </div>

        <!-- 图谱内容区 -->
        <div class="flex-1 overflow-auto p-3">
          <CausalGraph v-if="leftTab === 'causal'" />
          <RelationNetwork v-else />
        </div>
      </aside>

      <!-- 中间区域：场景卡片列表（自适应宽度） -->
      <main class="flex-1 overflow-auto p-4 space-y-4">
        <template v-if="loadStatus === 'loaded'" v-for="scene in scriptStore.scenes" :key="scene.id">
          <!-- 场景头部 -->
          <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl border border-slate-700/50 overflow-hidden">
            <div class="px-4 py-3 border-b border-slate-700/30 flex items-center justify-between">
              <div class="flex items-center gap-3">
                <span class="text-xs font-mono text-indigo-400">SCENE {{ scene.id }}</span>
                <span class="text-sm font-medium text-white">{{ scene.location }}</span>
                <span v-if="scene.time" class="text-xs text-slate-500">· {{ scene.time }}</span>
                <span class="px-1.5 py-0.5 text-xs rounded bg-slate-700/50 text-slate-400">
                  情绪 {{ scene.emotion_intensity.toFixed(1) }}
                </span>
              </div>
              <div v-if="scene.media_hints" class="flex items-center gap-2 text-xs text-slate-500">
                <span v-if="scene.media_hints.camera" title="镜头提示">
                  📷 {{ scene.media_hints.camera }}
                </span>
                <span v-if="scene.media_hints.music" title="音乐提示">
                  🎵 {{ scene.media_hints.music }}
                </span>
              </div>
            </div>

            <!-- Beat 卡片列表 -->
            <div class="divide-y divide-slate-700/20">
              <BeatCard
                v-for="(beat, idx) in scene.beats"
                :key="idx"
                :scene-id="scene.id"
                :beat-index="idx"
                :beat="beat"
                :is-active="scriptStore.currentSceneId === scene.id && scriptStore.currentBeatIndex === idx"
                @update-content="(content) => handleUpdateBeat(scene.id, idx, content)"
                @select-alt="(altIdx) => handleSelectAlt(scene.id, idx, altIdx)"
                @delete="handleDeleteBeat(scene.id, idx)"
              />

              <!-- 添加节拍按钮 -->
              <button
                @click="handleAddBeat(scene.id)"
                class="w-full px-4 py-2.5 text-xs text-slate-500 hover:text-indigo-400 hover:bg-slate-700/20 transition-colors flex items-center justify-center gap-1"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                </svg>
                添加节拍
              </button>
            </div>
          </div>
        </template>

        <!-- 状态展示区：加载中 / 已加载 / 空状态 / 错误 -->
        <div v-if="loadStatus === 'loading'" class="flex flex-col items-center justify-center h-full text-slate-400">
          <svg class="animate-spin w-12 h-12 mb-4 text-indigo-400" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p class="text-sm font-medium">正在加载剧本数据...</p>
          <p class="text-xs mt-1 text-slate-500">正在解析 YAML 并初始化编辑器</p>
        </div>

        <div v-else-if="loadStatus === 'error'" class="flex flex-col items-center justify-center h-full">
          <div class="w-16 h-16 mb-4 rounded-full bg-red-500/10 flex items-center justify-center">
            <svg class="w-8 h-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
          </div>
          <p class="text-sm font-medium text-red-300">剧本数据解析失败</p>
          <p class="text-xs mt-1 text-slate-500 max-w-xs text-center">{{ loadError }}</p>
          <div class="mt-4 flex gap-3">
            <NuxtLink to="/" class="px-4 py-2 rounded-lg text-sm bg-slate-700/60 hover:bg-slate-600/60 text-slate-300 transition-colors">
              返回首页
            </NuxtLink>
            <button @click="retryLoad" class="px-4 py-2 rounded-lg text-sm bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 transition-colors">
              重试加载
            </button>
          </div>
        </div>

        <div v-else-if="loadStatus === 'empty'" class="flex flex-col items-center justify-center h-full text-slate-500">
          <svg class="w-16 h-16 mb-4 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
          </svg>
          <p class="text-sm">暂无剧本数据</p>
          <p class="text-xs mt-1">请先在首页生成或导入剧本</p>
          <NuxtLink to="/" class="mt-3 inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 transition-colors">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
            返回首页生成剧本
          </NuxtLink>
        </div>
      </main>

      <!-- 右侧面板：剧本播放器 / YAML源码编辑器 (320px) -->
      <aside class="w-[320px] flex-shrink-0 border-l border-slate-700/50 bg-slate-900/30 overflow-hidden flex flex-col">
        <!-- 选项卡切换 -->
        <div class="flex-shrink-0 flex border-b border-slate-700/50">
          <button
            @click="rightTab = 'player'"
            class="flex-1 px-3 py-2 text-xs font-medium transition-colors"
            :class="rightTab === 'player'
              ? 'text-emerald-400 border-b-2 border-emerald-400 bg-emerald-500/5'
              : 'text-slate-400 hover:text-slate-200'"
          >
            播放器
          </button>
          <button
            @click="rightTab = 'source'"
            class="flex-1 px-3 py-2 text-xs font-medium transition-colors"
            :class="rightTab === 'source'
              ? 'text-amber-400 border-b-2 border-amber-400 bg-amber-500/5'
              : 'text-slate-400 hover:text-slate-200'"
          >
            YAML 源码
          </button>
        </div>

        <!-- 内容区 -->
        <div class="flex-1 overflow-auto">
          <Player v-if="rightTab === 'player'" />
          <div v-else class="p-3 h-full">
            <textarea
              v-model="yamlSource"
              spellcheck="false"
              class="w-full h-full min-h-[400px] bg-slate-900/70 border border-slate-700/50 rounded-lg p-3 text-xs text-slate-300 font-mono resize-none focus:outline-none focus:ring-1 focus:ring-indigo-500/50"
              placeholder="YAML 源码将在此显示..."
            ></textarea>
          </div>
        </div>
      </aside>
    </div>

    <!-- 右下角浮动聊天窗口 -->
    <CopilotChat />

    <!-- 设置弹窗 -->
    <Teleport to="body">
      <div v-if="showSettings" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="showSettings = false">
        <div class="bg-slate-800 rounded-xl border border-slate-700/50 w-full max-w-md p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-white mb-4">设置</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-slate-300 mb-1.5">DeepSeek API Key</label>
              <input
                v-model="deepSeek.apiKey"
                type="password"
                placeholder="sk-..."
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              />
            </div>
            <div>
              <label class="block text-sm text-slate-300 mb-1.5">Base URL</label>
              <input
                v-model="deepSeek.baseUrl"
                placeholder="https://api.deepseek.com/v1"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
              />
            </div>
          </div>
          <div class="mt-6 flex justify-end gap-2">
            <button @click="showSettings = false" class="px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors">
              取消
            </button>
            <button
              @click="saveSettings"
              class="px-4 py-2 text-sm bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-colors"
            >
              保存
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, provide, onMounted } from 'vue'
import yaml from 'js-yaml'
import { useScriptStore } from '~/stores/scriptStore'
import { useCopilotStore } from '~/stores/copilotStore'
import { useGraphStore } from '~/stores/graphStore'
import { useDeepSeekKey } from '~/composables/useDeepSeekKey'

import CausalGraph from '~/components/CausalGraph.vue'
import RelationNetwork from '~/components/RelationNetwork.vue'
import EmotionCurve from '~/components/EmotionCurve.vue'
import Player from '~/components/Player.vue'
import CopilotChat from '~/components/CopilotChat.vue'
import BeatCard from '~/components/BeatCard.vue'

const scriptStore = useScriptStore()
const copilotStore = useCopilotStore()
const graphStore = useGraphStore()
const deepSeek = useDeepSeekKey()

// ==================== UI 状态 ====================
type LoadStatus = 'loading' | 'loaded' | 'empty' | 'error'

const leftTab = ref<'causal' | 'relation'>('causal')
const rightTab = ref<'player' | 'source'>('player')
const showSettings = ref(false)
const isSaving = ref(false)
const yamlSource = ref('')
const loadStatus = ref<LoadStatus>('loading')
const loadError = ref('')

// ==================== 计算属性 ====================
const scriptData = computed(() => scriptStore.scriptData)

const emotionalCurveData = computed(() => {
  const curve = scriptStore.emotionalCurve
  if (!curve) return null
  return {
    chapters: curve.chapters,
    intensities: curve.intensities,
  }
})

const styleLabel = computed(() => {
  const map: Record<string, string> = {
    short_drama: '短剧',
    film: '电影',
    stage: '舞台剧',
  }
  return map[scriptData.value?.metadata?.style || ''] || scriptData.value?.metadata?.style || ''
})

// 全局事件：剧本更新
const scriptUpdateEvent = ref(0)
provide('scriptUpdate', scriptUpdateEvent)

function emitScriptUpdate() {
  scriptUpdateEvent.value++
}

// ==================== YAML 初始化（公共逻辑） ====================

/** 从 YAML 文本解析并初始化编辑器所有数据 */
function initFromYaml(rawYaml: string): boolean {
  try {
    const parsed = yaml.load(rawYaml) as any
    if (!parsed || !parsed.scenes || !parsed.scenes.length) {
      // YAML 存在但无有效场景数据
      return false
    }

    scriptStore.setScriptData(parsed)
    scriptStore.loadYaml(rawYaml)
    yamlSource.value = rawYaml

    // 初始化因果图谱
    if (parsed.causal_graph) {
      const cg = parsed.causal_graph
      const causalNodes = (cg.events || []).map((e: any) => ({
        id: e.id,
        name: e.description || e.id,
        category: e.chapter ? 'chapter_' + e.chapter : undefined,
      }))
      const causalEdges = (cg.edges || []).map((e: any) => ({
        source: e.from,
        target: e.to,
        value: e.strength,
        label: e.type === 'causal' ? '因果' : e.type === 'temporal' ? '时序' : '情感',
        lineStyle: {
          color: e.type === 'causal' ? '#60a5fa' : e.type === 'temporal' ? '#94a3b8' : '#f87171',
          width: Math.min(3, Math.max(1, e.strength * 3)),
        },
      }))
      graphStore.setCausalGraph(causalNodes, causalEdges)
    }

    // 初始化关系网络
    if (parsed.relation_network) {
      const rn = parsed.relation_network
      const charSet = new Set<string>()
      ;(rn.matrix || []).forEach((entry: any) => {
        charSet.add(entry.from)
        charSet.add(entry.to)
      })
      const relationNodes = Array.from(charSet).map(name => ({ id: name, name }))
      const relationEdges = (rn.matrix || []).map((entry: any) => ({
        source: entry.from,
        target: entry.to,
        value: entry.intimacy,
      }))
      graphStore.setRelationNetwork(relationNodes, relationEdges)
    }

    return true
  } catch (e) {
    console.error('YAML 解析失败:', e)
    loadError.value = e instanceof Error ? e.message : '未知解析错误'
    return false
  }
}

// ==================== 数据加载 ====================

async function loadData() {
  loadStatus.value = 'loading'
  loadError.value = ''

  // 使用 requestAnimationFrame 让 loading 状态先渲染，避免闪烁
  await new Promise(resolve => requestAnimationFrame(resolve))

  const rawYaml = sessionStorage.getItem('script_yaml')

  if (!rawYaml || !rawYaml.trim()) {
    loadStatus.value = 'empty'
    return
  }

  const success = initFromYaml(rawYaml)
  loadStatus.value = success ? 'loaded' : (loadError.value ? 'error' : 'empty')
}

function retryLoad() {
  loadData()
}

// 初始化
onMounted(() => {
  loadData()
})

// ==================== 保存 ====================
async function handleSave() {
  isSaving.value = true
  try {
    // 同步 YAML 源码
    const data = scriptStore.scriptData
    if (data) {
      yamlSource.value = yaml.dump(data, { lineWidth: -1, quotingType: '"', forceQuotes: true })
      sessionStorage.setItem('script_yaml', yamlSource.value)
    }
  } catch (e) {
    console.error('保存失败:', e)
  } finally {
    isSaving.value = false
  }
}

// 导入
function handleImport(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = (e) => {
    const text = e.target?.result as string || ''
    if (!text.trim()) return

    // 使用公共初始化函数
    const success = initFromYaml(text)
    if (success) {
      sessionStorage.setItem('script_yaml', text)
      loadStatus.value = 'loaded'
      emitScriptUpdate()
    } else {
      alert(loadError.value || '文件格式错误，无法解析为有效的 YAML 剧本数据')
      if (!loadError.value) loadStatus.value = 'error'
    }
  }
  reader.readAsText(file)
  target.value = '' // 重置 input 以便重复选择同一文件
}

// 设置保存
function saveSettings() {
  deepSeek.saveApiKey(deepSeek.apiKey)
  deepSeek.saveBaseUrl(deepSeek.baseUrl)
  showSettings.value = false
}

// Beat 操作
function handleUpdateBeat(sceneId: number, beatIndex: number, newContent: string) {
  scriptStore.updateBeatContent(sceneId, beatIndex, newContent)
  emitScriptUpdate()
}

function handleSelectAlt(sceneId: number, beatIndex: number, altIndex: number) {
  scriptStore.selectAlternative(sceneId, beatIndex, altIndex)
  emitScriptUpdate()
}

function handleDeleteBeat(sceneId: number, beatIndex: number) {
  if (confirm('确定删除该节拍？')) {
    scriptStore.removeBeat(sceneId, beatIndex)
    emitScriptUpdate()
  }
}

function handleAddBeat(sceneId: number) {
  scriptStore.addBeat(sceneId, {
    type: 'action',
    content: '',
    alternatives: [],
    selected: 0,
  })
  emitScriptUpdate()
}
</script>
