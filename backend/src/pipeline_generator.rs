use crate::models::*;
use crate::llm_client::{call_deepseek, select_model, call_parallel};
use crate::utils::safe_truncate;
use crate::{causal_graph, relation_network};
use tracing::{info, warn};

/// 通过三阶段流水线生成完整剧本
///
/// 返回元组：(最终剧本, 进度事件列表)
pub async fn generate_via_pipeline(
    text: &str,
    config: &GenConfig,
    api_key: &str,
    base_url: &str,
) -> Result<(ScriptYaml, Vec<PipelineProgressEvent>), String> {
    let mut progress_events = Vec::new();
    let is_mock = api_key.is_empty() || api_key == "mock";

    if is_mock {
        info!("Pipeline 使用模拟模式");
        let script = generate_mock_pipeline(text, config);
        progress_events.push(PipelineProgressEvent {
            stage: "analysis".to_string(),
            message: "模拟模式：跳过分析".to_string(),
            progress: 1.0,
            current_chapter: None,
            total_chapters: None,
            completed_chunks: None,
            total_chunks: None,
        });
        return Ok((script, progress_events));
    }

    // ========== Stage 1: 全局分析 ==========
    progress_events.push(PipelineProgressEvent {
        stage: "analysis".to_string(),
        message: "正在分析小说结构...".to_string(),
        progress: 0.0,
        current_chapter: None,
        total_chapters: None,
        completed_chunks: None,
        total_chunks: None,
    });

    let outline = match analyze_novel(text, api_key, base_url).await {
        Ok(o) => {
            info!(chapters = o.chapters.len(), characters = o.characters.len(), "Stage 1 分析完成");
            progress_events.push(PipelineProgressEvent {
                stage: "analysis".to_string(),
                message: format!("分析完成：识别 {} 个章节，{} 个角色", o.chapters.len(), o.characters.len()),
                progress: 1.0,
                current_chapter: None,
                total_chapters: Some(o.chapters.len() as i32),
                completed_chunks: None,
                total_chunks: None,
            });
            o
        }
        Err(e) => {
            warn!(error = %e, "Stage 1 失败，使用 mock 大纲");
            progress_events.push(PipelineProgressEvent {
                stage: "analysis".to_string(),
                message: format!("分析失败，使用备用方案: {}", e),
                progress: 1.0,
                current_chapter: None,
                total_chapters: None,
                completed_chunks: None,
                total_chunks: None,
            });
            generate_mock_outline(text)
        }
    };

    // 更新进度（使用实际 chapter 数量）
    if let Some(last) = progress_events.last_mut() {
        last.total_chapters = Some(outline.chapters.len() as i32);
    }

    // ========== Stage 2: 分块生成场景 ==========
    let total_chapters = outline.chapters.len() as i32;
    progress_events.push(PipelineProgressEvent {
        stage: "generation".to_string(),
        message: format!("开始分块生成（共 {} 章）...", total_chapters),
        progress: 0.0,
        current_chapter: None,
        total_chapters: Some(total_chapters),
        completed_chunks: Some(0),
        total_chunks: Some(total_chapters),
    });

    let chunks = match generate_chunks(text, &outline, config, api_key, base_url, &mut progress_events).await {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "Stage 2 失败，使用 mock 场景数据");
            generate_mock_chunks(&outline, config)
        }
    };

    progress_events.push(PipelineProgressEvent {
        stage: "generation".to_string(),
        message: "所有章节场景生成完成".to_string(),
        progress: 1.0,
        current_chapter: None,
        total_chapters: Some(total_chapters),
        completed_chunks: Some(total_chapters),
        total_chunks: Some(total_chapters),
    });

    // ========== Stage 3: 整合剧本 ==========
    progress_events.push(PipelineProgressEvent {
        stage: "integration".to_string(),
        message: "正在整合最终剧本...".to_string(),
        progress: 0.0,
        current_chapter: None,
        total_chapters: None,
        completed_chunks: None,
        total_chunks: None,
    });

    let mut script = integrate_script(&chunks, &outline, config);

    // 基于完整剧本生成因果图谱和关系网络
    if let Ok(yaml) = serde_yaml::to_string(&script) {
        // 生成因果图谱
        match causal_graph::build_causal_graph(&yaml, api_key, base_url).await {
            Ok(cg_data) => {
                script.causal_graph = Some(causal_data_to_response(&cg_data));
                info!("因果图谱生成成功: {} 个事件", cg_data.graph.node_count());
            }
            Err(e) => {
                warn!(error = %e, "因果图谱生成失败，使用空值");
            }
        }

        // 生成关系网络
        match relation_network::build_relation_network(&yaml, api_key, base_url).await {
            Ok(rn_data) => {
                script.relation_network = Some(relation_network::data_to_response(&rn_data));
                info!("关系网络生成成功");
            }
            Err(e) => {
                warn!(error = %e, "关系网络生成失败，使用空值");
            }
        }
    }

    progress_events.push(PipelineProgressEvent {
        stage: "integration".to_string(),
        message: "剧本整合完成".to_string(),
        progress: 1.0,
        current_chapter: None,
        total_chapters: None,
        completed_chunks: None,
        total_chunks: None,
    });

    Ok((script, progress_events))
}

// ==================== Stage 1: 全局分析 ====================

/// 调用 LLM 分析小说全文结构
async fn analyze_novel(
    text: &str,
    api_key: &str,
    base_url: &str,
) -> Result<AnalysisOutline, String> {
    // 截断策略：首尾采样
    let truncated_text = truncate_for_analysis(text);

    let prompt = format!(
        r#"你是一名专业的文学分析师。请对以下小说进行全局分析，输出 JSON 格式的分析大纲。

## 输出要求

严格输出以下 JSON 结构，不要包含任何其他文字：

{{
  "characters": [
    {{
      "id": "char_001",
      "name": "角色名",
      "traits": ["特征1", "特征2"],
      "voice": "语言风格描述",
      "arc_summary": "角色弧光/成长轨迹",
      "motivation": "核心动机或目标"
    }}
  ],
  "plot_lines": [
    {{
      "id": "plot_main",
      "name": "主线名称",
      "description": "情节线描述",
      "line_type": "main"
    }}
  ],
  "chapters": [
    {{
      "id": "ch_001",
      "title": "章节标题",
      "order": 1,
      "start_offset": 0,
      "end_offset": 5000,
      "summary": "本章摘要",
      "plot_lines": ["plot_main"],
      "key_characters": ["char_001"]
    }}
  ],
  "full_summary": "全文摘要（200字以内）"
}}

## 分析规则

1. 提取主要角色（3~15人），仅提取真正参与剧情推进的角色（有台词、有行动、有关系变化的）；背景路人/一次性提及人物不要提取。每个角色需有 id/name/traits/voice/arc_summary/motivation
2. 识别 N 条情节线（至少1条主线 + 支线），每条有 id/name/description/type(main/supporting)
3. 按叙事完整性划分章节单元（而非机械切分），一个叙事单元 = 一个完整的情节转折或情感弧段；建议每章对应 2~5 个场景；禁止将单一连续对话拆分为多个场景。每个单元有 title/order/start_offset/end_offset/summary/key_characters
4. 生成全文摘要（200字以内）
5. start_offset 和 end_offset 是原文中的字符位置（从0开始）

## 忠实度要求（最高优先级）
本章分析结果必须严格基于提供的小说原文。
- 不得编造原文中不存在的人物、事件或设定
- 章节摘要必须准确反映该章实际发生的核心事件
- 角色特征和弧光必须有原文依据

## 小说文本

{}"#,
        truncated_text
    );

    let model = select_model("analysis");
    let response = call_deepseek(&prompt, api_key, base_url, Some("你是专业文学分析师。严格输出JSON格式。"), true, model).await?;

    // 尝试提取 JSON
    let json_str = extract_json_from_response(&response);
    serde_json::from_str::<AnalysisOutline>(&json_str)
        .map_err(|e| format!("解析分析大纲失败: {}", e))
}

/// 截断策略：首尾采样
fn truncate_for_analysis(text: &str) -> String {
    const MAX_CHARS: usize = 50000;
    const HEAD_SIZE: usize = 20000;
    const TAIL_SIZE: usize = 15000;
    const SAMPLE_SIZE: usize = 3000;
    const SAMPLE_INTERVAL: usize = 10000;

    if text.chars().count() <= MAX_CHARS {
        return text.to_string();
    }

    let chars: Vec<char> = text.chars().collect();
    let total_len = chars.len();

    let mut result = String::new();

    // 取前 HEAD_SIZE 字符
    result.extend(chars.iter().take(HEAD_SIZE).cloned());
    result.push_str("\n\n... [中间省略] ...\n\n");

    // 中间采样：每 SAMPLE_INTERVAL 取 SAMPLE_SIZE 字符
    let mut pos = HEAD_SIZE;
    while pos + SAMPLE_SIZE < total_len - TAIL_SIZE {
        let end = (pos + SAMPLE_SIZE).min(total_len - TAIL_SIZE);
        result.extend(chars[pos..end].iter().cloned());
        result.push_str("\n\n");
        pos += SAMPLE_INTERVAL;
    }

    // 取最后 TAIL_SIZE 字符
    if total_len > TAIL_SIZE {
        result.push_str("\n\n... [继续到结尾] ...\n\n");
        result.extend(chars[total_len - TAIL_SIZE..].iter().cloned());
    }

    result
}

/// 从 LLM 响应中提取 JSON（通用版，保留原始结构）
/// 用于 Stage 1 等期望 JSON 对象的场景
fn extract_json_from_response(response: &str) -> String {
    let trimmed = response.trim();

    // 处理 markdown 代码块包裹
    if trimmed.starts_with("```") {
        if let Some(start) = trimmed.find('\n') {
            if let Some(end) = trimmed[start + 1..].find("```") {
                return extract_json_object(trimmed[start + 1..start + 1 + end].trim());
            }
        }
    }

    extract_json_object(trimmed)
}

/// 提取 JSON 对象（用于 Stage 1：AnalysisOutline 等对象类型响应）
fn extract_json_object(text: &str) -> String {
    let t = text.trim();

    // 直接是对象 → 返回完整对象
    if t.starts_with('{') {
        if let Some(e) = t.rfind('}') { return t[..=e].to_string(); }
        return t.to_string();
    }

    // 尝试在文本中找 { ... } 对象片段
    if let Some(s) = t.find('{') {
        if let Some(e) = t.rfind('}') {
            if e > s { return t[s..=e].to_string(); }
        }
    }

    t.to_string()
}

/// 从 LLM 响应中提取 JSON 数组（专用版，处理对象包裹数组的情况）
/// 用于 Stage 2 等期望 JSON 数组的场景：LLM 可能返回 {"scenes": [...]} 而非直接 [...]
fn extract_json_array_from_response(response: &str) -> String {
    let trimmed = response.trim();

    // 处理 markdown 代码块包裹
    if trimmed.starts_with("```") {
        if let Some(start) = trimmed.find('\n') {
            if let Some(end) = trimmed[start + 1..].find("```") {
                return extract_json_array_inner(trimmed[start + 1..start + 1 + end].trim());
            }
        }
    }

    extract_json_array_inner(trimmed)
}

/// 内部逻辑：从文本中提取 JSON 数组，智能处理对象包裹情况
fn extract_json_array_inner(text: &str) -> String {
    let t = text.trim();

    // A: 直接以 [ 开头 → 就是数组，直接返回
    if t.starts_with('[') {
        if let Some(e) = t.rfind(']') { return t[..=e].to_string(); }
        return t.to_string();
    }

    // B: 以 { 开头 → 可能是对象包裹了数组，尝试提取
    if t.starts_with('{') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(t) {
            if v.is_array() { return t.to_string(); }
            if let Some(o) = v.as_object() {
                // 按优先级查找常见包裹字段名
                for k in &["scenes", "data", "result", "output", "items"] {
                    if let Some(a) = o.get(*k).and_then(|x| x.as_array()) {
                        if let Ok(s) = serde_json::to_string(a) { return s; }
                    }
                }
                // 兜底：取第一个值是数组的字段
                for (_, val) in o {
                    if val.is_array() {
                        if let Ok(s) = serde_json::to_string(val) { return s; }
                    }
                }
            }
        }
        // 无法提取数组 → 返回原始对象文本（让上层报错）
        if let Some(e) = t.rfind('}') { return t[..=e].to_string(); }
        return t.to_string();
    }

    // C: 其他格式，找 [ ... ] 数组片段
    if let Some(s) = t.find('[') {
        if let Some(e) = t.rfind(']') {
            if e > s { return t[s..=e].to_string(); }
        }
    }

    t.to_string()
}

// ==================== Stage 2: 分块生成场景 ====================

/// 按章节分块并行生成场景
async fn generate_chunks(
    text: &str,
    outline: &AnalysisOutline,
    config: &GenConfig,
    api_key: &str,
    base_url: &str,
    progress_events: &mut Vec<PipelineProgressEvent>,
) -> Result<Vec<SceneChunk>, String> {
    let chapters = &outline.chapters;
    if chapters.is_empty() {
        return Err("大纲中没有章节信息".to_string());
    }

    let mut futures = Vec::new();
    for (idx, ch) in chapters.iter().enumerate() {
        let chapter_text = extract_chapter_text(text, ch);
        let prev_summary = if idx > 0 {
            chapters[idx - 1].summary.clone()
        } else {
            String::new()
        };
        let chapter_id = ch.id.clone();
        let outline_clone = outline.clone();
        let config_clone = config.clone();
        let api_key = api_key.to_string();
        let base_url = base_url.to_string();
        let ch_idx = idx;

        futures.push(async move {
            generate_single_chunk(
                &chapter_text,
                &chapter_id,
                &outline_clone,
                &config_clone,
                &prev_summary,
                &api_key,
                &base_url,
                ch_idx,
            ).await
        });
    }

    // 并行调用，最多同时 3 路
    let results = call_parallel(futures, 3).await;

    let mut chunks = Vec::new();
    for (idx, result) in results.into_iter().enumerate() {
        // 更新进度
        progress_events.push(PipelineProgressEvent {
            stage: "generation".to_string(),
            message: format!("第 {} 章生成完成", idx + 1),
            progress: (idx + 1) as f64 / chapters.len() as f64,
            current_chapter: Some(chapters.get(idx).map(|c| c.title.as_str()).unwrap_or("").to_string()),
            total_chapters: Some(chapters.len() as i32),
            completed_chunks: Some((idx + 1) as i32),
            total_chunks: Some(chapters.len() as i32),
        });

        match result {
            Ok(chunk) => chunks.push(chunk),
            Err(e) => {
                warn!(chapter_idx = idx, error = %e, "单章生成失败，将使用 mock 数据");
                // fallback 到 mock
                let ch = &chapters[idx];
                chunks.push(generate_single_chunk_mock(ch, config));
            }
        }
    }

    Ok(chunks)
}

/// 提取单个章节对应的原文片段
fn extract_chapter_text(text: &str, chapter: &ChapterOutline) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = chapter.start_offset.min(chars.len());
    let end = chapter.end_offset.min(chars.len());

    if start >= end {
        return String::new();
    }

    let chunk: String = chars[start..end].iter().collect();
    // 上限 12000 字符
    safe_truncate(&chunk, 12000).to_string()
}

/// 为单个章节调用 LLM 生成场景
async fn generate_single_chunk(
    chapter_text: &str,
    chapter_id: &str,
    outline: &AnalysisOutline,
    config: &GenConfig,
    prev_summary: &str,
    api_key: &str,
    base_url: &str,
    _chapter_idx: usize,
) -> Result<SceneChunk, String> {
    // 构建角色表上下文
    let characters_context: String = outline.characters.iter()
        .map(|c| format!(
            "- {} ({}): 特征={}, 语言={}, 弧光={}, 动机={}",
            c.name, c.id,
            c.traits.join(","),
            c.voice.as_deref().unwrap_or("未指定"),
            c.arc_summary.as_deref().unwrap_or("未指定"),
            c.motivation.as_deref().unwrap_or("未指定")
        ))
        .collect::<Vec<_>>()
        .join("\n");

    // 构建情节线上下文
    let plot_context: String = outline.plot_lines.iter()
        .map(|p| format!("- {} ({}): {} [{}]", p.name, p.id, p.description, p.line_type))
        .collect::<Vec<_>>()
        .join("\n");

    let style_desc = match config.style.as_str() {
        "film" => "电影剧本",
        "short_drama" => "短剧剧本",
        "stage" => "舞台剧剧本",
        _ => "短剧剧本",
    };

    let prev_context = if prev_summary.is_empty() {
        String::new()
    } else {
        format!("\n\n## 前一章摘要\n{}\n请确保与上一章内容连贯。", prev_summary)
    };

    let prompt = format!(
        r#"你是一名专业编剧。

## 最高优先级：忠实改编原则（违反即视为无效输出）

你正在将一部**已有的小说**改编为剧本。你的首要任务是**忠实还原原著**：
1. **情节不可篡改**：场景中发生的所有事件必须能在原文章节中找到对应依据，不得编造原文没有的情节转折
2. **人物设定不可偏离**：角色的性格、说话方式、动机必须严格遵循角色表中的定义和原文表现
3. **对话必须源自原著**：所有对白应基于原文中的实际对话进行改写和扩展，而非凭空创作。可以合理扩展但核心语义必须一致
4. **场景顺序遵循原文**：场景的时间线和空间切换应尊重原文的叙事顺序

请为以下章节生成{}场景。

## 角色表

{}

## 情节线定义

{}

## 本章信息

- 章节 ID: {}
- 标题: {}

{}

## 输出要求（严格遵循）

输出 JSON 数组，每个场景必须包含**具体、真实、可拍摄的内容**：

[
  {{
    "id": 1,
    "location": "大学图书馆 · 靠窗角落",
    "time": "午后 · 阳光斜射",
    "emotion_intensity": 0.3,
    "chapter_id": "{}",
    "characters_present": ["char_001"],
    "plot_lines": ["plot_main"],
    "beats": [
      {{
        "type": "action",
        "content": "阳光透过落地窗洒在木质书桌上，尘埃在光柱中缓缓浮动。林曦正埋头在一堆参考书中，时不时推一下滑落的眼镜，笔尖在笔记本上飞快地记录着什么。",
        "speaker": null,
        "emotion": null,
        "participants": [],
        "stage_direction": "镜头从窗外阳光缓慢推进至林曦的侧脸，特写她专注的眉眼",
        "plot_line_tags": ["plot_main"],
        "alternatives": [
          {{"content": "一道阴影落在书页上，林曦抬起头，撞进一双平静如水的眼睛里。", "tone": "文艺"}}
        ],
        "selected": 0
      }},
      {{
        "type": "dialogue",
        "content": "这里有人吗？",
        "speaker": "char_002",
        "emotion": "平淡中带着一丝期待",
        "participants": [
          {{"character_id": "char_002", "role": "speaker", "dialogue": "这里有人吗？"}},
          {{"character_id": "char_001", "role": "listener", "dialogue": null}}
        ],
        "stage_direction": null,
        "plot_line_tags": ["plot_main"],
        "alternatives": [
          {{"content": "请问，这个位置……有人吗？", "tone": "礼貌"}},
          {{"content": "呃，打扰了——这里有人坐吗？", "tone": "犹豫"}}
        ],
        "selected": 0
      }},
      {{
        "type": "action",
        "content": "林曦猛地抬头，手里的笔"啪"地掉在桌上。她愣了一秒，慌忙把散开的笔记往旁边收了收。",
        "speaker": null,
        "emotion": "惊讶后迅速恢复镇定",
        "participants": [],
        "stage_direction": "特写→中景：捕捉林曦从惊讶到掩饰的微表情变化",
        "plot_line_tags": ["plot_main"],
        "alternatives": [
          {{"content": "林曦没有立刻回答，而是用余光快速扫了一眼对面空荡荡的座位，才轻轻摇了摇头。", "tone": "内敛"}}
        ],
        "selected": 0
      }},
      {{
        "type": "dialogue",
        "content": "没、没人。你坐吧。",
        "speaker": "char_001",
        "emotion": "有些局促，声音比平时低了一些",
        "participants": [
          {{"character_id": "char_001", "role": "speaker", "dialogue": "没、没人。你坐吧。"}},
          {{"character_id": "char_002", "role": "listener", "dialogue": null}}
        ],
        "stage_direction": "林曦低头避开对方的目光，手指无意识地捏着笔帽",
        "plot_line_tags": ["plot_main"],
        "alternatives": [
          {{"content": "空着呢，随便坐。", "tone": "随意"}},
          {{"content": "……没人。", "tone": "简短冷淡"}}
        ],
        "selected": 0
      }},
      {{
        "type": "dialogue",
        "content": "谢谢。你这笔记记得挺认真的啊，期末复习？",
        "speaker": "char_002",
        "emotion": "轻松自然，带着善意的调侃",
        "participants": [
          {{"character_id": "char_002", "role": "speaker", "dialogue": "谢谢。你这笔记记得挺认真的啊，期末复习？"}},
          {{"character_id": "char_001", "role": "listener", "dialogue": null}}
        ],
        "stage_direction": "对方拉开椅子坐下，目光落在林曦摊开的笔记本上",
        "plot_line_tags": ["plot_main"],
        "alternatives": [
          {{"content": "谢了。你在看什么书？这么厚。", "tone": "好奇"}},
          {{"content": "多谢。——你也是文学系的？", "tone": "试探"}}
        ],
        "selected": 0
      }}
    ],
    "media_hints": {{ "camera": "全景→中景", "music": "轻钢琴曲" }}
  }}
]

## 质量规则（必须遵守，违反将导致输出无效）

1. **禁止使用任何占位符或泛化表述**：
   - ❌ "对话内容"、"场景地点 N"、"时间待定"、"动作描述"、"舞台指示"
   - ✅ "你什么时候回来的？怎么不说一声。"、"咖啡馆·靠窗座位"、"傍晚6:20"
2. **地点必须具体可感**：写出实际空间特征（如"老城区巷弄·青石板路旁"而非"地点"）
3. **时间必须有叙事意义**：标注对剧情有影响的时间信息（如"暴雨来临前夜"而非"时间"）
4. **对话必须是角色会说的话**：符合人物性格、推动情节发展、包含潜台词
5. **动作描写要有画面感**：包含感官细节（视觉/听觉/触觉），能直接指导拍摄
6. **情感状态要精确**：避免笼统的"平静"，使用"平静但暗藏波澜""压抑到极点后突然爆发"
7. **群戏参与者必须完整**：多人场景中每个人物都要有明确的 role 和对应的 dialogue/stage_direction
8. **备选方案（alternatives）规则**：
   - 对话/独白节拍**必须提供 2~3 个备选方案**，每个有不同 tone（语气/风格）
   - 动作节拍至少提供 1 个备选方案
   - tone 取值示例：礼貌/犹豫/激动/冷淡/温柔/愤怒/文艺/口语化/正式
   - ❌ "alternatives": []   ✅ "alternatives": [{{"content":"不同措辞版本", "tone":"语气标签"}}]
9. **场景数量控制（重要）**：
   - 每章生成 **2~5 个场景**（宁少勿多）
   - 一个场景 = 一个连续的时空单元（同一地点+同一时间段内发生的事件集合）
   - **禁止过度拆分**：如果两个事件发生在同一地点且时间连贯，应合并在同一场景中
   - **禁止的场景拆分方式**：❌ 把一段连续对话拆成3个场景  ❌ 把同一场戏按"动作-对话-动作"拆分
   - **正确的场景边界**：✅ 地点转换  ✅ 时间跳跃（如"三天后"）  ✅ 叙事视角切换
10. **节拍丰富度要求（重要）**：
    - 每个场景至少包含 **5~10 个节拍（beats）**，不应只有1-2个
    - 对话场景：需包含足够的来回对话（至少3轮以上），展现人物互动和情感推进
    - 动作场景：需有起承转合的动作序列（铺垫→发展→高潮→收尾），而非单句描述
    - 每段对话内容应在 **15~80 字**之间，太短无法传递信息，太长不适合影视节奏
    - 独白/内心戏同样需要有层次感，不能只是一句话
11. **原文映射要求（最重要）**：
    - 生成每个场景前，先在脑海中标注该场景对应原文中的哪些段落
    - 场景中的关键对话必须在原文中有迹可循（可改写、扩展、精炼，但不可无中生有）
    - 如果原文某章有大段精彩对话，必须将其转化为对应的 dialogue beats，不可省略为一句概括
    - 场景的情感强度（emotion_intensity）应与原文该段的情感基调一致

## 输出格式（最后确认）

**你的完整输出必须且只能是一个 JSON 数组，以 `[` 开头，以 `]` 结尾。**
- ✅ 正确：`[{{"id":1,"location":"...","beats":[...}}, {{"id":2,...}}]`
- ❌ 错误：`{{"scenes":[...]}}` 或 `{{"data":[...]}}` 或任何包含外层键名包裹的结构
- 不要添加任何解释文字、markdown 标记或代码块符号

## 本章原文

{}"#,
        style_desc,
        characters_context,
        plot_context,
        chapter_id,
        outline.chapters.iter().find(|c| c.id == *chapter_id).map(|c| c.title.as_str()).unwrap_or("未知"),
        prev_context,
        chapter_id,
        chapter_text
    );

    let model = select_model("generation");
    let response = call_deepseek(&prompt, api_key, base_url, Some("你是专业编剧。你的输出必须且只能是一个JSON数组（以[开头、]结尾），不要用任何对象包裹，不要添加任何解释文字。"), true, model).await?;

    let json_str = extract_json_array_from_response(&response);
    let scenes: Vec<Scene> = serde_json::from_str(&json_str)
        .map_err(|e| format!("解析场景数据失败: {}", e))?;

    Ok(SceneChunk {
        chapter_id: chapter_id.to_string(),
        scenes,
    })
}

// ==================== Stage 3: 整合剧本 ====================

/// 合并所有 chunk 并构建完整 ScriptYaml
fn integrate_script(
    chunks: &[SceneChunk],
    outline: &AnalysisOutline,
    config: &GenConfig,
) -> ScriptYaml {
    let mut all_scenes = Vec::new();
    let mut global_scene_id = 1i32;
    let mut chapters_output = Vec::new();

    for chunk in chunks {
        let mut chapter_scene_ids = Vec::new();

        for mut scene in chunk.scenes.clone() {
            // 分配全局唯一 ID
            let old_id = scene.id;
            scene.id = global_scene_id;
            global_scene_id += 1;
            chapter_scene_ids.push(scene.id);
            all_scenes.push(scene);
        }

        // 构建 Chapter 输出
        if let Some(ch_outline) = outline.chapters.iter().find(|c| c.id == chunk.chapter_id) {
            chapters_output.push(Chapter {
                id: chunk.chapter_id.clone(),
                title: ch_outline.title.clone(),
                plot_line: ch_outline.plot_lines.first().cloned(),
                summary: ch_outline.summary.clone(),
                scene_ids: chapter_scene_ids,
                order: ch_outline.order,
            });
        }
    }

    // 校验一致性
    validate_consistency(&all_scenes, &outline.characters);

    // 构建完整 ScriptYaml
    ScriptYaml {
        metadata: ScriptMetadata {
            title: "改编剧本".to_string(),
            source_novel: "用户上传的小说".to_string(),
            adaptation_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            style: config.style.clone(),
            emotional_curve: Some(EmotionalCurve {
                chapters: (1..=chapters_output.len() as i32).collect(),
                intensities: generate_emotional_curve(chapters_output.len()),
            }),
        },
        characters: outline.characters.clone(),
        causal_graph: None,
        relation_network: None,
        scenes: all_scenes,
        chapters: chapters_output,
    }
}

/// 校验一致性
fn validate_consistency(scenes: &[Scene], characters: &[Character]) {
    let char_ids: std::collections::HashSet<&str> = characters.iter().map(|c| c.id.as_str()).collect();

    for scene in scenes {
        // 检查 characters_present
        for cid in &scene.characters_present {
            if !char_ids.contains(cid.as_str()) {
                warn!(scene_id = scene.id, character_id = %cid, "场景引用了不存在的角色");
            }
        }

        // 检查 speaker
        for beat in &scene.beats {
            if let Some(ref speaker) = beat.speaker {
                // speaker 是名字而非 ID，这里做宽松检查
                let _ = (speaker, &char_ids);
            }
            // 检查 participants
            for p in &beat.participants {
                if !char_ids.contains(p.character_id.as_str()) {
                    warn!(scene_id = scene.id, participant_char = %p.character_id, "参与者引用了不存在的角色");
                }
            }
        }
    }
}

/// 生成占位情感曲线
fn generate_emotional_curve(chapter_count: usize) -> Vec<f64> {
    if chapter_count == 0 {
        return vec![];
    }
    (0..chapter_count)
        .map(|i| {
            let t = i as f64 / (chapter_count.max(1) - 1) as f64;
            // 简单的 S 形曲线模拟
            0.3 + 0.5 * ((t * std::f64::consts::PI - std::f64::consts::FRAC_PI_2).sin() + 1.0) / 2.0
        })
        .collect()
}

// ==================== Mock 模式 ====================

/// 生成基于文本长度的 mock 剧本
pub fn generate_mock_pipeline(text: &str, config: &GenConfig) -> ScriptYaml {
    let outline = generate_mock_outline(text);
    let chunks = generate_mock_chunks(&outline, config);
    integrate_script(&chunks, &outline, config)
}

/// 根据文本长度动态生成 mock 大纲
fn generate_mock_outline(text: &str) -> AnalysisOutline {
    let char_count = text.chars().count();

    let (num_chars, num_chapters, num_scenes_per_chapter) = if char_count < 5000 {
        (3, 1, vec![4])
    } else if char_count < 20000 {
        (6, 2, vec![5, 6])
    } else if char_count < 100000 {
        (15, 4, vec![4, 5, 6, 5])
    } else {
        (35, 8, vec![4, 5, 4, 6, 5, 4, 5, 4])
    };

    let mut characters = Vec::with_capacity(num_chars);
    let names = ["林曦", "顾言", "苏晴", "陈默", "夏雨", "陆远", "沈清", "方知", "叶晨", "周然",
                 "江宁", "许诺", "何欢", "宋词", "唐风", "云深", "白露", "莫染", "顾北", "南风",
                 "程诺", "安歌", "楚天", "洛离", "萧寒", "墨染", "青禾", "素锦", "流年", "浮生",
                 "初见", "微凉", "浅唱", "低吟", "独白", "回眸", "转身", "遗忘", "重逢", "告别",
                 "等待", "追寻", "迷失", "觉醒", "蜕变", "救赎", "成全", "守护", "承诺", "约定"];

    for i in 0..num_chars {
        characters.push(Character {
            id: format!("char_{:03}", i + 1),
            name: names[i % names.len()].to_string(),
            traits: vec!["坚韧".to_string(), "善良".to_string()],
            voice: Some("自然".to_string()),
            arc_summary: Some(format!("角色{}的成长弧光", i + 1)),
            motivation: Some(format!("追求目标{}", i + 1)),
            relationships: vec![],
            first_scene_id: Some(1),
        });
    }

    let plot_lines = vec![
        PlotLineDef {
            id: "plot_main".to_string(),
            name: "主线".to_string(),
            description: "核心故事线".to_string(),
            line_type: "main".to_string(),
        },
    ];

    let total_len = char_count;
    let chunk_size = total_len / num_chapters.max(1);
    let mut chapters = Vec::with_capacity(num_chapters);
    let mut offset = 0usize;

    for i in 0..num_chapters {
        let start = offset;
        let end = if i == num_chapters - 1 { total_len } else { offset + chunk_size };
        chapters.push(ChapterOutline {
            id: format!("ch_{:03}", i + 1),
            title: format!("第{}章", i + 1),
            order: (i + 1) as i32,
            start_offset: start,
            end_offset: end,
            summary: format!("第{}章的摘要", i + 1),
            plot_lines: vec!["plot_main".to_string()],
            key_characters: if !characters.is_empty() {
                vec![characters[0].id.clone()]
            } else {
                vec![]
            },
        });
        offset = end;
    }

    AnalysisOutline {
        characters,
        plot_lines,
        chapters,
        full_summary: format!("这是一部约{}字的小说的摘要", char_count),
    }
}

/// 生成 mock 场景块
fn generate_mock_chunks(outline: &AnalysisOutline, config: &GenConfig) -> Vec<SceneChunk> {
    outline.chapters.iter().map(|ch| {
        generate_single_chunk_mock(ch, config)
    }).collect()
}

/// 场景地点池 — 根据章节序号和场景索引生成有意义的地点
fn generate_scene_location(chapter: &ChapterOutline, scene_idx: usize) -> String {
    let locations = match chapter.order {
        1 => vec!["城市街道 · 清晨", "咖啡馆 · 靠窗座位", "办公室 · 会议室", "公寓 · 客厅", "公园 · 长椅", "地铁站台"],
        2 => vec!["医院走廊", "学校操场 · 傍晚", "便利店 · 夜间", "天台", "老城区巷弄", "江边步道"],
        3 => vec!["机场候机厅", "火车站月台", "酒店大堂", "宴会厅", "停车场地下层"],
        4 => vec!["海边悬崖", "山间小屋", "废弃工厂", "暴雨中的街角", "深夜的书房"],
        _ => vec!["室内空间 A", "室外场景 B", "过渡区域 C", "特殊场所 D"],
    };
    locations.get(scene_idx % locations.len()).copied().unwrap_or("未知地点").to_string()
}

/// 时间标注池 — 根据场景索引递进时间
fn generate_scene_time(chapter_order: i32, scene_idx: usize) -> &'static str {
    let times = match chapter_order {
        1 => ["清晨 6:30", "上午 9:00", "中午 12:15", "下午 3:40", "傍晚 6:20", "夜晚 9:00"],
        2 => ["凌晨 2:00", "早上 7:10", "午后 1:30", "黄昏 5:50", "深夜 11:20", "午夜时分"],
        3 => ["数日后的清晨", "某个阴雨午后", "风暴来临前夜", "黎明时分", "最后的时刻", "转折之后"],
        4 => ["多年前的回忆（闪回）", "现实与回忆交错处", "一切结束之后", "新的开始", "未知的明天", "终章前夕"],
        _ => ["时间点待具体化", "时间过渡中", "关键时刻", "尾声临近", "故事延续", "新的篇章"],
    };
    times.get(scene_idx % times.len()).copied().unwrap_or("时间待定")
}

/// 动作描述池 — 基于章节标题和角色生成有意义的动作
fn generate_action_content(chapter_title: &str, scene_idx: usize, chars: &[String]) -> String {
    let char_name = chars.first().map(|s| s.as_str()).unwrap_or("主角");
    let actions = [
        format!("阳光从{}的侧面照进来，{}站在窗边，目光落在远处的城市轮廓上。", char_name, char_name),
        format!("{}推开门，脚步声在空旷的房间里回荡。空气中弥漫着灰尘的味道。", char_name),
        format!("一场突如其来的大雨把{}困在了屋檐下。{}看了看手机——没有信号。", char_name, char_name),
        format!("桌上摊开的旧照片吸引了{}的注意。照片边缘已经泛黄，上面的人影模糊不清。", char_name),
        format!("{}转过身，发现{}不知何时站在了身后，表情复杂地看着自己。", char_name, chars.get(1).map(|s| s.as_str()).unwrap_or("对方")),
        format!("风把桌上的文件吹散了一地。{}蹲下身一张张捡起，手指在某一页停住了。", char_name),
        format!("电话铃声打破了沉默。{}盯着响个不停的手机屏幕，犹豫了三秒才接通。", char_name),
        format!("镜子里的倒影让{}愣了一下——这还是自己认识的那个样子吗？", char_name),
    ];
    actions.get((scene_idx + chapter_title.len()) % actions.len()).cloned().unwrap_or_default()
}

/// 对话内容池 — 基于角色名和场景上下文生成真实对话
fn generate_dialogue_content(speaker: &str, listener: Option<&str>, scene_idx: usize) -> (String, Vec<BeatParticipant>) {
    let dialogues = [
        (format!("{}：你什么时候回来的？怎么不说一声。", speaker), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("你什么时候回来的？怎么不说一声。")) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "listener".to_string(), dialogue: None },
        ]),
        (format!("{}：有些事……不知道该从哪里说起。", speaker), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("有些事……不知道该从哪里说起。")) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "listener".to_string(), dialogue: None },
        ]),
        (format!("{}：你不用解释，我都明白。", listener.unwrap_or("对方")), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some("……".to_string()) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "speaker".to_string(), dialogue: Some(format!("你不用解释，我都明白。")) },
        ]),
        (format!("{}：这件事我必须自己做决定。不管结果如何，我不会后悔。", speaker), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("这件事我必须自己做决定。不管结果如何，我不会后悔。")) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "observer".to_string(), dialogue: None },
        ]),
        (format!("{}：（沉默良久）……走吧。", speaker), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("（沉默良久）……走吧。")) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "listener".to_string(), dialogue: None },
        ]),
        (format!("{}：你以为我想这样吗？", speaker), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("你以为我想这样吗？")) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "listener".to_string(), dialogue: None },
        ]),
        (format!("{}：从今天开始，我们各走各的路吧。", speaker), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("从今天开始，我们各走各的路吧。")) },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "listener".to_string(), dialogue: None },
        ]),
        (format!("{}：等等——你听我说完！", listener.unwrap_or("对方")), vec![
            BeatParticipant { character_id: speaker.to_string(), role: "listener".to_string(), dialogue: None },
            BeatParticipant { character_id: listener.unwrap_or("对方").to_string(), role: "speaker".to_string(), dialogue: Some(format!("等等——你听我说完！")) },
        ]),
    ];
    dialogues.get((scene_idx + speaker.len()) % dialogues.len()).cloned().unwrap_or_else(|| (
        format!("{}：这里的情况比我想象的要复杂得多。", speaker),
        vec![BeatParticipant { character_id: speaker.to_string(), role: "speaker".to_string(), dialogue: Some(format!("这里的情况比我想象的要复杂得多。")) }]
    ))
}

/// 舞台指示池
fn generate_stage_direction(speaker: &str, scene_idx: usize) -> String {
    let directions = [
        format!("{}缓缓抬起头，眼神中闪过一丝不易察觉的情绪波动。", speaker),
        format!("{}的手不自觉地攥紧了衣角，指节微微发白。", speaker),
        format!("空气仿佛凝固了。{}深吸一口气，试图让自己平静下来。", speaker),
        format!("窗外传来一声闷雷，{}的身体明显抖了一下。", speaker),
        format!("{}转身背对过去，肩膀微微颤抖。", speaker),
        format!("一阵风吹过，吹乱了{}的头发，也吹散了两人之间最后一点默契。", speaker),
    ];
    directions.get(scene_idx % directions.len()).cloned().unwrap_or_default()
}

/// 生成单个章节的 mock 数据（高质量，无占位符）
fn generate_single_chunk_mock(chapter: &ChapterOutline, _config: &GenConfig) -> SceneChunk {
    let num_scenes = 3 + (chapter.order as usize % 4); // 3-6 场景

    let scenes: Vec<Scene> = (0..num_scenes).map(|scene_idx| {
        let location = generate_scene_location(chapter, scene_idx);
        let time_str = generate_scene_time(chapter.order, scene_idx);
        let action_text = generate_action_content(&chapter.title, scene_idx, &chapter.key_characters);
        let speaker = chapter.key_characters.first().cloned().unwrap_or_default();
        let listener = chapter.key_characters.get(1).cloned();
        let (dialogue_text, participants) = generate_dialogue_content(&speaker, listener.as_deref(), scene_idx);
        let stage_dir = generate_stage_direction(&speaker, scene_idx);

        Scene {
            id: (scene_idx + 1) as i32,
            location,
            time: Some(time_str.to_string()),
            emotion_intensity: 0.3 + (scene_idx as f64) * 0.15,
            chapter_id: Some(chapter.id.clone()),
            characters_present: chapter.key_characters.clone(),
            plot_lines: chapter.plot_lines.clone(),
            beats: vec![
                Beat {
                    beat_type: "action".to_string(),
                    content: action_text,
                    alternatives: vec![],
                    selected: 0,
                    speaker: None,
                    emotion: None,
                    participants: vec![],
                    stage_direction: Some(stage_dir),
                    plot_line_tags: chapter.plot_lines.clone(),
                },
                Beat {
                    beat_type: "dialogue".to_string(),
                    content: dialogue_text,
                    alternatives: vec![],
                    selected: 0,
                    speaker: Some(speaker.clone()),
                    emotion: match scene_idx % 5 {
                        0 => Some("平静但暗藏波澜".to_string()),
                        1 => Some("压抑".to_string()),
                        2 => Some("激动".to_string()),
                        3 => Some("悲伤".to_string()),
                        _ => Some("坚定".to_string()),
                    },
                    participants: participants.clone(),
                    stage_direction: None,
                    plot_line_tags: chapter.plot_lines.clone(),
                },
            ],
            media_hints: Some(MediaHints {
                camera: match scene_idx % 6 {
                    0 => "全景 → 缓慢推进至中景".to_string(),
                    1 => "特写（面部表情）".to_string(),
                    2 => "过肩镜头".to_string(),
                    3 => "低角度仰拍".to_string(),
                    4 => "手持跟拍".to_string(),
                    _ => "固定机位 → 人物入画".to_string(),
                },
                music: match scene_idx % 5 {
                    0 => "钢琴独奏，缓慢渐入".to_string(),
                    1 => "环境音为主，远处车流声".to_string(),
                    2 => "弦乐四重奏，紧张感递增".to_string(),
                    3 => "静默（只有呼吸声）".to_string(),
                    _ => "吉他扫弦，情绪转折点".to_string(),
                },
            }),
        }
    }).collect();

    SceneChunk {
        chapter_id: chapter.id.clone(),
        scenes,
    }
}

/// 将 CausalGraphData (petgraph DiGraph) 转换为 CausalGraphResponse (events + edges 数组)
fn causal_data_to_response(data: &CausalGraphData) -> CausalGraphResponse {
    let mut events = Vec::new();
    let mut edges = Vec::new();

    // 提取所有节点作为事件
    let node_indices: Vec<_> = data.graph.node_indices().collect();
    for idx in &node_indices {
        if let Some(event) = data.graph.node_weight(*idx) {
            events.push(event.clone());
        }
    }

    // 提取所有边
    for edge_idx in data.graph.edge_indices() {
        if let Some(edge) = data.graph.edge_weight(edge_idx) {
            if let Some((source, target)) = data.graph.edge_endpoints(edge_idx) {
                let source_id = data.graph.node_weight(source)
                    .map(|e| e.id.clone())
                    .unwrap_or_default();
                let target_id = data.graph.node_weight(target)
                    .map(|e| e.id.clone())
                    .unwrap_or_default();
                edges.push(CausalEdge {
                    from: source_id,
                    to: target_id,
                    edge_type: edge.edge_type.clone(),
                    strength: edge.strength,
                    description: edge.description.clone(),
                });
            }
        }
    }

    CausalGraphResponse { events, edges }
}
