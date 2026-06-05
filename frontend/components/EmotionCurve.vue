<template>
  <div class="w-full" :class="compact ? 'h-full' : 'min-h-[120px]'">
    <VChart
      ref="chartRef"
      :option="chartOption"
      :init-options="{ renderer: 'canvas' }"
      autoresize
      :style="{ height: compact ? '100%' : '110px' }"
      @mousedown="handleMouseDown"
      @mouseup="handleMouseUp"
      @mousemove="handleMouseMove"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, inject, Ref } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  MarkPointComponent,
  MarkLineComponent,
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

use([LineChart, GridComponent, TooltipComponent, MarkPointComponent, MarkLineComponent, CanvasRenderer])

interface Props {
  data: {
    chapters: number[]
    intensities: number[]
  }
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
})

// 内部可变数据副本（不直接修改 props）
const internalData = ref({
  chapters: [...(props.data?.chapters || [])],
  intensities: [...(props.data?.intensities || [])],
})

// 同步 props 变化到内部数据
watch(() => props.data, (newData) => {
  if (newData) {
    internalData.value = {
      chapters: [...newData.chapters],
      intensities: [...newData.intensities],
    }
  }
}, { deep: true })

// 定义事件
const emit = defineEmits<{
  'update:intensity': [chapterIndex: number, newValue: number]
}>()

const chartRef = ref<InstanceType<typeof VChart>>()
const isDragging = ref(false)
const dragPointIndex = ref(-1)

// 注入全局更新事件
const scriptUpdateEvent = inject<Ref<number>>('scriptUpdate')

// 当前章节标注（取最大强度值的章节）
const currentChapter = computed(() => {
  const intensities = internalData.value.intensities
  const chapters = internalData.value.chapters
  if (!intensities.length) return 0
  let maxIdx = 0
  let maxVal = -1
  intensities.forEach((val, idx) => {
    if (val > maxVal) {
      maxVal = val
      maxIdx = idx
    }
  })
  return chapters[maxIdx]
})

const chartOption = computed(() => {
  const chapters = internalData.value.chapters.map(String)
  const intensities = [...internalData.value.intensities]

  return {
    grid: props.compact
        ? { top: 8, right: 20, bottom: 20, left: 32 }
        : { top: 25, right: 25, bottom: 30, left: 45 },
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(15, 23, 42, 0.95)',
      borderColor: '#334155',
      borderWidth: 1,
      textStyle: { color: '#e2e8f0', fontSize: 11 },
      formatter: (params: any) => {
        const p = params[0]
        return `第${p.axisValue}章<br/>情绪强度: <strong>${p.data.toFixed(2)}</strong>`
      },
    },
    xAxis: {
      type: 'category',
      data: chapters,
      axisLabel: {
        color: '#64748b',
        fontSize: props.compact ? 9 : 11,
        formatter: (val: string) => `Ch.${val}`,
      },
      axisLine: { lineStyle: { color: '#334155' } },
      splitLine: { show: false },
    },
    yAxis: {
      type: 'value',
      min: 0,
      max: 1,
      axisLabel: {
        color: '#64748b',
        fontSize: props.compact ? 9 : 11,
        formatter: (val: number) => val.toFixed(1),
      },
      axisLine: { show: false },
      splitLine: {
        lineStyle: { color: '#1e293b', type: 'dashed' },
      },
    },
    series: [
      {
        type: 'line',
        data: intensities,
        smooth: 0.35,
        symbol: 'circle',
        symbolSize: props.compact ? 4 : 6,
        showSymbol: !props.compact,
        itemStyle: {
          color: '#818cf8',
          borderColor: '#6366f1',
          borderWidth: 2,
        },
        lineStyle: {
          color: '#818cf8',
          width: 2,
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0, y: 0, x2: 0, y2: 1,
            colorStops: [
              { offset: 0, color: 'rgba(99, 102, 241, 0.35)' },
              { offset: 1, color: 'rgba(99, 102, 241, 0.02)' },
            ],
          },
        },
        markPoint: {
          data: intensities.length > 0
            ? [{
                coord: [String(internalData.value.chapters[currentChapter.value] || internalData.value.chapters[intensities.indexOf(Math.max(...intensities))]), Math.max(...intensities)],
                symbol: 'pin',
                symbolSize: props.compact ? 25 : 40,
                itemStyle: { color: '#f59e0b' },
                label: {
                  show: !props.compact,
                  color: '#fbbf24',
                  fontSize: 10,
                  formatter: `当前`,
                },
              }]
            : [],
        },
        markLine: {
          silent: true,
          symbol: 'none',
          data: [{ yAxis: 0.5, lineStyle: { color: '#ef4444', type: 'dashed', width: 1 } }],
          label: { show: false },
        },
      },
    ],
  }
})

// 数据点拖拽调节
let dragStartY = 0
let dragStartValue = 0

function handleMouseDown(params: any) {
  if (params.event?.event?.which !== 1) return // 只响应左键
  if (params.componentType !== 'series') return

  isDragging.value = true
  dragPointIndex.value = params.dataIndex
  dragStartY = params.event.offsetY
  dragStartValue = internalData.value.intensities[params.dataIndex]
}

function handleMouseMove(params: any) {
  if (!isDragging.value || dragPointIndex.value < 0) return
  if (dragPointIndex.value >= internalData.value.intensities.length) return

  const deltaY = dragStartY - (params.event?.offsetY || 0)
  // 基于图表高度估算变化量（约 180px 对应 1.0）
  const deltaValue = deltaY / 200
  const newValue = Math.max(0, Math.min(1, dragStartValue + deltaValue))

  internalData.value.intensities[dragPointIndex.value] = parseFloat(newValue.toFixed(3))
  chartRef.value?.setOption(chartOption.value)
}

function handleMouseUp() {
  if (isDragging.value && dragPointIndex.value >= 0) {
    // 通知父组件数据已变更
    emit('update:intensity', dragPointIndex.value, internalData.value.intensities[dragPointIndex.value])
    isDragging.value = false
    dragPointIndex.value = -1
  }
}
</script>
