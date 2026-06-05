use crate::models::*;
use crate::llm_client::call_deepseek;
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
            "请分析以下剧本，提取关键事件及其因果关系。\n\
            返回JSON格式：{{\"events\": [{{\"id\", \"description\", \"scene_ids\", \"chapter\"}}], \
            \"edges\": [{{\"from\", \"to\", \"type\", \"strength\"}}]}}\n\n\
            剧本内容（前5000字）：\n{}",
            &script_yaml[..script_yaml.len().min(5000)]
        );

        let response = call_deepseek(&prompt, api_key, base_url, None, true).await?;
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
            },
            CausalEdge {
                from: "evt_002".to_string(),
                to: "evt_003".to_string(),
                edge_type: "temporal".to_string(),
                strength: 1.0,
            },
            CausalEdge {
                from: "evt_003".to_string(),
                to: "evt_004".to_string(),
                edge_type: "emotional".to_string(),
                strength: 0.7,
            },
        ],
    };

    causal_response_to_data(&response)
}
