use crate::models::*;
use crate::llm_client::call_deepseek;
use crate::emotion_analyzer::analyze_emotions;

/// 生成完整剧本 YAML
pub async fn generate_script(
    text: &str,
    config: &GenConfig,
    api_key: &str,
    base_url: &str,
) -> Result<String, String> {
    // 如果有 API Key，尝试调用 LLM 生成；失败则自动降级为模拟模式
    if !api_key.is_empty() && api_key != "mock" {
        match generate_with_llm(text, config, api_key, base_url).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!(error = %e, "LLM 生成失败，自动降级为模拟模式");
                // 不返回错误，而是降级到模拟模式
            }
        }
    }

    // 模拟模式：返回预设的示例剧本
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
        "你是一名专业编剧。请将以下小说改编为{}。\n\
        要求：\n\
        1. 提取主要角色（含 traits 和 voice）\n\
        2. 将故事拆分为场景（scenes），每个场景包含节拍（beats）\n\
        3. 节拍类型包括：action（动作描述）、dialogue（对话）、monologue（独白）、parenthetical（括号说明）\n\
        {}\n\
        {}\n\
        返回完整的 JSON 格式剧本数据（符合 ScriptYaml 结构）。\n\n\
        小说文本（前8000字）：\n{}",
        style_desc,
        blind_mode_note,
        media_note,
        &text[..text.len().min(8000)]
    );

    let response = call_deepseek(&prompt, api_key, base_url, None, true).await?;

    // 尝试解析为 ScriptYaml 并序列化为 YAML
    if let Ok(script) = serde_json::from_str::<ScriptYaml>(&response) {
        // 补充元数据
        let mut script = script;
        script.metadata.adaptation_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        script.metadata.emotional_curve = Some(emotional_curve);
        serde_yaml::to_string(&script)
            .map_err(|e| format!("序列化剧本 YAML 失败: {}", e))
    } else {
        // LLM 返回的不是有效 JSON，返回原始响应作为 YAML 内容
        Ok(response)
    }
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
            },
            BeatAlternative {
                content: "备选版本：阳光斜斜地照进来，在地板上画出金色的格子。林曦揉了揉发酸的眼睛。".to_string(),
                tone: Some("慵懒".to_string()),
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
            },
            Character {
                id: "char_002".to_string(),
                name: "顾言".to_string(),
                traits: vec!["冷静".to_string(), "理性".to_string(), "内敛".to_string()],
                voice: Some("简洁，低沉".to_string()),
            },
            Character {
                id: "char_003".to_string(),
                name: "苏晴".to_string(),
                traits: vec!["活泼".to_string(), "直率".to_string(), "热心".to_string()],
                voice: Some("语速快，音调高".to_string()),
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
                    },
                    CausalEdge {
                        from: "evt_002".to_string(),
                        to: "evt_003".to_string(),
                        edge_type: "temporal".to_string(),
                        strength: 1.0,
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
                        history: vec![],
                    },
                    RelationEntry {
                        from: "顾言".to_string(),
                        to: "林曦".to_string(),
                        intimacy: 0.6,
                        power_gap: 0.3,
                        trust: 0.65,
                        history: vec![],
                    },
                    RelationEntry {
                        from: "林曦".to_string(),
                        to: "苏晴".to_string(),
                        intimacy: 0.8,
                        power_gap: 0.1,
                        trust: 0.9,
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
                    },
                    Beat {
                        beat_type: "action".to_string(),
                        content: "一个身影停在林曦对面。林曦抬头——是顾言，那个总是一个人坐在最后一排的男生。".to_string(),
                        alternatives: vec![
                            BeatAlternative {
                                content: "一道阴影落在书页上。林曦抬起头，撞进一双平静如水的眼睛里。".to_string(),
                                tone: Some("文艺".to_string()),
                            },
                        ],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "这里有人吗？".to_string(),
                        speaker: Some("顾言".to_string()),
                        emotion: Some("平淡".to_string()),
                        alternatives: vec![
                            BeatAlternative { content: "请问，这个位置……".to_string(), tone: Some("礼貌".to_string()) },
                            BeatAlternative { content: "介意我坐这儿吗？".to_string(), tone: Some("随意".to_string()) },
                        ],
                        selected: 0,
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "啊，没有，你坐。".to_string(),
                        speaker: Some("林曦".to_string()),
                        emotion: Some("惊讶→镇定".to_string()),
                        alternatives: vec![
                            BeatAlternative { content: "没、没有，请坐……".to_string(), tone: Some("慌乱".to_string()) },
                        ],
                        selected: 0,
                    },
                    Beat {
                        beat_type: "action".to_string(),
                        content: "顾言坐下，从背包里取出一本深蓝色封皮的笔记本，推到林曦面前。".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "你的。上次落在我桌上的。".to_string(),
                        speaker: Some("顾言".to_string()),
                        emotion: Some("平静".to_string()),
                        alternatives: vec![],
                        selected: 0,
                    },
                    Beat {
                        beat_type: "monologue".to_string(),
                        content: "林曦愣住了。她完全不记得自己丢过笔记本。接过笔记本的瞬间，她的指尖触到了封面上微微凸起的字迹。".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                ],
                media_hints: media_hints.clone(),
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
                            },
                        ],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                    Beat {
                        beat_type: "monologue".to_string(),
                        content: "\"如果你看到了这些字，说明我们之间有某种联系。——G\"".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                    Beat {
                        beat_type: "dialogue".to_string(),
                        content: "G……顾言？".to_string(),
                        speaker: Some("林曦".to_string()),
                        emotion: Some("震惊→困惑".to_string()),
                        alternatives: vec![
                            BeatAlternative { content: "这个 G 是……".to_string(), tone: Some("自言自语".to_string()) },
                        ],
                        selected: 0,
                    },
                    Beat {
                        beat_type: "action".to_string(),
                        content: "林曦快速翻动笔记本。每一页的角落都有一句话，像一条隐秘的线索，串起一段不为人知的故事。".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                    Beat {
                        beat_type: "parenthetical".to_string(),
                        content: "（林曦的手微微颤抖）".to_string(),
                        alternatives: vec![],
                        selected: 0,
                        speaker: None,
                        emotion: None,
                    },
                ],
                media_hints: Some(MediaHints {
                    camera: "特写（笔记本字迹）→ 侧脸光影".to_string(),
                    music: "悬疑弦乐，低沉".to_string(),
                }),
            },
        ],
    };

    serde_yaml::to_string(&script).unwrap_or_else(|_| "模拟剧本生成失败".to_string())
}
