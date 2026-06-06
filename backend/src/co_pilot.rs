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
        context_info.push_str(&format!(
            "\n\n当前剧本预览（前{}字）：\n{}",
            preview_len,
            &yaml[..preview_len]
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

    // 构建上下文信息
    let mut context_parts = Vec::new();
    context_parts.push(format!("目标类型: {}", req.target));
    context_parts.push(format!("场景 ID: {}", req.scene_id));
    if let Some(bi) = req.beat_index {
        context_parts.push(format!("节拍索引: {}", bi));
    }
    if let Some(content) = &req.current_content {
        context_parts.push(format!("当前内容:\n{}", content));
    }

    // 截取完整剧本作为上下文（限制长度避免超 token）
    if let Some(yaml) = &req.full_script_yaml {
        let preview_len = yaml.len().min(4000);
        context_parts.push(format!(
            "\n完整剧本上下文（前{}字）：\n{}",
            preview_len,
            &yaml[..preview_len]
        ));
    }

    let user_prompt = format!(
        "{}\n\n修改意图: {}",
        context_parts.join("\n"),
        req.instruction
    );

    let response = call_deepseek(
        &user_prompt,
        api_key,
        base_url,
        Some(REGENERATE_SYSTEM_PROMPT),
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
