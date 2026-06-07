use serde::{Deserialize, Serialize};
use crate::llm_client::{call_deepseek, Message};
use crate::utils::safe_truncate;

/// 副编剧上下文
#[derive(Debug, Deserialize, Default)]
pub struct CoPilotContext {
    #[serde(default)]
    pub scene_id: Option<i32>,
    #[serde(default)]
    pub beat_index: Option<usize>,
    #[serde(default)]
    pub selected_text: Option<String>,
}

/// 副编剧聊天请求
#[derive(Debug, Deserialize)]
pub struct CoPilotChatRequest {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub context: Option<CoPilotContext>,
    #[serde(default)]
    pub full_script_yaml: Option<String>,
}

/// 副编剧聊天响应
#[derive(Debug, Serialize)]
pub struct CoPilotChatResponse {
    pub reply: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
}

/// 建议项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Suggestion {
    #[serde(rename = "type")]
    pub suggestion_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_scene_id: Option<i32>,
}

/// 副编剧建议请求
#[derive(Debug, Deserialize)]
pub struct CoPilotSuggestRequest {
    pub script_yaml: String,
}

/// 副编剧建议响应
#[derive(Debug, Serialize)]
pub struct CoPilotSuggestResponse {
    pub suggestions: Vec<Suggestion>,
}

/// 系统提示词
const CO_PILOT_SYSTEM_PROMPT: &str =
    "你是专业编剧副手（AI 副编剧），熟悉剧本结构、角色弧光、因果逻辑。\
     你可以：\n\
     1. 分析角色动机和性格一致性\n\
     2. 优化剧本节奏和张力\n\
     3. 检查因果逻辑和叙事连贯性\n\
     4. 生成备选对白或动作描述\n\
     5. 提供影视化改编建议\n\n\
     回答要求：\n\
     - 简洁有力，必要时使用 Markdown 列表\n\
     - 提供具体可操作的建议\n\
     - 如果涉及修改，给出明确的修改方案";

/// 副编剧聊天
pub async fn chat(
    messages: &[Message],
    context: &CoPilotContext,
    script_yaml: Option<&str>,
    api_key: &str,
    base_url: &str,
) -> Result<CoPilotChatResponse, String> {
    // 模拟模式
    if api_key.is_empty() || api_key == "mock" {
        return Ok(mock_chat_response(context));
    }

    // 构建带上下文的用户消息
    let mut context_info = String::new();
    if let Some(sid) = context.scene_id {
        context_info.push_str(&format!("\n当前场景 ID: {}", sid));
    }
    if let Some(bi) = context.beat_index {
        context_info.push_str(&format!("\n当前节拍索引: {}", bi));
    }
    if let Some(text) = &context.selected_text {
        context_info.push_str(&format!("\n选中文本: {}", text));
    }

    // 如果有剧本内容，截取一部分作为上下文
    if let Some(yaml) = script_yaml {
        let preview_len = yaml.len().min(3000);
        let safe_len = yaml.floor_char_boundary(preview_len);
        context_info.push_str(&format!(
            "\n\n当前剧本预览（前{}字）：\n{}",
            safe_len,
            &yaml[..safe_len]
        ));
    }

    // 将最后一条消息与上下文合并
    let user_content = if let Some(last) = messages.last() {
        format!("{}\n{}", last.content, context_info)
    } else {
        context_info
    };

    let response = call_deepseek(
        &user_content,
        api_key,
        base_url,
        Some(CO_PILOT_SYSTEM_PROMPT),
        false,
        "deepseek-chat",
    )
    .await?;

    Ok(CoPilotChatResponse {
        reply: response,
        suggestions: None,
    })
}

/// 主动分析剧本并返回建议
pub async fn suggest(
    script_yaml: &str,
    api_key: &str,
    base_url: &str,
) -> Result<Vec<Suggestion>, String> {
    // 模拟模式
    if api_key.is_empty() || api_key == "mock" {
        return Ok(mock_suggestions());
    }

    let prompt = format!(
        "请分析以下剧本，找出可以改进的地方。\n\
        返回JSON数组格式：[{{\"type\": \"节奏/角色/逻辑/对白/结构\", \"message\": \"建议内容\", \"target_scene_id\": 场景ID或null}}]\n\n\
        剧本内容：\n{}",
        safe_truncate(script_yaml, 5000)
    );

    let response = call_deepseek(&prompt, api_key, base_url, None, true, "deepseek-chat").await?;

    match serde_json::from_str::<Vec<Suggestion>>(&response) {
        Ok(suggestions) => Ok(suggestions),
        Err(_) => Ok(mock_suggestions()),
    }
}

/// 模拟模式的聊天响应
fn mock_chat_response(_context: &CoPilotContext) -> CoPilotChatResponse {
    CoPilotChatResponse {
        reply: "这是模拟模式下的回复。在真实模式下，我会根据你的剧本提供专业建议。请配置 DeepSeek API Key 开始创作。".to_string(),
        suggestions: Some(vec![
            "尝试在第二幕增加一个转折点".to_string(),
            "角色的对白可以更有个性".to_string(),
            "考虑增加一个象征性的道具".to_string(),
        ]),
    }
}

/// 模拟模式的建议列表
fn mock_suggestions() -> Vec<Suggestion> {
    vec![
        Suggestion {
            suggestion_type: "节奏".to_string(),
            message: "第一幕铺垫稍长，可以增加一个小冲突来吸引观众注意力。".to_string(),
            target_scene_id: Some(1),
        },
        Suggestion {
            suggestion_type: "角色".to_string(),
            message: "林曦在第二场的情绪转变可以更细腻一些，增加过渡节拍。".to_string(),
            target_scene_id: Some(2),
        },
        Suggestion {
            suggestion_type: "对白".to_string(),
            message: "顾言的对白偏少，可以考虑增加一些暗示性台词来丰富人物层次。".to_string(),
            target_scene_id: Some(1),
        },
        Suggestion {
            suggestion_type: "结构".to_string(),
            message: "目前只有两个场景，建议至少扩展到3-4个场景以完整呈现故事弧线。".to_string(),
            target_scene_id: None,
        },
    ]
}

// ==================== AI 重生成 ====================

/// 重生成请求
#[derive(Debug, Deserialize)]
pub struct RegenerateRequest {
    /// 目标类型：'beat'（单个节拍）或 'scene'（整场场景）
    pub target: String,
    /// 场景 ID
    pub scene_id: i32,
    /// 节拍索引（target=beat 时必填）
    #[serde(default)]
    pub beat_index: Option<usize>,
    /// 用户修改意图/指令
    pub instruction: String,
    /// 当前节拍原始内容（target=beat 时）
    #[serde(default)]
    pub current_content: Option<String>,
    /// 完整剧本 YAML（用于上下文感知）
    #[serde(default)]
    pub full_script_yaml: Option<String>,
    /// 群戏参与者列表（群戏节拍时必填，含 character_id / role / dialogue）
    #[serde(default)]
    pub participants: Option<serde_json::Value>,
}

/// 重生成响应
#[derive(Debug, Serialize)]
pub struct RegenerateResponse {
    /// 生成的新内容
    pub content: String,
    /// 额外的备选方案
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<BeatAlternative>>,
    /// 如果是整场重生成，返回所有节拍
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beats: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BeatAlternative {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion: Option<String>,
}

/// 备选方案生成请求
#[derive(Debug, Deserialize)]
pub struct AlternativesRequest {
    /// 目标类型：'beat'
    pub target: String,
    /// 场景 ID
    pub scene_id: i32,
    /// 节拍索引
    #[serde(default)]
    pub beat_index: Option<usize>,
    /// 用户指定的生成方向/意图（可空）
    #[serde(default)]
    pub instruction: Option<String>,
    /// 当前节拍内容
    #[serde(default)]
    pub current_content: Option<String>,
    /// 当前节拍类型
    #[serde(default)]
    pub current_type: Option<String>,
    /// 当前说话角色
    #[serde(default)]
    pub speaker: Option<String>,
    /// 完整剧本 YAML
    #[serde(default)]
    pub full_script_yaml: Option<String>,
    /// 期望生成的备选数量
    #[serde(default = "default_num_alts")]
    pub num_alternatives: usize,
    /// 群戏参与者列表（群戏节拍时必填）
    #[serde(default)]
    pub participants: Option<serde_json::Value>,
}

fn default_num_alts() -> usize { 3 }

/// 备选方案生成响应
#[derive(Debug, Serialize)]
pub struct AlternativesResponse {
    /// 生成的备选方案列表
    pub alternatives: Vec<BeatAlternative>,
}

const ALTERNATIVES_SYSTEM_PROMPT: &str =
    "你是一名专业编剧。用户要求你为剧本中的某个节拍生成多个不同风格或角度的备选版本。\n\
     要求：\n\
     1. 每个备选版本必须在保持原意的前提下，从不同角度或风格重新表达\n\
     2. 保持与原剧本的角色设定、情节发展、世界观一致\n\
     3. 每个版本的语气、节奏、情感强度应有所区别\n\
     4. 为每个版本标注其风格/情感特征（tone）\n\
     5. 返回 JSON 格式结果\n\n\
     输出格式：\n\
     {\"alternatives\": [{\"content\": \"备选版本1的内容\", \"name\": \"正式版\", \"tone\": \"正式严肃\", \"emotion\": \"紧张\"}, ...]}";

// ==================== 群戏节拍专用提示词 ====================

/// 群戏节拍 AI 重生成系统提示词
/// 群戏（group beat）是多人同时参与的场景，每个参与者有独立的角色（speaker/listener/observer）和对话内容
const GROUP_BEAT_REGENERATE_SYSTEM_PROMPT: &str =
    "你是一名专业编剧，擅长处理多人群戏场景的改写。用户要求你重新生成一个【群戏节拍】。\n\n\
     **群戏节拍的独特结构**：\n\
     - 一个群戏节拍包含多个参与者（participants），每人有 character_id、role（speaker/listener/observer）、dialogue\n\
     - speaker 是当前发言者，listener 在倾听/反应，observer 在旁观\n\
     - 节拍的 content 字段通常是对整体氛围或动作的概括性描述\n\
     - 每个参与者的 dialogue 是该角色的具体台词/动作\n\n\
     **重写要求**：\n\
     1. 必须保持所有参与者的角色设定、性格特征不变\n\
     2. 每个参与者的对话必须符合其 role 定位（speaker 主动发言、listener 反应式回应、observer 旁白式观察）\n\
     3. 多人对话之间要有自然的互动感和节奏感（抢话、沉默、打断等真实细节）\n\
     4. 保持与原剧本情节发展、世界观一致\n\
     5. 生成的内容应自然衔接前后文\n\n\
     **输出格式（必须严格遵循）**：\n\
     返回 JSON：\n\
     {\n\
       \"content\": \"对整个群戏节拍的概括性描述（动作/氛围）\",\n\
       \"participants\": [\n\
         {\"character_id\": \"char_XXX\", \"role\": \"speaker\", \"dialogue\": \"该角色的台词\"},\n\
         {\"character_id\": \"char_YYY\", \"role\": \"listener\", \"dialogue\": \"该角色的反应台词\"}\n\
       ],\n\
       \"alternatives\": [{\"content\": \"备选概括\", \"participants\": [...], \"tone\": \"风格描述\"}]\n\
     }";

/// 群戏节拍 AI 备选方案系统提示词
const GROUP_BEAT_ALTERNATIVES_SYSTEM_PROMPT: &str =
    "你是一名专业编剧，擅长为多人群戏场景生成不同风格的备选方案。\n\n\
     **群戏节拍结构说明**：\n\
     - 群戏节拍含多个参与者（participants），每人有 character_id、role（speaker/listener/observer）、dialogue\n\
     - 备选方案必须为每个参与者都提供新的对话内容，不能遗漏任何参与者\n\n\
     **生成要求**：\n\
     1. 每个备选方案必须在保持原意和参与者完整的前提下，从不同角度或风格重新表达\n\
     2. 保持与原剧本的角色设定、情节发展、世界观一致\n\
     3. 不同方案的语气、节奏、互动模式应有所区别（如：紧张对抗版 / 轻松调侃版 / 压抑沉默版）\n\
     4. 每个方案中所有 participants 必须完整保留（不可减少或增加参与者）\n\
     5. 为每个方案标注其风格/情感特征（tone）\n\n\
     **输出格式（必须严格遵循）**：\n\
     {\n\
       \"alternatives\": [\n\
         {\n\
           \"content\": \"方案概括描述\",\n\
           \"name\": \"方案名称\",\n\
           \"tone\": \"风格描述\",\n\
           \"emotion\": \"情感基调\",\n\
           \"participants\": [\n\
             {\"character_id\": \"char_XXX\", \"role\": \"speaker\", \"dialogue\": \"新台词\"},\n\
             {\"character_id\": \"char_YYY\", \"role\": \"listener\", \"dialogue\": \"新反应\"}\n\
           ]\n\
         },\n\
         ...\n\
       ]\n\
     }";

/// AI 生成备选方案
pub async fn alternatives(
    req: &AlternativesRequest,
    api_key: &str,
    base_url: &str,
) -> Result<AlternativesResponse, String> {
    // Mock 模式
    if api_key.is_empty() || api_key == "mock" {
        return Ok(mock_alternatives_response(req));
    }

    // 判断是否为群戏节拍
    let is_group_beat = req.participants.as_ref()
        .map(|p| p.is_array())
        .unwrap_or(false);

    let instruction = req.instruction.as_deref().unwrap_or(
        if is_group_beat {
            "请为这个群戏节拍生成多个不同互动风格的备选版本，每个版本中所有参与者都应有新的对话内容"
        } else {
            "请为这个节拍生成多个不同风格的备选版本，每个版本应有独特的语气和表达角度"
        }
    );

    // 构建用户消息
    let mut user_msg = if is_group_beat {
        format!(
            "请为以下【群戏节拍】生成 {} 个不同风格的备选版本：\n",
            req.num_alternatives
        )
    } else {
        format!(
            "请为以下节拍生成 {} 个不同风格的备选版本：\n",
            req.num_alternatives
        )
    };
    if let Some(t) = &req.current_type {
        user_msg.push_str(&format!("- 节拍类型: {}\n", t));
    }
    if let Some(s) = &req.speaker {
        user_msg.push_str(&format!("- 说话人: {}\n", s));
    }
    if let Some(c) = &req.current_content {
        user_msg.push_str(&format!("- 当前内容:\n{}\n", c));
    }
    // 群戏节拍：附加参与者信息
    if let Some(ref pts) = req.participants {
        if pts.is_array() {
            user_msg.push_str(&format!(
                "- 参与者列表:\n{}\n",
                serde_json::to_string_pretty(pts).unwrap_or_default()
            ));
        }
    }
    user_msg.push_str(&format!("\n- 生成方向: {}", instruction));

    // 截取上下文
    if let Some(yaml) = &req.full_script_yaml {
        let preview_len = yaml.len().min(3000);
        let safe_len = yaml.floor_char_boundary(preview_len);
        user_msg.push_str(&format!(
            "\n\n剧本上下文（前{}字）：\n{}",
            safe_len,
            &yaml[..safe_len]
        ));
    }

    // 根据是否为群戏选择不同的系统提示词
    let system_prompt = if is_group_beat {
        GROUP_BEAT_ALTERNATIVES_SYSTEM_PROMPT
    } else {
        ALTERNATIVES_SYSTEM_PROMPT
    };

    let response = call_deepseek(
        &user_msg,
        api_key,
        base_url,
        Some(system_prompt),
        true, // JSON 模式
        "deepseek-chat",
    ).await?;

    // 解析响应中的 alternatives 数组
    let parsed: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| format!("解析备选方案 JSON 失败: {}", e))?;

    let alts = parse_alternatives_from_json(&parsed)?;

    tracing::info!(num_alts = alts.len(), "AI 备选方案生成成功");

    Ok(AlternativesResponse { alternatives: alts })
}

/// 从 JSON 值中提取 alternatives 数组
fn parse_alternatives_from_json(value: &serde_json::Value) -> Result<Vec<BeatAlternative>, String> {
    // 直接是数组
    if let Some(arr) = value.as_array() {
        return parse_alt_array(arr);
    }

    // 是对象，找 alternatives 字段
    if let Some(obj) = value.as_object() {
        for key in ["alternatives", "alts", "options", "versions"] {
            if let Some(arr) = obj.get(key).and_then(|v| v.as_array()) {
                return parse_alt_array(arr);
            }
        }
    }

    // 兜底：将整个值作为单个 content 包装
    Ok(vec![BeatAlternative {
        content: value.to_string(),
        name: None,
        tone: None,
        emotion: None,
    }])
}

fn parse_alt_array(arr: &[serde_json::Value]) -> Result<Vec<BeatAlternative>, String> {
    let mut result = Vec::new();
    for item in arr {
        match item {
            serde_json::Value::String(s) => {
                result.push(BeatAlternative {
                    content: s.clone(),
                    name: None,
                    tone: None,
                    emotion: None,
                });
            }
            serde_json::Value::Object(obj) => {
                result.push(BeatAlternative {
                    content: obj.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: obj.get("name").and_then(|v| v.as_str()).map(String::from),
                    tone: obj.get("tone").and_then(|v| v.as_str()).map(String::from),
                    emotion: obj.get("emotion").and_then(|v| v.as_str()).map(String::from),
                });
            }
            _ => {}
        }
    }
    if result.is_empty() {
        return Err("未找到有效的备选方案数据".to_string());
    }
    Ok(result)
}

fn mock_alternatives_response(req: &AlternativesRequest) -> AlternativesResponse {
    // 判断是否为群戏节拍
    let is_group = req.participants.as_ref()
        .map(|p| p.is_array())
        .unwrap_or(false);

    if is_group {
        // 群戏节拍 mock：返回含 participants 的备选方案
        AlternativesResponse {
            alternatives: vec![
                BeatAlternative {
                    content: "[Mock] 紧张对抗版 — 众人针锋相对，气氛剑拔弩张。".to_string(),
                    name: Some("紧张对抗版".to_string()),
                    tone: Some("激烈冲突".to_string()),
                    emotion: Some("愤怒/紧张".to_string()),
                },
                BeatAlternative {
                    content: "[Mock] 轻松调侃版 — 用幽默化解紧张，人物间互相打趣。".to_string(),
                    name: Some("轻松调侃版".to_string()),
                    tone: Some("轻松幽默".to_string()),
                    emotion: Some("愉快/放松".to_string()),
                },
                BeatAlternative {
                    content: "[Mock] 压抑沉默版 — 沉默中暗流涌动，用动作和眼神代替语言。".to_string(),
                    name: Some("压抑沉默版".to_string()),
                    tone: Some("内敛压抑".to_string()),
                    emotion: Some("沉重/不安".to_string()),
                },
            ],
        }
    } else {
        AlternativesResponse {
            alternatives: vec![
                BeatAlternative {
                    content: "[Mock] 备选方案 A — 更正式的表述方式，适合严肃场合的对白。".to_string(),
                    name: Some("正式版".to_string()),
                    tone: Some("正式严肃".to_string()),
                    emotion: Some("沉稳".to_string()),
                },
                BeatAlternative {
                    content: "[Mock] 备选方案 B — 更轻松幽默的表达，增加人物亲和力。".to_string(),
                    name: Some("轻松版".to_string()),
                    tone: Some("轻松幽默".to_string()),
                    emotion: Some("愉快".to_string()),
                },
                BeatAlternative {
                    content: "[Mock] 备选方案 C — 更具冲突感的对白，增强戏剧张力。".to_string(),
                    name: Some("冲突版".to_string()),
                    tone: Some("紧张激烈".to_string()),
                    emotion: Some("愤怒".to_string()),
                },
            ],
        }
    }
}

const REGENERATE_SYSTEM_PROMPT: &str =
    "你是一名专业编剧。用户要求你根据其修改意图重新生成剧本中的某个内容。\n\
     要求：\n\
     1. 必须保持与原剧本的角色设定、情节发展、世界观一致\n\
     2. 保持角色语气和性格特征不变\n\
     3. 生成的内容应自然衔接前后文\n\
     4. 返回 JSON 格式结果\n\n\
     输出格式：\n\
     {\"content\": \"主要生成的内容\", \"alternatives\": [{\"content\": \"备选1\", \"tone\": \"语气描述\"}]}";

/// AI 重生成（针对单个节拍或整场场景）
pub async fn regenerate(
    req: &RegenerateRequest,
    api_key: &str,
    base_url: &str,
) -> Result<RegenerateResponse, String> {
    // 模拟模式
    if api_key.is_empty() || api_key == "mock" {
        return Ok(mock_regenerate_response(req));
    }

    // 判断是否为群戏节拍
    let is_group_beat = req.participants.as_ref()
        .map(|p| p.is_array())
        .unwrap_or(false);

    // 构建上下文信息
    let mut context_parts = Vec::new();
    context_parts.push(format!("目标类型: {}", req.target));
    context_parts.push(format!("场景 ID: {}", req.scene_id));
    if let Some(bi) = req.beat_index {
        context_parts.push(format!("节拍索引: {}", bi));
    }
    if is_group_beat {
        context_parts.push("** 注意：这是一个【群戏节拍】，包含多个参与者 **".to_string());
    }
    if let Some(content) = &req.current_content {
        context_parts.push(format!("当前内容:\n{}", content));
    }
    // 群戏节拍：附加参与者信息
    if let Some(ref pts) = req.participants {
        if pts.is_array() {
            context_parts.push(format!(
                "当前参与者列表:\n{}",
                serde_json::to_string_pretty(pts).unwrap_or_default()
            ));
        }
    }

    // 截取完整剧本作为上下文（限制长度避免超 token）
    if let Some(yaml) = &req.full_script_yaml {
        let preview_len = yaml.len().min(4000);
        // 确保截断位置在字符边界上（UTF-8 中文 3 字节，不能从字节中间切）
        let safe_len = yaml.floor_char_boundary(preview_len);
        context_parts.push(format!(
            "\n完整剧本上下文（前{}字）：\n{}",
            safe_len,
            &yaml[..safe_len]
        ));
    }

    let user_prompt = format!(
        "{}\n\n修改意图: {}",
        context_parts.join("\n"),
        req.instruction
    );

    // 根据是否为群戏选择不同的系统提示词
    let system_prompt = if is_group_beat {
        GROUP_BEAT_REGENERATE_SYSTEM_PROMPT
    } else {
        REGENERATE_SYSTEM_PROMPT
    };

    let response = call_deepseek(
        &user_prompt,
        api_key,
        base_url,
        Some(system_prompt),
        true, // JSON 模式
        "deepseek-chat",
    )
    .await?;

    // 解析 AI 返回的 JSON
    match serde_json::from_str::<serde_json::Value>(&response) {
        Ok(json) => {
            let content = json.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or(&response)
                .to_string();

            // 解析 alternatives
            let alternatives = json.get("alternatives").and_then(|arr| {
                arr.as_array().map(|items| {
                    items.iter().filter_map(|item| {
                        Some(BeatAlternative {
                            content: item.get("content")?.as_str()?.to_string(),
                            name: item.get("name").and_then(|v| v.as_str()).map(String::from),
                            tone: item.get("tone").and_then(|v| v.as_str()).map(String::from),
                            emotion: item.get("emotion").and_then(|v| v.as_str()).map(String::from),
                        })
                    }).collect::<Vec<_>>()
                })
            });

            Ok(RegenerateResponse {
                content,
                alternatives,
                beats: None,
            })
        }
        Err(_) => {
            // JSON 解析失败时直接使用原文
            tracing::warn!("AI 重生成返回的不是有效 JSON，使用原始文本");
            Ok(RegenerateResponse {
                content: response,
                alternatives: None,
                beats: None,
            })
        }
    }
}

fn mock_regenerate_response(req: &RegenerateRequest) -> RegenerateResponse {
    // 判断是否为群戏节拍
    let is_group = req.participants.as_ref()
        .map(|p| p.is_array())
        .unwrap_or(false);

    if req.target == "scene" {
        RegenerateResponse {
            content: format!("已重新生成场景 {} 的全部内容（模拟模式）", req.scene_id),
            alternatives: None,
            beats: Some(serde_json::json!([
                {
                    "type": "action",
                    "content": format!("[AI重生成] 场景{} - 动作描述（模拟）", req.scene_id),
                    "alternatives": [],
                    "selected": 0
                },
                {
                    "type": "dialogue",
                    "content": "[AI重生成] 对白内容（模拟）",
                    "alternatives": [],
                    "selected": 0,
                    "speaker": "角色A"
                },
                {
                    "type": "action",
                    "content": "[AI重生成] 结尾动作（模拟）",
                    "alternatives": [],
                    "selected": 0
                }
            ])),
        }
    } else if is_group {
        // 群戏节拍 mock：返回含 participants 的重生成结果
        RegenerateResponse {
            content: format!(
                "[AI重生成·群戏] 根据「{}」的要求重新生成了此群戏节拍（模拟模式）",
                req.instruction
            ),
            alternatives: Some(vec![
                BeatAlternative {
                    content: "[备选·群戏A] 紧张对抗风格的群戏版本（模拟）".to_string(),
                    tone: Some("激烈冲突".to_string()),
                    name: None,
                    emotion: None,
                },
            ]),
            beats: None,
        }
    } else {
        RegenerateResponse {
            content: format!(
                "[AI重生成] 根据「{}」的要求重新生成了此节拍内容（模拟模式）",
                req.instruction
            ),
            alternatives: Some(vec![
                BeatAlternative {
                    content: "[备选方案A] 另一种表达方式（模拟）".to_string(),
                    tone: Some("正式".to_string()),
                    name: None,
                    emotion: None,
                },
                BeatAlternative {
                    content: "[备选方案B] 更口语化的版本（模拟）".to_string(),
                    tone: Some("轻松".to_string()),
                    name: None,
                    emotion: None,
                },
            ]),
            beats: None,
        }
    }
}

// ==================== 项目信息自动生成 ====================

/// 项目信息生成请求
#[derive(Debug, Deserialize)]
pub struct ProjectInfoRequest {
    /// 小说文本（用于分析主题）
    pub novel_text: String,
    /// 剧本风格
    #[serde(default = "default_style")]
    pub style: String,
}

fn default_style() -> String {
    "short_drama".to_string()
}

/// 项目信息生成响应
#[derive(Debug, Serialize)]
pub struct ProjectInfoResponse {
    pub name: String,
    pub description: String,
}

const PROJECT_INFO_SYSTEM_PROMPT: &str =
    "你是一个专业的影视项目命名和描述助手。\
     根据提供的小说文本片段和剧本风格，生成一个简洁有吸引力的项目名称（不超过20字）\
     和一段准确的项目描述（不超过100字）。\n\n\
     要求：\n\
     - 名称：体现故事核心主题或情感基调，具有辨识度\n\
     - 描述：概括故事主线、主要角色关系、风格特点\n\
     - 输出严格的 JSON 格式\n\n\
     输出格式：\n\
     {\"name\": \"项目名称\", \"description\": \"项目描述\"}";

/// 根据小说文本自动生成项目名称和描述
pub async fn generate_project_info(
    req: &ProjectInfoRequest,
    api_key: &str,
    base_url: &str,
) -> Result<ProjectInfoResponse, String> {
    // 模拟模式
    if api_key.is_empty() || api_key == "mock" {
        return Ok(mock_project_info_response(&req.novel_text));
    }

    let novel_preview = safe_truncate(&req.novel_text, 2000);
    let style_label = match req.style.as_str() {
        "film" => "电影",
        "stage" => "舞台剧",
        _ => "短剧",
    };

    let user_prompt = format!(
        "小说文本预览:\n{}\n\n目标剧本风格: {}",
        novel_preview, style_label
    );

    let response = call_deepseek(
        &user_prompt,
        api_key,
        base_url,
        Some(PROJECT_INFO_SYSTEM_PROMPT),
        true, // JSON 模式
        "deepseek-chat",
    )
    .await?;

    match serde_json::from_str::<serde_json::Value>(&response) {
        Ok(json) => Ok(ProjectInfoResponse {
            name: json.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("未命名项目")
                .to_string(),
            description: json.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("AI 生成的剧本项目")
                .to_string(),
        }),
        Err(_) => {
            tracing::warn!("项目信息生成返回非 JSON 格式");
            Ok(mock_project_info_response(&req.novel_text))
        }
    }
}

fn mock_project_info_response(novel_text: &str) -> ProjectInfoResponse {
    let preview = safe_truncate(novel_text, 300);
    let name_hint = preview
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim())
        .unwrap_or("未命名");

    let name = if name_hint.len() > 18 {
        let safe_cut = name_hint.floor_char_boundary(16);
        format!("{}…改编剧本", &name_hint[..safe_cut])
    } else {
        format!("{}改编剧本", name_hint)
    };

    ProjectInfoResponse {
        name,
        description: format!(
            "基于小说文本生成的短剧风格剧本。{}",
            if novel_text.len() > 500 {
                "涵盖多章节叙事结构，包含完整的人物关系与情节发展。"
            } else {
                "包含核心场景与人物对话。"
            }
        ),
    }
}
