use serde::{Deserialize, Serialize};

/// LLM 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// DeepSeek API 请求体
#[derive(Debug, Serialize)]
struct DeepSeekRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<DeepSeekResponseFormat>,
}

#[derive(Debug, Serialize)]
struct DeepSeekResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

/// DeepSeek API 响应体
#[derive(Debug, Deserialize)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
    #[allow(dead_code)]
    usage: Option<DeepSeekUsage>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessage,
}

#[derive(Debug, Deserialize)]
struct DeepSeekMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct DeepSeekUsage {
    #[allow(dead_code)]
    total_tokens: u32,
}

/// 调用 DeepSeek API（或模拟模式）
///
/// # 参数
/// - `prompt`: 用户提示词或系统提示+用户消息
/// - `api_key`: API Key，为空或 "mock" 时启用模拟模式
/// - `base_url`: API 基础 URL
/// - `system_prompt`: 可选的系统提示词
/// - `json_mode`: 是否要求返回 JSON 格式
/// - `model`: 模型名称（如 "deepseek-chat"）
pub async fn call_deepseek(
    prompt: &str,
    api_key: &str,
    base_url: &str,
    system_prompt: Option<&str>,
    json_mode: bool,
    model: &str,
) -> Result<String, String> {
    // 模式模式判断
    let is_mock = api_key.is_empty() || api_key == "mock";

    if is_mock {
        tracing::warn!("⚠️  当前为模拟模式，未调用 DeepSeek API（无 token 消耗）");
        return Ok(mock_response(prompt, json_mode));
    }

    // 构建消息列表
    let mut messages = Vec::new();
    if let Some(sys) = system_prompt {
        messages.push(Message {
            role: "system".to_string(),
            content: sys.to_string(),
        });
    }
    messages.push(Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    });

    // 构建请求体
    let msg_count = messages.len();
    let mut request = DeepSeekRequest {
        model: model.to_string(),
        messages,
        temperature: Some(0.7),
        response_format: None,
    };

    if json_mode {
        request.response_format = Some(DeepSeekResponseFormat {
            format_type: "json_object".to_string(),
        });
    }

    // 发送 HTTP 请求
    // DeepSeek OpenAI 兼容格式：base_url = https://api.deepseek.com，实际端点为 /v1/chat/completions
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(15))
        .user_agent("Novel2Script-Pro/1.0")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    let base_url_trimmed = base_url.trim_end_matches('/');
    let url = format!("{}/v1/chat/completions", base_url_trimmed);

    tracing::info!(url = %url, model = %model, msg_count = msg_count, "Calling DeepSeek API");

    // 验证 URL 有效性
    let parsed_url = reqwest::Url::parse(&url)
        .map_err(|e| format!("API Base URL 无效 '{}': {}", url, e))?;

    let response = client
        .post(parsed_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, url = %url, "DeepSeek API HTTP 请求失败");
            // 提供更具体的诊断信息
            if e.is_timeout() {
                format!("请求 DeepSeek API 超时（{}s）：请检查网络连接或 API 地址是否可达", 120)
            } else if e.is_connect() {
                format!("无法连接到 DeepSeek API ({})：请检查网络连接、防火墙或代理设置", url)
            } else if e.to_string().contains("dns") || e.to_string().contains("DNS") {
                format!("DNS 解析失败：无法解析域名 {}，请检查网络设置", base_url)
            } else if e.to_string().contains("certificate") || e.to_string().contains("tls") || e.to_string().contains("ssl") {
                format!("TLS/SSL 握手失败 ({}): 请检查系统证书或使用 native-tls 后端", url)
            } else {
                format!("请求 DeepSeek API 失败 ({})：请检查网络连接和 API 地址是否正确", e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("DeepSeek API 返回错误 ({}): {}", status, body));
    }

    let deepseek_resp: DeepSeekResponse = response
        .json()
        .await
        .map_err(|e| format!("解析 DeepSeek API 响应失败: {}", e))?;

    deepseek_resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "DeepSeek API 返回空响应".to_string())
}

/// 模拟模式：根据提示词关键词返回预设的示例数据
fn mock_response(prompt: &str, _json_mode: bool) -> String {
    let prompt_lower = prompt.to_lowercase();

    // 根据提示词内容返回不同的模拟数据
    if prompt_lower.contains("分析") || prompt_lower.contains("emotion") || prompt_lower.contains("情感") {
        r#"{"chapters":[1,2,3],"intensities":[0.2,0.7,0.9]}"#.to_string()
    } else if prompt_lower.contains("角色") || prompt_lower.contains("character") || prompt_lower.contains("抽取") {
        r#"[{"id":"char_001","name":"林曦","traits":["敏感","执着","善良"],"voice":"轻柔，慢速，偶尔停顿"},{"id":"char_002","name":"顾言","traits":["冷静","理性","内敛"],"voice":"简洁，低沉，条理清晰"},{"id":"char_003","name":"苏晴","traits":["活泼","直率","热心"],"voice":"语速快，音调高，喜欢用感叹号"}]"#.to_string()
    } else if prompt_lower.contains("因果") || prompt_lower.contains("causal") || prompt_lower.contains("事件") {
        r#"{"events":[{"id":"evt_001","description":"林曦在图书馆偶遇顾言","scene_ids":[1],"chapter":1},{"id":"evt_002","description":"顾言借给林曦一本笔记","scene_ids":[1],"chapter":1},{"id":"evt_003","description":"林曦发现笔记中的秘密留言","scene_ids":[2],"chapter":2},{"id":"evt_004","description":"苏晴告诉林曦顾言的过去","scene_ids":[3],"chapter":3}],"edges":[{"from":"evt_001","to":"evt_002","type":"causal","strength":0.9,"description":"偶遇引发初次互动"},{"from":"evt_002","to":"evt_003","type":"causal","strength":1.0,"description":"借出笔记直接导致秘密被发现"},{"from":"evt_003","to":"evt_004","type":"emotional","strength":0.7,"description":"情感冲击促使寻求真相"}]}"#.to_string()
    } else if prompt_lower.contains("关系") || prompt_lower.contains("relation") || prompt_lower.contains("网络") {
        r#"{"matrix":[{"from":"林曦","to":"顾言","intimacy":0.6,"power_gap":-0.3,"trust":0.7,"relation_type":"暧昧","description":"图书馆相遇后产生的好感与试探","history":[{"scene_id":1,"delta_intimacy":0.2,"delta_power":0.0,"delta_trust":0.3}]},{"from":"顾言","to":"林曦","intimacy":0.6,"power_gap":0.3,"trust":0.65,"relation_type":"暧昧","description":"对林曦的关注与保护欲","history":[{"scene_id":1,"delta_intimacy":0.2,"delta_power":0.0,"delta_trust":0.25}]},{"from":"林曦","to":"苏晴","intimacy":0.8,"power_gap":0.1,"trust":0.9,"relation_type":"闺蜜","description":"无话不谈的挚友关系","history":[]},{"from":"苏晴","to":"林曦","intimacy":0.8,"power_gap":-0.1,"trust":0.88,"relation_type":"闺蜜","description":"最信任的朋友","history":[]},{"from":"顾言","to":"苏晴","intimacy":0.3,"power_gap":0.2,"trust":0.4,"relation_type":"疏远","description":"因林曦产生的微妙竞争感","history":[]},{"from":"苏晴","to":"顾言","intimacy":0.3,"power_gap":-0.2,"trust":0.35,"relation_type":"提防","description":"担心顾言伤害林曦","history":[]}]}"#.to_string()
    } else if prompt_lower.contains("剧本") || prompt_lower.contains("script") || prompt_lower.contains("生成") || prompt_lower.contains("节拍") {
        // 返回完整剧本 YAML 的 JSON 表示（后续会被转为 YAML）
        mock_script_json()
    } else if prompt_lower.contains("副编剧") || prompt_lower.contains("copilot") || prompt_lower.contains("建议") {
        r#"{"reply":"这是模拟模式下的回复。在真实模式下，我会根据你的剧本提供专业建议。请配置 DeepSeek API Key 开始创作。","suggestions":["尝试在第二幕增加一个转折点","角色的对白可以更有个性","考虑增加一个象征性的道具"]}"#.to_string()
    } else {
        // 默认模拟回复
        "这是模拟模式下的回复。请配置 DeepSeek API Key 以获得真实的 AI 分析结果。".to_string()
    }
}

/// 模拟模式的完整剧本数据（JSON 格式，后续序列化为 YAML）
fn mock_script_json() -> String {
    r#"{
  "metadata": {
    "title": "《遇见》",
    "source_novel": "示例小说",
    "adaptation_date": "2025-01-15",
    "style": "short_drama",
    "emotional_curve": {"chapters": [1, 2, 3], "intensities": [0.2, 0.7, 0.9]}
  },
  "characters": [
    {"id": "char_001", "name": "林曦", "traits": ["敏感", "执着", "善良"], "voice": "轻柔，慢速"},
    {"id": "char_002", "name": "顾言", "traits": ["冷静", "理性", "内敛"], "voice": "简洁，低沉"},
    {"id": "char_003", "name": "苏晴", "traits": ["活泼", "直率", "热心"], "voice": "语速快，音调高"}
  ],
  "causal_graph": {
    "events": [
      {"id": "evt_001", "description": "林曦在图书馆偶遇顾言", "scene_ids": [1], "chapter": 1},
      {"id": "evt_002", "description": "顾言借给林曦一本笔记", "scene_ids": [1], "chapter": 1},
      {"id": "evt_003", "description": "林曦发现笔记中的秘密留言", "scene_ids": [2], "chapter": 2}
    ],
    "edges": [
      {"from": "evt_001", "to": "evt_002", "type": "causal", "strength": 0.9, "description": "偶遇引发初次互动与对话"},
      {"from": "evt_002", "to": "evt_003", "type": "causal", "strength": 1.0, "description": "借出笔记直接导致秘密被发现"}
    ]
  },
  "relation_network": {
    "matrix": [
      {"from": "林曦", "to": "顾言", "intimacy": 0.6, "power_gap": -0.3, "trust": 0.7,
       "relation_type": "暧昧", "description": "图书馆相遇后产生的好感", "history": []},
      {"from": "顾言", "to": "林曦", "intimacy": 0.6, "power_gap": 0.3, "trust": 0.65,
       "relation_type": "暧昧", "description": "对林曦的关注与保护欲", "history": []},
      {"from": "林曦", "to": "苏晴", "intimacy": 0.8, "power_gap": 0.1, "trust": 0.9,
       "relation_type": "闺蜜", "description": "无话不谈的挚友", "history": []}
    ]
  },
  "scenes": [
    {
      "id": 1,
      "location": "大学图书馆 · 靠窗角落",
      "time": "午后",
      "emotion_intensity": 0.3,
      "beats": [
        {"type": "action", "content": "阳光透过落地窗洒在木质书桌上。林曦正埋头在一堆参考书中，时不时推一下滑落的眼镜。", "alternatives": [{"content": "午后的图书馆安静得能听见翻书声。林曦坐在靠窗的位置，面前摊开着三本厚重的专业书。", "tone": "静谧"}], "selected": 0},
        {"type": "action", "content": "一个身影停在林曦对面。林曦抬头——是顾言，那个总是一个人坐在最后一排的男生。", "alternatives": [{"content": "一道阴影落在书页上。林曦抬起头，撞进一双平静如水的眼睛里。", "tone": "文艺"}], "selected": 0},
        {"type": "dialogue", "content": "这里有人吗？", "speaker": "顾言", "emotion": "平淡", "alternatives": [{"content": "请问，这个位置……", "tone": "礼貌"}, {"content": "介意我坐这儿吗？", "tone": "随意"}], "selected": 0},
        {"type": "dialogue", "content": "啊，没有，你坐。", "speaker": "林曦", "emotion": "惊讶→镇定", "alternatives": [{"content": "没、没有，请坐……", "tone": "慌乱"}], "selected": 0},
        {"type": "action", "content": "顾言坐下，从背包里取出一本深蓝色封皮的笔记本，推到林曦面前。", "alternatives": [], "selected": 0},
        {"type": "dialogue", "content": "你的。上次落在我桌上的。", "speaker": "顾言", "emotion": "平静", "alternatives": [], "selected": 0},
        {"type": "monologue", "content": "林曦愣住了。她完全不记得自己丢过笔记本。接过笔记本的瞬间，她的指尖触到了封面上微微凸起的字迹。", "alternatives": [], "selected": 0}
      ],
      "media_hints": {"camera": "中景 → 特写（笔记本封面）", "music": "轻钢琴曲，渐入"}
    },
    {
      "id": 2,
      "location": "林曦宿舍 · 书桌前",
      "time": "深夜",
      "emotion_intensity": 0.75,
      "beats": [
        {"type": "action", "content": "台灯下，林曦翻开那本深蓝色的笔记本。第一页是空白的，但对着光看时，隐约可见铅笔写下的字迹。", "alternatives": [{"content": "林曦关上宿舍门，迫不及待地打开笔记本。借着台灯的光，她发现每一页的角落都有细小的铅笔字。", "tone": "悬疑"}], "selected": 0},
        {"type": "monologue", "content": "\"如果你看到了这些字，说明我们之间有某种联系。——G\"", "alternatives": [], "selected": 0},
        {"type": "dialogue", "content": "G……顾言？", "speaker": "林曦", "emotion": "震惊→困惑", "alternatives": [{"content": "这个 G 是……", "tone": "自言自语"}], "selected": 0},
        {"type": "action", "content": "林曦快速翻动笔记本。每一页的角落都有一句话，像一条隐秘的线索，串起一段不为人知的故事。", "alternatives": [], "selected": 0},
        {"type": "parenthetical", "content": "（林曦的手微微颤抖）", "alternatives": [], "selected": 0}
      ],
      "media_hints": {"camera": "特写（笔记本字迹）→ 侧脸光影", "music": "悬疑弦乐，低沉"}
    }
  ]
}"#.to_string()
}

// ==================== 辅助函数 =====

/// 根据任务类型选择模型
pub fn select_model(task_type: &str) -> &'static str {
    match task_type {
        "analysis" | "emotion" => "deepseek-chat",
        "generation" => "deepseek-chat",
        _ => "deepseek-chat",
    }
}

/// 粗略估算中文字符数对应的 token 数（中文约 1.5 token/字符）
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count() * 3 / 2  // 粗略估算
}

/// 并行调用多个 LLM 请求，返回所有结果
/// 使用 tokio::sync::Semaphore 控制并发数
pub async fn call_parallel<Fut, T>(
    futures: Vec<Fut>,
    max_concurrent: usize,
) -> Vec<Result<T, String>>
where
    Fut: std::future::Future<Output = Result<T, String>> + Send + 'static,
    T: Send + 'static,
{
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::with_capacity(futures.len());

    for fut in futures {
        let permit = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            fut.await
        }));
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap_or(Err("任务执行失败".to_string())));
    }
    results
}
