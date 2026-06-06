use crate::models::*;
use crate::llm_client::call_deepseek;
use crate::utils::safe_truncate;

/// 关系网络内部数据结构
#[derive(Debug, Clone)]
pub struct RelationNetworkData {
    /// 关系矩阵：key = (from_char, to_char), value = 关系指标
    pub relations: std::collections::HashMap<(String, String), RelationMetrics>,
}

/// 单个关系指标
#[derive(Debug, Clone)]
pub struct RelationMetrics {
    pub intimacy: f64,   // 亲密度 0..1
    pub power_gap: f64,  // 权力差
    pub trust: f64,      // 信任度 0..1
    /// 关系类型标签（如"爱情""仇人"等）
    pub relation_type: Option<String>,
    /// 关系描述文本
    pub description: Option<String>,
    pub history: Vec<RelationHistoryItem>,
}

impl RelationNetworkData {
    pub fn new() -> Self {
        Self {
            relations: std::collections::HashMap::new(),
        }
    }

    /// 获取两个角色之间的关系（不可变引用）
    pub fn get_relation(&self, from: &str, to: &str) -> Option<&RelationMetrics> {
        self.relations.get(&(from.to_string(), to.to_string()))
    }

    /// 获取两个角色之间的关系（可变引用）
    pub fn get_mut_relation(&mut self, from: &str, to: &str) -> Option<&mut RelationMetrics> {
        self.relations.get_mut(&(from.to_string(), to.to_string()))
    }

    /// 设置/更新关系
    pub fn set_relation(&mut self, from: String, to: String, metrics: RelationMetrics) {
        self.relations.insert((from, to), metrics);
    }

    /// 获取所有角色名称列表
    pub fn characters(&self) -> std::collections::HashSet<String> {
        let mut chars = std::collections::HashSet::new();
        for (f, t) in self.relations.keys() {
            chars.insert(f.clone());
            chars.insert(t.clone());
        }
        chars
    }
}

impl Default for RelationNetworkData {
    fn default() -> Self {
        Self::new()
    }
}

/// 从剧本 YAML 构建关系网络
pub async fn build_relation_network(
    script_yaml: &str,
    api_key: &str,
    base_url: &str,
) -> Result<RelationNetworkData, String> {
    // 尝试从已有 YAML 解析
    if let Ok(script) = serde_yaml::from_str::<ScriptYaml>(script_yaml) {
        if let Some(rn) = script.relation_network {
            return Ok(relation_response_to_data(&rn));
        }
    }

    // 如果有 API Key，使用 LLM 分析
    if !api_key.is_empty() && api_key != "mock" {
        let prompt = format!(
            "你是一名专业编剧分析师。请分析以下剧本中角色之间的社会关系网络。\n\n\
            ## 输出要求\n\n\
            返回严格的 JSON 格式：\n\n\
            {{\n\
              \"matrix\": [\n\
                {{\"from\":\"角色A\",\"to\":\"角色B\",\"intimacy\":0.0~1.0,\"power_gap\":-1.0~1.0,\"trust\":0.0~1.0,\n\
                 \"relation_type\":\"关系类型标签\",\"description\":\"关系描述（可选）\",\n\
                 \"history\":[{{\"scene_id\":场景号,\"delta_intimacy\":变化量,\"delta_power\":变化量,\"delta_trust\":变化量}}]}}\n\
              ]\n\
            }}\n\n\
            ## 关系矩阵的关键规则\n\n\
            ### 1. 双向关系原则（最重要）\n\
            - **每对角色关系必须生成两条记录**：A→B 和 B→A，分别描述各自视角的关系状态\n\
            - **对立/敌对关系必须双向体现**：如果 A 对立 B，则 B 也应对立 A（双方互为反对立场）\n\
              例如：「张三」和「李四」是仇人 → 两条记录的 relation_type 均应为「仇人」\n\
            - **亲密关系同样双向**：A 爱慕 B ≠ B 爱慕 A 时需在数值上体现差异\n\n\
            ### 2. relation_type 字段（必填）\n\
            用简短中文标注关系的本质类型，用于连线上显示。取值示例：\n\
            - 亲密类：「暧昧」/「恋人」/「闺蜜」/「知己」/「家人」/「师徒」\n\
            - 友好类：「朋友」/「同事」/「同学」/「盟友」/「仰慕者」\n\
            - 中性类：「熟人」/「邻居」/「路人」/「交易关系」\n\
            - 对立类：「仇人」/「对手」/「政敌」/「情敌」/「背叛者」\n\
            - 不对称类：「单恋」/「追随者」/「提防对象」/「利用关系」\n\n\
            ### 3. 数值字段含义\n\
            - intimacy（亲密度）：0=完全疏远/陌生，0.5=普通认识，0.8=亲密好友，1.0=至亲/灵魂伴侣\n\
            - power_gap（权力差距）：负值=from 弱于 to，正值=from 强于 to，0=平等。范围 -1.0 ~ 1.0\n\
            - trust（信任度）：0=完全不信任/怀疑，0.5=一般信任，0.9=高度信赖，<0.25=明显不信任/敌意\n\n\
            ### 4. 视觉区分规则\n\
            - **高亲密度 + 高信任**（intimacy>0.7, trust>0.7）→ 关系类型偏正向（亲密/友好），视觉用暖色线\n\
            - **低信任度**（trust<0.35）→ 关系类型偏向对立/紧张，视觉用冷色/红色线\n\
            - **极端对立**（trust<0.2, intimacy<0.3）→ relation_type 应明确标记为「仇人」「对手」等对抗性词汇\n\n\
            ## 剧本内容（前5000字）：\n{}",
            safe_truncate(script_yaml, 5000)
        );

        let response = call_deepseek(&prompt, api_key, base_url, None, true).await?;
        if let Ok(rn) = serde_json::from_str::<RelationNetworkResponse>(&response) {
            return Ok(relation_response_to_data(&rn));
        }
    }

    // 模拟模式
    Ok(mock_relation_network())
}

/// 根据场景变更增量更新关系网络
pub async fn update_with_scenes(
    scene_ids: &[i32],
    script_yaml: &str,
    api_key: &str,
    base_url: &str,
) -> Result<RelationNetworkResponse, String> {
    // 先获取当前关系网络
    let mut current = build_relation_network(script_yaml, api_key, base_url).await?;

    // 如果有 API Key，使用 LLM 增量更新
    if !api_key.is_empty() && api_key != "mock" {
        let prompt = format!(
            "以下剧本中的场景 {:?} 发生了变化。请分析这些变化对角色关系的影响，返回增量更新。\n\
            返回JSON格式：{{\"matrix\": [{{\"from\", \"to\", \"delta_intimacy\", \"delta_power\", \"delta_trust\"}}]}}\n\n\
            剧本内容：\n{}",
            scene_ids,
            safe_truncate(script_yaml, 5000)
        );

        if let Ok(response) = call_deepseek(&prompt, api_key, base_url, None, true).await {
            if let Ok(updates) = serde_json::from_str::<serde_json::Value>(&response) {
                if let Some(matrix) = updates.get("matrix").and_then(|m| m.as_array()) {
                    for item in matrix {
                        let from = item.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let to = item.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let di = item.get("delta_intimacy").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let dp = item.get("delta_power").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let dt = item.get("delta_trust").and_then(|v| v.as_f64()).unwrap_or(0.0);

                        if let Some(rel) = current.get_mut_relation(&from, &to) {
                            rel.intimacy = (rel.intimacy + di).clamp(0.0, 1.0);
                            rel.power_gap = (rel.power_gap + dp).clamp(-1.0, 1.0);
                            rel.trust = (rel.trust + dt).clamp(0.0, 1.0);
                            rel.history.push(RelationHistoryItem {
                                scene_id: scene_ids.first().copied().unwrap_or(0),
                                delta_intimacy: di,
                                delta_power: dp,
                                delta_trust: dt,
                            });
                        }
                    }
                }
            }
        }
    } else {
        // 模拟模式：对每个场景添加小的随机变化
        for (_key, rel) in current.relations.iter_mut() {
            let di = 0.05;
            let dp = 0.02;
            let dt = 0.03;
            rel.intimacy = (rel.intimacy + di).clamp(0.0, 1.0);
            rel.power_gap = (rel.power_gap + dp).clamp(-1.0, 1.0);
            rel.trust = (rel.trust + dt).clamp(0.0, 1.0);
            for &sid in scene_ids {
                rel.history.push(RelationHistoryItem {
                    scene_id: sid,
                    delta_intimacy: di,
                    delta_power: dp,
                    delta_trust: dt,
                });
            }
        }
    }

    // 转换回响应格式
    Ok(data_to_response(&current))
}

/// 将 API 响应转换为内部数据结构
fn relation_response_to_data(response: &RelationNetworkResponse) -> RelationNetworkData {
    let mut data = RelationNetworkData::new();
    for entry in &response.matrix {
        data.set_relation(
            entry.from.clone(),
            entry.to.clone(),
            RelationMetrics {
                intimacy: entry.intimacy,
                power_gap: entry.power_gap,
                trust: entry.trust,
                relation_type: entry.relation_type.clone(),
                description: entry.description.clone(),
                history: entry.history.clone(),
            },
        );
    }
    data
}

/// 将内部数据转换为响应格式（公开方法，供 handlers 调用）
pub fn data_to_response(data: &RelationNetworkData) -> RelationNetworkResponse {
    let matrix = data
        .relations
        .iter()
        .map(|((from, to), metrics)| RelationEntry {
            from: from.clone(),
            to: to.clone(),
            intimacy: metrics.intimacy,
            power_gap: metrics.power_gap,
            trust: metrics.trust,
            relation_type: metrics.relation_type.clone(),
            description: metrics.description.clone(),
            history: metrics.history.clone(),
        })
        .collect();

    RelationNetworkResponse { matrix }
}

/// 模拟模式的关系网络
fn mock_relation_network() -> RelationNetworkData {
    let mut data = RelationNetworkData::new();

    data.set_relation(
        "林曦".to_string(),
        "顾言".to_string(),
        RelationMetrics {
            intimacy: 0.6,
            power_gap: -0.3,
            trust: 0.7,
            relation_type: None,
            description: None,
            history: vec![RelationHistoryItem {
                scene_id: 1,
                delta_intimacy: 0.2,
                delta_power: 0.0,
                delta_trust: 0.3,
            }],
        },
    );

    data.set_relation(
        "顾言".to_string(),
        "林曦".to_string(),
        RelationMetrics {
            intimacy: 0.6,
            power_gap: 0.3,
            trust: 0.65,
            relation_type: None,
            description: None,
            history: vec![RelationHistoryItem {
                scene_id: 1,
                delta_intimacy: 0.2,
                delta_power: 0.0,
                delta_trust: 0.25,
            }],
        },
    );

    data.set_relation(
        "林曦".to_string(),
        "苏晴".to_string(),
        RelationMetrics {
            intimacy: 0.8,
            power_gap: 0.1,
            trust: 0.9,
            relation_type: None,
            description: None,
            history: vec![],
        },
    );

    data.set_relation(
        "苏晴".to_string(),
        "林曦".to_string(),
        RelationMetrics {
            intimacy: 0.8,
            power_gap: -0.1,
            trust: 0.88,
            relation_type: None,
            description: None,
            history: vec![],
        },
    );

    data.set_relation(
        "顾言".to_string(),
        "苏晴".to_string(),
        RelationMetrics {
            intimacy: 0.3,
            power_gap: 0.2,
            trust: 0.4,
            relation_type: None,
            description: None,
            history: vec![],
        },
    );

    data.set_relation(
        "苏晴".to_string(),
        "顾言".to_string(),
        RelationMetrics {
            intimacy: 0.3,
            power_gap: -0.2,
            trust: 0.35,
            relation_type: None,
            description: None,
            history: vec![],
        },
    );

    data
}
