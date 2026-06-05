# Novel2Script Pro - AI 智能小说改编剧本平台

> 基于 DeepSeek 大模型的 AI 辅助小说转剧本工具，支持情感热力图、因果图谱、关系网络、AI 编剧等创新功能。

## 功能特性

### 核心功能
- **智能剧本生成** - 输入小说文本，自动生成结构化剧本（YAML 格式），支持短剧/电影/舞台剧/动漫等多种风格
- **情感曲线分析** - 基于规则引擎的情感强度分析，可视化展示章节情绪起伏，支持手动拖拽调整
- **因果图谱 (Causal Graph)** - 基于 petgraph 的有向图分析，DFS 下游事件影响评估
- **角色关系网络 (Relation Network)** - 角色间关系度量（亲密度、冲突度、信任度），力导向图可视化
- **AI 编剧助手 (Co-Pilot)** - 实时对话式编剧建议，基于上下文的智能推荐

### 创新特性
- **盲演模式** - 隐藏角色名/场景信息，专注台词本身
- **跨媒体桥接** - 自动标注镜头语言、音效、灯光提示
- **反套路检测器** - 识别常见陈词滥调并给出替代建议
- **LLM 自动降级** - API 调用失败时自动降级为 Mock 模式，保证服务可用性

## 技术架构

```
┌─────────────────────────────┐         ┌──────────────────────┐
│   Frontend (Nuxt 3.21)      │  ────►  │   Backend (Rust/Axum) │
│   Vue 3.5 + TypeScript      │         │   + PostgreSQL        │
│   ECharts + Monaco + Vite 7│         │   + DeepSeek API      │
└─────────────────────────────┘         └──────────────────────┘
```

### 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| **前端框架** | Nuxt 3.21 (Vue 3.5 + TypeScript) | SSR 支持，Pinia 状态管理，Vite 7 构建 |
| **后端框架** | Axum 0.7 (Rust) | 高性能异步 Web 框架 |
| **数据库** | PostgreSQL + sqlx | 异步数据库访问 |
| **AI 引擎** | DeepSeek API (OpenAI 兼容) | 剧本生成与 AI 对话 |
| **可视化** | ECharts 5 | 因果图、关系网络、情感曲线 |
| **代码编辑** | Monaco Editor | VS Code 同款编辑器 |
| **图算法** | petgraph (Rust) | DFS 因果分析 |

## 项目结构

```
s1d3_director/
├── backend/                 # Rust 后端
│   ├── src/
│   │   ├── main.rs          # 入口、路由、DB 迁移
│   │   ├── handlers.rs      # API 处理器 (7 个端点)
│   │   ├── models.rs        # 请求/响应数据模型
│   │   ├── llm_client.rs    # DeepSeek API 客户端
│   │   ├── script_generator.rs  # 核心剧本生成引擎
│   │   ├── causal_graph.rs  # 因果图谱分析
│   │   ├── relation_network.rs  # 关系网络计算
│   │   ├── co_pilot.rs      # AI 编剧助手
│   │   └── emotion_analyzer.rs  # 情感分析器
│   ├── Cargo.toml
│   ├── .env.example         # 环境变量模板
│   └── .env                 # ⚠️ 不提交到 Git
├── frontend/                # Nuxt 3.21 前端
│   ├── pages/
│   │   ├── index.vue        # 首页（输入小说 → 生成剧本）
│   │   └── editor.vue       # 编辑器（三栏布局）
│   ├── components/
│   │   ├── BeatCard.vue     # 节拍卡片
│   │   ├── EmotionCurve.vue # 情感曲线
│   │   ├── CausalGraph.vue  # 因果图谱
│   │   ├── RelationNetwork.vue  # 关系网络
│   │   ├── CopilotChat.vue  # AI 对话
│   │   └── Player.vue       # TTS 播放器
│   ├── stores/              # Pinia 状态管理
│   ├── composables/         # 组合式函数
│   ├── nuxt.config.ts
│   └── package.json
├── 1.md                     # 项目规格文档
├── README.md                # 本文件
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
2. **输入小说文本** - 在首页粘贴或输入待改编的小说内容
3. **选择风格** - 选择目标剧本风格（短剧/电影/舞台剧/动漫）
4. **生成剧本** - 点击"生成剧本"，等待 AI 分析和转换
5. **编辑优化** - 在编辑器中查看生成的剧本，可：
   - 直接编辑节拍内容（Monaco Editor）
   - 调整情感曲线（拖拽节点）
   - 查看因果图谱和关系网络
   - 使用 AI 编剧助手获取建议
6. **导出 YAML** - 剧本数据自动同步为 YAML 格式

## API 接口

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| POST | `/api/analyze` | 小说文本分析 |
| POST | `/api/generate-script` | 生成剧本（核心） |
| POST | `/api/causal/impact` | 因果影响分析 |
| POST | `/api/relation/update` | 更新关系网络 |
| POST | `/api/co-pilot/chat` | AI 对话 |
| POST | `/api/co-pilot/suggest` | AI 建议 |

## 已知限制

- 当前 LLM 集成仅支持 DeepSeek API（OpenAI 兼容格式）
- Mock 模式下返回固定示例数据，不进行真实的语义分析
- TTS 播放器为前端基础实现，暂未接入专业 TTS 服务
- 数据库表结构已创建但部分高级存储功能仍在完善中

## 开源协议

MIT License

## 更新记录

### v0.2.0 (2026-06-05) - Nuxt 版本升级

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

- 完整的前后端架构搭建
- 7 个 API 端点实现
- 前端可视化组件（ECharts 因果图/关系网络/情感曲线）
- AI 编剧助手功能
- PostgreSQL 数据库集成
- LLM Mock 模式降级策略
