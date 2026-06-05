<template>
  <div class="w-full h-full flex flex-col">
    <VChart
      ref="chartRef"
      :option="chartOption"
      :init-options="{ renderer: 'canvas' }"
      autoresize
      class="flex-1 min-h-[300px]"
      @click="handleNodeClick"
    />

    <!-- 角色关系详情面板 -->
    <div
      v-if="detailVisible"
      class="flex-shrink-0 mx-2 mb-2 p-3 bg-slate-800/70 rounded-lg border border-slate-700/50"
    >
      <div class="flex items-center justify-between mb-2">
        <h4 class="text-xs font-medium text-slate-300">关系详情</h4>
        <button @click="closeDetail" class="text-slate-500 hover:text-white transition-colors">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div v-if="selectedRelation" class="space-y-2">
        <div class="flex items-center gap-2 text-sm">
          <span class="font-medium text-indigo-300">{{ selectedRelation.source }}</span>
          <svg class="w-4 h-4 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 8l4 4m0 0l-4 4m4-4H3" />
          </svg>
          <span class="font-medium text-purple-300">{{ selectedRelation.target }}</span>
        </div>

        <div class="grid grid-cols-2 gap-2 text-xs">
          <div class="bg-slate-900/50 rounded p-2">
            <div class="text-slate-500 mb-1">亲密度</div>
            <div class="text-emerald-400 font-medium">{{ selectedRelation.intimacy?.toFixed(2) || '-' }}</div>
          </div>
          <div class="bg-slate-900/50 rounded p-2">
            <div class="text-slate-500 mb-1">信任度</div>
            <div :class="trustColorClass" class="font-medium">{{ selectedRelation.trust?.toFixed(2) || '-' }}</div>
          </div>
        </div>

        <!-- 调整关系值滑块 -->
        <div class="space-y-2 pt-1">
          <label class="text-[10px] text-slate-500 block">调整亲密度</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="selectedRelation.intimacy || 0"
            @input="(e) => updateRelation('intimacy', Number((e.target as HTMLInputElement).value))"
            class="w-full h-1.5 bg-slate-700 rounded-full appearance-none cursor-pointer accent-emerald-400"
          />
          <label class="text-[10px] text-slate-500 block mt-2">调整信任度</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="selectedRelation.trust || 0"
            @input="(e) => updateRelation('trust', Number((e.target as HTMLInputElement).value))"
            class="w-full h-1.5 bg-slate-700 rounded-full appearance-none cursor-pointer"
            :class="selectedRelation.trust > 0.5 ? 'accent-green-400' : 'accent-red-400'"
          />
        </div>
      </div>

      <p v-else class="text-xs text-slate-500">点击两个角色查看详情</p>
    </div>

    <!-- 图例 -->
    <div class="flex-shrink-0 px-2 py-1 text-[10px] text-slate-500 flex items-center gap-2 border-t border-slate-700/30">
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-gradient-to-r from-emerald-400 to-emerald-600 rounded"></span>高信任
      </span>
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-gradient-to-r from-yellow-400 to-yellow-600 rounded"></span>中等
      </span>
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-gradient-to-r from-red-400 to-red-600 rounded"></span>低信任
      </span>
      <span v-if="graphStore.selectedCharacters.length" class="ml-auto text-purple-400">
        已选: {{ graphStore.selectedCharacters.join(' / ') }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, nextTick } from 'vue'
import { useGraphStore } from '~/stores/graphStore'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { GraphChart } from 'echarts/charts'
import { TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([GraphChart, TooltipComponent, CanvasRenderer])

const graphStore = useGraphStore()
const chartRef = ref<InstanceType<typeof VChart>>()
const detailVisible = ref(false)

interface RelationDetail {
  source: string
  target: string
  intimacy?: number
  trust?: number
}

const selectedRelation = ref<RelationDetail | null>(null)

const trustColorClass = computed(() => {
  if (!selectedRelation.value?.trust) return 'text-slate-400'
  if (selectedRelation.value.trust > 0.7) return 'text-emerald-400'
  if (selectedRelation.value.trust > 0.35) return 'text-yellow-400'
  return 'text-red-400'
})

// 根据信任度获取边颜色
function getEdgeColor(trust?: number): string {
  if (trust === undefined || trust === null) return '#94a3b8'
  if (trust > 0.7) return '#10b981'   // 绿色 - 高信任
  if (trust > 0.35) return '#f59e0b'  // 黄色 - 中等
  return '#ef4444'                     // 红色 - 低信任
}

// 根据亲密度获取边宽度
function getEdgeWidth(intimacy?: number): number {
  if (intimacy === undefined || intimacy === null) return 1
  return Math.max(1, Math.min(6, intimacy * 8))
}

// 构建图表配置
const chartOption = computed(() => {
  const nodes = graphStore.relationNodes.map((node) => ({
    id: node.id,
    name: node.name,
    symbolSize: 30,
    itemStyle: {
      color: graphStore.selectedCharacters.includes(node.name)
        ? '#a78bfa'
        : '#475569',
      borderColor: graphStore.selectedCharacters.includes(node.name)
        ? '#8b5cf6'
        : '#64748b',
      borderWidth: graphStore.selectedCharacters.includes(node.name) ? 2.5 : 1.5,
    },
    label: {
      show: true,
      fontSize: 11,
      color: graphStore.selectedCharacters.includes(node.name) ? '#e2e8f0' : '#94a3b8',
      fontWeight: graphStore.selectedCharacters.includes(node.name) ? 'bold' : 'normal',
    },
  }))

  const edges = graphStore.relationEdges.map((edge) => ({
    source: edge.source,
    target: edge.target,
    value: edge.value || 1,
    lineStyle: {
      color: getEdgeColor(edge.value),
      width: getEdgeWidth(edge.value),
      opacity: 0.75,
      curveness: 0.2,
    },
  }))

  return {
    tooltip: {
      trigger: 'item',
      backgroundColor: 'rgba(15, 23, 42, 0.95)',
      borderColor: '#334155',
      borderWidth: 1,
      padding: [8, 12],
      textStyle: {
        color: '#e2e8f0',
        fontSize: 12,
      },
      formatter: (params: any) => {
        if (params.dataType === 'node') {
          return `<strong>${params.name}</strong><br/><span style="color:#94a3b8">角色</span>`
        }
        if (params.dataType === 'edge') {
          const edgeData = graphStore.relationEdges.find(
            (e) => e.source === params.data.source && e.target === params.data.target
          )
          return `${params.data.source} ↔ ${params.data.target}<br/>亲密度: ${(edgeData?.value || 0).toFixed(2)}`
        }
        return ''
      },
    },
    animationDuration: 800,
    animationEasingUpdate: 'cubicInOut',
    series: [
      {
        type: 'graph',
        layout: 'force',
        data: nodes,
        links: edges,
        roam: true,
        draggable: true,
        force: {
          repulsion: 150,
          gravity: 0.08,
          edgeLength: [100, 250],
          layoutAnimation: true,
        },
        emphasis: {
          focus: 'adjacency',
          lineStyle: {
            width: 4,
          },
        },
        lineStyle: {},
        circular: {
          rotateLabel: false,
        },
      },
    ],
  }
})

// 节点点击：选中/取消选中角色
function handleNodeClick(params: any) {
  if (params.dataType !== 'node') return

  const charName = params.data.name
  graphStore.selectCharacter(charName)

  // 如果已选中两个角色，显示详情
  if (graphStore.selectedCharacters.length >= 2) {
    showDetailForPair(graphStore.selectedCharacters[0], graphStore.selectedCharacters[1])
  }

  nextTick(() => {
    chartRef.value?.setOption(chartOption.value)
  })
}

// 显示两个角色的关系详情
function showDetailForPair(charA: string, charB: string) {
  const edge = graphStore.relationEdges.find(
    (e) =>
      (e.source === charA && e.target === charB) ||
      (e.source === charB && e.target === charA)
  )

  if (edge) {
    selectedRelation.value = {
      source: charA,
      target: charB,
      intimacy: edge.value,
      trust: edge.value,
    }
  } else {
    selectedRelation.value = {
      source: charA,
      target: charB,
      intimacy: 0.5,
      trust: 0.5,
    }
  }
  detailVisible.value = true
}

// 更新关系值
function updateRelation(field: 'intimacy' | 'trust', value: number) {
  if (!selectedRelation.value) return
  ;(selectedRelation.value as any)[field] = value

  // 同步到 store 的边数据中
  const edgeIndex = graphStore.relationEdges.findIndex(
    (e) =>
      (e.source === selectedRelation.value!.source &&
        e.target === selectedRelation.value!.target) ||
      (e.source === selectedRelation.value!.target &&
        e.target === selectedRelation.value!.source)
  )
  if (edgeIndex >= 0) {
    graphStore.relationEdges[edgeIndex].value = value
  }

  nextTick(() => {
    chartRef.value?.setOption(chartOption.value)
  })
}

function closeDetail() {
  detailVisible.value = false
  selectedRelation.value = null
}
</script>
