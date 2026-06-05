use serde::{Deserialize, Serialize};
use crate::llm_client::{call_deepseek, Message};

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
        &script_yaml[..script_yaml.len().min(5000)]
    );

    let response = call_deepseek(&prompt, api_key, base_url, None, true).await?;

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
