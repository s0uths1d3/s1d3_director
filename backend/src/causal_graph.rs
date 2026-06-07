use crate::models::*;
use crate::llm_client::call_deepseek;
use crate::utils::safe_truncate;
use petgraph::graph::{DiGraph, NodeIndex};

/// 从剧本 YAML 中构建因果图谱
pub async fn build_causal_graph(
    script_yaml: &str,
    api_key: &str,
    base_url: &str,
) -> Result<CausalGraphData, String> {
    // 尝试从已有 YAML 中解析因果图谱
    if let Ok(script) = serde_yaml::from_str::<ScriptYaml>(script_yaml) {
        if let Some(cg) = script.causal_graph {
            return Ok(causal_response_to_data(&cg));
        }
    }

    // 如果有 API Key，使用 LLM 提取
    if !api_key.is_empty() && api_key != "mock" {
        let prompt = format!(
            "你是一名专业编剧分析师。请分析以下剧本，提取关键事件及其因果关系。\n\n\
            ## 核心原则：基于实际剧本内容分析\n\
            你必须仔细阅读下方提供的完整剧本内容，从中提取真实发生的事件和它们之间的因果关系。\n\
            - 事件（events）必须是剧本中**实际发生的关键情节节点**，而非泛化的概念\n\
            - 因果边（edges）必须反映**剧本中明确呈现的逻辑关系**，不可凭空推断不存在的关联\n\
            - 如果两个事件在剧本中没有直接的因果联系，就不要强行建立边\n\n\
            ## 输出要求\n\n\
            返回严格的 JSON 格式，包含 events（事件节点）和 edges（因果连线）两个数组：\n\n\
            {{\n\
              \"events\": [{{\"id\":\"事件ID\",\"description\":\"事件描述文本\",\"scene_ids\":[场景编号],\"chapter\":章节号}}],\n\
              \"edges\": [{{\"from\":\"源事件ID\",\"to\":\"目标事件ID\",\"type\":\"causal|temporal|emotional\",\"strength\":0.0~1.0,\"description\":\"具体因果关系描述\"}}]\n\
            }}\n\n\
            ### 事件提取指南\n\
            - 从每个场景（scene）中提取 1~2 个最具影响力的事件\n\
            - 事件的 description 应包含：谁 + 做了什么 + 结果如何（具体到人名和行动）\n\
            - 好的事件示例：\"林曦发现笔记本角落的铅笔字迹\" / \"顾言承认策划了整次相遇\"\n\
            - 差的事件示例：\"角色相遇\" / \"发生冲突\" / \"情感变化\"\n\
            - scene_ids 和 chapter 必须准确填写该事件实际发生的场景编号和章节号\n\n\
            ### 边（因果关系）提取指南\n\
            - 只建立**确实存在的**因果关系，宁可少建也不要错建\n\
            - 每条边的 description 必须用一句话说明具体的因果机制\n\
            - strength 应反映因果关系的确定性：必然发生=0.9+, 很可能=0.7-0.9, 有可能=0.4-0.7, 弱关联=0.1-0.4\n\
            - type 选择要准确：如果A直接导致B用causal，如果只是先后发生用temporal，如果是情绪驱动行为用emotional\n\n\
            ## 因果边（edges）的关键规则\n\n\
            1. **每条边必须包含 description 字段**：用简洁的中文描述具体的因果机制，例如：\n\
               - \"直接导致\" / \"触发\" / \"引发\" / \"促使\" / \"间接造成\" / \"时间推进后必然发生\"\n\
               - \"情感转折引发\" / \"信息暴露导致信任崩塌\" / \"误会加深使矛盾激化\"\n\
               - **禁止使用泛化标签**，如仅写「因果」或「关联」，必须写出具体原因\n\
            2. **type 字段取值**：\n\
               - causal: 直接因果（一个事件直接导致另一个事件）\n\
               - temporal: 时序关系（时间上先后发生，有逻辑承接但不一定是强因果）\n\
               - emotional: 情感驱动（情绪/心理状态变化导致的行为或事件）\n\
            3. **strength 字段**：0.1~1.0，表示因果强度/确定性。1.0=必然发生，0.5=较可能，0.1=弱相关\n\
            4. **description 是连线上显示的核心文本**，请确保它准确、具体、可读性强\n\n\
            ## 剧本内容（前15000字）：\n{}",
            safe_truncate(script_yaml, 15000)
        );

        let response = call_deepseek(
            &prompt,
            api_key,
            base_url,
            Some("你是专业的剧本分析师，擅长从文本中准确提取事件和因果关系。只输出基于事实的分析结果。"),
            true,
            "deepseek-chat",
        ).await?;
        if let Ok(cg) = serde_json::from_str::<CausalGraphResponse>(&response) {
            return Ok(causal_response_to_data(&cg));
        }
    }

    // 模拟模式：返回预设数据
    Ok(mock_causal_graph())
}

/// 将 API 响应格式转换为内部图数据结构
fn causal_response_to_data(response: &CausalGraphResponse) -> CausalGraphData {
    let mut graph: DiGraph<CausalEvent, CausalEdge> = DiGraph::new();
    let mut id_map = std::collections::HashMap::new();

    // 添加节点
    for event in &response.events {
        let idx = graph.add_node(event.clone());
        id_map.insert(event.id.clone(), idx);
    }

    // 添加边
    for edge in &response.edges {
        if let (Some(&from_idx), Some(&to_idx)) =
            (id_map.get(&edge.from), id_map.get(&edge.to))
        {
            graph.add_edge(from_idx, to_idx, edge.clone());
        }
    }

    CausalGraphData { graph, id_map }
}

/// 查找下游受影响事件
pub fn find_downstream_events(
    graph_data: &CausalGraphData,
    event_id: &str,
) -> Vec<String> {
    graph_data.find_downstream(event_id)
}

/// 模拟模式的因果图谱
fn mock_causal_graph() -> CausalGraphData {
    let response = CausalGraphResponse {
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
            CausalEvent {
                id: "evt_004".to_string(),
                description: "苏晴告诉林曦顾言的过去".to_string(),
                scene_ids: vec![3],
                chapter: 3,
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
            CausalEdge {
                from: "evt_003".to_string(),
                to: "evt_004".to_string(),
                edge_type: "emotional".to_string(),
                strength: 0.7,
                description: Some("情感转折引发".to_string()),
            },
        ],
    };

    causal_response_to_data(&response)
}
