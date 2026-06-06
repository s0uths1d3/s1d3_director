import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import yaml from 'js-yaml'

// 类型定义（与后端 YAML Schema 对齐）
export interface BeatAlternative {
  content: string
  tone?: string
}

export interface Beat {
  type: string
  content: string
  alternatives: BeatAlternative[]
  selected: number
  speaker?: string
  emotion?: string
}

export interface MediaHints {
  camera: string
  music: string
}

export interface Scene {
  id: number
  location: string
  time?: string
  emotion_intensity: number
  beats: Beat[]
  media_hints?: MediaHints
}

export interface Character {
  id: string
  name: string
  traits: string[]
  voice?: string
}

export interface EmotionalCurve {
  chapters: number[]
  intensities: number[]
}

export interface ScriptMetadata {
  title: string
  source_novel: string
  adaptation_date: string
  style: string
  emotional_curve?: EmotionalCurve
}

export interface ScriptData {
  metadata: ScriptMetadata
  characters: Character[]
  causal_graph?: any
  relation_network?: any
  scenes: Scene[]
}

/** 单次修改记录 */
export interface ModificationRecord {
  timestamp: number
  target: string // 格式 "scene:{id}" 或 "scene:{id}:beat:{idx}"
  field: string // 'content' | 'type' | 'speaker' | 'emotion' | 'location' | 'time' 等
  oldValue: string
  newValue: string
  source: 'manual' | 'ai' | 'alternative' // 修改来源
}

/** 节拍类型选项 */
export const BEAT_TYPES = [
  { value: 'action', label: '动作', color: 'slate' },
  { value: 'dialogue', label: '对白', color: 'blue' },
  { value: 'monologue', label: '独白', color: 'purple' },
  { value: 'parenthetical', label: '括号说明', color: 'yellow' },
] as const

export const useScriptStore = defineStore('script', () => {
  // 状态
  const yamlText = ref('')
  const scriptData = ref<ScriptData | null>(null)
  const currentSceneId = ref<number | null>(null)
  const currentBeatIndex = ref<number | null>(null)
  const isPlaying = ref(false)
  const playPosition = ref(0) // 当前播放到的全局 beat 索引
  const history = ref<string[]>([]) // YAML 历史版本

  // 修改历史记录
  const modifications = ref<ModificationRecord[]>([])
  // AI 重生成加载状态：key 为 "sceneId:beatIndex" 或 "sceneId"
  const regeneratingKeys = ref<Set<string>>(new Set())

  // 计算属性
  const scenes = computed(() => scriptData.value?.scenes || [])
  const characters = computed(() => scriptData.value?.characters || [])
  const emotionalCurve = computed(() => scriptData.value?.metadata?.emotional_curve || null)
  const totalBeats = computed(() =>
    scenes.value.reduce((sum, scene) => sum + scene.beats.length, 0)
  )

  // 当前选中的场景
  const currentScene = computed(() =>
    scenes.value.find(s => s.id === currentSceneId.value) || null
  )

  // 获取所有 beats 的扁平列表（用于播放器）
  const flatBeats = computed(() => {
    const result: { sceneId: number; beatIndex: number; beat: Beat }[] = []
    scenes.value.forEach(scene => {
      scene.beats.forEach((beat, idx) => {
        result.push({ sceneId: scene.id, beatIndex: idx, beat })
      })
    })
    return result
  })

  // 当前选中的 beat
  const currentBeat = computed(() => {
    if (currentScene.value && currentBeatIndex.value !== null) {
      return currentScene.value.beats[currentBeatIndex.value] || null
    }
    return null
  })

  // 某个目标是否正在 AI 重生成中
  function isRegenerating(key: string): boolean {
    return regeneratingKeys.value.has(key)
  }

  // ==================== 场景编辑方法 ====================

  /** 更新场景基本信息 */
  function updateSceneInfo(sceneId: number, updates: Partial<Pick<Scene, 'location' | 'time' | 'emotion_intensity'>>) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (!scene) return

    for (const [field, newValue] of Object.entries(updates)) {
      const oldVal = String((scene as any)[field] ?? '')
      ;(scene as any)[field] = newValue
      pushModification(`scene:${sceneId}`, field, oldVal, String(newValue), 'manual')
    }
    syncYamlFromData()
  }

  /** 更新场景 media_hints */
  function updateSceneMediaHints(sceneId: number, hints: Partial<MediaHints>) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (!scene) return
    if (!scene.media_hints) {
      scene.media_hints = { camera: '', music: '' }
    }
    Object.assign(scene.media_hints, hints)
    syncYamlFromData()
  }

  /** 删除整个场景 */
  function removeScene(sceneId: number) {
    if (!scriptData.value) return
    const idx = scriptData.value.scenes.findIndex(s => s.id === sceneId)
    if (idx !== -1) {
      scriptData.value.scenes.splice(idx, 1)
      syncYamlFromData()
    }
  }

  /** 添加新场景 */
  function addScene(scene: Omit<Scene, 'id'>): Scene {
    if (!scriptData.value) throw new Error('剧本数据未初始化')
    const maxId = scriptData.value.scenes.reduce((max, s) => Math.max(max, s.id), 0)
    const newScene: Scene = { ...scene, id: maxId + 1 }
    scriptData.value.scenes.push(newScene)
    syncYamlFromData()
    return newScene
  }

  // ==================== 节拍编辑方法 ====================

  /** 更新单个 beat 内容 */
  function updateBeatContent(sceneId: number, beatIndex: number, newContent: string) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const oldVal = scene.beats[beatIndex].content
      scene.beats[beatIndex].content = newContent
      if (oldVal !== newContent) {
        pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'content', oldVal, newContent, 'manual')
      }
      syncYamlFromData()
    }
  }

  /** 更新节拍类型 */
  function updateBeatType(sceneId: number, beatIndex: number, newType: string) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const oldVal = scene.beats[beatIndex].type
      scene.beats[beatIndex].type = newType
      pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'type', oldVal, newType, 'manual')
      syncYamlFromData()
    }
  }

  /** 更新节拍说话人 */
  function updateBeatSpeaker(sceneId: number, beatIndex: number, speaker: string) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const oldVal = scene.beats[beatIndex].speaker || ''
      scene.beats[beatIndex].speaker = speaker || undefined
      pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'speaker', oldVal, speaker, 'manual')
      syncYamlFromData()
    }
  }

  /** 更新节拍情绪 */
  function updateBeatEmotion(sceneId: number, beatIndex: number, emotion: string) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const oldVal = scene.beats[beatIndex].emotion || ''
      scene.beats[beatIndex].emotion = emotion || undefined
      pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'emotion', oldVal, emotion, 'manual')
      syncYamlFromData()
    }
  }

  /** 选择备选项 */
  function selectAlternative(sceneId: number, beatIndex: number, altIndex: number) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const beat = scene.beats[beatIndex]
      if (altIndex >= 0 && altIndex < beat.alternatives.length) {
        const oldVal = beat.content
        beat.selected = altIndex
        beat.content = beat.alternatives[altIndex].content
        pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'content', oldVal, beat.content, 'alternative')
        syncYamlFromData()
      }
    }
  }

  /** AI 重生成结果应用到节拍 — 将生成的内容作为新的备选项加入 */
  function applyAIRegeneration(
    sceneId: number,
    beatIndex: number,
    generatedContent: string,
    alternatives?: BeatAlternative[],
  ) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const beat = scene.beats[beatIndex]
      const oldVal = beat.content

      // 将原内容保存为备选（如果还没有的话）
      const hasOriginalAsAlt = beat.alternatives.some(a => a.content === oldVal)
      if (!hasOriginalAsAlt && oldVal.trim()) {
        beat.alternatives.unshift({ content: oldVal, tone: '原始版本' })
      }

      // 将 AI 生成内容设为主内容
      beat.content = generatedContent
      beat.selected = 0

      // 追加额外的备选项
      if (alternatives && alternatives.length > 0) {
        for (const alt of alternatives) {
          const exists = beat.alternatives.some(a => a.content === alt.content)
          if (!exists) {
            beat.alternatives.push(alt)
          }
        }
      }

      pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'content', oldVal, generatedContent, 'ai')
      syncYamlFromData()

      // 清除重生成状态
      regeneratingKeys.value.delete(`${sceneId}:${beatIndex}`)
    }
  }

  /** AI 重生成整场场景 */
  function applyAISceneRegeneration(
    sceneId: number,
    generatedBeats: Beat[],
  ) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (!scene) return

    // 保存旧节拍作为历史
    const oldBeatsJson = JSON.stringify(scene.beats)

    // 替换所有节拍
    scene.beats = generatedBeats

    pushModification(`scene:${sceneId}`, 'beats', oldBeatsJson, JSON.stringify(generatedBeats), 'ai')
    syncYamlFromData()
    regeneratingKeys.value.delete(String(sceneId))
  }

  /** 设置/清除重生成状态 */
  function setRegenerating(key: string, loading: boolean) {
    if (loading) {
      regeneratingKeys.value.add(key)
    } else {
      regeneratingKeys.value.delete(key)
    }
  }

  /** 添加 beat */
  function addBeat(sceneId: number, beat: Beat) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene) {
      scene.beats.push(beat)
      pushModification(`scene:${sceneId}:beat:${scene.beats.length - 1}`, 'content', '', beat.content || '', 'manual')
      syncYamlFromData()
    }
  }

  /** 删除 beat */
  function removeBeat(sceneId: number, beatIndex: number) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && beatIndex >= 0 && beatIndex < scene.beats.length) {
      const removed = scene.beats.splice(beatIndex, 1)[0]
      pushModification(`scene:${sceneId}:beat:${beatIndex}`, 'content', removed.content || '', '[已删除]', 'manual')
      syncYamlFromData()
    }
  }

  /** 在指定位置插入 beat */
  function insertBeat(sceneId: number, beatIndex: number, beat: Beat) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && beatIndex >= 0 && beatIndex <= scene.beats.length) {
      scene.beats.splice(beatIndex, 0, beat)
      syncYamlFromData()
    }
  }

  /** 上移/下移 beat */
  function moveBeat(sceneId: number, beatIndex: number, direction: 'up' | 'down') {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (!scene) return
    const targetIdx = direction === 'up' ? beatIndex - 1 : beatIndex + 1
    if (targetIdx < 0 || targetIdx >= scene.beats.length) return
    const temp = scene.beats[beatIndex]
    scene.beats[beatIndex] = scene.beats[targetIdx]
    scene.beats[targetIdx] = temp
    syncYamlFromData()
  }

  // ==================== 修改历史 ====================

  /** 推送一条修改记录 */
  function pushModification(target: string, field: string, oldValue: string, newValue: string, source: ModificationRecord['source']) {
    modifications.value.push({
      timestamp: Date.now(),
      target,
      field,
      oldValue,
      newValue,
      source,
    })
    // 最多保留 200 条记录
    if (modifications.value.length > 200) {
      modifications.value.shift()
    }
  }

  /** 获取某个目标的修改历史 */
  function getTargetHistory(target: string): ModificationRecord[] {
    return modifications.value.filter(m => m.target === target).reverse()
  }

  /** 清空修改历史 */
  function clearModifications() {
    modifications.value = []
  }

  // ==================== 数据同步 ====================

  /** 从数据同步到 YAML */
  function syncYamlFromData() {
    if (!scriptData.value) return
    try {
      const newYaml = yaml.dump(scriptData.value, {
        lineWidth: -1,
        quotingType: '"',
        forceQuotes: false,
        indent: 2,
      })
      yamlText.value = newYaml
      // 同步到 sessionStorage
      if (typeof window !== 'undefined') {
        sessionStorage.setItem('script_yaml', newYaml)
      }
    } catch (e) {
      console.error('YAML 序列化失败:', e)
    }
  }

  // 方法：加载 YAML 数据
  function loadYaml(yamlStr: string) {
    yamlText.value = yamlStr
    try {
      history.value.push(yamlStr)
      if (history.value.length > 20) {
        history.value.shift()
      }
    } catch (e) {
      console.error('YAML 解析失败:', e)
    }
  }

  // 方法：设置解析后的剧本数据
  function setScriptData(data: ScriptData) {
    scriptData.value = data
    if (data.scenes.length > 0) {
      currentSceneId.value = data.scenes[0].id
    }
  }

  // 方法：播放控制
  function togglePlay() {
    isPlaying.value = !isPlaying.value
  }

  function stopPlay() {
    isPlaying.value = false
    playPosition.value = 0
  }

  function nextBeat() {
    if (playPosition.value < totalBeats.value - 1) {
      playPosition.value++
      const current = flatBeats.value[playPosition.value]
      if (current) {
        currentSceneId.value = current.sceneId
        currentBeatIndex.value = current.beatIndex
      }
    } else {
      stopPlay()
    }
  }

  // 重置
  function reset() {
    yamlText.value = ''
    scriptData.value = null
    currentSceneId.value = null
    currentBeatIndex.value = null
    isPlaying.value = false
    playPosition.value = 0
    history.value = []
    modifications.value = []
    regeneratingKeys.value.clear()
  }

  return {
    yamlText,
    scriptData,
    currentSceneId,
    currentBeatIndex,
    isPlaying,
    playPosition,
    history,
    modifications,
    regeneratingKeys,
    scenes,
    characters,
    emotionalCurve,
    totalBeats,
    currentScene,
    currentBeat,
    flatBeats,
    isRegenerating,
    loadYaml,
    setScriptData,
    // 场景操作
    updateSceneInfo,
    updateSceneMediaHints,
    removeScene,
    addScene,
    // 节拍操作
    updateBeatContent,
    updateBeatType,
    updateBeatSpeaker,
    updateBeatEmotion,
    selectAlternative,
    applyAIRegeneration,
    applyAISceneRegeneration,
    setRegenerating,
    addBeat,
    removeBeat,
    insertBeat,
    moveBeat,
    // 修改历史
    getTargetHistory,
    clearModifications,
    // 播放控制
    togglePlay,
    stopPlay,
    nextBeat,
    reset,
  }
})
