<template>
  <div class="w-full h-full flex flex-col">
    <VChart
      ref="chartRef"
      :option="chartOption"
      :init-options="{ renderer: 'canvas' }"
      autoresize
      class="flex-1 min-h-[300px]"
      @click="handleChartClick"
      @contextmenu.prevent="handleContextMenu"
    />

    <!-- 右键菜单 -->
    <Teleport to="body">
      <div
        v-if="contextMenu.visible"
        class="fixed z-50 bg-slate-800 border border-slate-600 rounded-lg shadow-xl py-1 min-w-[160px]"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      >
        <button
          @click="simulateRemove"
          class="w-full px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700 transition-colors flex items-center gap-2"
        >
          <svg class="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          模拟删除
        </button>
      </div>
    </Teleport>

    <!-- 图例说明 -->
    <div class="flex-shrink-0 px-2 py-1.5 text-[10px] text-slate-500 flex items-center gap-3 border-t border-slate-700/30">
      <span class="flex items-center gap-1">
        <span class="w-2.5 h-0.5 bg-blue-400 rounded"></span>因果
      </span>
      <span class="flex items-center gap-1">
        <span class="w-2.5 h-0.5 bg-slate-400 rounded"></span>时序
      </span>
      <span class="flex items-center gap-1">
        <span class="w-2.5 h-0.5 bg-red-400 rounded"></span>情感
      </span>
      <span v-if="graphStore.highlightedEventId" class="ml-auto text-indigo-400">
        已选中: {{ graphStore.highlightedEventId }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, inject, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useGraphStore } from '~/stores/graphStore'
import { useDialog } from '~/composables/useDialog'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { GraphChart } from 'echarts/charts'
import {
  TooltipComponent,
  LegendComponent,
  MarkLineComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([GraphChart, TooltipComponent, LegendComponent, MarkLineComponent, CanvasRenderer])

const graphStore = useGraphStore()
const dialog = useDialog()
const chartRef = ref<InstanceType<typeof VChart>>()

// 右键菜单状态
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  nodeId: '',
})

// 边类型颜色映射
const edgeColorMap: Record<string, string> = {
  causal: '#60a5fa',     // 蓝色
  temporal: '#94a3b8',   // 灰色
  emotional: '#f87171',   // 红色
}

// 构建图表配置
const chartOption = computed(() => {
  const nodes = graphStore.causalNodes.map((node) => ({
    id: node.id,
    name: node.name,
    category: node.category || 0,
    symbolSize: getSymbolSize(node.id),
    itemStyle: getNodeStyle(node.id),
    label: {
      show: true,
      fontSize: 11,
      color: '#e2e8f0',
      formatter: truncateText(node.name, 12),
    },
  }))

  const edges = graphStore.causalEdges.map((edge) => ({
    source: edge.source,
    target: edge.target,
    value: edge.value || 1,
    lineStyle: {
      color: edge.lineStyle?.color || edgeColorMap[edge.label as string] || '#60a5fa',
      width: edge.lineStyle?.width || (edge.value ? Math.min(3, Math.max(1, edge.value)) : 2),
      type: edge.lineStyle?.type || 'solid',
      curveness: 0.15,
    },
    symbol: ['none', 'arrow'],
    symbolSize: [0, 8],
    label: {
      show: !!edge.label,
      formatter: edge.label || '',
      fontSize: 9,
      color: '#94a3b8',
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
          return `<strong>${params.name}</strong>`
        }
        if (params.dataType === 'edge') {
          return `${params.data.source} → ${params.data.target}<br/>类型: ${params.data.label || '关联'}`
        }
        return ''
      },
    },
    animationDurationUpdate: 300,
    animationEasingUpdate: 'cubicOut',
    series: [
      {
        type: 'graph',
        layout: 'force',
        data: nodes,
        links: edges,
        roam: true,
        draggable: true,
        force: {
          repulsion: 200,
          gravity: 0.05,
          edgeLength: [80, 200],
          layoutAnimation: true,
        },
        emphasis: {
          focus: 'adjacency',
          lineStyle: {
            width: 4,
          },
        },
        lineStyle: {
          opacity: 0.7,
        },
        itemStyle: {
          borderColor: '#475569',
          borderWidth: 1,
        },
        categories: [
          { name: '事件' },
          { name: '转折点' },
          { name: '高潮' },
        ],
      },
    ],
  }
})

function getSymbolSize(nodeId: string): number {
  // 高亮节点更大
  if (nodeId === graphStore.highlightedEventId) return 28
  if (graphStore.affectedEvents.includes(nodeId)) return 22
  return 16
}

function getNodeStyle(nodeId: string) {
  if (nodeId === graphStore.highlightedEventId) {
    return {
      color: '#fbbf24',
      borderColor: '#f59e0b',
      borderWidth: 3,
      shadowBlur: 10,
      shadowColor: 'rgba(251, 191, 36, 0.5)',
    }
  }
  if (graphStore.affectedEvents.includes(nodeId)) {
    return {
      color: '#34d399',
      borderColor: '#10b981',
      borderWidth: 2,
    }
  }
  return {
    color: '#475569',
    borderColor: '#64748b',
  }
}

function truncateText(text: string, maxLen: number): string {
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text
}

// 点击节点高亮
function handleChartClick(params: any) {
  hideContextMenu()
  if (params.dataType !== 'node') return

  const eventId = params.data.id
  if (graphStore.highlightedEventId === eventId) {
    graphStore.clearHighlight()
  } else {
    // 计算下游节点（BFS）
    const downstream = findDownstream(eventId)
    graphStore.highlightDownstream(eventId, downstream)
  }

  nextTick(() => {
    chartRef.value?.setOption(chartOption.value)
  })
}

// BFS 查找下游节点
function findDownstream(nodeId: string): string[] {
  const visited = new Set<string>()
  const queue = [...graphStore.causalEdges.filter(e => e.source === nodeId).map(e => e.target)]
  while (queue.length > 0) {
    const current = queue.shift()!
    if (visited.has(current)) continue
    visited.add(current)
    for (const edge of graphStore.causalEdges) {
      if (edge.source === current && !visited.has(edge.target)) {
        queue.push(edge.target)
      }
    }
  }
  return Array.from(visited)
}

// 右键菜单
function handleContextMenu(params: any) {
  if (params.dataType !== 'node') return

  contextMenu.value = {
    visible: true,
    x: params.event.offsetX + 8,
    y: params.event.offsetY + 8,
    nodeId: params.data.id,
  }
}

function hideContextMenu() {
  contextMenu.value.visible = false
}

async function simulateRemove() {
  const nodeId = contextMenu.value.nodeId
  if (!nodeId) return
  hideContextMenu()

  // 找出所有受影响的下游节点
  const downstream = findDownstream(nodeId)
  const affectedList = [nodeId, ...downstream]

  await dialog.alert(`模拟删除事件 "${nodeId}"\n将影响以下下游事件:\n${affectedList.join(', ')}`, { title: '删除事件影响分析' })
}

// 点击空白处关闭右键菜单
function handleClickOutside(e: MouseEvent) {
  if (contextMenu.value.visible) {
    const target = e.target as HTMLElement
    if (!target.closest('.fixed')) {
      hideContextMenu()
    }
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>
