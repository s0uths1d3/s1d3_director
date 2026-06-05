import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface GraphNode {
  id: string
  name: string
  category?: string
}

export interface GraphEdge {
  source: string
  target: string
  value?: number
  label?: string
  lineStyle?: {
    type?: string
    color?: string
    width?: number
  }
}

export const useGraphStore = defineStore('graph', () => {
  // 因果图谱数据
  const causalNodes = ref<GraphNode[]>([])
  const causalEdges = ref<GraphEdge[]>([])
  const highlightedEventId = ref<string | null>(null)
  const affectedEvents = ref<string[]>([])

  // 关系网络数据
  const relationNodes = ref<GraphNode[]>([])
  const relationEdges = ref<GraphEdge[]>([])
  const selectedCharacters = ref<string[]>([])

  // 当前活跃的图谱视图：'causal' | 'relation'
  const activeView = ref<'causal' | 'relation'>('causal')

  // 方法：设置因果图谱数据
  function setCausalGraph(nodes: GraphNode[], edges: GraphEdge[]) {
    causalNodes.value = nodes
    causalEdges.value = edges
    highlightedEventId.value = null
    affectedEvents.value = []
  }

  // 方法：高亮事件及其下游影响
  function highlightDownstream(eventId: string, downstreamIds: string[]) {
    highlightedEventId.value = eventId
    affectedEvents.value = downstreamIds
  }

  // 方法：清除高亮
  function clearHighlight() {
    highlightedEventId.value = null
    affectedEvents.value = []
  }

  // 方法：设置关系网络数据
  function setRelationNetwork(nodes: GraphNode[], edges: GraphEdge[]) {
    relationNodes.value = nodes
    relationEdges.value = edges
    selectedCharacters.value = []
  }

  // 方法：选中角色（用于显示详情）
  function selectCharacter(charName: string) {
    if (selectedCharacters.value.includes(charName)) {
      selectedCharacters.value = selectedCharacters.value.filter(n => n !== charName)
    } else {
      if (selectedCharacters.value.length < 2) {
        selectedCharacters.value.push(charName)
      } else {
        selectedCharacters.value = [selectedCharacters.value[1], charName]
      }
    }
  }

  // 方法：切换视图
  function switchView(view: 'causal' | 'relation') {
    activeView.value = view
  }

  return {
    causalNodes,
    causalEdges,
    highlightedEventId,
    affectedEvents,
    relationNodes,
    relationEdges,
    selectedCharacters,
    activeView,
    setCausalGraph,
    highlightDownstream,
    clearHighlight,
    setRelationNetwork,
    selectCharacter,
    switchView,
  }
})
