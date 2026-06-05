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

export const useScriptStore = defineStore('script', () => {
  // 状态
  const yamlText = ref('')
  const scriptData = ref<ScriptData | null>(null)
  const currentSceneId = ref<number | null>(null)
  const currentBeatIndex = ref<number | null>(null)
  const isPlaying = ref(false)
  const playPosition = ref(0) // 当前播放到的全局 beat 索引
  const history = ref<string[]>([]) // YAML 历史版本

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

  // 方法：加载 YAML 数据
  function loadYaml(yaml: string) {
    yamlText.value = yaml
    try {
      // 使用 js-yaml 解析（在组件中引入）
      // 这里先存储原始文本，解析在组件中进行
      history.value.push(yaml)
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

  // 方法：更新单个 beat 内容
  function updateBeatContent(sceneId: number, beatIndex: number, newContent: string) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      scene.beats[beatIndex].content = newContent
      syncYamlFromData()
    }
  }

  // 方法：选择备选项
  function selectAlternative(sceneId: number, beatIndex: number, altIndex: number) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && scene.beats[beatIndex]) {
      const beat = scene.beats[beatIndex]
      if (altIndex >= 0 && altIndex < beat.alternatives.length) {
        beat.selected = altIndex
        // 将选中项的内容替换为主内容
        beat.content = beat.alternatives[altIndex].content
        syncYamlFromData()
      }
    }
  }

  // 方法：添加 beat
  function addBeat(sceneId: number, beat: Beat) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene) {
      scene.beats.push(beat)
      syncYamlFromData()
    }
  }

  // 方法：删除 beat
  function removeBeat(sceneId: number, beatIndex: number) {
    if (!scriptData.value) return
    const scene = scriptData.value.scenes.find(s => s.id === sceneId)
    if (scene && beatIndex >= 0 && beatIndex < scene.beats.length) {
      scene.beats.splice(beatIndex, 1)
      syncYamlFromData()
    }
  }

  // 方法：从数据同步到 YAML
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
  }

  return {
    yamlText,
    scriptData,
    currentSceneId,
    currentBeatIndex,
    isPlaying,
    playPosition,
    history,
    scenes,
    characters,
    emotionalCurve,
    totalBeats,
    currentScene,
    flatBeats,
    loadYaml,
    setScriptData,
    updateBeatContent,
    selectAlternative,
    addBeat,
    removeBeat,
    togglePlay,
    stopPlay,
    nextBeat,
    reset,
  }
})
