<template>
  <div class="h-screen flex flex-col bg-[var(--bg-dark)] overflow-hidden">
    <!-- 快捷键保存反馈浮动提示 -->
    <Transition name="feedback-fade">
      <div
        v-if="saveFeedbackVisible"
        class="fixed top-4 left-1/2 -translate-x-1/2 z-[200] flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-500/90 text-white text-sm font-medium shadow-lg shadow-emerald-500/30 backdrop-blur-sm pointer-events-none"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        {{ saveFeedbackMessage }}
      </div>
    </Transition>

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
          <!-- 保存（更新当前项目） -->
          <button
            @click="handleSave"
            :disabled="isSaving"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
            :class="currentProjectId
              ? 'bg-emerald-600/80 hover:bg-emerald-500 text-white'
              : 'bg-slate-700/60 hover:bg-slate-600/60 text-slate-200'"
            :title="currentProjectId ? '保存到当前项目 (Ctrl+S)' : '保存并创建新项目 (Ctrl+S)'"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
            </svg>
            {{ isSaving ? '保存中...' : '保存' }}
            <span class="text-[10px] font-mono opacity-50 ml-0.5">⌘S</span>
          </button>

          <!-- 保存为（另存为新项目副本） -->
          <button
            @click="handleSaveAs"
            :disabled="isSaving"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600/70 hover:bg-indigo-500 text-white text-sm font-medium transition-colors disabled:opacity-50"
            title="将当前内容另存为一个新的项目副本"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            保存为
          </button>

          <!-- 项目信息（手动编辑元数据） -->
          <button
            v-if="currentProjectId"
            @click="showExistingProjectSaveDialog()"
            :disabled="isSaving || isUpdatingProject"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors disabled:opacity-50 bg-slate-700/50 hover:bg-slate-600/50 text-slate-300 border border-slate-600/40 hover:border-slate-500/40"
            title="编辑项目名称、负责人、描述等元数据"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
            项目信息
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
            @click="resetToDefaults()"
            class="p-1.5 rounded-lg bg-slate-700/60 hover:bg-slate-600/60 text-slate-300 transition-colors"
            title="恢复面板默认大小"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5" />
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

    <!-- 情感曲线迷你图（可调整高度） -->
    <div
      v-if="emotionalCurveData && layout.emotionHeight > 0"
      class="flex-shrink-0 border-b border-slate-700/30 bg-slate-900/40 px-4 overflow-hidden"
      :style="{ height: layout.emotionHeight + 'px' }"
    >
      <EmotionCurve :data="emotionalCurveData" compact />
    </div>
    <!-- 情感曲线拖拽条 -->
    <div
      v-if="emotionalCurveData"
      class="flex-shrink-0 h-1 cursor-ns-resize group relative hover:bg-indigo-500/60 active:bg-indigo-400 transition-colors"
      @mousedown="startDragEmotion($event)"
    >
      <span class="absolute inset-x-0 top-1/2 -translate-y-1/2 h-[3px] w-8 mx-auto rounded-full bg-slate-600 group-hover:bg-indigo-400 transition-colors opacity-0 group-hover:opacity-100"></span>
    </div>

    <!-- 三栏主体布局 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 左侧面板：因果图谱 / 关系网络（可调整宽度） -->
      <aside
        class="flex-shrink-0 border-r border-slate-700/50 bg-slate-900/30 overflow-hidden flex flex-col"
        :style="{ width: layout.leftWidth + 'px' }"
      >
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
          <ClientOnly>
            <CausalGraph v-if="leftTab === 'causal'" />
            <RelationNetwork v-else />
            <template #fallback>
              <div class="flex items-center justify-center h-full text-slate-500 text-sm">
                加载图谱中...
              </div>
            </template>
          </ClientOnly>
        </div>
      </aside>

      <!-- 左侧面板拖拽条（水平调整宽度） -->
      <div
        class="flex-shrink-0 w-1 cursor-ew-resize group relative hover:bg-indigo-500/60 active:bg-indigo-400 transition-colors"
        @mousedown="startDragLeft($event)"
      >
        <span class="absolute inset-y-0 left-1/2 -translate-x-1/2 w-[3px] h-8 my-auto rounded-full bg-slate-600 group-hover:bg-indigo-400 transition-colors opacity-0 group-hover:opacity-100"></span>
      </div>

      <!-- 中间区域：场景卡片列表（自适应宽度） -->
      <main class="flex-1 overflow-auto p-4 space-y-4">
        <template v-if="loadStatus === 'loaded'" v-for="scene in scriptStore.scenes" :key="scene.id">
          <!-- 场景头部（可编辑） -->
          <div class="bg-slate-800/50 backdrop-blur-sm rounded-xl border border-slate-700/50 overflow-hidden">
            <!-- 场景标题栏 -->
            <div class="px-4 py-3 border-b border-slate-700/30">
              <div class="flex items-center justify-between mb-2">
                <div class="flex items-center gap-3">
                  <span class="text-xs font-mono text-indigo-400">SCENE {{ scene.id }}</span>
                  <!-- 场景地点（可编辑） -->
                  <input
                    :value="scene.location"
                    @input="(e: Event) => updateSceneField(scene.id, 'location', (e.target as HTMLInputElement).value)"
                    class="text-sm font-medium bg-transparent text-white border-b border-slate-600/50 focus:border-indigo-400 outline-none px-1 min-w-[120px] placeholder:text-slate-600"
                    placeholder="场景地点"
                    title="点击编辑场景地点"
                  />
                  <span v-if="scene.time" class="text-xs text-slate-500">·</span>
                  <!-- 场景时间（可编辑） -->
                  <input
                    v-if="scene.time || editingSceneTime === scene.id"
                    :value="scene.time ?? ''"
                    @blur="editingSceneTime = null; if (!(scene as any).time) (scene as any).time = undefined"
                    @input="(e: Event) => updateSceneField(scene.id, 'time', (e.target as HTMLInputElement).value)"
                    class="text-xs bg-transparent text-slate-400 border-b border-slate-600/50 focus:border-indigo-400 outline-none px-1 w-24 min-w-0 placeholder:text-slate-600"
                    placeholder="时间"
                    title="点击编辑时间"
                  />
                  <button
                    v-else-if="!scene.time"
                    @click="editingSceneTime = scene.id"
                    class="text-xs text-slate-600 hover:text-slate-400 px-1 rounded hover:bg-slate-700/30 transition-colors"
                  >+ 时间</button>
                </div>

                <!-- 右侧操作 -->
                <div class="flex items-center gap-2">
                  <!-- AI 重生成整场按钮 -->
                  <button
                    v-if="!scriptStore.isRegenerating(String(scene.id))"
                    @click="handleAIRegenerateScene(scene.id)"
                    class="flex items-center gap-1 px-2 py-1 text-[10px] rounded-md text-emerald-400/70 hover:text-emerald-300 hover:bg-emerald-500/10 transition-all"
                    title="AI 重生成整场内容"
                  >
                    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    AI 重生成
                  </button>
                  <div v-else class="flex items-center gap-1 px-2 py-1 text-[10px] rounded-md bg-emerald-500/10 text-emerald-400">
                    <svg class="animate-spin w-3 h-3" fill="none" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    AI 生成中...
                  </div>

                  <!-- 删除场景 -->
                  <button
                    @click="handleDeleteScene(scene.id)"
                    class="p-1 rounded text-slate-600 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-scene:hover:opacity-100"
                    title="删除此场景"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
                  </button>
                </div>
              </div>

              <!-- 第二行：情绪滑块 + media_hints -->
              <div class="flex items-center gap-4">
                <!-- 情绪强度滑块 -->
                <div class="flex items-center gap-2 flex-1 max-w-xs">
                  <span class="text-[10px] text-slate-500 whitespace-nowrap">情绪强度</span>
                  <input
                    type="range"
                    min="0"
                    max="10"
                    step="0.1"
                    :value="scene.emotion_intensity"
                    @input="(e: Event) => updateSceneField(scene.id, 'emotion_intensity', parseFloat((e.target as HTMLInputElement).value))"
                    class="flex-1 h-1 appearance-none bg-slate-700 rounded-full cursor-pointer accent-indigo-500 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-indigo-400 [&::-webkit-slider-thumb]:shadow-[0_0_6px_rgba(129,140,248,0.5)]"
                  />
                  <span class="text-[10px] font-mono text-slate-400 w-6 text-right">{{ scene.emotion_intensity.toFixed(1) }}</span>
                </div>

                <!-- 媒体提示 -->
                <div v-if="scene.media_hints" class="flex items-center gap-2 text-xs text-slate-500 ml-auto">
                  <span v-if="scene.media_hints.camera" title="镜头提示">📷 {{ scene.media_hints.camera }}</span>
                  <span v-if="scene.media_hints.music" title="音乐提示">🎵 {{ scene.media_hints.music }}</span>
                </div>
              </div>
            </div>

            <!-- Beat 卡片列表 -->
            <div class="divide-y divide-slate-700/20 group-scene">
              <BeatCard
                v-for="(beat, idx) in scene.beats"
                :key="idx"
                :scene-id="scene.id"
                :beat-index="idx"
                :beat="beat"
                :is-active="scriptStore.currentSceneId === scene.id && scriptStore.currentBeatIndex === idx"
                :is-regenerating="scriptStore.isRegenerating(`${scene.id}:${idx}`)"
                @update-content="(content) => handleUpdateBeat(scene.id, idx, content)"
                @update-type="(newType) => handleUpdateBeatType(scene.id, idx, newType)"
                @update-speaker="(speaker) => handleUpdateSpeaker(scene.id, idx, speaker)"
                @update-emotion="(emotion) => handleUpdateEmotion(scene.id, idx, emotion)"
                @select-alt="(altIdx) => handleSelectAlt(scene.id, idx, altIdx)"
                @remove-alt="(altIdx) => scriptStore.removeAlternative(scene.id, idx, altIdx)"
                @add-alt="(content, opts) => scriptStore.addCustomAlternative(scene.id, idx, content, opts)"
                @delete="handleDeleteBeat(scene.id, idx)"
                @ai-regenerate="() => handleAIRegenerateBeat(scene.id, idx)"
                @move-up="() => scriptStore.moveBeat(scene.id, idx, 'up')"
                @move-down="() => scriptStore.moveBeat(scene.id, idx, 'down')"
                @toggle-history="() => showHistoryPanel(scene.id, idx)"
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

        <!-- 修改历史面板（覆盖层） -->
        <Teleport to="body">
          <div
            v-if="historyPanelVisible"
            class="fixed inset-0 z-[100] flex items-center justify-center"
            @click.self="historyPanelVisible = false"
          >
            <!-- 遮罩 -->
            <div class="absolute inset-0 bg-black/50 backdrop-blur-sm" />

            <!-- 面板 -->
            <div class="relative z-10 w-full max-w-2xl max-h-[70vh] bg-slate-800 rounded-xl border border-slate-700/50 shadow-xl overflow-hidden">
              <!-- 标题栏 -->
              <div class="px-5 py-3 border-b border-slate-700/30 flex items-center justify-between">
                <div>
                  <h3 class="text-sm font-medium text-white">修改历史</h3>
                  <p v-if="historyTarget" class="text-xs text-slate-500 mt-0.5">
                    场景 {{ historyTarget.sceneId }} · 节拍 {{ historyTarget.beatIndex + 1 }}
                  </p>
                </div>
                <button
                  @click="historyPanelVisible = false"
                  class="p-1.5 rounded-lg text-slate-500 hover:text-slate-300 hover:bg-slate-700/50 transition-colors"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
                </button>
              </div>

              <!-- 历史记录列表 -->
              <div class="p-5 overflow-y-auto max-h-[calc(70vh-60px)] space-y-3">
                <template v-if="historyRecords.length > 0">
                  <div
                    v-for="(record, idx) in historyRecords"
                    :key="idx"
                    class="rounded-lg border p-3"
                    :class="{
                      'border-emerald-500/30 bg-emerald-500/5': record.source === 'ai',
                      'border-indigo-500/30 bg-indigo-500/5': record.source === 'alternative',
                      'border-slate-700/50 bg-slate-800/50': record.source === 'manual',
                    }"
                  >
                    <div class="flex items-center gap-2 mb-2">
                      <span
                        class="text-[10px] font-medium px-1.5 py-0.5 rounded"
                        :class="{
                          'bg-emerald-500/15 text-emerald-300': record.source === 'ai',
                          'bg-indigo-500/15 text-indigo-300': record.source === 'alternative',
                          'bg-slate-700 text-slate-400': record.source === 'manual',
                        }"
                      >
                        {{ record.source === 'ai' ? 'AI 重生成' : record.source === 'alternative' ? '选择备选' : '手动修改' }}
                      </span>
                      <span class="text-[10px] text-slate-500">{{ formatTime(record.timestamp) }}</span>
                      <span class="text-[10px] text-slate-600 ml-auto">字段: {{ record.field }}</span>
                    </div>

                    <!-- 对比视图 -->
                    <div class="space-y-2 text-xs">
                      <div v-if="record.oldValue && record.oldValue !== '[已删除]'">
                        <span class="text-red-400/80 line-through break-all">{{ truncate(record.oldValue, 200) }}</span>
                      </div>
                      <div>
                        <span class="text-emerald-300 break-all">{{ truncate(record.newValue, 200) }}</span>
                      </div>
                    </div>
                  </div>
                </template>
                <div v-else class="text-center py-8 text-slate-500 text-sm">
                  暂无修改记录
                </div>
              </div>

              <!-- 底部操作 -->
              <div class="px-5 py-3 border-t border-slate-700/30 flex justify-end">
                <button
                  @click="historyPanelVisible = false"
                  class="px-4 py-1.5 text-xs rounded-lg bg-slate-700/60 hover:bg-slate-600/60 text-slate-300 transition-colors"
                >关闭</button>
              </div>
            </div>
          </div>
        </Teleport>

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

      <!-- 右侧面板拖拽条（水平调整宽度） -->
      <div
        class="flex-shrink-0 w-1 cursor-ew-resize group relative hover:bg-indigo-500/60 active:bg-indigo-400 transition-colors"
        @mousedown="startDragRight($event)"
      >
        <span class="absolute inset-y-0 left-1/2 -translate-x-1/2 w-[3px] h-8 my-auto rounded-full bg-slate-600 group-hover:bg-indigo-400 transition-colors opacity-0 group-hover:opacity-100"></span>
      </div>

      <!-- 右侧面板：剧本播放器 / YAML源码编辑器（可调整宽度） -->
      <aside
        class="flex-shrink-0 border-l border-slate-700/50 bg-slate-900/30 overflow-hidden flex flex-col"
        :style="{ width: layout.rightWidth + 'px' }"
      >
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
        <div class="bg-slate-800 rounded-xl border border-slate-700/50 w-full max-w-lg p-6 shadow-xl max-h-[85vh] overflow-y-auto">
          <h3 class="text-lg font-semibold text-white mb-4">设置</h3>
          <div class="space-y-5">
            <!-- API 设置 -->
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

            <!-- 分割线 -->
            <div class="border-t border-slate-700/50"></div>

            <!-- 快捷键设置 -->
            <div>
              <div class="flex items-center justify-between mb-3">
                <label class="block text-sm font-medium text-slate-200">键盘快捷键</label>
                <span class="text-[11px] text-slate-500">自定义或禁用快捷键</span>
              </div>

              <div class="space-y-2.5">
                <div
                  v-for="sc in getRegisteredShortcuts()"
                  :key="sc.id"
                  class="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-slate-900/50 border border-slate-700/40 group"
                >
                  <!-- 快捷键名称 -->
                  <div class="flex-1 min-w-0">
                    <div class="text-sm text-slate-200">{{ sc.name }}</div>
                    <div v-if="shortcutEditId !== sc.id" class="text-xs font-mono mt-0.5" :class="sc.userBinding ? 'text-indigo-400' : 'text-slate-500'">
                      {{ sc.userBinding ? keyLabel(sc.userBinding) + ' (自定义)' : keyLabel(sc.defaultBinding) }}
                    </div>
                  </div>

                  <!-- 编辑模式：输入框 -->
                  <template v-if="shortcutEditId === sc.id">
                    <div class="flex-1 max-w-[180px]">
                      <input
                        ref="shortcutInputRef"
                        v-model="shortcutEditValue"
                        type="text"
                        placeholder="如 ctrl+s, ctrl+shift+s"
                        class="w-full bg-slate-950 border border-indigo-500/50 rounded px-2 py-1 text-xs font-mono text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500/50"
                        @keydown.enter="confirmShortcutEdit(sc.id)"
                        @keydown.escape="cancelShortcutEdit"
                        @blur="confirmShortcutEdit(sc.id)"
                      />
                      <p v-if="shortcutConflictWarning" class="mt-1 text-[11px] text-amber-400">{{ shortcutConflictWarning }}</p>
                    </div>
                    <button @click="confirmShortcutEdit(sc.id)" class="px-2 py-1 text-xs bg-emerald-600/80 hover:bg-emerald-500 text-white rounded transition-colors">
                      确定
                    </button>
                    <button @click="cancelShortcutEdit" class="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 text-slate-300 rounded transition-colors">
                      取消
                    </button>
                  </template>

                  <!-- 非编辑模式：操作按钮 -->
                  <template v-else>
                    <button
                      @click="startShortcutEdit(sc)"
                      class="px-2 py-1 text-xs bg-slate-700/60 hover:bg-slate-600 text-slate-300 rounded transition-colors opacity-0 group-hover:opacity-100"
                      title="修改快捷键"
                    >
                      修改
                    </button>
                    <button
                      v-if="sc.userBinding"
                      @click="resetUserBinding(sc.id); shortcutEditId = ''"
                      class="px-2 py-1 text-xs text-amber-400/70 hover:text-amber-300 transition-colors opacity-0 group-hover:opacity-100"
                      title="恢复默认"
                    >
                      重置
                    </button>
                    <button
                      @click="setUserBinding(sc.id, null); shortcutEditId = ''"
                      class="px-2 py-1 text-xs text-red-400/60 hover:text-red-300 transition-colors opacity-0 group-hover:opacity-100"
                      title="禁用此快捷键"
                    >
                      禁用
                    </button>
                  </template>
                </div>

                <!-- 无快捷键提示 -->
                <div v-if="getRegisteredShortcuts().length === 0" class="text-center py-3 text-xs text-slate-600">
                  暂无已注册的快捷键
                </div>
              </div>
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

      <!-- 保存成功弹窗：填写工程名称 -->
      <div v-if="showSaveDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="closeSaveDialog">
        <div class="bg-slate-800 rounded-xl border border-slate-700/50 w-full max-w-md p-6 shadow-xl">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <div>
              <h3 class="text-lg font-semibold text-white">保存成功</h3>
              <p class="text-xs text-slate-400">剧本内容已保存到服务器</p>
            </div>
          </div>

          <div class="space-y-4">
            <div>
              <label class="block text-sm text-slate-300 mb-1.5">工程名称 <span class="text-red-400">*</span></label>
              <input
                ref="projectNameInput"
                v-model="newProjectName"
                type="text"
                placeholder="请输入工程名称，如「校园青春短剧」"
                maxlength="100"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/50 transition-colors"
                @keyup.enter="confirmProjectName"
              />
              <p class="mt-1.5 text-xs text-slate-500">为这个剧本创建一个项目，方便在首页管理和继续编辑</p>
            </div>

            <div v-if="saveError" class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-sm text-red-300">
              {{ saveError }}
            </div>
          </div>

          <div class="mt-6 flex justify-end gap-2">
            <button
              @click="skipNaming"
              class="px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors"
            >
              跳过
            </button>
            <button
              @click="confirmProjectName"
              :disabled="!newProjectName.trim() || isCreatingProject"
              class="px-4 py-2 text-sm bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
            >
              <svg v-if="isCreatingProject" class="animate-spin w-3.5 h-3.5" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isCreatingProject ? '创建中...' : '确认创建' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 已有项目保存：元数据编辑弹窗 -->
      <div v-if="showSaveMetaDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="showSaveMetaDialog = false">
        <div class="bg-slate-800 rounded-xl border border-slate-700/50 w-full max-w-lg p-6 shadow-xl">
          <!-- 标题 -->
          <div class="flex items-center gap-3 mb-5">
            <div class="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
              </svg>
            </div>
            <div>
              <h3 class="text-lg font-semibold text-white">保存并更新项目信息</h3>
              <p class="text-xs text-slate-400 mt-0.5">剧本内容已保存，可在此修改项目元数据</p>
            </div>
          </div>

          <!-- 表单 -->
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-slate-300 mb-1.5">项目名称</label>
              <input
                v-model="saveMetaForm.title"
                type="text"
                placeholder="输入项目名称"
                maxlength="100"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/50 transition-colors"
              />
            </div>

            <div>
              <label class="block text-sm text-slate-300 mb-1.5">创建人 / 负责人</label>
              <input
                v-model="saveMetaForm.owner"
                type="text"
                placeholder="输入负责人姓名"
                maxlength="100"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/50 transition-colors"
              />
            </div>

            <div>
              <label class="block text-sm text-slate-300 mb-1.5">项目描述</label>
              <textarea
                v-model="saveMetaForm.description"
                rows="2"
                placeholder="可选：添加项目描述..."
                maxlength="500"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 focus:border-emerald-500/50 transition-colors resize-none"
              ></textarea>
            </div>

            <div>
              <label class="block text-sm text-slate-300 mb-1.5">项目状态</label>
              <div class="flex gap-2 flex-wrap">
                <button
                  v-for="opt in projectStatusOptions"
                  :key="opt.value"
                  @click="saveMetaForm.status = opt.value"
                  :class="[
                    'px-3 py-1.5 rounded-md text-xs font-medium transition-all border',
                    saveMetaForm.status === opt.value
                      ? `${opt.bg} ${opt.color} border-current`
                      : 'bg-slate-900/30 text-slate-500 border-slate-700/50 hover:border-slate-600 hover:text-slate-300'
                  ]"
                >
                  {{ opt.label }}
                </button>
              </div>
            </div>

            <!-- 备份信息展示 -->
            <div v-if="backups.length > 0" class="rounded-lg bg-slate-900/60 border border-slate-700/40 p-3">
              <div class="flex items-center justify-between text-xs mb-2">
                <span class="text-slate-400">本地自动备份</span>
                <button @click="clearAllBackups()" class="text-slate-600 hover:text-red-400 transition-colors">清除所有备份</button>
              </div>
              <div class="max-h-28 overflow-y-auto space-y-1">
                <div
                  v-for="b in backups.slice(0, 5)"
                  :key="b.id"
                  class="flex items-center justify-between text-xs px-2 py-1.5 rounded bg-slate-800/50"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="shrink-0 text-emerald-400">{{ b.label }}</span>
                    <span class="text-slate-500 truncate">{{ backupTimeAgo(b.timestamp) }}</span>
                  </div>
                  <div class="flex items-center gap-2 shrink-0 ml-2">
                    <span class="text-slate-600">{{ backupFormatSize(b.size) }}</span>
                    <button @click="removeBackup(b.id)" class="text-slate-600 hover:text-red-400 transition-colors" title="删除此备份">&times;</button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="mt-6 flex justify-end gap-3">
            <button
              @click="showSaveMetaDialog = false"
              class="px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors"
            >跳过</button>
            <button
              @click="confirmSaveMeta()"
              :disabled="isUpdatingProject"
              class="px-5 py-2 text-sm bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
            >
              <svg v-if="isUpdatingProject" class="animate-spin w-3.5 h-3.5" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isUpdatingProject ? '保存中...' : '确认保存' }}
            </button>
          </div>
        </div>
      </div>

      <!-- 保存为：另存为新项目弹窗 -->
      <div v-if="showSaveAsDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click.self="showSaveAsDialog = false">
        <div class="bg-slate-800 rounded-xl border border-slate-700/50 w-full max-w-md p-6 shadow-xl">
          <!-- 标题 -->
          <div class="flex items-center gap-3 mb-5">
            <div class="w-10 h-10 rounded-full bg-indigo-500/20 flex items-center justify-center">
              <svg class="w-5 h-5 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
            <div>
              <h3 class="text-lg font-semibold text-white">另存为新项目</h3>
              <p class="text-xs text-slate-400 mt-0.5">创建当前内容的独立副本，原项目不受影响</p>
            </div>
          </div>

          <!-- 表单 -->
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-slate-300 mb-1.5">新项目名称 <span class="text-red-400">*</span></label>
              <input
                ref="saveAsInputRef"
                v-model="saveAsName"
                type="text"
                placeholder="输入新项目名称，如「校园青春短剧 v2」"
                maxlength="100"
                class="w-full bg-slate-900/50 border border-slate-600 rounded-lg px-3 py-2.5 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-colors"
                @keyup.enter="confirmSaveAs()"
              />
              <p v-if="currentProjectId" class="mt-1.5 text-[11px] text-slate-500 flex items-center gap-1">
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                将基于当前项目创建独立副本，包含完整的剧本内容
              </p>
            </div>

            <div v-if="saveAsError" class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-500/20 text-sm text-red-300">
              {{ saveAsError }}
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="mt-6 flex justify-end gap-3">
            <button
              @click="showSaveAsDialog = false"
              class="px-4 py-2 text-sm text-slate-400 hover:text-white transition-colors"
            >取消</button>
            <button
              @click="confirmSaveAs()"
              :disabled="isCreatingSaveAs || !saveAsName.trim()"
              class="px-5 py-2 text-sm bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
            >
              <svg v-if="isCreatingSaveAs" class="animate-spin w-3.5 h-3.5" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ isCreatingSaveAs ? '创建中...' : '确认另存为' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, provide, onMounted, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import yaml from 'js-yaml'
import { useScriptStore } from '~/stores/scriptStore'
import { useCopilotStore } from '~/stores/copilotStore'
import { useGraphStore } from '~/stores/graphStore'
import { useProjectStore } from '~/stores/projectStore'
import { useDeepSeekKey } from '~/composables/useDeepSeekKey'
import { useDialog } from '~/composables/useDialog'
import { useEditorLayout } from '~/composables/useEditorLayout'
import { useAutoBackup } from '~/composables/useAutoBackup'
import { useKeyboardShortcuts, bindingToString } from '~/composables/useKeyboardShortcuts'

import CausalGraph from '~/components/CausalGraph.vue'
import RelationNetwork from '~/components/RelationNetwork.vue'
import EmotionCurve from '~/components/EmotionCurve.vue'
import Player from '~/components/Player.vue'
import CopilotChat from '~/components/CopilotChat.vue'
import BeatCard from '~/components/BeatCard.vue'

const route = useRoute()
const scriptStore = useScriptStore()
const copilotStore = useCopilotStore()
const graphStore = useGraphStore()
const projectStore = useProjectStore()
const deepSeek = useDeepSeekKey()
const dialog = useDialog()
const { layout, resetToDefaults, setLeftWidth, setRightWidth, setEmotionHeight } = useEditorLayout()
const {
  backups,
  lastBackupTime,
  createBackup,
  removeBackup,
  clearAllBackups,
  startAutoSave,
  stopAutoSave,
  timeAgo: backupTimeAgo,
  formatSize: backupFormatSize,
} = useAutoBackup()
const {
  registerShortcut,
  unregisterShortcut,
  getRegisteredShortcuts,
  setUserBinding,
  resetUserBinding,
  detectConflicts,
  bindingToString: keyLabel,
  parseBindingString,
} = useKeyboardShortcuts()

// ==================== UI 状态 ====================
type LoadStatus = 'loading' | 'loaded' | 'empty' | 'error'

const leftTab = ref<'causal' | 'relation'>('causal')
const rightTab = ref<'player' | 'source'>('player')
const showSettings = ref(false)
const isSaving = ref(false)
const yamlSource = ref('')
const loadStatus = ref<LoadStatus>('loading')
const loadError = ref('')

// 保存相关状态
const currentScriptId = ref<string>('')
const currentProjectId = ref<string>('')
const showSaveDialog = ref(false)
const newProjectName = ref('')
const saveError = ref('')
const isCreatingProject = ref(false)
const projectNameInput = ref<HTMLInputElement | null>(null)

// 已有项目保存：元数据编辑弹窗
const showSaveMetaDialog = ref(false)
const saveMetaForm = ref({
  title: '',
  owner: '',
  description: '',
  status: 'draft',
})

// 项目状态选项
const projectStatusOptions = [
  { value: 'draft', label: '草稿', color: 'text-slate-400', bg: 'bg-slate-500/20' },
  { value: 'in_progress', label: '进行中', color: 'text-blue-400', bg: 'bg-blue-500/20' },
  { value: 'reviewing', label: '审核中', color: 'text-yellow-400', bg: 'bg-yellow-500/20' },
  { value: 'completed', label: '已完成', color: 'text-emerald-400', bg: 'bg-emerald-500/20' },
]
const isUpdatingProject = ref(false)

// 保存为：另存为新项目弹窗
const showSaveAsDialog = ref(false)
const saveAsName = ref('')
const saveAsError = ref('')
const isCreatingSaveAs = ref(false)
const saveAsInputRef = ref<HTMLInputElement | null>(null)

// 快捷键保存反馈提示
const saveFeedbackVisible = ref(false)
const saveFeedbackMessage = ref('')
let saveFeedbackTimer: ReturnType<typeof setTimeout> | null = null

function showSaveFeedback(message: string) {
  saveFeedbackMessage.value = message
  saveFeedbackVisible.value = true
  if (saveFeedbackTimer) clearTimeout(saveFeedbackTimer)
  saveFeedbackTimer = setTimeout(() => {
    saveFeedbackVisible.value = false
  }, 2000)
}

// 快捷键设置面板状态
const showShortcutSettings = ref(false)
const shortcutEditId = ref('')
const shortcutEditValue = ref('')
const shortcutConflictWarning = ref('')

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

// ==================== 工具函数 ====================

/** 格式化时间戳 */
function formatTime(timestamp: number): string {
  const d = new Date(timestamp)
  return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}:${d.getSeconds().toString().padStart(2, '0')}`
}

/** 截断文本 */
function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text
  return text.slice(0, maxLen) + '...'
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
        // 优先使用具体描述，否则用类型中文标签
        description: e.description || undefined,
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
        trust: entry.trust,
        description: entry.relation_type || entry.description || undefined,
        label: entry.relation_type || undefined,
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

  // 从 URL 参数读取 script_id 和 project_id
  currentScriptId.value = (route.query.script_id as string) || ''
  currentProjectId.value = (route.query.project_id as string) || ''

  // 使用 requestAnimationFrame 让 loading 状态先渲染，避免闪烁
  await new Promise(resolve => requestAnimationFrame(resolve))

  const rawYaml = sessionStorage.getItem('script_yaml')

  if (!rawYaml || !rawYaml.trim()) {
    loadStatus.value = 'empty'
    return
  }

  const success = initFromYaml(rawYaml)
  loadStatus.value = success ? 'loaded' : (loadError.value ? 'error' : 'empty')

  // 数据加载成功后启动自动备份定时器，并自动保存（创建项目+剧本）
  if (success) {
    startAutoSave(() => serializeToYaml())
    // 首次加载且无关联项目时 → 自动执行一次保存（相当于用户点了一次"保存"）
    if (!currentProjectId.value && !currentScriptId.value) {
      autoSaveAfterGeneration()
    }
  }
}

function retryLoad() {
  loadData()
}

/**
 * 生成剧本后自动保存：创建项目 + 保存剧本内容
 * 相当于用户在编辑器中手动点了一次"保存"，静默执行不弹窗
 */
async function autoSaveAfterGeneration() {
  const yamlContent = serializeToYaml()
  if (!yamlContent) return

  try {
    // 1) 创建剧本记录
    const scriptRes = await fetch('/api/scripts', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: scriptData.value?.metadata?.title || '未命名剧本',
        style: scriptData.value?.metadata?.style || 'short_drama',
        yaml_content: yamlContent,
      }),
    })
    if (!scriptRes.ok) return // 后端不可用时静默跳过
    const scriptDataRes = await scriptRes.json()
    const newScriptId = scriptDataRes.id || scriptDataRes.script_id
    currentScriptId.value = newScriptId

    // 2) 创建项目并关联剧本
    const styleMap: Record<string, string> = { short_drama: '短剧', film: '电影', stage: '舞台剧' }
    const projTitle = scriptData.value?.metadata?.title || `${styleMap[scriptData.value?.metadata?.style || 'short_drama'] || '短剧'} - ${new Date().toLocaleString('zh-CN')}`
    const projRes = await fetch('/api/projects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: projTitle,
        description: `${projTitle} - ${scriptStore.scenes.length} 场景`,
        style: scriptData.value?.metadata?.style || 'short_drama',
        status: 'draft',
      }),
    })
    if (!projRes.ok) return
    const project = await projRes.json()
    currentProjectId.value = project.id

    // 3) 关联剧本到项目（更新项目的 script_id）
    await fetch(`/api/projects/${project.id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ script_id: newScriptId }),
    }).catch(() => {})

    // 4) 更新本地项目 store
    projectStore.addProjectToLocal(project)

    // 5) 同步 sessionStorage + 本地备份
    yamlSource.value = yamlContent
    sessionStorage.setItem('script_yaml', yamlContent)
    createBackup(yamlContent, '自动保存')

    console.log(`[自动保存] 项目已创建, project_id=${project.id}, script_id=${newScriptId}`)
    showSaveFeedback('已自动保存')
  } catch (e) {
    console.warn('[自动保存] 失败，用户可稍后手动保存:', e)
  }
}

// 初始化
onMounted(() => {
  loadData()

  // 注册 Ctrl+S 快捷键 → 触发保存（防抖 1s，允许在输入框中触发）
  registerShortcut('save', '保存文档', { key: 's', ctrl: true }, async () => {
    if (isSaving.value) return // 正在保存时跳过
    await handleSave()
    showSaveFeedback('已保存')
  }, {
    debounceMs: 1000,
    allowInInput: true,  // 在 textarea / input 中也能触发保存
  })
})

// ==================== 保存 ====================

/** 将当前编辑器内容序列化为 YAML 字符串 */
function serializeToYaml(): string | null {
  const data = scriptStore.scriptData
  if (!data) return null
  return yaml.dump(data, { lineWidth: -1, quotingType: '"', forceQuotes: true })
}

/** 保存剧本到后端，返回 script_id */
async function saveScriptToBackend(yamlContent: string): Promise<string> {
  // 如果已有 script_id，直接更新
  if (currentScriptId.value) {
    const res = await fetch(`/api/scripts/${currentScriptId.value}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/yaml; charset=utf-8' },
      body: yamlContent,
    })
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: `HTTP ${res.status}` }))
      throw new Error(err.error || err.message || `保存失败 (HTTP ${res.status})`)
    }
    return currentScriptId.value
  }

  // 无 script_id：需要先创建新剧本记录（通过 generate-script 的保存逻辑或单独创建）
  // 这里通过 POST 一个最小化的创建请求来获取新 ID
  const createRes = await fetch('/api/scripts', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      title: scriptData.value?.metadata?.title || '未命名剧本',
      style: scriptData.value?.metadata?.style || 'short_drama',
      yaml_content: yamlContent,
    }),
  })

  if (!createRes.ok) {
    // 如果 POST /api/scripts 不存在，尝试用 PUT 创建（后端可能不支持 POST scripts）
    // 回退方案：提示用户需要先在首页生成
    throw new Error('当前剧本尚未关联到服务器记录，请先在首页生成剧本后再编辑保存')
  }

  const result = await createRes.json()
  const newId = result.id || result.script_id
  if (!newId) throw new Error('服务器未返回剧本 ID')
  currentScriptId.value = newId
  return newId
}

/**
 * 「保存」按钮 — 严格更新当前项目，绝不创建新副本
 * - 已有 project_id → 更新剧本 + 更新项目元数据
 * - 无 project_id → 提示用户使用「保存为」或先关联项目
 */
async function handleSave() {
  isSaving.value = true
  try {
    const yamlContent = serializeToYaml()
    if (!yamlContent) {
      await dialog.alert('没有可保存的剧本数据', { variant: 'warning' })
      return
    }

    // 同步本地 YAML 源码视图 + sessionStorage
    yamlSource.value = yamlContent
    sessionStorage.setItem('script_yaml', yamlContent)

    // === 场景 A：已有项目 → 直接更新 ===
    if (currentProjectId.value) {
      // 1) 保存/更新剧本内容到后端
      const savedScriptId = await saveScriptToBackend(yamlContent)
      console.log(`[保存] 剧本已更新, script_id=${savedScriptId}`)

      // 2) 本地自动备份
      createBackup(yamlContent, '手动保存')

      // 3) 后端快照
      if (savedScriptId) {
        try {
          await fetch(`/api/scripts/${savedScriptId}/snapshots`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/yaml; charset=utf-8' },
            body: yamlContent,
          })
        } catch { /* 快照失败不阻断 */ }
      }

      // 4) 保存完成（不再自动弹出项目信息弹窗）
      return
    }

    // === 场景 B：无关联项目 → 引导用户选择 ===
    const choice = await dialog.confirm(
      '当前剧本尚未关联到任何项目。\n\n• 点击「保存为」可创建一个全新的项目副本\n• 如需更新已有项目，请从首页重新打开该项目',
      {
        title: '未关联项目',
        variant: 'warning',
        confirmText: '前往保存为',
        cancelText: '取消',
      }
    )
    if (choice) {
      handleSaveAs()
    }
  } catch (e: any) {
    console.error('保存失败:', e)
    await dialog.alert(e.message || '保存失败，请检查网络连接', { variant: 'danger' })
  } finally {
    isSaving.value = false
  }
}

/**
 * 「保存为」按钮 — 将当前内容另存为一个全新的项目副本
 * 流程：输入新名称 → 创建新剧本记录 → 创建新项目 → 关联两者
 */
async function handleSaveAs() {
  // 预填充默认名称
  const baseName = scriptData.value?.metadata?.title || '未命名项目'
  saveAsName.value = currentProjectId.value ? `${baseName} (副本)` : baseName
  saveAsError.value = ''
  showSaveAsDialog.value = true
  await nextTick()
  saveAsInputRef.value?.focus()
  saveAsInputRef.value?.select()
}

/** 确认「保存为」：创建新项目副本 */
async function confirmSaveAs() {
  const name = saveAsName.value.trim()
  if (!name) {
    saveAsError.value = '请输入项目名称'
    return
  }

  isCreatingSaveAs.value = true
  saveAsError.value = ''

  try {
    const yamlContent = serializeToYaml()
    if (!yamlContent) throw new Error('没有可保存的剧本数据')

    // 同步本地
    yamlSource.value = yamlContent
    sessionStorage.setItem('script_yaml', yamlContent)

    // 1) 创建新剧本记录（独立的副本）
    const scriptRes = await fetch('/api/scripts', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: name,
        style: scriptData.value?.metadata?.style || 'short_drama',
        yaml_content: yamlContent,
      }),
    })
    if (!scriptRes.ok) throw new Error(`创建剧本副本失败 (HTTP ${scriptRes.status})`)
    const scriptDataRes = await scriptRes.json()
    const newScriptId = scriptDataRes.id || scriptDataRes.script_id
    if (!newScriptId) throw new Error('服务器未返回新剧本 ID')

    // 2) 创建新项目并关联剧本
    const style = scriptData.value?.metadata?.style || 'short_drama'
    const projRes = await fetch('/api/projects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: name,
        description: `${name} - ${scriptStore.scenes.length} 场景（副本）`,
        style,
      }),
    })
    if (!projRes.ok) throw new Error(`创建项目副本失败 (HTTP ${projRes.status})`)
    const newProject = await projRes.json()

    // 3) 关联剧本到新项目
    if (newProject.id && newScriptId) {
      await fetch(`/api/projects/${newProject.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ script_id: newScriptId }),
      }).catch(() => {})
    }

    // 4) 切换当前上下文到新项目（用户现在在编辑这个新副本）
    currentScriptId.value = newScriptId
    currentProjectId.value = newProject.id

    // 5) 本地备份
    createBackup(yamlContent, `另存为: ${name}`)

    // 6) 更新本地项目列表
    projectStore.addProjectToLocal({
      id: newProject.id,
      title: name,
      description: '',
      style,
      status: 'draft',
      owner: 'anonymous',
      novel_preview: null,
      script_id: newScriptId,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    })

    showSaveAsDialog.value = false

    await dialog.alert(
      `已成功将内容另存为新项目「${name}」。\n原项目不受影响，您现在正在编辑新副本。`,
      { variant: 'success', title: '另存为成功' }
    )
  } catch (e: any) {
    saveAsError.value = e.message || '另存为失败，请重试'
  } finally {
    isCreatingSaveAs.value = false
  }
}

/** 已有项目保存时：显示元数据编辑弹窗 */
async function showExistingProjectSaveDialog() {
  // 先获取当前项目的最新信息用于填充表单
  try {
    const res = await fetch(`/api/projects/${currentProjectId.value}`)
    if (res.ok) {
      const project = await res.json()
      saveMetaForm.value = {
        title: project.title || '',
        owner: project.owner || '',
        description: project.description || '',
        status: project.status || 'draft',
      }
    }
  } catch { /* 使用默认值 */ }

  if (!saveMetaForm.value.title) {
    saveMetaForm.value.title = scriptData.value?.metadata?.title || ''
  }
  showSaveMetaDialog.value = true
}

/** 确认元数据修改并更新项目 */
async function confirmSaveMeta() {
  isUpdatingProject.value = true
  try {
    const res = await fetch(`/api/projects/${currentProjectId.value}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: saveMetaForm.value.title || undefined,
        owner: saveMetaForm.value.owner || undefined,
        description: saveMetaForm.value.description || undefined,
        status: saveMetaForm.value.status || undefined,
      }),
    })

    if (!res.ok) throw new Error(`更新项目失败 (HTTP ${res.status})`)

    showSaveMetaDialog.value = false

    // 构建成功反馈信息
    const parts = [`剧本内容已保存`]
    if (backups.value.length > 0) {
      parts.push(`本地备份 #${backups.value.length} 已创建`)
    }
    await dialog.alert(parts.join('，') + '。', {
      variant: 'success',
      title: '保存成功',
    })
  } catch (e: any) {
    await dialog.alert(e.message || '更新项目信息失败', { variant: 'danger' })
  } finally {
    isUpdatingProject.value = false
  }
}

// ==================== 工程命名弹窗 ====================

async function confirmProjectName() {
  const name = newProjectName.value.trim()
  if (!name) return

  isCreatingProject.value = true
  saveError.value = ''

  try {
    const style = scriptData.value?.metadata?.style || 'short_drama'
    const res = await fetch('/api/projects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: name,
        description: `${scriptData.value?.metadata?.title || ''} - ${scriptStore.scenes.length} 场景`,
        style,
      }),
    })

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: `HTTP ${res.status}` }))
      throw new Error(err.error || err.detail || '创建项目失败')
    }

    const project = await res.json()
    currentProjectId.value = project.id

    // 关联剧本到项目（更新项目的 script_id）
    if (currentScriptId.value && project.id) {
      await fetch(`/api/projects/${project.id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ script_id: currentScriptId.value }),
      }).catch(() => {}) // 非关键操作，失败不阻断
    }

    // 更新本地 project store
    projectStore.addProjectToLocal({
      id: project.id,
      title: name,
      description: '',
      style,
      status: 'draft',
      owner: 'anonymous',
      novel_preview: null,
      script_id: currentScriptId.value,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    })

    showSaveDialog.value = false
    newProjectName.value = ''
    await dialog.alert(`工程「${name}」创建成功！可在首页查看和管理。`, { variant: 'success' })
  } catch (e: any) {
    saveError.value = e.message || '创建失败，请重试'
  } finally {
    isCreatingProject.value = false
  }
}

function skipNaming() {
  showSaveDialog.value = false
  newProjectName.value = ''
  saveError.value = ''
}

function closeSaveDialog() {
  if (!isCreatingProject.value) {
    skipNaming()
  }
}

// 导入
async function handleImport(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  const reader = new FileReader()
  reader.onload = async (e) => {
    const text = e.target?.result as string || ''
    if (!text.trim()) return

    // 使用公共初始化函数
    const success = initFromYaml(text)
    if (success) {
      sessionStorage.setItem('script_yaml', text)
      loadStatus.value = 'loaded'
      emitScriptUpdate()
    } else {
      await dialog.alert(loadError.value || '文件格式错误，无法解析为有效的 YAML 剧本数据', { variant: 'danger' })
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

async function handleDeleteBeat(sceneId: number, beatIndex: number) {
  if (await dialog.confirm('确定删除该节拍？', { variant: 'danger' })) {
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

// ==================== 场景编辑 ====================
const editingSceneTime = ref<number | null>(null)

function updateSceneField(sceneId: number, field: 'location' | 'time' | 'emotion_intensity', value: any) {
  scriptStore.updateSceneInfo(sceneId, { [field]: value } as any)
  emitScriptUpdate()
}

async function handleDeleteScene(sceneId: number) {
  if (await dialog.confirm('确定删除整个场景？该场景下所有节拍将被删除。', { variant: 'danger' })) {
    scriptStore.removeScene(sceneId)
    emitScriptUpdate()
  }
}

// ==================== 节拍增强编辑 ====================

function handleUpdateBeatType(sceneId: number, beatIndex: number, newType: string) {
  scriptStore.updateBeatType(sceneId, beatIndex, newType)
  emitScriptUpdate()
}

function handleUpdateSpeaker(sceneId: number, beatIndex: number, speaker: string) {
  scriptStore.updateBeatSpeaker(sceneId, beatIndex, speaker)
  emitScriptUpdate()
}

function handleUpdateEmotion(sceneId: number, beatIndex: number, emotion: string) {
  scriptStore.updateBeatEmotion(sceneId, beatIndex, emotion)
  emitScriptUpdate()
}

// ==================== AI 重生成 ====================

/** AI 重生成单个节拍 */
async function handleAIRegenerateBeat(sceneId: number, beatIndex: number) {
  const scene = scriptStore.scenes.find(s => s.id === sceneId)
  if (!scene || !scene.beats[beatIndex]) return
  const beat = scene.beats[beatIndex]

  // 弹出输入框让用户输入修改意图
  const instruction = await dialog.prompt(
    `请描述你希望如何修改此【${beat.type === 'action' ? '动作' : beat.type === 'dialogue' ? '对白' : beat.type === 'monologue' ? '独白' : '括号说明'}】内容：`,
    { title: 'AI 重生成', placeholder: '例如：让对话更自然、增加紧张感、改变语气...' }
  )
  if (!instruction) return // 用户取消

  const regenKey = `${sceneId}:${beatIndex}`
  scriptStore.setRegenerating(regenKey, true)

  try {
    const res = await fetch('/api/co-pilot/regenerate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        target: 'beat',
        scene_id: sceneId,
        beat_index: beatIndex,
        instruction,
        current_content: beat.content,
        full_script_yaml: scriptStore.yamlText,
      }),
    })

    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data = await res.json()

    scriptStore.applyAIRegeneration(
      sceneId,
      beatIndex,
      data.content,
      data.alternatives?.map((a: any) => ({ content: a.content, tone: a.tone }))
    )
    emitScriptUpdate()

    await dialog.alert('AI 已重新生成内容，原版本已保存为备选方案。', { variant: 'success', title: '重生成完成' })
  } catch (e: any) {
    await dialog.alert(`AI 重生成失败：${e.message}`, { variant: 'danger', title: '错误' })
  } finally {
    scriptStore.setRegenerating(regenKey, false)
  }
}

/** AI 重生成整场场景 */
async function handleAIRegenerateScene(sceneId: number) {
  const instruction = await dialog.prompt(
    '请描述你希望如何重写整个场景的内容：',
    { title: 'AI 重生成场景', placeholder: '例如：调整整体节奏、增强冲突感、改变氛围...' }
  )
  if (!instruction) return

  scriptStore.setRegenerating(String(sceneId), true)

  try {
    const res = await fetch('/api/co-pilot/regenerate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        target: 'scene',
        scene_id: sceneId,
        instruction,
        full_script_yaml: scriptStore.yamlText,
      }),
    })

    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data = await res.json()

    // 如果返回了 beats 数组，替换整场
    if (data.beats && Array.isArray(data.beats)) {
      scriptStore.applyAISceneRegeneration(sceneId, data.beats as any[])
    }
    emitScriptUpdate()

    await dialog.alert('场景已由 AI 重新生成。', { variant: 'success', title: '重生成完成' })
  } catch (e: any) {
    await dialog.alert(`AI 重生成失败：${e.message}`, { variant: 'danger', title: '错误' })
  } finally {
    scriptStore.setRegenerating(String(sceneId), false)
  }
}

// ==================== 修改历史面板 ====================

const historyPanelVisible = ref(false)
const historyTarget = ref<{ sceneId: number; beatIndex: number } | null>(null)

const historyRecords = computed(() => {
  if (!historyTarget.value) return []
  return scriptStore.getTargetHistory(`scene:${historyTarget.value.sceneId}:beat:${historyTarget.value.beatIndex}`)
})

function showHistoryPanel(sceneId: number, beatIndex: number) {
  historyTarget.value = { sceneId, beatIndex }
  historyPanelVisible.value = true
}

// ==================== 面板拖拽调整大小 ====================

/** 当前正在进行的拖拽类型 */
const dragging = ref<'left' | 'right' | 'emotion' | null>(null)

function startDragLeft(e: MouseEvent) {
  e.preventDefault()
  dragging.value = 'left'
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
}

function startDragRight(e: MouseEvent) {
  e.preventDefault()
  dragging.value = 'right'
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
}

function startDragEmotion(e: MouseEvent) {
  e.preventDefault()
  dragging.value = 'emotion'
  document.addEventListener('mousemove', onDragMove)
  document.addEventListener('mouseup', onDragEnd)
}

function onDragMove(e: MouseEvent) {
  if (!dragging.value) return
  if (dragging.value === 'left') {
    setLeftWidth(e.clientX)
  } else if (dragging.value === 'right') {
    // 右侧面板：从窗口右边缘算起
    setRightWidth(window.innerWidth - e.clientX)
  } else if (dragging.value === 'emotion') {
    const container = document.querySelector('.h-screen.flex.flex-col')
    if (container) {
      const rect = (container as HTMLElement).getBoundingClientRect()
      const headerHeight = 56 // header 高度约 56px
      setEmotionHeight(e.clientY - rect.top - headerHeight)
    }
  }
}

function onDragEnd() {
  dragging.value = null
  document.removeEventListener('mousemove', onDragMove)
  document.removeEventListener('mouseup', onDragEnd)
}

// ==================== 快捷键设置编辑 ====================

const shortcutInputRef = ref<HTMLInputElement | null>(null)

/** 开始编辑某个快捷键 */
function startShortcutEdit(sc: { id: string; defaultBinding: any; userBinding: any }) {
  shortcutEditId.value = sc.id
  // 预填充当前绑定值
  const current = sc.userBinding || sc.defaultBinding
  shortcutEditValue.value = keyLabel(current).toLowerCase()
  shortcutConflictWarning.value = ''
  nextTick(() => {
    shortcutInputRef.value?.focus()
    shortcutInputRef.value?.select()
  })
}

/** 确认快捷键修改（含冲突检测） */
function confirmShortcutEdit(id: string) {
  const raw = shortcutEditValue.value.trim()
  if (!raw) {
    cancelShortcutEdit()
    return
  }

  const parsed = parseBindingString(raw)
  if (!parsed) {
    shortcutConflictWarning.value = '无法解析的快捷键格式，示例: ctrl+s'
    return
  }

  // 冲突检测
  const conflicts = detectConflicts(parsed, id)
  if (conflicts.length > 0) {
    const names = getRegisteredShortcuts()
      .filter(s => conflicts.includes(s.id))
      .map(s => s.name)
      .join('、')
    shortcutConflictWarning.value = `与「${names}」快捷键冲突，请更换`
    return
  }

  // 应用修改
  setUserBinding(id, parsed)
  shortcutEditId.value = ''
  shortcutConflictWarning.value = ''
}

/** 取消编辑 */
function cancelShortcutEdit() {
  shortcutEditId.value = ''
  shortcutEditValue.value = ''
  shortcutConflictWarning.value = ''
}
</script>

<style scoped>
/* 快捷键保存反馈过渡动画 */
.feedback-fade-enter-active {
  transition: all 0.25s ease-out;
}
.feedback-fade-leave-active {
  transition: all 0.4s ease-in;
}
.feedback-fade-enter-from {
  opacity: 0;
  transform: translate(-50%, -12px);
}
.feedback-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, -8px);
}
</style>
