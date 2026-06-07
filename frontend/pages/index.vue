<template>
  <div class="min-h-screen bg-gradient-to-br from-slate-900 via-indigo-950 to-slate-900">
    <!-- 顶部导航 -->
    <nav class="border-b border-slate-700/50 bg-slate-900/80 backdrop-blur-sm sticky top-0 z-50">
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
          <button @click="refreshProjects" :disabled="loadingProjects"
            class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 transition-colors disabled:opacity-50">
            <svg :class="{ 'animate-spin': loadingProjects }" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            刷新
          </button>
        </div>
      </div>
    </nav>

    <main class="max-w-7xl mx-auto px-4 py-8 space-y-10">
      <!-- ==================== 项目数据展示区域 ==================== -->
      <section>
        <div class="flex items-center justify-between mb-5">
          <h2 class="text-xl font-semibold text-white flex items-center gap-2">
            <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            我的项目
            <span v-if="projectList.total > 0" class="text-sm font-normal text-slate-400">({{ projectList.total }})</span>
          </h2>

          <!-- 搜索 + 筛选栏 -->
          <div class="flex items-center gap-3">
            <!-- 搜索框 -->
            <div class="relative">
              <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <input
                v-model="searchKeyword"
                @input="handleSearchInput"
                type="text"
                placeholder="搜索项目..."
                class="pl-9 pr-3 py-1.5 w-48 bg-slate-800/80 border border-slate-600 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-transparent"
              />
            </div>

            <!-- 状态筛选 -->
            <select
              v-model="statusFilter"
              @change="fetchProjects(1)"
              class="bg-slate-800/80 border border-slate-600 rounded-lg px-3 py-1.5 text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
            >
              <option value="">全部状态</option>
              <option value="draft">草稿</option>
              <option value="completed">已完成</option>
              <option value="archived">已归档</option>
            </select>

          </div>
        </div>

        <!-- 加载状态 -->
        <div v-if="loadingProjects && !projectList.projects.length" class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-12 text-center">
          <svg class="animate-spin w-10 h-10 mx-auto text-indigo-400 mb-4" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p class="text-slate-400">正在加载项目列表...</p>
        </div>

        <!-- 错误状态 -->
        <div v-else-if="loadError" class="bg-red-900/20 backdrop-blur-sm rounded-xl border border-red-700/30 p-8 text-center">
          <svg class="w-10 h-10 mx-auto text-red-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
          <p class="text-red-300 mb-1">{{ loadError }}</p>
          <p class="text-red-400/60 text-sm mb-4">请检查后端服务是否正常运行</p>
          <button @click="retryLoadProjects"
            class="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-sm font-medium bg-red-600/20 hover:bg-red-600/30 text-red-300 border border-red-600/30 transition-colors">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            重试加载
          </button>
        </div>

        <!-- 空状态 -->
        <div v-else-if="!projectList.projects.length && !loadingProjects" class="bg-slate-800/40 backdrop-blur-sm rounded-xl border border-slate-700/30 p-12 text-center">
          <svg class="w-14 h-14 mx-auto text-slate-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-slate-400 mb-1">暂无项目</p>
          <p class="text-slate-500 text-sm">在下方生成剧本后，系统将自动为您创建项目</p>
        </div>

        <!-- 项目卡片网格 -->
        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div
            v-for="proj in projectList.projects"
            :key="proj.id"
            class="group bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-5 hover:border-indigo-500/40 hover:bg-slate-800/80 transition-all duration-200 cursor-pointer relative overflow-hidden"
            @click="openProject(proj)"
          >
            <!-- 状态标签 -->
            <div class="absolute top-4 right-4">
              <span :class="statusBadgeClass(proj.status)" class="px-2 py-0.5 text-xs font-medium rounded-full">
                {{ statusLabel(proj.status) }}
              </span>
            </div>

            <!-- 项目标题 -->
            <h3 class="text-white font-medium pr-16 group-hover:text-indigo-300 transition-colors line-clamp-1">{{ proj.title }}</h3>

            <!-- 描述 -->
            <p v-if="proj.description" class="text-slate-400 text-sm mt-1 line-clamp-2">{{ proj.description }}</p>

            <!-- 信息行 -->
            <div class="mt-4 pt-3 border-t border-slate-700/50 grid grid-cols-2 gap-y-1.5 text-xs">
              <div class="flex items-center gap-1.5 text-slate-500">
                <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
                </svg>
                <span>{{ styleLabel(proj.style) }}</span>
              </div>
              <div class="flex items-center gap-1.5 text-slate-500">
                <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
                <span>{{ proj.owner || 'anonymous' }}</span>
              </div>
              <div class="flex items-center gap-1.5 text-slate-500 col-span-2">
                <svg class="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>{{ formatTime(proj.created_at) }}</span>
              </div>
            </div>

            <!-- 操作按钮（hover 显示） -->
            <div class="mt-3 pt-3 border-t border-slate-700/50 flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
              <button @click.stop="openProject(proj)"
                class="flex-1 py-1.5 text-xs rounded-md bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 transition-colors">
                打开编辑
              </button>
              <button @click.stop="confirmDelete(proj)"
                class="py-1.5 px-2.5 text-xs rounded-md bg-red-600/10 hover:bg-red-600/20 text-red-400 transition-colors">
                删除
              </button>
            </div>
          </div>
        </div>

        <!-- 分页控制 -->
        <div v-if="projectList.total > projectList.page_size" class="flex items-center justify-center gap-4 mt-6 pt-4">
          <button
            @click="fetchProjects(projectList.page - 1)"
            :disabled="projectList.page <= 1 || loadingProjects"
            class="px-3 py-1.5 text-sm rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            上一页
          </button>
          <span class="text-sm text-slate-400">
            第 {{ projectList.page }} / {{ totalPages }} 页 (共 {{ projectList.total }} 条)
          </span>
          <button
            @click="fetchProjects(projectList.page + 1)"
            :disabled="projectList.page >= totalPages || loadingProjects"
            class="px-3 py-1.5 text-sm rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            下一页
          </button>
        </div>
      </section>

      <!-- 分割线 -->
      <div class="border-t border-slate-700/30"></div>

      <!-- ==================== 原有：小说转剧本功能区 ==================== -->
      <section>
        <div class="text-center mb-8">
          <h2 class="text-3xl font-bold text-white mb-3">
            小说转剧本
            <span class="bg-gradient-to-r from-indigo-400 to-purple-400 bg-clip-text text-transparent">
              智能创作平台
            </span>
          </h2>
          <p class="text-base text-slate-400 max-w-2xl mx-auto">
            上传你的小说文本，AI 将自动分析情感节奏、提取角色、生成专业剧本，
            支持因果图谱、关系网络、盲演模式等创新功能。
          </p>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 items-stretch">
          <!-- 左侧：小说输入 -->
          <div class="lg:col-span-2 flex flex-col">
            <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6 flex-1 flex flex-col">
              <label class="block text-sm font-medium text-slate-300 mb-3">小说文本输入</label>

              <!-- 文件上传 -->
              <div
                class="border-2 border-dashed border-slate-600 rounded-lg p-8 text-center hover:border-indigo-500/50 transition-colors cursor-pointer mb-4"
                @click="triggerFileInput"
                @dragover.prevent="isDragging = true"
                @dragleave.prevent="isDragging = false"
                @drop.prevent="handleFileDrop"
                :class="{ 'border-indigo-500 bg-indigo-500/5': isDragging }"
              >
                <input ref="fileInput" type="file" accept=".txt,.md" class="hidden" @change="handleFileUpload" />
                <svg class="w-12 h-12 mx-auto text-slate-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"
                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                </svg>
                <p class="text-slate-400 text-sm">拖拽 .txt / .md 文件到此处，或点击上传</p>
                <p class="text-slate-500 text-xs mt-1">建议至少包含 3 个章节</p>
              </div>

              <!-- 文本粘贴区 -->
              <textarea
                v-model="novelText"
                placeholder="或者直接在此粘贴小说内容..."
                class="w-full flex-1 min-h-64 bg-slate-900/50 border border-slate-600 rounded-lg p-4 text-slate-200 placeholder-slate-500 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-transparent"
              ></textarea>

              <div class="flex justify-between items-center mt-2">
                <span class="text-xs text-slate-500">{{ novelText.length }} 字符</span>
                <button v-if="novelText" @click="novelText = ''" class="text-xs text-slate-500 hover:text-red-400 transition-colors">清空文本</button>
              </div>
            </div>
          </div>

          <!-- 右侧：配置面板 -->
          <div class="flex flex-col gap-6">
            <!-- 项目信息（可选） -->
            <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6">
              <label class="block text-sm font-medium text-slate-300 mb-3">项目信息</label>
              <div class="space-y-3">
                <div>
                  <label class="block text-xs text-slate-400 mb-1">项目名称 <span class="text-slate-600">(可选，留空则 AI 自动生成)</span></label>
                  <input v-model="projectName" type="text" placeholder="例如：校园爱情改编剧本"
                    class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50" />
                </div>
                <div>
                  <label class="block text-xs text-slate-400 mb-1">项目描述 <span class="text-slate-600">(可选，留空则 AI 自动生成)</span></label>
                  <textarea v-model="projectDescription" rows="2" placeholder="简要描述这个项目的目标..."
                    class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500/50"></textarea>
                </div>
              </div>
            </div>

            <!-- 剧本风格 -->
            <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6">
              <label class="block text-sm font-medium text-slate-300 mb-3">剧本风格</label>
              <select v-model="config.style"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-slate-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/50">
                <option value="short_drama">短剧</option>
                <option value="film">电影</option>
                <option value="stage">舞台剧</option>
              </select>
            </div>

            <!-- 功能开关 -->
            <div class="bg-slate-800/60 backdrop-blur-sm rounded-xl border border-slate-700/50 p-6">
              <label class="block text-sm font-medium text-slate-300 mb-4">功能选项</label>
              <div class="space-y-3">
                <!-- 章节处理模式（突出显示） -->
                <div class="rounded-lg bg-slate-900/50 border border-slate-600/50 p-3">
                  <label class="flex items-center justify-between cursor-pointer group">
                    <div class="flex items-center gap-2.5 min-w-0">
                      <svg class="w-4 h-4 text-indigo-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                      </svg>
                      <span class="text-sm text-slate-300 group-hover:text-white transition-colors">AI 自动分析章节</span>
                    </div>
                    <button
                      type="button"
                      role="switch"
                      :aria-checked="config.aiChapterAnalysis"
                      @click="config.aiChapterAnalysis = !config.aiChapterAnalysis"
                      class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:ring-offset-2 focus:ring-offset-slate-800"
                      :class="config.aiChapterAnalysis ? 'bg-indigo-500' : 'bg-slate-600'"
                    >
                      <span
                        class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                        :class="config.aiChapterAnalysis ? 'translate-x-5' : 'translate-x-0'"
                      />
                    </button>
                  </label>
                  <!-- 模式说明 -->
                  <p class="text-[11px] text-slate-500 mt-2 leading-relaxed pl-6.5">
                    <template v-if="config.aiChapterAnalysis">
                      <span class="text-indigo-400/80 font-medium">AI 分析模式</span>
                      — 系统将基于叙事完整性智能划分章节，适合无明确章节标记的小说
                    </template>
                    <template v-else>
                      <span class="text-emerald-400/80 font-medium">保留原始章节</span>
                      — 系统将识别原文中的「第X章」等标记，按用户原有章节结构搭建项目
                    </template>
                  </p>
                  <!-- 章节预览（检测到章节时显示） -->
                  <div v-if="!config.aiChapterAnalysis && detectedChapters.length > 0" class="mt-2 pl-6.5">
                    <div class="flex items-center gap-1.5 text-[10px] text-emerald-400/70 mb-1.5">
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                      检测到 {{ detectedChapters.length }} 个章节
                    </div>
                    <div class="max-h-28 overflow-y-auto space-y-0.5 pr-1">
                      <div v-for="(ch, idx) in detectedChapters" :key="idx"
                        class="flex items-center gap-2 px-2 py-1 rounded bg-slate-800/60 text-[10px]">
                        <span class="text-slate-500 shrink-0 w-5 text-right">{{ idx + 1 }}</span>
                        <span class="text-slate-300 truncate">{{ ch.title }}</span>
                        <span class="text-slate-600 ml-auto shrink-0">{{ ch.charCount }}字</span>
                      </div>
                    </div>
                  </div>
                  <div v-else-if="!config.aiChapterAnalysis && novelText.length > 100 && detectedChapters.length === 0" class="mt-2 pl-6.5">
                    <div class="flex items-center gap-1.5 text-[10px] text-amber-400/70">
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
                      </svg>
                      未检测到标准章节标记（如「第X章」），建议启用 AI 分析模式
                    </div>
                  </div>
                </div>

                <label class="flex items-center justify-between cursor-pointer group">
                  <span class="text-sm text-slate-300 group-hover:text-white transition-colors">盲演模式</span>
                  <input type="checkbox" v-model="config.blindActingMode" class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
                </label>
                <label class="flex items-center justify-between cursor-pointer group">
                  <span class="text-sm text-slate-300 group-hover:text-white transition-colors">因果图谱</span>
                  <input type="checkbox" v-model="config.includeCausalGraph" class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
                </label>
                <label class="flex items-center justify-between cursor-pointer group">
                  <span class="text-sm text-slate-300 group-hover:text-white transition-colors">关系网络</span>
                  <input type="checkbox" v-model="config.includeRelationNetwork" class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
                </label>
                <label class="flex items-center justify-between cursor-pointer group">
                  <span class="text-sm text-slate-300 group-hover:text-white transition-colors">媒体提示</span>
                  <input type="checkbox" v-model="config.includeMediaHints" class="w-4 h-4 rounded border-slate-600 text-indigo-500 focus:ring-indigo-500/50" />
                </label>
              </div>
            </div>

            <!-- 生成按钮 -->
            <button
              @click="generateScript"
              :disabled="!novelText.trim() || isGenerating"
              class="w-full mt-auto py-3 px-4 rounded-xl font-semibold text-white transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 hover:shadow-lg hover:shadow-indigo-500/25 active:scale-[0.98]"
            >
              <span v-if="!isGenerating" class="flex items-center justify-center gap-2">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                生成剧本
              </span>
              <span v-else class="flex flex-col items-center justify-center gap-2">
                <svg class="animate-spin w-5 h-5" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <span class="text-xs">{{ generateMessage || '正在处理...' }}</span>
                <!-- 进度条 -->
                <div v-if="generateProgress > 0" class="w-full bg-slate-700/50 rounded-full h-1.5 overflow-hidden">
                  <div class="h-full bg-white/60 rounded-full transition-all duration-500 ease-out" :style="{ width: generateProgress + '%' }"></div>
                </div>
                <span v-if="generateProgress > 0" class="text-[10px] opacity-60">{{ generateProgress }}%</span>
              </span>
            </button>
          </div>
        </div>

        <!-- 示例小说按钮 -->
        <div class="mt-6 text-center">
          <button @click="loadSampleNovel"
            class="text-sm text-slate-400 hover:text-indigo-400 transition-colors underline underline-offset-4">
            没有小说？点击加载示例数据体验完整流程
          </button>
        </div>
      </section>
    </main>

    <!-- 删除确认弹窗 -->
    <Teleport to="body">
      <div v-if="deleteTarget" class="fixed inset-0 z-[100] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm" @click.self="deleteTarget = null">
        <div class="bg-slate-800 rounded-2xl border border-slate-700 shadow-2xl w-full max-w-sm p-6 text-center">
          <div class="w-12 h-12 mx-auto mb-4 rounded-full bg-red-600/10 flex items-center justify-center">
            <svg class="w-6 h-6 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </div>
          <h3 class="text-lg font-semibold text-white mb-2">确认删除？</h3>
          <p class="text-sm text-slate-400 mb-5">删除项目「{{ deleteTarget?.title }}」后将无法恢复。</p>
          <div class="flex justify-center gap-3">
            <button @click="deleteTarget = null"
              class="px-4 py-2 rounded-lg text-sm text-slate-300 hover:bg-slate-700 transition-colors">
              取消
            </button>
            <button @click="doDelete" :disabled="deleting"
              class="px-4 py-2 rounded-lg text-sm font-medium bg-red-600 hover:bg-red-500 text-white disabled:opacity-50 transition-colors">
              {{ deleting ? '删除中...' : '确认删除' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import { useProjectStore } from '../stores/projectStore'
import { useDialog } from '../composables/useDialog'

const router = useRouter()
const projectStore = useProjectStore()
const dialog = useDialog()

// ==================== 项目管理状态（使用全局 Store） ====================
const { projectList, loading: loadingProjects } = storeToRefs(projectStore)
// 兼容模板中的 loadError
const loadError = computed(() => projectStore.error || '')
const searchKeyword = ref('')
const statusFilter = ref('')
const deleteTarget = ref<any>(null)
const deleting = ref(false)

// 防抖定时器
let searchTimer: ReturnType<typeof setTimeout> | null = null

// 项目名称和描述（生成剧本时使用）
const projectName = ref('')
const projectDescription = ref('')

// 分页总页数
const totalPages = computed(() => Math.ceil(projectList.value.total / projectList.value.page_size))

// ==================== 原有功能状态 ====================
const novelText = ref('')
const fileInput = ref<HTMLInputElement | null>(null)
const isDragging = ref(false)
const isGenerating = ref(false)

// 生成进度状态（SSE）
const generateProgress = ref(0)
const generateStage = ref('')
const generateMessage = ref('')

const config = reactive({
  style: 'short_drama',
  blindActingMode: true,
  aiChapterAnalysis: true,
  includeCausalGraph: true,
  includeRelationNetwork: true,
  includeMediaHints: true,
})

// ==================== 章节检测（原始章节保留模式） ====================

/** 章节标记正则：匹配「第X章」、Markdown标题、Chapter X 等常见格式 */
const CHAPTER_PATTERN = /^(?:\s{0,4}#{1,6}\s+)?(?:第[一二三四五六七八九十百千零〇0-9]+[章节回卷集部]|[Cc]hapter\s+\d+|[Pp]art\s+\d+|\d+[\.、．]\s*\S.*)/m

interface DetectedChapter {
  title: string
  startOffset: number
  endOffset: number
  charCount: number
}

/** 检测小说文本中的章节结构 */
const detectedChapters = computed<DetectedChapter[]>(() => {
  const text = novelText.value
  if (!text || text.length < 50) return []

  // 使用带全局标志的正则，通过 exec 迭代获取每个匹配的精确位置
  const globalPattern = new RegExp(CHAPTER_PATTERN.source, CHAPTER_PATTERN.flags.replace('m', '') + 'gm')
  const chapters: DetectedChapter[] = []
  let match: RegExpExecArray | null
  let lastEnd = 0

  while ((match = globalPattern.exec(text)) !== null) {
    const title = match[0].replace(/^[#\s]+/, '').trim()
    const start = match.index

    // 关闭上一个章节
    if (chapters.length > 0) {
      chapters[chapters.length - 1].endOffset = start
      chapters[chapters.length - 1].charCount =
        chapters[chapters.length - 1].endOffset - chapters[chapters.length - 1].startOffset
    }

    chapters.push({
      title,
      startOffset: start,
      endOffset: text.length,
      charCount: 0,
    })
    lastEnd = start
  }

  // 关闭最后一个章节
  if (chapters.length > 0) {
    chapters[chapters.length - 1].endOffset = text.length
    chapters[chapters.length - 1].charCount =
      chapters[chapters.length - 1].endOffset - chapters[chapters.length - 1].startOffset
  }

  return chapters
})

// ==================== 项目 API 操作 ====================

async function fetchProjects(page: number = 1) {
  await projectStore.fetchProjects(page, { search: searchKeyword.value, status: statusFilter.value })
}

function handleSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => fetchProjects(1), 350)
}

function retryLoadProjects() {
  loadError.value = ''
  fetchProjects(1)
}

function refreshProjects() {
  fetchProjects(projectList.value.page)
}

function confirmDelete(proj: any) {
  deleteTarget.value = proj
}

async function doDelete() {
  if (!deleteTarget.value) return
  deleting.value = true

  try {
    const res = await fetch(`/api/projects/${deleteTarget.value.id}`, { method: 'DELETE' })
    if (!res.ok) throw new Error(`HTTP ${res.status}`)

    const deletedId = deleteTarget.value.id
    deleteTarget.value = null
    // 乐观更新：直接从列表移除
    projectStore.removeProjectLocal(deletedId)
  } catch (e: any) {
    await dialog.alert('删除失败: ' + e.message, { variant: 'danger' })
  } finally {
    deleting.value = false
  }
}

async function openProject(proj: any) {
  if (proj.script_id) {
    // 从后端加载剧本数据到 sessionStorage，确保编辑器能正确显示
    try {
      const res = await fetch(`/api/scripts/${proj.script_id}`)
      if (res.ok) {
        const yamlText = await res.text()
        sessionStorage.setItem('script_yaml', yamlText)
      } else {
        console.warn('加载剧本失败，将以空编辑器打开')
      }
    } catch (e) {
      console.error('请求剧本数据出错:', e)
    }
    router.push({
      path: '/editor',
      query: {
        script_id: proj.script_id || '',
        project_id: proj.id || '',
      },
    })
  } else {
    await dialog.alert('该项目暂无关联剧本，请在下方生成剧本', { variant: 'warning' })
  }
}

// ==================== 辅助函数 ====================

function styleLabel(style: string): string {
  const map: Record<string, string> = { short_drama: '短剧', film: '电影', stage: '舞台剧', anime: '动漫' }
  return map[style] || style
}

function statusLabel(status: string): string {
  const map: Record<string, string> = { draft: '草稿', completed: '已完成', archived: '已归档' }
  return map[status] || status
}

function statusBadgeClass(status: string): string {
  const map: Record<string, string> = {
    draft: 'bg-yellow-500/20 text-yellow-300',
    completed: 'bg-green-500/20 text-green-300',
    archived: 'bg-slate-500/20 text-slate-400',
  }
  return map[status] || 'bg-slate-500/20 text-slate-400'
}

function formatTime(isoStr: string): string {
  try {
    const d = new Date(isoStr)
    const now = new Date()
    const diffMs = now.getTime() - d.getTime()
    const diffMin = Math.floor(diffMs / 60000)
    if (diffMin < 1) return '刚刚'
    if (diffMin < 60) return `${diffMin} 分钟前`
    const diffHour = Math.floor(diffMin / 60)
    if (diffHour < 24) return `${diffHour} 小时前`
    const diffDay = Math.floor(diffHour / 24)
    if (diffDay < 30) return `${diffDay} 天前`
    return d.toLocaleDateString('zh-CN')
  } catch {
    return isoStr
  }
}

// ==================== 原有功能函数 ====================

function triggerFileInput() {
  fileInput.value?.click()
}

function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    const reader = new FileReader()
    reader.onload = (e) => { novelText.value = e.target?.result as string || '' }
    reader.readAsText(file)
  }
}

function handleFileDrop(event: DragEvent) {
  isDragging.value = false
  const file = event.dataTransfer?.files[0]
  if (file && (file.name.endsWith('.txt') || file.name.endsWith('.md'))) {
    const reader = new FileReader()
    reader.onload = (e) => { novelText.value = e.target?.result as string || '' }
    reader.readAsText(file)
  }
}

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

窗外，夕阳正好。`
}

async function generateScript() {
  if (!novelText.value.trim()) return

  isGenerating.value = true
  generateProgress.value = 0
  generateStage.value = 'analysis'
  generateMessage.value = '正在分析小说结构...'

  try {
    let yamlText: string

    // 尝试 SSE 流式模式
    const useSSE = await tryStreamGenerate(novelText.value)
    if (useSSE) {
      yamlText = await streamGenerateScript(novelText.value)
    } else {
      // 降级为普通 POST 模式
      generateMessage.value = '正在生成剧本（可能需要 30~120 秒）...'
      const requestBody: Record<string, any> = {
        text: novelText.value,
        config: {
          style: config.style,
          blind_acting_mode: config.blindActingMode,
          max_alternatives: 2,
          include_media_hints: config.includeMediaHints,
          include_causal_graph: config.includeCausalGraph,
          include_relation_network: config.includeRelationNetwork,
          ai_chapter_analysis: config.aiChapterAnalysis,
        },
      }
      // 保留原始章节模式时，传递前端检测到的章节数据
      if (!config.aiChapterAnalysis && detectedChapters.value.length > 0) {
        requestBody.pre_parsed_chapters = detectedChapters.value.map((ch, idx) => ({
          id: `ch_${String(idx + 1).padStart(3, '0')}`,
          title: ch.title,
          order: idx + 1,
          start_offset: ch.startOffset,
          end_offset: ch.endOffset,
        }))
      }
      const response = await fetch('/api/generate-script', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestBody),
      })
      if (!response.ok) throw new Error(`生成失败: ${response.status}`)
      yamlText = await response.text()
    }

    // 确定项目名称和描述：用户填写优先，否则 AI 自动生成
    let finalName = projectName.value.trim()
    let finalDescription = projectDescription.value.trim()

    if (!finalName || !finalDescription) {
      generateMessage.value = '正在生成项目信息...'
      try {
        const infoRes = await fetch('/api/generate-project-info', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            novel_text: novelText.value.slice(0, 5000),
            style: config.style,
          }),
        })
        if (infoRes.ok) {
          const info = await infoRes.json()
          if (!finalName) finalName = info.name
          if (!finalDescription) finalDescription = info.description
        }
      } catch (e) {
        console.warn('AI 生成项目信息失败，使用默认值:', e)
      }
    }

    // 兜底：如果仍然为空，使用默认值
    if (!finalName) {
      const styleMap: Record<string, string> = { short_drama: '短剧', film: '电影', stage: '舞台剧' }
      finalName = `${styleMap[config.style] || '短剧'} - ${new Date().toLocaleString('zh-CN', { month: 'numeric', day: 'numeric', hour: '2-digit', minute: '2-digit' })}`
    }
    if (!finalDescription) {
      finalDescription = `基于小说文本生成的剧本项目`
    }

    // 乐观更新：在导航前先将项目添加到"我的项目"列表
    const now = new Date().toISOString()
    projectStore.addProjectToLocal({
      id: 'pending-' + Date.now(),
      title: finalName,
      description: finalDescription,
      style: config.style,
      status: 'completed',
      owner: 'anonymous',
      novel_preview: novelText.value.slice(0, 200),
      script_id: null,
      created_at: now,
      updated_at: now,
    } as any)

    sessionStorage.setItem('script_yaml', yamlText)
    router.push('/editor')
  } catch (error: any) {
    console.error('生成剧本失败:', error)
    await dialog.alert('生成失败: ' + error.message, { variant: 'danger' })
  } finally {
    isGenerating.value = false
    generateProgress.value = 0
    generateStage.value = ''
    generateMessage.value = ''
  }
}

/** 检测后端是否支持 SSE 流式接口 */
async function tryStreamGenerate(_text: string): Promise<boolean> {
  try {
    const probeRes = await fetch('/api/generate-script/stream', {
      method: 'GET',
      headers: { Accept: 'text/event-stream' },
    })
    return probeRes.ok
  } catch {
    return false
  }
}

/** 使用 SSE 流式生成剧本 */
async function streamGenerateScript(text: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const encodedText = encodeURIComponent(text)
    const chapterMode = config.aiChapterAnalysis ? 'true' : 'false'
    const eventSource = new EventSource(`/api/generate-script/stream?text=${encodedText}&ai_chapter_analysis=${chapterMode}`)

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (data.stage) {
          generateStage.value = data.stage
          generateProgress.value = Math.round((data.progress || 0) * 100)
        }
        if (data.message) {
          generateMessage.value = data.message
        }
        if (data.current_chapter && data.total_chapters) {
          generateMessage.value = `正在按章节生成剧本（第 ${data.current_chapter}/${data.total_chapters} 章）...`
        }
        if (data.completed_chunks !== undefined && data.total_chunks !== undefined) {
          generateMessage.value = `正在整合校验中... (${data.completed_chunks}/${data.total_chunks})`
        }

        // 最终结果
        if (data.yaml) {
          eventSource.close()
          resolve(data.yaml)
        }
        if (data.error) {
          eventSource.close()
          reject(new Error(data.error))
        }
      } catch (e) {
        // 非 JSON 消息，忽略
      }
    }

    eventSource.onerror = () => {
      eventSource.close()
      reject(new Error('SSE 连接中断，请重试'))
    }

    // 超时保护：120 秒
    setTimeout(() => {
      if (eventSource.readyState !== EventSource.CLOSED) {
        eventSource.close()
        reject(new Error('生成超时（>120秒），请稍后重试或缩短文本长度'))
      }
    }, 120_000)
  })
}

// ==================== 初始化 ====================

onMounted(() => {
  fetchProjects(1)
})
</script>
