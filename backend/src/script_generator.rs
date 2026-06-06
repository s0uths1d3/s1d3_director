use crate::models::*;
use crate::llm_client::call_deepseek;
use crate::emotion_analyzer::analyze_emotions;
use crate::utils::safe_truncate;

/// 生成完整剧本 YAML
pub async fn generate_script(
    text: &str,
    config: &GenConfig,
    api_key: &str,
    base_url: &str,
) -> Result<String, String> {
    // 如果有 API Key，调用 LLM 生成
    if !api_key.is_empty() && api_key != "mock" {
        tracing::info!(api_key_len = api_key.len(), base_url = %base_url, "使用 DeepSeek LLM 生成剧本");

        match generate_with_llm(text, config, api_key, base_url).await {
            Ok(result) => {
                tracing::info!("LLM 剧本生成成功");
                return Ok(result);
            }
            Err(e) => {
                tracing::error!(error = %e, "LLM 剧本生成失败，将错误返回给前端（不再静默降级为模拟模式）");
                return Err(format!("AI 生成失败: {}。请检查 API Key 和网络连接，或暂时清空 API Key 使用模拟模式。", e));
            }
        }
    }

    // 模拟模式
    tracing::warn!("API Key 未配置或为 mock 模式，使用预设示例数据");
    Ok(generate_mock_script(text, config))
}

/// 使用 LLM 生成剧本
async fn generate_with_llm(
    text: &str,
    config: &GenConfig,
    api_key: &str,
    base_url: &str,
) -> Result<String, String> {
    // 1. 分析情感曲线
    let emotional_curve = analyze_emotions(text, api_key, base_url).await?;

    // 2. 构建 LLM 提示词生成剧本
    let style_desc = match config.style.as_str() {
        "film" => "电影剧本",
        "short_drama" => "短剧剧本",
        "stage" => "舞台剧剧本",
        _ => "短剧剧本",
    };

    let blind_mode_note = if config.blind_acting_mode {
        "每个节拍必须提供至少2个备选内容（alternatives），包含不同语气（tone）的变体。"
    } else {
        ""
    };

    let media_note = if config.include_media_hints {
        "每个场景必须包含 media_hints 字段，包含 camera（镜头建议）和 music（配乐风格）。"
    } else {
        ""
    };

    let prompt = format!(
        r#"你是一名专业编剧。请将以下小说改编为{}。

## 输出要求

你必须输出一个完整的 JSON 对象，严格遵循以下结构。不要输出任何其他文字、解释或 markdown 标记。

### JSON 结构定义

{{
  "metadata": {{
    "title": "剧本标题",
    "source_novel": "原小说名称或描述",
    "adaptation_date": "YYYY-MM-DD",
    "style": "{}",
    "emotional_curve": {{ "chapters": [1,2], "intensities": [0.3,0.7] }}
  }},
  "characters": [
    {{
      "id": "char_001",
      "name": "角色名",
      "traits": ["特征1", "特征2"],
      "voice": "声音特点描述"
    }}
  ],
  "causal_graph": {{
    "events": [{{ "id":"evt_001","description":"事件描述","scene_ids":[1],"chapter":1 }}],
    "edges": [
      {{ "from":"evt_001","to":"evt_002","type":"causal|temporal|emotional","strength":0.9,
         "description":"具体的因果关系描述，如'直接导致''触发''情感转折引发'等" }}
    ]
  }},
  "relation_network": {{
    "matrix": [
      {{ "from":"角色A","to":"角色B","intimacy":0.6,"power_gap":-0.3,"trust":0.7,
         "relation_type":"关系类型标签（如暧昧/仇人/闺蜜/对手）",
         "description":"关系详细描述（可选）",
         "history":[] }},
      {{ "from":"角色B","to":"角色A","intimacy":0.6,"power_gap":0.3,"trust":0.65,
         "relation_type":"关系类型标签（双向关系的另一方向）",
         "history":[] }}
    ]
  }},
  "scenes": [
    {{
      "id": 1,
      "location": "大学图书馆 · 靠窗角落",
      "time": "午后 · 阳光斜射",
      "emotion_intensity": 0.3,
      "media_hints": {{ "camera": "全景→缓慢推进至中景", "music": "轻钢琴曲渐入" }},
      "beats": [
        {{
          "type": "action",
          "content": "阳光透过落地窗洒在木质书桌上。林曦正埋头在一堆参考书中，时不时推一下滑落的眼镜。",
          "alternatives": [
            {{ "content": "一道阴影落在书页上。林曦抬起头，撞进一双平静如水的眼睛里。", "tone": "文艺" }}
          ],
          "selected": 0,
          "speaker": null,
          "emotion": null
        }},
        {{
          "type": "dialogue",
          "content": "这里有人吗？",
          "alternatives": [
            {{ "content": "请问，这个位置……", "tone": "礼貌" }}
          ],
          "selected": 0,
          "speaker": "char_002",
          "emotion": "平淡中带着一丝期待"
        }}
      ]
    }}
  ]
}}

## 质量规则（必须遵守）

1. 提取主要角色，每个角色必须有唯一的 id（格式 char_NNN）
2. 将故事拆分为场景（scenes），每个场景有唯一数字 id
3. 场景中包含节拍（beats），节拍类型：action（动作描述）、dialogue（对话）、monologue（独白）、parenthetical（括号说明）
4. 节拍的 alternatives 中，每条备选的 content 字段是实际内容文本
5. **禁止使用任何占位符或泛化表述**：
   - ❌ "对话内容"、"节拍内容文本"、"地点描述"、"时间描述（可选）"
   - ✅ "你什么时候回来的？怎么不说一声。"、"咖啡馆·靠窗座位"、"傍晚6:20"
6. **地点必须具体可感**：写出实际空间特征和环境氛围
7. **对话必须是角色会说的话**：符合人物性格、推动情节发展、包含潜台词
8. **动作描写要有画面感**：包含感官细节，能直接指导拍摄

{}

## 小说文本（前8000字）

{}"#,
        style_desc,
        style_desc,
        blind_mode_note,
        safe_truncate(text, 8000)
    );

    let response = call_deepseek(&prompt, api_key, base_url, None, true, "deepseek-chat").await?;

    tracing::info!(response_len = response.len(), "LLM 原始响应长度");

    // 尝试提取 JSON（处理可能的 markdown 代码块包裹）
    let json_str = extract_json_from_response(&response);

    // 尝试解析为 ScriptYaml 并序列化为 YAML
    if let Ok(script) = serde_json::from_str::<ScriptYaml>(&json_str) {
        // 补充元数据
        let mut script = script;
        script.metadata.adaptation_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        script.metadata.emotional_curve = Some(emotional_curve);
        let yaml_output = serde_yaml::to_string(&script)
            .map_err(|e| format!("序列化剧本 YAML 失败: {}", e))?;
        tracing::info!(yaml_len = yaml_output.len(), scene_count = script.scenes.len(), "成功生成标准 YAML");
        return Ok(yaml_output);
    }

    // 尝试智能转换非标准格式 → ScriptYaml
    match convert_to_script_yaml(&json_str, emotional_curve) {
        Ok(yaml_output) => {
            tracing::info!("通过智能转换成功生成 YAML");
            return Ok(yaml_output);
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                raw_preview = safe_truncate(&json_str, 500),
                "LLM 返回的 JSON 无法解析为标准格式，且智能转换也失败"
            );
            return Err(format!(
                "AI 生成的数据格式异常（{}）。请重试或检查小说内容是否足够丰富。",
                e
            ));
        }
    }
}

/// 从 LLM 响应中提取 JSON 字符串（去除可能的 markdown 代码块包裹）
fn extract_json_from_response(response: &str) -> String {
    let trimmed = response.trim();

    // 如果被 ```json ... ``` 包裹，提取内部内容
    if trimmed.starts_with("```") {
        if let Some(start) = trimmed.find('\n') {
            if let Some(end) = trimmed[start + 1..].find("```") {
                return trimmed[start + 1..start + 1 + end].trim().to_string();
            }
        }
    }

    // 如果整个响应就是 JSON（以 { 开头），直接返回
    if trimmed.starts_with('{') {
        return trimmed.to_string();
    }

    // 尝试在响应中找第一个 { 到最后一个 }
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            return trimmed[start..=end].to_string();
        }
    }

    response.to_string()
}

/// 智能转换：将 LLM 非标准 JSON 输出转换为 ScriptYaml 格式
fn convert_to_script_yaml(json_str: &str, emotional_curve: EmotionalCurve) -> Result<String, String> {
    // 先尝试作为通用 Value 解析
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON 解析失败: {}", e))?;

    tracing::info!(keys = ?value.as_object().map(|o| o.keys().collect::<Vec<_>>()), "尝试智能转换 LLM 输出");

    // 提取标题
    let title = value.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("未命名剧本")
        .to_string();

    // 提取 characters
    let characters = value.get("characters")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().enumerate().map(|(i, c)| Character {
                id: c.get("id").and_then(|v| v.as_str()).unwrap_or(&format!("char_{:03}", i + 1)).to_string(),
                name: c.get("name").and_then(|v| v.as_str()).unwrap_or("未知角色").to_string(),
                traits: c.get("traits").and_then(|v| v.as_array())
                    .map(|t| t.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
                voice: c.get("voice").and_then(|v| v.as_str()).map(String::from),
                arc_summary: c.get("arc_summary").and_then(|v| v.as_str()).map(String::from),
                motivation: c.get("motivation").and_then(|v| v.as_str()).map(String::from),
                relationships: vec![],
                first_scene_id: None,
            }).collect()
        })
        .unwrap_or_default();

    // 提取 scenes
    let scenes_raw = value.get("scenes").and_then(|v| v.as_array());
    let scenes = match scenes_raw {
        Some(arr) => {
            arr.iter().enumerate().map(|(idx, s)| {
                // location 可能叫 location 也可能叫 name/setting
                let location = s.get("location")
                    .or_else(|| s.get("name"))
                    .or_else(|| s.get("setting"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                // time 可能叫 time 或从 setting 推断
                let time = s.get("time").and_then(|v| v.as_str()).map(String::from);

                // emotion_intensity 默认值
                let emotion_intensity = s.get("emotion_intensity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);

                // beats
                let beats = s.get("beats").and_then(|v| v.as_array())
                    .map(|b_arr| {
                        b_arr.iter().map(|b| {
                            let beat_type = b.get("type")
                                .or_else(|| b.get("beat_type"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("action")
                                .to_string();
                            let content = b.get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            // alternatives: 统一提取 content 字段
                            let alternatives = b.get("alternatives").and_then(|v| v.as_array())
                                .map(|a_arr| {
                                    a_arr.iter().map(|a| {
                                        // 优先取 content，其次按 beat_type 取对应字段
                                        let alt_content = a.get("content")
                                            .or_else(|| a.get(beat_type.as_str()))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let tone = a.get("tone").and_then(|v| v.as_str()).map(String::from);
                                        BeatAlternative { content: alt_content, tone, name: None, emotion: None }
                                    }).collect()
                                })
                                .unwrap_or_default();

                            let speaker = b.get("speaker").and_then(|v| v.as_str()).map(String::from);
                            let emotion = b.get("emotion").and_then(|v| v.as_str()).map(String::from);

                            Beat {
                                beat_type,
                                content,
                                alternatives,
                                selected: 0,
                                speaker,
                                emotion,
                                participants: vec![],
                                stage_direction: None,
                                plot_line_tags: vec![],
                            }
                        }).collect()
                    })
                    .unwrap_or_default();

                // media_hints
                let media_hints = s.get("media_hints").and_then(|mh| {
                    let camera = mh.get("camera")?.as_str()?;
                    let music = mh.get("music")?.as_str()?;
                    Some(MediaHints { camera: camera.to_string(), music: music.to_string() })
                });

                Scene {
                    id: (idx + 1) as i32,
                    location,
                    time,
                    emotion_intensity,
                    beats,
                    media_hints,
                    chapter_id: None,
                    characters_present: vec![],
                    plot_lines: vec![],
                }
            }).collect()
        }
        None => vec![],
    };

    let script = ScriptYaml {
        metadata: ScriptMetadata {
            title,
            source_novel: "用户上传的小说".to_string(),
            adaptation_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            style: "short_drama".to_string(),
            emotional_curve: Some(emotional_curve),
        },
        characters,
        causal_graph: None,
        relation_network: None,
        scenes,
        chapters: vec![],
    };

    serde_yaml::to_string(&script).map_err(|e| format!("YAML 序列化失败: {}", e))
}

/// 模拟模式：生成示例剧本 YAML
fn generate_mock_script(_text: &str, config: &GenConfig) -> String {
    let style = config.style.clone();

    // 根据配置决定是否包含备选项
    let alternatives = if config.blind_acting_mode {
        vec![
            BeatAlternative {
                content: "备选版本：午后的图书馆安静得能听见翻书声。林曦坐在靠窗的位置，面前摊开着三本厚重的专业书。".to_string(),
                tone: Some("静谧".to_string()),
                name: None,
                emotion: None,
            },
            BeatAlternative {
                content: "备选版本：阳光斜斜地照进来，在地板上画出金色的格子。林曦揉了揉发酸的眼睛。".to_string(),
                tone: Some("慵懒".to_string()),
                name: None,
                emotion: None,
            },
        ]
    } else {
        vec![]
    };

    let media_hints = if config.include_media_hints {
        Some(MediaHints {
            camera: "中景 → 特写（笔记本封面）".to_string(),
            music: "轻钢琴曲，渐入".to_string(),
        })
    } else {
        None
    };

    let script = ScriptYaml {
        metadata: ScriptMetadata {
            title: "《遇见》".to_string(),
            source_novel: "用户上传的小说".to_string(),
            adaptation_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            style,
            emotional_curve: Some(EmotionalCurve {
                chapters: vec![1, 2, 3],
                intensities: vec![0.2, 0.7, 0.9],
            }),
        },
        characters: vec![
            Character {
                id: "char_001".to_string(),
                name: "林曦".to_string(),
                traits: vec!["敏感".to_string(), "执着".to_string(), "善良".to_string()],
                voice: Some("轻柔，慢速".to_string()),
                arc_summary: None,
                motivation: None,
                relationships: vec![],
                first_scene_id: None,
            },
            Character {
                id: "char_002".to_string(),
                name: "顾言".to_string(),
                traits: vec!["冷静".to_string(), "理性".to_string(), "内敛".to_string()],
                voice: Some("简洁，低沉".to_string()),
                arc_summary: None,
                motivation: None,
                relationships: vec![],
                first_scene_id: None,
            },
            Character {
                id: "char_003".to_string(),
                name: "苏晴".to_string(),
                traits: vec!["活泼".to_string(), "直率".to_string(), "热心".to_string()],
                voice: Some("语速快，音调高".to_string()),
                arc_summary: None,
                motivation: None,
                relationships: vec![],
                first_scene_id: None,
            },
        ],
        causal_graph: if config.include_causal_graph {
            Some(CausalGraphResponse {
                events: vec![
                    CausalEvent {
                        id: "evt_001".to_string(),
                        description: "林曦在图书馆偶遇顾言".to_string(),
                        scene_ids: vec![1],
                        chapter: 1,
                    },
                    CausalEvent {
                        id: "evt_002".to_string(),
                        description: "顾言借给林曦一本笔记".to_string(),
                        scene_ids: vec![1],
                        chapter: 1,
                    },
                    CausalEvent {
                        id: "evt_003".to_string(),
                        description: "林曦发现笔记中的秘密留言".to_string(),
                        scene_ids: vec![2],
                        chapter: 2,
                    },
                ],
                edges: vec![
                    CausalEdge {
                        from: "evt_001".to_string(),
                        to: "evt_002".to_string(),
                        edge_type: "causal".to_string(),
                        strength: 0.9,
                        description: Some("直接导致".to_string()),
                    },
                    CausalEdge {
                        from: "evt_002".to_string(),
                        to: "evt_003".to_string(),
                        edge_type: "temporal".to_string(),
                        strength: 1.0,
                        description: Some("时间推进后触发".to_string()),
                    },
                ],
            })
        } else {
            None
        },
        relation_network: if config.include_relation_network {
            Some(RelationNetworkResponse {
                matrix: vec![
                    RelationEntry {
                        from: "林曦".to_string(),
                        to: "顾言".to_string(),
                        intimacy: 0.6,
                        power_gap: -0.3,
                        trust: 0.7,
                        relation_type: Some("暧昧".to_string()),
                        description: None,
                        history: vec![],
                    },
                    RelationEntry {
                        from: "顾言".to_string(),
                        to: "林曦".to_string(),
                        intimacy: 0.6,
                        power_gap: 0.3,
                        trust: 0.65,
                        relation_type: Some("暧昧".to_string()),
                        description: None,
                        history: vec![],
                    },
                    RelationEntry {
                        from: "林曦".to_string(),
                        to: "苏晴".to_string(),
                        intimacy: 0.8,
                        power_gap: 0.1,
                        trust: 0.9,
                        relation_type: Some("闺蜜".to_string()),
                        description: None,
                        history: vec![],
                    },
                ],
            })
        } else {
            None
        },
        scenes: vec![
            Scene {
                id: 1,
                location: "大学图书馆 · 靠窗角落".to_string(),
                time: Some("午后".to_string()),
                emotion_intensity: 0.3,
                beats: vec![
                    Beat {
                        beat_type: "action".to_string(),
                        content: "阳光透过落地窗洒在木质书桌上。林曦正埋头在一堆参考书中，时不时推一下滑落的眼镜。".to_string(),
                        alternatives: alternatives.clone(),
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "action".to_string(),
                        content: "一个身影停在林曦对面。林曦抬头——是顾言，那个总是一个人坐在最后一排的男生。".to_string(),
                        alternatives: vec![
                            BeatAlternative {
                                content: "一道阴影落在书页上。林曦抬起头，撞进一双平静如水的眼睛里。".to_string(),
                                tone: Some("文艺".to_string()),
                                name: None,
                                emotion: None,
                            },
                        ],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "这里有人吗？".to_string(),
                        speaker: Some("顾言".to_string()),
                        emotion: Some("平淡".to_string()),
                        alternatives: vec![
                            BeatAlternative { content: "请问，这个位置……".to_string(), tone: Some("礼貌".to_string()), name: None, emotion: None },
                            BeatAlternative { content: "介意我坐这儿吗？".to_string(), tone: Some("随意".to_string()), name: None, emotion: None },
                        ],
                        selected: 0,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "啊，没有，你坐。".to_string(),
                        speaker: Some("林曦".to_string()),
                        emotion: Some("惊讶→镇定".to_string()),
                        alternatives: vec![
                            BeatAlternative { content: "没、没有，请坐……".to_string(), tone: Some("慌乱".to_string()), name: None, emotion: None },
                        ],
                        selected: 0,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "action".to_string(),
                        content: "顾言坐下，从背包里取出一本深蓝色封皮的笔记本，推到林曦面前。".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "你的。上次落在我桌上的。".to_string(),
                        speaker: Some("顾言".to_string()),
                        emotion: Some("平静".to_string()),
                        alternatives: vec![],
                        selected: 0,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "monologue".to_string(),
                        content: "林曦愣住了。她完全不记得自己丢过笔记本。接过笔记本的瞬间，她的指尖触到了封面上微微凸起的字迹。".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                ],
                media_hints: media_hints.clone(),
                chapter_id: None,
                characters_present: vec![],
                plot_lines: vec![],
            },
            Scene {
                id: 2,
                location: "林曦宿舍 · 书桌前".to_string(),
                time: Some("深夜".to_string()),
                emotion_intensity: 0.75,
                beats: vec![
                    Beat {
                        beat_type: "action".to_string(),
                        content: "台灯下，林曦翻开那本深蓝色的笔记本。第一页是空白的，但对着光看时，隐约可见铅笔写下的字迹。".to_string(),
                        alternatives: vec![
                            BeatAlternative {
                                content: "林曦关上宿舍门，迫不及待地打开笔记本。借着台灯的光，她发现每一页的角落都有细小的铅笔字。".to_string(),
                                tone: Some("悬疑".to_string()),
                                name: None,
                                emotion: None,
                            },
                        ],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "monologue".to_string(),
                        content: "\"如果你看到了这些字，说明我们之间有某种联系。——G\"".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "G……顾言？".to_string(),
                        speaker: Some("林曦".to_string()),
                        emotion: Some("震惊→困惑".to_string()),
                        alternatives: vec![
                            BeatAlternative { content: "这个 G 是……".to_string(), tone: Some("自言自语".to_string()), name: None, emotion: None },
                        ],
                        selected: 0,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "action".to_string(),
                        content: "林曦快速翻动笔记本。每一页的角落都有一句话，像一条隐秘的线索，串起一段不为人知的故事。".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                    Beat {
                        beat_type: "parenthetical".to_string(),
                        content: "（林曦的手微微颤抖）".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                        participants: vec![],
                        stage_direction: None,
                        plot_line_tags: vec![],
                    },
                ],
                media_hints: Some(MediaHints {
                    camera: "特写（笔记本字迹）→ 侧脸光影".to_string(),
                    music: "悬疑弦乐，低沉".to_string(),
                }),
                chapter_id: None,
                characters_present: vec![],
                plot_lines: vec![],
            },
        ],
        chapters: vec![],
    };

    serde_yaml::to_string(&script).unwrap_or_else(|_| "模拟剧本生成失败".to_string())
}
