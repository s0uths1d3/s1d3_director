# Novel2Script Pro — 剧本 YAML Schema 规范 (v1.0)

> 本文档定义了 Novel2Script Pro 平台剧本数据的完整 YAML Schema，包含字段规范、类型约束、枚举值与设计决策说明。

***

## 目录

- [一、Schema 概览](#一schema-概览)
- [二、根结构体](#二根结构体-scriptyaml) [`ScriptYaml`](#二根结构体-scriptyaml)
- [三、`metadata`](#三metadata-剧本元信息) [剧本元信息](#三metadata-剧本元信息)
- [四、`characters`](#四characters-角色列表) [角色列表](#四characters-角色列表)
- [五、`chapters`](#五chapters-章节列表) [章节列表](#五chapters-章节列表)
- [六、`scenes`](#六scenes-场景列表核心) [场景列表（核心）](#六scenes-场景列表核心)
- [七、`Beat`](#七beat-节拍最小叙事单元) [节拍（最小叙事单元）](#七beat-节拍最小叙事单元)
- [八、分析产物（可选扩展）](#八分析产物可选扩展)
- [九、完整 YAML 示例](#九完整-yaml-示例)
- [十、设计决策说明](#十设计决策说明)

***

## 一、Schema 概览

### 层级关系图

```
ScriptYaml (根)
├── metadata          # 元信息（必填）
│   └── emotional_curve  # 情感曲线
├── characters[]      # 角色表（必填）
│   └── relationships[]  # 角色间关系
├── chapters[]        # 章节/幕表（可选）
├── scenes[]          # 场景表（必填，核心）
│   ├── beats[]       # 节拍列表
│   │   ├── alternatives[]  # 备选方案
│   │   └── participants[]  # 群戏参与者（可选）
│   └── media_hints   # 影视化提示
├── causal_graph      # 因果图谱（可选，分析产物）
│   ├── events[]
│   └── edges[]
└── relation_network  # 关系网络（可选，分析产物）
    └── matrix[]
```

### 数据流

```
原著小说 → LLM Pipeline (Stage1 + Stage2) → ScriptYaml → 编辑器 ↔ 用户交互 → 导出 YAML
                                        ↓
                              sessionStorage / localStorage / 后端持久化
```

***

## 二、根结构体 `ScriptYaml`

| 字段                 | 类型                    | 必填   | 默认值    | 说明            |
| ------------------ | --------------------- | ---- | ------ | ------------- |
| `metadata`         | **object**            | ✅ 必填 | —      | 剧本级元信息        |
| `characters`       | **array\[Character]** | ✅ 必填 | `[]`   | 全局角色注册表       |
| `scenes`           | **array\[Scene]**     | ✅ 必填 | `[]`   | 场景列表（按 id 排序） |
| `chapters`         | **array\[Chapter]**   | 可选   | `[]`   | 章/幕划分容器       |
| `causal_graph`     | **object / null**     | 可选   | `null` | 因果图谱分析产物      |
| `relation_network` | **object / null**     | 可选   | `null` | 关系网络分析产物      |

**YAML 示例：**

```yaml
metadata: { ... }
characters: [...]
scenes: [...]
chapters: [...]
causal_graph: { events: [], edges: [] }
relation_network: { matrix: [] }
```

***

## 三、`metadata` 剧本元信息

| 字段                | 类型                | 必填   | 默认值    | 说明                    |
| ----------------- | ----------------- | ---- | ------ | --------------------- |
| `title`           | **string**        | ✅ 必填 | —      | 剧本标题                  |
| `source_novel`    | **string**        | ✅ 必填 | —      | 原著小说名称                |
| `adaptation_date` | **string**        | ✅ 必填 | —      | 改编日期（格式 `YYYY-MM-DD`） |
| `style`           | **enum**          | ✅ 必填 | —      | 目标风格（见下方枚举）           |
| `emotional_curve` | **object / null** | 可选   | `null` | 情感曲线数据                |

### `style` 枚举

| 值               | 含义     |
| --------------- | ------ |
| `"film"`        | 电影剧本   |
| `"short_drama"` | 短剧/微短剧 |
| `"stage"`       | 舞台剧    |
| `"anime"`       | 动漫剧本   |

### `emotional_curve` 子对象

| 字段            | 类型            | 必填 | 说明                   |
| ------------- | ------------- | -- | -------------------- |
| `chapters`    | **number\[]** | ✅  | 章节序号数组 `[1, 2, ...]` |
| `intensities` | **number\[]** | ✅  | 对应情感强度值（0.0\~1.0+）   |

**设计约束：** `chapters.length` 必须等于 `intensities.length`。

**YAML 示例：**

```yaml
metadata:
  title: "改编剧本"
  source_novel: "用户上传的小说"
  adaptation_date: "2026-06-06"
  style: film
  emotional_curve:
    chapters: [1, 2, 3]
    intensities: [0.3, 0.55, 0.8]
```

***

## 四、`characters` 角色列表

全局角色注册表。每个角色在此处定义一次，通过 `id` 在全文档中被引用。

### Character 对象

| 字段               | 类型                   | 必填   | 默认值    | 说明                          |
| ---------------- | -------------------- | ---- | ------ | --------------------------- |
| `id`             | **string**           | ✅ 必填 | —      | 唯一标识符，格式 `char_XXX`（三位数字补零） |
| `name`           | **string**           | ✅ 必填 | —      | 角色显示名（如"林曦"、"顾言"）           |
| `traits`         | **string\[]**        | 可选   | `[]`   | 性格特征标签                      |
| `voice`          | **string / null**    | 可选   | `null` | 语言风格描述（供 AI 生成时参考语气）        |
| `arc_summary`    | **string / null**    | 可选   | `null` | 角色弧光/成长轨迹描述                 |
| `motivation`     | **string / null**    | 可选   | `null` | 核心动机或目标                     |
| `relationships`  | **array\[Relation]** | 可选   | `[]`   | 与其他角色的关系                    |
| `first_scene_id` | **number / null**    | 可选   | `null` | 首次出场的场景 ID                  |

### Relation（嵌套在 Character 内）

| 字段                    | 类型                | 必填 | 说明                            |
| --------------------- | ----------------- | -- | ----------------------------- |
| `target_character_id` | **string**        | ✅  | 目标角色 ID（引用 `characters[].id`） |
| `relation_type`       | **enum**          | ✅  | 关系类型（见下方枚举）                   |
| `description`         | **string / null** | 可选 | 自由文本描述                        |

### `relation_type` 枚举

| 值            | 含义    |
| ------------ | ----- |
| `"family"`   | 亲属    |
| `"romance"`  | 恋爱/暧昧 |
| `"rivalry"`  | 对手/竞争 |
| `"mentor"`   | 师徒/指导 |
| `"alliance"` | 同盟/伙伴 |
| `"enemy"`    | 敌对    |
| `"other"`    | 其他    |

**YAML 示例：**

```yaml
characters:
  - id: char_001
    name: 林曦
    traits: [勤奋, 敏感, 善良]
    voice: 温柔而略带犹豫，内心独白细腻
    arc_summary: 从单纯的大学生成长为更自信的职场新人
    motivation: 追求学业和事业成功
    relationships:
      - target_character_id: char_002
        relation_type: romance
        description: 暗恋对象，后发展为恋人
    first_scene_id: 1
```

***

## 五、`chapters` 章节列表

章/幕的容器结构，用于组织场景的层级归属和导航。

### Chapter 对象

| 字段          | 类型                | 必填   | 默认值    | 说明                              |
| ----------- | ----------------- | ---- | ------ | ------------------------------- |
| `id`        | **string**        | ✅ 必填 | —      | 章节标识，格式 `ch_XXX`                |
| `title`     | **string**        | ✅ 必填 | —      | 章节标题                            |
| `order`     | **number**        | ✅ 必填 | —      | 排序序号（从 1 开始递增）                  |
| `plot_line` | **string / null** | 可选   | `null` | 主情节线标签（如 `"plot_main"`）         |
| `summary`   | **string**        | 可选   | `""`   | 本章摘要                            |
| `scene_ids` | **number\[]**     | 可选   | `[]`   | 本章包含的场景 ID 列表（引用 `scenes[].id`） |

**YAML 示例：**

```yaml
chapters:
  - id: ch_001
    title: 阳光下的相遇
    order: 1
    plot_line: plot_main
    summary: 林曦与顾言的初次相遇
    scene_ids: [1, 2, 3, 4, 5]
```

***

## 六、`scenes` 场景列表（核心）

剧本的核心数据层。每个 Scene 代表一个时空连续的戏剧片段。

### Scene 对象

| 字段                   | 类型                | 必填   | 默认值    | 说明                               |
| -------------------- | ----------------- | ---- | ------ | -------------------------------- |
| `id`                 | **number**        | ✅ 必填 | —      | 全局唯一整数 ID（从 1 开始递增）              |
| `location`           | **string**        | ✅ 必填 | —      | 地点（如"咖啡馆 · 靠窗座位"）                |
| `time`               | **string / null** | 可选   | `null` | 时间标注（如"清晨 6:30"、"午后 · 阳光斜射"）     |
| `emotion_intensity`  | **number**        | 可选   | `0.0`  | 情感强度浮点数（典型范围 0.0\~1.0）           |
| `beats`              | **array\[Beat]**  | 可选   | `[]`   | 节拍列表（见下文详述）                      |
| `media_hints`        | **object / null** | 可选   | `null` | 影视化提示（镜头/音效）                     |
| `chapter_id`         | **string / null** | 可选   | `null` | 所属章节 ID（引用 `chapters[].id`）      |
| `characters_present` | **string\[]**     | 可选   | `[]`   | 出场角色 ID 列表（引用 `characters[].id`） |
| `plot_lines`         | **string\[]**     | 可选   | `[]`   | 关联的情节线标签                         |

### MediaHints（嵌套）

| 字段       | 类型         | 必填 | 说明      |
| -------- | ---------- | -- | ------- |
| `camera` | **string** | ✅  | 镜头指导    |
| `music`  | **string** | ✅  | 音乐/音效提示 |

**YAML 示例：**

```yaml
scenes:
  - id: 1
    location: "城市街道 · 清晨"
    time: "清晨 6:30"
    emotion_intensity: 0.3
    chapter_id: ch_001
    characters_present: [char_001, char_002]
    plot_lines: [plot_main]
    beats:
      - type: action
        content: "林曦抬头，看见苏晴端着两杯咖啡走过来..."
        speaker: null
        emotion: 焦虑中带着期待
        participants: []
        plot_line_tags: [plot_main]
      - type: dialogue
        content: "别说了，我感觉我要挂了。"
        speaker: char_001
        emotion: 焦虑中带着期待
        participants: []
        plot_line_tags: [plot_main]
    media_hints:
      camera: "全景 → 缓慢推进至中景"
      music: "钢琴独奏，缓慢渐入"
```

***

## 七、`Beat` 节拍（最小叙事单元）

剧本的最小原子单位。每个 Beat 是一个不可分割的动作或对话片段。

### Beat 对象

| 字段                | 类型                             | 必填   | 默认值    | 说明                                                   |
| ----------------- | ------------------------------ | ---- | ------ | ---------------------------------------------------- |
| `type`            | **enum**                       | ✅ 必填 | —      | 节拍类型（见下方枚举）                                          |
| `content`         | **string**                     | ✅ 必填 | —      | 正文内容（动作描写或台词）                                        |
| `speaker`         | **string / null**              | 可选   | `null` | 说话人（仅 dialogue/monologue 有值；可为 `char_XXX` 占位符或实际角色名） |
| `emotion`         | **string / null**              | 可选   | `null` | 情绪标签（自由文本）                                           |
| `participants`    | **array\[Participant] / null** | 可选   | `[]`   | 群戏参与者（多人场景专用）                                        |
| `stage_direction` | **string / null**              | 可选   | `null` | 舞台指示（导演级动作描写，与 content 分离）                           |
| `alternatives`    | **array\[Alternative]**        | 可选   | `[]`   | 备选方案列表                                               |
| `selected`        | **number**                     | 可选   | `0`    | 当前选中索引（0=原始版本，1+=备选方案）                               |
| `plot_line_tags`  | **string\[] / null**           | 可选   | `[]`   | 关联情节线标签                                              |

### `type` 枚举

| 值                 | 标签   | 含义          | 典型内容示例                  |
| ----------------- | ---- | ----------- | ----------------------- |
| `"action"`        | 动作   | 第三人称动作/环境描写 | "林曦抬头，看见苏晴端着两杯咖啡走过来..." |
| `"dialogue"`      | 对白   | 角色对话        | "别说了，我感觉我要挂了。"          |
| `"monologue"`     | 独白   | 内心独白/旁白     | （内心声音的完整段落）             |
| `"parenthetical"` | 括号说明 | 台词的情绪/动作指示  | "(犹豫地)" / "(突然停住)"      |

### Participant（群戏参与者，嵌套于 Beat）

| 字段             | 类型                | 必填 | 说明                          |
| -------------- | ----------------- | -- | --------------------------- |
| `character_id` | **string**        | ✅  | 角色 ID（引用 `characters[].id`） |
| `role`         | **enum**          | ✅  | 在本场中的角色分工（见下方枚举）            |
| `dialogue`     | **string / null** | 可选 | 该角色的台词（仅 `speaker` 有值）      |

### `role` 枚举（Participant）

| 值            | 含义       | dialogue 是否有值 |
| ------------ | -------- | ------------- |
| `"speaker"`  | 当前说话人    | ✅ 有台词         |
| `"listener"` | 倾听者/反应者  | ❌ null        |
| `"observer"` | 旁观者/背景人物 | ❌ null        |

### Alternative（备选方案，嵌套于 Beat）

| 字段             | 类型                             | 必填 | 说明                                     |
| -------------- | ------------------------------ | -- | -------------------------------------- |
| `content`      | **string**                     | ✅  | 替代内容文本                                 |
| `name`         | **string / null**              | 可选 | 方案名称（如"方案1（幽默）"）                       |
| `tone`         | **string / null**              | 可选 | 语气/风格标签                                |
| `emotion`      | **string / null**              | 可选 | 方案关联情绪值                                |
| `participants` | **array\[Participant] / null** | 可选 | 群戏备选方案的参与者（与 Beat 级 participants 结构一致） |

**单人对白 Beat 示例：**

```yaml
- type: dialogue
  content: "别说了，我感觉我要挂了。"
  speaker: char_001           # ← 占位符，前端渲染时替换为"林曦"
  emotion: 焦虑中带着期待
  alternatives:
    - content: "行了行了，我快不行了。"
      name: "方案1（口语化）"
      tone: 口语化
      emotion: 放松
  selected: 0
  plot_line_tags: [plot_main]
```

**群戏 Beat 示例（多人同时在场）：**

```yaml
- type: dialogue
  content: ""                 # 群戏时 content 为空，内容分散在 participants 中
  speaker: null
  participants:
    - character_id: char_001
      role: speaker
      dialogue: "别说了，我感觉我要挂了。"
    - character_id: char_002
      role: listener
      dialogue: null
    - character_id: char_003
      role: observer
      dialogue: null
  alternatives:
    - content: ""
      name: "方案2（紧张版）"
      tone: 紧张对抗
      participants:
        - character_id: char_001
          role: speaker
          dialogue: "你到底想怎样？说清楚！"
        - character_id: char_002
          role: listener
          dialogue: null
        - character_id: char_003
          role: observer
          dialogue: null
  plot_line_tags: [plot_main]
```

***

## 八、分析产物（可选扩展）

以下两个顶层字段为 AI 分析管线生成的衍生数据，不属于剧本创作本身，但作为附加信息嵌入同一 YAML 文件中以便统一管理。

### `causal_graph` 因果图谱

| 字段       | 类型                      | 必填 | 说明     |
| -------- | ----------------------- | -- | ------ |
| `events` | **array\[CausalEvent]** | ✅  | 因果事件节点 |
| `edges`  | **array\[CausalEdge]**  | ✅  | 因果关系边  |

#### CausalEvent

| 字段            | 类型            | 必填 | 说明         |
| ------------- | ------------- | -- | ---------- |
| `id`          | **string**    | ✅  | 事件唯一标识     |
| `description` | **string**    | ✅  | 事件描述文本     |
| `scene_ids`   | **number\[]** | 可选 | 关联场景 ID 列表 |
| `chapter`     | **number**    | ✅  | 所属章节号      |

#### CausalEdge

| 字段            | 类型                | 必填 | 说明                                            |
| ------------- | ----------------- | -- | --------------------------------------------- |
| `from`        | **string**        | ✅  | 起始事件 ID                                       |
| `to`          | **string**        | ✅  | 目标事件 ID                                       |
| `type`        | **enum**          | ✅  | 边类型：`"temporal"` / `"emotional"` / `"causal"` |
| `strength`    | **number**        | ✅  | 强度值（0.0\~1.0）                                 |
| `description` | **string / null** | 可选 | 因果关系描述                                        |

### `relation_network` 关系网络

| 字段       | 类型                        | 必填 | 说明     |
| -------- | ------------------------- | -- | ------ |
| `matrix` | **array\[RelationEntry]** | ✅  | 关系矩阵条目 |

#### RelationEntry

| 字段              | 类型                      | 必填 | 说明                 |
| --------------- | ----------------------- | -- | ------------------ |
| `from`          | **string**              | ✅  | 源角色名               |
| `to`            | **string**              | ✅  | 目标角色名              |
| `intimacy`      | **number**              | ✅  | 亲密度（-1.0\~1.0）     |
| `power_gap`     | **number**              | ✅  | 权力差距（负=from 弱于 to） |
| `trust`         | **number**              | ✅  | 信任度（-1.0\~1.0）     |
| `relation_type` | **string / null**       | 可选 | 关系类型标签             |
| `description`   | **string / null**       | 可选 | 关系描述               |
| `history`       | **array\[HistoryItem]** | 可选 | 变化历史               |

#### HistoryItem

| 字段               | 类型         | 必填 | 说明         |
| ---------------- | ---------- | -- | ---------- |
| `scene_id`       | **number** | ✅  | 发生变化的场景 ID |
| `delta_intimacy` | **number** | ✅  | 亲密度变化量     |
| `delta_power`    | **number** | ✅  | 权力差距变化量    |
| `delta_trust`    | **number** | ✅  | 信任度变化量     |

***

## 九、完整 YAML 示例

以下是一个最小可用的完整剧本 YAML 文件：

```yaml
metadata:
  title: "改编剧本"
  source_novel: "用户上传的小说"
  adaptation_date: "2026-06-06"
  style: film
  emotional_curve:
    chapters: [1, 2, 3]
    intensities: [0.3, 0.55, 0.8]

characters:
  - id: char_001
    name: 林曦
    traits: [勤奋, 敏感, 善良]
    voice: 温柔而略带犹豫
    arc_summary: 从单纯到成熟
    motivation: 追求事业成功
    relationships:
      - target_character_id: char_002
        relation_type: romance
        description: 暗恋对象
    first_scene_id: 1
  - id: char_002
    name: 顾言
    traits: [成熟, 稳重, 内敛]
    voice: 简洁而富有磁性
    relationships: []
    first_scene_id: 1

chapters:
  - id: ch_001
    title: 阳光下的相遇
    order: 1
    plot_line: plot_main
    summary: 初遇场景
    scene_ids: [1, 2]

scenes:
  - id: 1
    location: "城市街道 · 清晨"
    time: "清晨 6:30"
    emotion_intensity: 0.3
    chapter_id: ch_001
    characters_present: [char_001, char_002]
    plot_lines: [plot_main]
    media_hints:
      camera: "全景 → 缓慢推进至中景"
      music: "钢琴独奏，缓慢渐入"
    beats:
      - type: action
        content: "林曦抬头，看见苏晴端着两杯咖啡走过来。"
        speaker: null
        emotion: 焦虑中带着期待
        participants: []
        plot_line_tags: [plot_main]
      - type: dialogue
        content: "别说了，我感觉我要挂了。"
        speaker: char_001
        emotion: 焦虑中带着期待
        participants: []
        alternatives:
          - content: "行了行了，我快不行了。"
            name: "方案1（口语化）"
            tone: 口语化
        selected: 0
        plot_line_tags: [plot_main]

causal_graph:
  events:
    - id: e1
      description: 大雨困住林曦
      scene_ids: [1]
      chapter: 1
  edges:
    - from: e1
      to: e2
      type: causal
      strength: 0.8
      description: 大雨导致两人被困，触发后续对话

relation_network:
  matrix:
    - from: 林曦
      to: 顾言
      intimacy: 0.6
      power_gap: -0.3
      trust: 0.5
      relation_type: 暧昧
      description: 互相有好感但未挑明
      history:
        - scene_id: 1
          delta_intimacy: 0.2
          delta_power: -0.1
          delta_trust: 0.15
```

***

## 十、设计决策说明

本文档记录了 Schema 设计过程中的关键决策及其原因，便于后续维护和理解。

### 决策 1：为什么选择 YAML 而非 JSON？

| 维度      | YAML                     | JSON      | <br />       |
| ------- | ------------------------ | --------- | :----------- |
| 人类可读性   | 高（无多余引号/逗号）              | 低（大量语法噪音） | <br />       |
| 注释支持    | ✅ 原生支持 `#` 注释            | ❌ 不支持     | <br />       |
| 多行字符串   | ✅ 原生支持（\`                | `/`>\`）   | ❌ 需要 `\n` 转义 |
| 工具链成熟度  | 成熟（js-yaml, serde\_yaml） | 最广泛       | <br />       |
| 嵌套深度可读性 | 通过缩进直观呈现层级               | 深层嵌套难以阅读  | <br />       |

**结论：** 剧本是高度人工编辑的内容，可读性和注释能力优先级高于机器解析便利性。YAML 的多行字符串天然适合存储大段台词和舞台指示。

### 决策 2：为什么 `speaker` 使用 `char_XXX` 占位符而非直接写角色名？

**问题：** LLM 生成阶段无法预知最终的角色 ID 映射关系。Stage 1 先提取角色并分配 `char_001`, `char_002` 等 ID，Stage 2 才生成具体场景内容。

**方案对比：**

| 方案                 | 优点                 | 缺点                  |
| ------------------ | ------------------ | ------------------- |
| A. 直接用角色名          | 直观易读               | 名字可能重复/变更，无法建立稳定引用  |
| B. 用 `char_XXX` ID | 稳定引用，前后一致          | 不够直观，需前端渲染时转换       |
| **C. 采用方案 B（当前）**  | **ID 稳定 + 前端自动解析** | **增加一层 resolve 逻辑** |

**实现：** 前端通过 `resolvePlaceholders(text)` 函数将 `char_00X` 正则匹配后替换为 `scriptStore.getCharacterName(id)` 返回的实际名称。Player.vue、BeatCard.vue、SceneCard.vue 三处统一使用此机制。

### 决策 3：为什么 Beat 同时有 `content` 和 `participants.dialogue`？

这是为了区分 **单人节拍** 与 **群戏节拍** 两种截然不同的叙事模式：

**单人节拍（普通模式）：**

```yaml
- type: dialogue
  content: "别说了，我感觉我要挂了。"   # ← 所有内容集中在这里
  speaker: char_001
  participants: []
```

一个 beat = 一个人的一个动作/一句台词。简单、直观、兼容传统剧本格式。

**群戏节拍（多人互动）：**

```yaml
- type: dialogue
  content: ""                              # ← content 为空
  participants:
    - character_id: char_001               # ← 内容按角色分散
      role: speaker
      dialogue: "别说了，我感觉我要挂了。"
    - character_id: char_002
      role: listener                       # listener 无台词但标记在场
      dialogue: null
```

一个 beat = 一个多人互动瞬间。每个参与者的反应/台词独立存储，便于：

- AI 分别为不同角色生成差异化回应
- 备选方案切换时保持完整的多人状态
- TTS 按角色顺序依次朗读

**为什么不用两个独立的 Beat 类型？** 因为群戏和单人 beat 共享同一个位置序列（同一个场景的 beats 数组），拆分会导致时间线断裂。用 `participants` 是否非空来区分是更优雅的多态方案。

### 决策 4：为什么 `alternatives` 嵌套在 Beat 内部而非外置？

**内嵌方案（当前）：**

```yaml
beats:
  - content: "原文"
    alternatives:
      - content: "备选A"
      - content: "备选B"
```

**外置方案（备选）：**

```yaml
beats:
  - id: beat_001
    content: "原文"
alternatives:
  beat_001:
    - content: "备选A"
    - content: "备选B"
```

**选择内嵌的原因：**

1. **局部性原则** — 备选方案与其所属 beat 高度耦合，放在一起编辑时无需跨区域查找
2. **删除安全** — 删除一个 beat 时其备选方案自然一并删除，不会产生孤儿数据
3. **YAML 可读性** — 内嵌结构在视觉上就是「主方案 + N 个备选」的自然分组
4. **API 简化** — 后端 regenerate/alternatives API 的请求/响应都是 beat 级别的，无需额外关联查询

### 决策 5：为什么 `selected` 是 number 而非 boolean？

`selected: 0` 表示使用原始版本（`content` 字段的值），`selected: 1` 表示使用 `alternatives[0]`，以此类推。

**为什么不直接用 boolean 标记是否启用某个备选？**
因为需要支持 **N 个备选方案之间的切换**（不只是"原始 vs 备选A"）。用户可能生成 3\~5 个备选方案并逐一对比，`selected` 作为数组索引天然支持这种多选一切换模式。

### 决策 6：为什么 Scene.id 是 number 而 Chapter.id 是 string？

| 实体        | ID 格式                 | 原因                                                    |
| --------- | --------------------- | ----------------------------------------------------- |
| Scene     | `number` (1, 2, 3...) | 场景是线性时间轴上的点，自然编号即可；也便于 `scrollIntoView` 等基于位置的 DOM 操作 |
| Chapter   | `string` (`ch_001`)   | 章节是逻辑分组容器，前缀 `ch_` 使其在日志和调试中一眼可辨；预留未来可能的非连续编号需求       |
| Character | `string` (`char_001`) | 角色需要在全文中被字符串引用（如 `speaker` 字段），字符串 ID 更通用且自描述         |

### 决策 7：为什么 `causal_graph` 和 `relation_network` 放在根级别而非单独文件？

这两个分析产物虽然不是剧本创作的核心数据，但具有强关联性：

- 因果事件的 `scene_ids` 直接引用场景 ID
- 关系网络的 `history` 记录的是随场景推进的关系变化
- 导出时用户期望获得一份**完整的、自包含的**项目文件

放在同一 YAML 中保证了：

1. **原子性** — 单文件备份/恢复，不会出现剧本与分析数据版本不一致
2. **引用完整性** — 无需跨文件验证 ID 引用
3. **分发便捷** — 一份 YAML 即可分享给协作者或导入其他工具

代价是文件体积增大，但对于小说改编场景（通常 < 100KB），完全可接受。

### 决策 8：为什么 emotional\_curve 放在 metadata 内而非 scenes 级别？

情感曲线是**章级别的聚合视图**（每章一个强度值），不是每个场景/节拍的属性。将其放在 metadata 下表示「这是对整个剧本的情感概览」，与 `title`、`style` 等元信息同级。

如果未来需要更细粒度的情感标注（如每个 beat 都有 emotion 值），那属于 Beat.emotion 字段的职责，与本字段不冲突——两者是不同粒度的情感表达。

### 决策 9：为什么 Beat.type 包含 `parenthetical`（括号说明）？

传统好莱坞剧本格式中，括号说明（parenthetical）是写在角色名和台词之间的一行小字，用于指示说话方式：

```
JOHN
(angrily)
I told you not to come here.
```

在本 Schema 中，它被建模为独立的 beat type 而非 dialogue 的子属性，原因是：

1. **统一处理** — 所有 beat 都是同级的一等公民，编辑器可以用统一的卡片组件渲染
2. **AI 生成友好** — LLM 输出时不需要判断"这是括号说明还是台词"，每种 type 都是独立条目
3. **灵活性** — parenthetical 可以独立存在（不带 dialogue），例如纯动作指示中的括号注释

### 决策 10：`media_hints` 为什么是 Scene 级别而非 Beat 级别？

镜头语言和音乐通常以**场景为单位**规划（一场戏一个镜头方案），而不是每个节拍都切换镜头。这符合影视制作惯例——分镜表是按场（scene）拆分的。

如果未来需要 beat 级别的精细媒体提示（如某句台词时的特写镜头），可以通过扩展 Beat 新增 `media_hint` 字段实现，不影响现有结构。
