<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-900 via-indigo-950 to-slate-900">
    <!-- 顶部导航 -->
    <nav class="border-b border-slate-700/50 bg-slate-900/80 backdrop-blur-sm">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center">
              <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
              </svg>
            </div>
            <h1 class="text-xl font-bold text-white">Novel2Script Pro</h1>
            <span class="px-2 py-0.5 text-xs font-medium bg-indigo-500/20 text-indigo-300 rounded-full">AI 驱动</span>
          </div>
        </div>
      </div>
    </nav>

    <main class="max-w-5xl mx-auto px-4 py-12">
      <!-- 标题区域 -->
      <div class="text-center mb-12">
        <h2 class="text-4xl font-bold text-white mb-4">
          小说转剧本
          <span class="bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
            智能创作平台
          </span>
        </h2>
        <p class="text-lg text-slate-400 max-w-2xl mx-auto">
          上传你的小说文本，AI 将自动分析情感节奏、提取角色、生成专业剧本，
          支持因果图谱、关系网络、盲演模式等创新功能。
        </p>
      </div>

      <!-- 主要输入区 -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- 左侧：小说输入 -->
        <div class="lg:col-span-2 space-y-6">
          <!-- 文本输入卡片 -->
          <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6">
            <label class="block text-sm font-medium text-slate-300 mb-3">
              小说文本输入
            </label>

            <!-- 文件上传 -->
            <div
              class="border-2 border-dashed border-slate-600 rounded-lg p-8 text-center hover:border-indigo-500/50 transition-colors cursor-pointer mb-4"
              @click="triggerFileInput"
              @dragover.prevent="isDragging = true"
              @dragleave.prevent="isDragging = false"
              @drop.prevent="handleFileDrop"
              :class="{ 'border-indigo-500 bg-indigo-500/5': isDragging }"
            >
              <input
                ref="fileInput"
                type="file"
                accept=".txt,.md"
                class="hidden"
                @change="handleFileUpload"
              />
              <svg class="w-12 h-12 mx-auto text-slate-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                  d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
              </svg>
              <p class="text-slate-400 text-sm">
                拖拽 .txt / .md 文件到此处，或点击上传
              </p>
              <p class="text-slate-500 text-xs mt-1">建议至少包含 3 个章节</p>
            </div>

            <!-- 文本粘贴区 -->
            <textarea
              v-model="novelText"
              placeholder="或者直接在此粘贴小说内容..."
              class="w-full h-64 bg-slate-900/50 border border-slate-600 rounded-lg p-4 text-slate-200 placeholder-slate-500 resize-y focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-transparent"
            ></textarea>

            <div class="flex justify-between items-center mt-2">
              <span class="text-xs text-slate-500">{{ novelText.length }} 字符</span>
              <button
                v-if="novelText"
                @click="novelText = ''"
                class="text-xs text-slate-500 hover:text-red-400 transition-colors"
              >
                清空文本
              </button>
            </div>
          </div>
        </div>

        <!-- 右侧：配置面板 -->
        <div class="space-y-6">
          <!-- 剧本风格 -->
          <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6">
            <label class="block text-sm font-medium text-slate-300 mb-3">剧本风格</label>
            <select
              v-model="config.style"
              class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-slate-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
            >
              <option value="short_drama">短剧</option>
              <option value="film">电影</option>
              <option value="stage">舞台剧</option>
            </select>
          </div>

          <!-- 功能开关 -->
          <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6">
            <label class="block text-sm font-medium text-slate-300 mb-4">功能选项</label>
            <div class="space-y-3">
              <label class="flex items-center justify-between cursor-pointer group">
                <span class="text-sm text-slate-300 group-hover:text-white transition-colors">盲演模式</span>
                <input type="checkbox" v-model="config.blindActingMode"
                  class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
              </label>
              <label class="flex items-center justify-between cursor-pointer group">
                <span class="text-sm text-slate-300 group-hover:text-white transition-colors">因果图谱</span>
                <input type="checkbox" v-model="config.includeCausalGraph"
                  class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
              </label>
              <label class="flex items-center justify-between cursor-pointer group">
                <span class="text-sm text-slate-300 group-hover:text-white transition-colors">关系网络</span>
                <input type="checkbox" v-model="config.includeRelationNetwork"
                  class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
              </label>
              <label class="flex items-center justify-between cursor-pointer group">
                <span class="text-sm text-slate-300 group-hover:text-white transition-colors">媒体提示</span>
                <input type="checkbox" v-model="config.includeMediaHints"
                  class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
              </label>
            </div>
          </div>

          <!-- 生成按钮 -->
          <button
            @click="generateScript"
            :disabled="!novelText.trim() || isGenerating"
            class="w-full py-3 px-4 rounded-xl font-semibold text-white transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 hover:shadow-lg hover:shadow-indigo-500/25 active:scale-[0.98]"
          >
            <span v-if="!isGenerating" class="flex items-center justify-center gap-2">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              生成剧本
            </span>
            <span v-else class="flex items-center justify-center gap-2">
              <svg class="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z">
                </path>
              </svg>
              正在分析并生成...
            </span>
          </button>
        </div>
      </div>

      <!-- 示例小说按钮 -->
      <div class="mt-8 text-center">
        <button
          @click="loadSampleNovel"
          class="text-sm text-slate-400 hover:text-indigo-400 transition-colors underline underline-offset-4"
        >
          没有小说？点击加载示例数据体验完整流程
        </button>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

// 状态
const novelText = ref('')
const fileInput = ref<HTMLInputElement | null>(null)
const isDragging = ref(false)
const isGenerating = ref(false)

// 配置
const config = reactive({
  style: 'short_drama',
  blindActingMode: true,
  includeCausalGraph: true,
  includeRelationNetwork: true,
  includeMediaHints: true,
})

// 触发文件选择
function triggerFileInput() {
  fileInput.value?.click()
}

// 处理文件上传
function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    const reader = new FileReader()
    reader.onload = (e) => {
      novelText.value = e.target?.result as string || ''
    }
    reader.readAsText(file)
  }
}

// 处理拖拽
function handleFileDrop(event: DragEvent) {
  isDragging.value = false
  const file = event.dataTransfer?.files[0]
  if (file && (file.name.endsWith('.txt') || file.name.endsWith('.md'))) {
    const reader = new FileReader()
    reader.onload = (e) => {
      novelText.value = e.target?.result as string || ''
    }
    reader.readAsText(file)
  }
}

// 测试 API 连接（已移除：API 配置固定在后端 .env）

// 加载示例小说
function loadSampleNovel() {
  novelText.value = `第一章 初遇

午后的阳光透过大学图书馆的落地窗，在木质地板上画出一片片金色的光斑。林曦坐在靠窗的角落，面前摊开着三本厚厚的参考书和一台笔记本电脑。

她推了推滑落的眼镜，眉头微皱。距离期末考试还有两周，而她连第一章都还没复习完。

"又在这里啊。"

林曦抬头，看见苏晴端着两杯咖啡走过来。苏晴是她室友，也是她在大学里最好的朋友。

"别说了，我感觉我要挂了。"林曦接过咖啡，叹了口气。

"你每次都说要挂，结果哪次不是年级前三？"苏晴在她旁边坐下，"对了，听说顾言今天也来了。"

林曦的手顿了一下。

顾言。那个总是一个人坐在图书馆最后一排、从不与人交谈的男生。她不知道为什么，每次听到这个名字，心里就会有一种奇怪的感觉。

"他跟我有什么关系？"林曦低头看书，假装不在意。

苏晴意味深长地看了她一眼："哦？真的没关系？那上个月是谁在食堂多看人家好几眼？"

"苏晴！"林曦的脸微微泛红。

就在这时，一个身影停在了她们桌前。

林曦抬起头——是一双平静如水的眼睛。

"这里有人吗？"他的声音低沉而干净。

第二章 秘密

深夜。宿舍里只剩下林曦还亮着灯。

她的面前放着那本深蓝色的笔记本——顾言今天下午递给她的。"你的，上次落在我桌上的。"

但林曦完全不记得自己丢过任何笔记本。

她翻开第一页。空白的。

她又翻到第二页。还是空白的。

正当她准备合上时，台灯的光线恰好以某个角度照在纸页上。隐约间，她看见了角落里有细小的铅笔字迹。

林曦的心跳突然加速了。

她把笔记本对着光，仔细辨认那些字：

"如果你看到了这些字，说明我们之间有某种联系。——G"

G……顾言？

林曦快速翻动整本笔记本。每一页的角落都有一句话，像一条隐秘的线索：

第四页："有些人注定会相遇，无论绕多少路。"
第十二页："图书馆的阳光和你一样温暖。"
第二十三页："我不知道该怎么开口告诉你。"
最后一页："如果你读到了这里，来老地方找我。——G"

林曦的手开始微微颤抖。她不知道这是什么意思，但她知道自己必须去找他。

第三章 真相

第二天傍晚。图书馆。同一个靠窗的位置。

林曦来得比平时早了一个小时。她手里紧紧攥着那本深蓝色笔记本。

五点三十分，顾言出现了。

他还是那样，一个人，背着书包，面无表情地走过一排排书架。但当他的目光扫过靠窗的位置时，脚步停住了。

林曦站起来。

他们隔着一张桌子对视。周围的一切仿佛都静止了。

"你……"林曦开口，声音有些发颤。

顾言看着她，眼神中闪过一丝她从未见过的情绪。

"你看到了。"他说。这不是疑问句。

"这是什么意思？"林曦举起笔记本，"你为什么不直接告诉我？"

顾言沉默了几秒。然后他走过来，在她对面坐下。

"因为有些话，我不知道该怎么说。"他的声音很轻，"从大一入学那天起，我就注意到你了。每次在图书馆看到你认真学习的样子，我都想跟你说话，但我不知道怎么开口。"

林曦愣住了。

"后来我开始在笔记里写东西，想着万一有一天你能看到……虽然我知道这个可能性很小。"顾言苦笑了一下，"结果你真的丢了——不，是我故意制造机会让你拿到它的。"

"所以那次相遇不是偶然？"

"不是。"顾言直视她的眼睛，"我策划了很久。只是没想到，你会花这么长时间才打开它。"

林曦感觉自己的心跳声大得整个图书馆都能听见。

"那你现在想说什么？"她问。

顾言没有回答。他从书包里拿出另一本笔记本——崭新的，浅绿色的封皮——推到她面前。

林曦翻开第一页，上面只有一行字：

"故事的下一章，我们一起写好吗？"

窗外，夕阳正好。
`
}

// 生成剧本
async function generateScript() {
  if (!novelText.value.trim()) return

  isGenerating.value = true

  try {
    // 调用后端生成剧本接口（API Key 由后端 .env 配置管理）
    const response = await fetch('/api/generate-script', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        text: novelText.value,
        config: {
          style: config.style,
          blind_acting_mode: config.blindActingMode,
          max_alternatives: 2,
          include_media_hints: config.includeMediaHints,
          include_causal_graph: config.includeCausalGraph,
          include_relation_network: config.includeRelationNetwork,
        },
      }),
    })

    if (!response.ok) {
      throw new Error(`生成失败: ${response.status}`)
    }

    const yamlText = await response.text()

    // 将 YAML 存储到 sessionStorage，跳转到编辑器
    sessionStorage.setItem('script_yaml', yamlText)

    router.push('/editor')
  } catch (error: any) {
    console.error('生成剧本失败:', error)
    alert('生成失败: ' + error.message)
  } finally {
    isGenerating.value = false
  }
}
</script>
