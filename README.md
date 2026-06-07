# Novel2Script Pro - AI 智能小说改编剧本平台

> 基于 DeepSeek 大模型的 AI 辅助小说转剧本工具，支持情感热力图、因果图谱、关系网络、TTS 朗读、AI 编剧等创新功能。

## 功能特性

### 核心功能
- **智能流水线生成** - 两阶段 Pipeline 架构：Stage 1 全局分析（角色提取、章节划分）→ Stage 2 分块场景生成，忠实还原原著情节与人物设定
- **TTS 语音朗读** - 基于 Web Speech API 的节拍朗读引擎，支持播放/暂停/恢复完整控制，自动等待当前节拍读完再切换下一节拍
- **情感曲线分析** - 基于规则引擎的情感强度分析，可视化展示章节情绪起伏，支持手动拖拽调整
- **因果图谱 (Causal Graph)** - 基于 petgraph 的有向图分析，DFS 下游事件影响评估，事件与边均锚定原文证据
- **角色关系网络 (Relation Network)** - 角色间关系度量（亲密度、冲突度、信任度），每个数值附带判定依据说明
- **AI 编剧助手 (Co-Pilot)** - 实时对话式编剧建议，基于上下文的智能推荐
- **AI 节拍重写 & 备选方案** - 单节拍级别 AI 重新生成，支持普通节拍与群戏节拍（多人参与者）的差异化提示词；多备选方案一键切换对比

### 编辑器特性
- **三栏布局** - 左侧章节导航（单击定位 / 双击跳转首个场景）、中间场景编辑区、右侧大纲面板（因果图/关系图/情感曲线切换）
- **盲演模式** - 隐藏角色名/场景信息，专注台词本身
- **跨媒体桥接** - 自动标注镜头语言、音效、灯光提示
- **反套路检测器** - 识别常见陈词滥调并给出替代建议
- **自动备份** - 基于防抖的本地存储自动保存机制
- **键盘快捷键** - 全局快捷键支持（保存、撤销等）
- **LLM 自动降级** - API 调用失败时自动降级为 Mock 模式，保证服务可用性

## 技术架构

```
┌─────────────────────────────┐         ┌──────────────────────┐
│   Frontend (Nuxt 3.21)      │  ────►  │   Backend (Rust/Axum) │
│   Vue 3.5 + TypeScript      │         │   + PostgreSQL        │
│   ECharts + NaiveUI + Vite 7│         │   + DeepSeek API      │
│   Pinia 状态管理             │         │   Pipeline 双阶段     │
└─────────────────────────────┘         └──────────────────────┘
```

### 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| **前端框架** | Nuxt 3.21 (Vue 3.5 + TypeScript) | SSR 支持，Pinia 状态管理，Vite 7 构建 |
| **UI 组件库** | Naive UI 2.40 | 高质量 Vue 3 组件库 |
| **CSS 方案** | Tailwind CSS 3.4 | 原子化 CSS 工具类 |
| **可视化** | ECharts 5 + vue-echarts 7 | 因果图、关系网络、情感曲线 |
| **后端框架** | Axum 0.7 (Rust) | 高性能异步 Web 框架 |
| **数据库** | PostgreSQL 15 + sqlx 0.7 | 异步数据库访问（tokio + rustls） |
| **AI 引擎** | DeepSeek API (OpenAI 兼容) | 剧本生成与 AI 对话 |
| **图算法** | petgraph 0.6 (Rust) | DFS 因果分析与关系网络计算 |
| **HTTP 客户端** | reqwest 0.12 | DeepSeek API 调用（JSON + Stream） |

## 项目结构

```
s1d3_director/
├── backend/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs             # 应用入口、路由注册、DB 迁移
│   │   ├── handlers.rs         # HTTP API 处理器
│   │   ├── models.rs           # 请求/响应数据模型定义
│   │   ├── utils.rs            # 工具函数集
│   │   ├── llm_client.rs       # DeepSeek API 客户端封装
│   │   ├── pipeline_generator.rs  # ★ 核心流水线生成器（两阶段）
│   │   ├── script_generator.rs  # 剧本生成引擎
│   │   ├── causal_graph.rs     # 因果图谱分析与可视化数据
│   │   ├── relation_network.rs # 关系网络计算与度量
│   │   ├── co_pilot.rs         # AI 编剧助手逻辑
│   │   └── emotion_analyzer.rs # 情感强度分析器
│   ├── Cargo.toml
│   ├── .env.example            # 环境变量模板
│   └── .env                    # ⚠️ 不提交到 Git
├── frontend/                   # Nuxt 3 前端
│   ├── pages/
│   │   ├── index.vue           # 首页（小说输入 → 风格选择 → 生成剧本）
│   │   └── editor.vue          # 编辑器主页面（三栏布局）
│   ├── components/
│   │   ├── BeatCard.vue        # 单人节拍卡片（台词编辑 + TTS）
│   │   ├── SceneCard.vue       # 场景卡片（群戏节拍 + 舞台指示）
│   │   ├── ChapterCard.vue     # 章节折叠卡片容器
│   │   ├── CausalGraph.vue     # 因果图谱可视化
│   │   ├── RelationNetwork.vue # 关系网络力导向图
│   │   ├── EmotionCurve.vue    # 情感曲线拖拽图表
│   │   ├── CopilotChat.vue     # AI 对话交互组件
│   │   ├── Player.vue          # TTS 播放控制条
│   │   └── AppDialog.vue       # 全局对话框组件
│   ├── stores/                 # Pinia 全局状态
│   │   ├── scriptStore.ts      # 剧本数据（章节/场景/节拍）
│   │   ├── graphStore.ts       # 图谱数据（因果/关系/情感）
│   │   ├── copilotStore.ts     # AI 对话状态
│   │   └── projectStore.ts     # 项目元信息
│   ├── composables/            # 组合式函数
│   │   ├── useAutoBackup.ts    # 自动备份（防抖保存）
│   │   ├── useDeepSeekKey.ts   # API Key 管理
│   │   ├── useDialog.ts        # 对话框状态管理
│   │   ├── useEditorLayout.ts  # 编辑器布局控制
│   │   └── useKeyboardShortcuts.ts  # 键盘快捷键绑定
│   ├── assets/
│   │   └── main.css            # 全局样式（滚动条美化等）
│   ├── nuxt.config.ts
│   └── package.json
├── 1.md                        # 项目规格文档
├── README.md                   # 本文件
└── .gitignore
```

## 安装指南

### 前置要求

- [Rust](https://rustup.rs/) >= 1.75
- [Node.js](https://nodejs.org/) >= 18
- [PostgreSQL](https://www.postgresql.org/) >= 15
- npm 或 pnpm 或 yarn

### 1. 克隆项目

```bash
git clone https://github.com/s0uths1d3/s1d3_director.git
cd s1d3_director
```

### 2. 后端配置

```bash
cd backend
cp .env.example .env
```

编辑 `.env` 文件：

```env
# DeepSeek API 配置（必填以启用真实 AI 功能，留空则使用 Mock 模式）
DEEPSEEK_API_KEY=sk-your-api-key-here
DEEPSEEK_BASE_URL=https://api.deepseek.com/v1

# 数据库连接
DATABASE_URL=postgres://postgres:your-password@localhost:5432/novel2script_pro

# 服务端口
PORT=8081
```

### 3. 启动后端

```bash
cd backend
cargo run
```

后端默认监听 `http://localhost:8081`

### 4. 启动前端

```bash
cd frontend
npm install
npm run dev
```

前端默认运行在 `http://localhost:3000`，自动代理 `/api` 到后端 `8081`。

## 使用方法

1. **打开前端页面** - 访问 `http://localhost:3000`
2. **输入小说文本** - 在首页粘贴或上传待改编的小说内容（支持 .txt / .md）
3. **配置项目** - 填写项目名称和描述（可选，留空则由 AI 自动生成）
4. **选择风格** - 选择目标剧本风格（短剧/电影/舞台剧/动漫）
5. **功能选项** - 可选开启：有痕模式、风网要语、关系统网、媒体提示
6. **生成剧本** - 点击"生成剧本"，AI 将执行两阶段流水线：
   - Stage 1：全局分析 → 提取角色（3~15 人）、划分章节（每章 2~5 场景）、生成摘要
   - Stage 2：分块生成 → 逐章生成场景卡片、节拍、台词、舞台指示
7. **编辑优化** - 在编辑器中查看生成的剧本，可：
   - 直接编辑节拍内容（富文本 contenteditable，支持占位符解析）
   - 调整情感曲线（拖拽节点）
   - 查看因果图谱和关系网络
   - 使用 TTS 朗读验证台词节奏
   - 使用 AI 编剧助手获取建议
   - 左侧导航：单击定位章节 / 双击跳转首个场景
8. **导出 YAML** - 剧本数据自动同步为 YAML 格式

## API 接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| POST | `/api/generate-script` | 流水线剧本生成（核心：两阶段） |
| POST | `/api/causal/impact` | 因果影响分析 |
| POST | `/api/relation/update` | 更新关系网络 |
| POST | `/api/co-pilot/chat` | AI 对话 |
| POST | `/api/co-pilot/suggest` | AI 建议 |
| POST | `/api/co-pilot/regenerate` | AI 重新生成节拍（支持群戏） |
| POST | `/api/co-pilot/alternatives` | AI 生成备选方案（支持群戏） |

## 已知限制

- 当前 LLM 集成仅支持 DeepSeek API（OpenAI 兼容格式）
- Mock 模式下返回固定示例数据，不进行真实的语义分析
- TTS 播放器基于浏览器 Web Speech API，效果取决于操作系统内置语音
- 数据库表结构已创建但部分高级存储功能仍在完善中
- 流水线生成对长篇小说（>10 万字）的处理时间较长

## 开源协议

MIT License

## 更新记录

### v0.3.0 (2026-06-07) - 流水线架构重构 & 编辑器体验升级

**后端重大更新：**
- 新增 `pipeline_generator.rs` — 两阶段流水线生成架构
  - Stage 1：全局分析（角色提取 3~15 人 / 章节按叙事完整性划分 2~5 场景/章）
  - Stage 2：分块场景生成（忠实改编铁律 / 节拍丰富度 5~10 个/场景 / 原文映射锚定）
  - LLM 输出容错：智能处理 JSON 对象包裹数组格式（`{"scenes":[...]}`）
  - JSON 提取函数拆分为对象/数组两套专用路径，修复 Stage1 解析回归问题
- 因果图谱：截断上限扩容 3x（5000→15000 字），新增事件/边提取指南与 strength 分级标准
- 关系网络：同步扩容截断上限，新增数值校准规则与判定依据要求

**前端重大更新：**
- **Player.vue TTS 引擎重写**：setInterval 固定定时器 → async/await Promise 驱动 playLoop()
  - TTS 开启时：await speakContent() 等朗读完毕后自动切换下一节拍
  - 暂停/恢复完整实现：speechSynthesis.pause() / resume() + isPaused 状态机
- **BeatCard.vue 台词编辑区全面重构**（6 项缺陷修复）：
  - 反向占位符解析（blur 时 char_XXX ID 保持不变）
  - 编辑状态保护（外部更新不干扰光标位置）
  - 富文本粘贴过滤（@paste 只保留纯文本）
  - class-based 占位符方案（替代 :empty 伪类）
  - 输入时光标上下边界自动跟随滚动
- **SceneCard.vue 群戏区域优化**：span→div 支持块级滚动、全区域粘贴安全
- **editor.vue 编辑器布局**：
  - 首页左右面板 Grid 等高对齐（items-stretch + flex-1 自适应）
  - 中间区域 min-h-0 修复 flex 子项滚动失效
  - 章节导航双击跳转首个场景（scrollIntoView 居中）+ select-none 防选中
- **全局滚动条美化**（main.css）：8px padding-box 内嵌边框 + active 状态

**新增 Composables：**
- `useAutoBackup.ts` — 防抖自动备份
- `useDeepSeekKey.ts` — API Key 管理
- `useDialog.ts` — 对话框状态
- `useEditorLayout.ts` — 布局控制
- `useKeyboardShortcuts.ts` — 快捷键绑定

**新增组件：**
- `ChapterCard.vue` — 章节折叠容器
- `SceneCard.vue` — 场景卡片（群戏节拍）
- `AppDialog.vue` — 全局对话框

### v0.2.0 (2026-06-05) - Nuxt 版本大升级

**依赖升级：**
| 包名 | 旧版本 | 新版本 |
|------|--------|--------|
| nuxt | ^3.12.4 | **^3.21.7** |
| vue | ^3.4.27 | **^3.5.13** |
| vue-router | ^4.3.2 | **^4.5.0** |
| pinia | ^2.1.7 | **^2.3.0** |
| @pinia/nuxt | ^0.5.1 | **^0.9.0** |
| vue-echarts | ^6.7.3 | **^7.0.3** |
| echarts | ^5.5.0 | **^5.5.1** |
| naive-ui | ^2.38.0 | **^2.40.0** |
| tailwindcss | ^3.4.4 | **^3.4.17** |
| @nuxtjs/tailwindcss | ^6.12.0 | **^6.13.0** |

**配套升级：**
- Nitro: 2.x → **2.13.4**
- Vite: 5.x → **7.3.5**
- compatibilityDate: 2024-06-08 → **2025-10-28**

### v0.1.0 (2026-06-05) - 项目初始化

- 完整的前后端架构搭建（Rust/Axum + Nuxt/Vue）
- 6 个 API 端点实现
- 前端可视化组件（ECharts 因果图/关系网络/情感曲线）
- AI 编剧助手功能
- PostgreSQL 数据库集成
- LLM Mock 模式降级策略
