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
    <div class="flex-shrink-0 px-2 py-1 text-[10px] text-slate-500 flex items-center gap-2 border-t border-slate-700/30 flex-wrap">
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-gradient-to-r from-emerald-400 to-emerald-600 rounded"></span>高信任
      </span>
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-gradient-to-r from-yellow-400 to-yellow-600 rounded"></span>中等
      </span>
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-red-500/70 rounded" style="border-top: 2px dashed #ef4444"></span>对立
      </span>
      <span class="flex items-center gap-1">
        <span class="w-3 h-0.5 bg-purple-400/85 rounded"></span>亲密
      </span>
      <span class="flex items-center gap-1 ml-1">
        <svg class="w-3 h-3 text-slate-500" viewBox="0 0 24 24" fill="none" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"/></svg>
        单向
      </span>
      <span class="flex items-center gap-1">
        <svg class="w-4 h-3 text-purple-400" viewBox="0 0 24 16" fill="none" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2 8h18M17 5l3 3-3 3M7 5L4 8l3 3"/></svg>
        双向
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

function truncateText(text: string, maxLen: number): string {
  return text.length > maxLen ? text.slice(0, maxLen) + '…' : text
}

// 根据亲密度/信任度推断关系类型描述（短标签优先完整显示）
function inferRelationLabel(edge: any): string {
  // 优先使用后端提供的 relation_type 标签
  if (edge.relation_type) return edge.relation_type.length > 8 ? truncateText(edge.relation_type, 8) : edge.relation_type
  // 其次使用 description
  if (edge.description) return edge.description.length > 8 ? truncateText(edge.description, 8) : edge.description
  if (edge.label && edge.label !== '关联') return edge.label.length > 8 ? truncateText(edge.label, 8) : edge.label

  const val = edge.value || 0
  const trust = edge.trust ?? val
  const intimacy = edge.intimacy ?? val

  // 根据数值范围推断关系类型
  if (intimacy > 0.75 && trust > 0.7) return '亲密'
  if (intimacy > 0.6 && trust > 0.5) return '友好'
  if (intimacy > 0.35 && intimacy <= 0.6) return '普通'
  if (trust < 0.25) return '对立'
  if (intimacy < 0.25) return '疏远'
  return '关联'
}

/** 判断是否为对立/敌对关系（用于视觉区分） */
function isOppositional(edge: any): boolean {
  // 优先检查 relation_type 是否包含对立关键词
  const label = inferRelationLabel(edge)
  const oppKeywords = ['仇人', '对手', '政敌', '情敌', '背叛者', '对立', '敌对', '敌意']
  if (oppKeywords.some(k => label.includes(k))) return true
  // 其次根据数值判断：低信任 + 低亲密度 = 对立
  const trust = edge.trust ?? (edge.value || 0)
  const intimacy = edge.intimacy ?? (edge.value || 0)
  return trust < 0.3 && intimacy < 0.35
}

/** 判断是否为亲密关系（用于视觉区分） */
function isIntimate(edge: any): boolean {
  const label = inferRelationLabel(edge)
  const intKeywords = ['恋人', '暧昧', '闺蜜', '知己', '家人', '至亲', '亲密', '爱人']
  if (intKeywords.some(k => label.includes(k))) return true
  const intimacy = edge.intimacy ?? (edge.value || 0)
  const trust = edge.trust ?? (edge.value || 0)
  return intimacy > 0.7 && trust > 0.7
}

/** 获取边的视觉样式配置 */
function getEdgeVisualStyle(edge: any): { color: string; type?: string; opacity: number } {
  if (isOppositional(edge)) {
    return {
      color: '#ef4444',   // 红色 — 对立
      type: 'dashed',     // 虚线 — 区分于亲密关系的实线
      opacity: 0.75,
    }
  }
  if (isIntimate(edge)) {
    return {
      color: '#a78bfa',   // 紫色 — 亲密
      opacity: 0.85,
    }
  }
  // 默认使用亲密度渐变色
  return {
    color: getEdgeColor(edge.value),
    opacity: 0.75,
  }
}

/** 检测双向关系并合并处理 — 弧线防重叠设计
 *
 * 核心策略：
 * 1. 同关系类型(A↔B都是"好友") → 单条直线 + 双向箭头 + 居中标签
 * 2. 异关系类型(A→B"暗恋", B→A"普通") → 两条大弧度曲线(±curveness)，标签沿法向偏移到弧线外侧
 * 3. 单向关系(A→B"暗恋") → 轻微弯曲的单箭头线 + 标签
 *
 * 防重叠机制：
 * - 弧线使用较大的 curveness 绝对值(0.4~0.55)，拉开两弧间距
 * - 标签使用 distance 偏移到各自弧线的凸侧（法向外侧）
 * - 标签强制水平显示（rotate: 0），避免斜向文字重叠
 * - 多对同节点边时动态递增 curveness，避免三边以上重叠
 */
function buildProcessedEdges() {
  const rawEdges = graphStore.relationEdges
  const processed: any[] = []
  const paired = new Set<string>()
  // 记录每对节点之间的边数量，用于多边场景递增曲率
  const pairEdgeCount = new Map<string, number>()

  // 预统计：同一对节点之间有多少条边
  for (const e of rawEdges) {
    const key = [e.source, e.target].sort().join('-')
    pairEdgeCount.set(key, (pairEdgeCount.get(key) || 0) + 1)
  }

  for (let i = 0; i < rawEdges.length; i++) {
    const e = rawEdges[i]
    const pairKey = [e.source, e.target].sort().join('-')
    const edgeCount = pairEdgeCount.get(pairKey) || 1

    // 查找反向边
    const reverseIdx = rawEdges.findIndex(
      (r, idx) => idx !== i && r.source === e.target && r.target === e.source
    )

    if (reverseIdx >= 0 && !paired.has(pairKey)) {
      // ===== 双向关系 =====
      const rev = rawEdges[reverseIdx]
      paired.add(pairKey)

      const labelA = inferRelationLabel(e)
      const labelB = inferRelationLabel(rev)

      if (labelA === labelB) {
        // ---- 情况1: 同类型 → 直线双箭头 ----
        const style = getEdgeVisualStyle(e)
        processed.push({
          source: e.source,
          target: e.target,
          value: Math.max(e.value || 0, rev.value || 0),
          lineStyle: {
            color: style.color,
            width: getEdgeWidth(Math.max(e.value || 0, rev.value || 0)),
            type: style.type || 'solid',
            opacity: style.opacity,
            curveness: 0,
          },
          symbol: ['arrow', 'arrow'],
          symbolSize: [7, 7],
          label: {
            show: true,
            formatter: labelA,
            fontSize: 10,
            color: isOppositional(e) ? '#fca5a5' : '#c4b5fd',
            position: 'middle',
            rotate: 0,
            backgroundColor: 'rgba(15, 23, 42, 0.85)',
            borderColor: 'transparent',
            borderRadius: 4,
            padding: [2, 6],
          },
          _direction: 'bidirectional',
        })
      } else {
        // ---- 情况2: 异类型 → 外弧+内弧分离模式 ----
        // 一条大曲率弧线绕到两节点连线的外侧（宽弧）
        // 一条小曲率弧线贴近连线的内侧（紧弧）
        // 形成类似图中红线的「外包围 + 内通道」效果
        const styleA = getEdgeVisualStyle(e)
        const styleB = getEdgeVisualStyle(rev)

        const curveA = 0.55    // 外弧：向一侧大幅弯曲，绕过节点外侧
        const curveB = -0.12    // 内弧：向另一侧轻微弯曲，贴近节点间直线

        // 弧线A: source → target，外弧（大弯度）
        processed.push({
          source: e.source,
          target: e.target,
          value: e.value || 1,
          lineStyle: {
            color: styleA.color,
            width: getEdgeWidth(e.value),
            type: styleA.type || 'solid',
            opacity: styleA.opacity,
            curveness: curveA,
          },
          symbol: ['none', 'arrow'],
          symbolOffset: [3, -5],
          symbolSize: [0, 9],
          label: {
            show: true,
            formatter: labelA,
            fontSize: 10,
            color: isOppositional(e) ? '#fca5a5' : '#94a3b8',
            position: 'middle',
            distance: [0, -14],
            rotate: 0,
            backgroundColor: 'rgba(15, 23, 42, 0.92)',
            borderColor: styleA.color,
            borderWidth: 1,
            borderRadius: 4,
            padding: [2, 6],
          },
          _direction: 'directed',
        })

        // 弧线B: target → source，内弧（小弯度，贴近直线）
        processed.push({
          source: rev.source,
          target: rev.target,
          value: rev.value || 1,
          lineStyle: {
            color: styleB.color,
            width: getEdgeWidth(rev.value),
            type: styleB.type || 'solid',
            opacity: styleB.opacity,
            curveness: curveB,
          },
          symbol: ['none', 'arrow'],
          symbolOffset: [-2, 3],
          symbolSize: [0, 9],
          label: {
            show: true,
            formatter: labelB,
            fontSize: 10,
            color: isOppositional(rev) ? '#fca5a5' : '#94a3b8',
            position: 'middle',
            distance: [0, 10],
            rotate: 0,
            backgroundColor: 'rgba(15, 23, 42, 0.92)',
            borderColor: styleB.color,
            borderWidth: 1,
            borderRadius: 4,
            padding: [2, 6],
          },
          _direction: 'directed',
        })
      }
    } else if (!paired.has(pairKey)) {
      // ---- 情况3: 单向关系 ----
      const label = inferRelationLabel(e)
      const style = getEdgeVisualStyle(e)
      processed.push({
        source: e.source,
        target: e.target,
        value: e.value || 1,
        lineStyle: {
          color: style.color,
          width: getEdgeWidth(e.value),
          type: style.type || 'solid',
          opacity: style.opacity,
          curveness: 0.08,     // 微小弯曲以区分于直线双箭头
        },
        symbol: ['none', 'arrow'],
        symbolSize: [0, 8],
        label: {
          show: true,
          formatter: label,
          fontSize: 9,
          color: isOppositional(e) ? '#fca5a5' : '#94a3b8',
          position: 'middle',
          rotate: 0,
          backgroundColor: 'rgba(15, 23, 42, 0.85)',
          borderColor: 'transparent',
          borderRadius: 4,
          padding: [2, 5],
        },
        _direction: 'directed',
      })
    }
  }
  return processed
}

// 构建图表配置
const chartOption = computed(() => {
  const nodes = graphStore.relationNodes.map((node) => {
    const isSelected = graphStore.selectedCharacters.includes(node.name)
    // 根据选中状态和连接数动态计算大小
    const edgeCount = graphStore.relationEdges.filter(
      (e) => e.source === node.id || e.target === node.id
    ).length
    return {
      id: node.id,
      name: node.name,
      symbolSize: isSelected ? 32 + edgeCount * 3 : 24 + edgeCount * 2,
      itemStyle: {
        color: isSelected ? '#a78bfa' : '#475569',
        borderColor: isSelected ? '#8b5cf6' : '#64748b',
        borderWidth: isSelected ? 2.5 : 1.5,
      },
      label: {
        show: true,
        fontSize: 11,
        color: isSelected ? '#e2e8f0' : '#94a3b8',
        fontWeight: isSelected ? 'bold' : 'normal',
        formatter: truncateText(node.name, 10),
      },
    }
  })

  const edges = buildProcessedEdges()

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
          const dirLabel = params.data._direction === 'bidirectional' ? '↔' : '→'
          const edgeData = graphStore.relationEdges.find(
            (e) => e.source === params.data.source && e.target === params.data.target
          )
          const relLabel = params.data.label?.formatter || inferRelationLabel(edgeData)
          return `${params.data.source} ${dirLabel} ${params.data.target}<br/><span style="color:#c4b5fd">${relLabel}</span><br/>亲密度: ${(edgeData?.value || 0).toFixed(2)}`
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
          repulsion: 300,           // 正常斥力，不拉远节点
          gravity: 0.03,            // 正常引力
          edgeLength: [120, 250],   // 正常边距
          layoutAnimation: true,
          friction: 0.6,
          preventOverlap: true,
        },
        initLayout: 'circular',
        emphasis: {
          focus: 'adjacency',
          lineStyle: {
            width: 4,
          },
        },
        lineStyle: {},
      },
    ],
  }
})

// 节点点击：选中/取消选中角色（使用 dispatchAction 避免重置布局）
function handleNodeClick(params: any) {
  if (params.dataType !== 'node') return

  const charName = params.data.name
  graphStore.selectCharacter(charName)

  // 如果已选中两个角色，显示详情
  if (graphStore.selectedCharacters.length >= 2) {
    showDetailForPair(graphStore.selectedCharacters[0], graphStore.selectedCharacters[1])
  }

  // 使用 dispatchAction 更新选中状态，不触发 setOption 重置布局
  nextTick(() => {
    if (!chartRef.value) return
    const instance = chartRef.value
    instance.dispatchAction({ type: 'downplay' })
    for (const name of graphStore.selectedCharacters) {
      instance.dispatchAction({
        type: 'highlight',
        seriesIndex: 0,
        name,
      })
    }
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

// 更新关系值（保留当前节点位置，避免力导向布局重置）
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

  // 使用保留位置的更新策略：提取当前坐标 → setOption → 力模拟从当前位置继续
  nextTick(() => {
    if (!chartRef.value) return
    const instance = chartRef.value
    // 从当前布局中提取所有节点的 x,y 坐标
    const option = instance.getOption() as any
    const currentNodes = option?.series?.[0]?.data || []
    const positionedNodes = chartOption.value.series[0].data.map((node: any, i: number) => ({
      ...node,
      // 如果当前布局有坐标则保留，让力模拟从当前位置继续演化
      ...(currentNodes[i] && currentNodes[i].x != null ? { x: currentNodes[i].x, y: currentNodes[i].y } : {}),
    }))
    instance.setOption({
      series: [{
        data: positionedNodes,
        links: chartOption.value.series[0].links,
      }],
    })
  })
}

function closeDetail() {
  detailVisible.value = false
  selectedRelation.value = null
}
</script>
