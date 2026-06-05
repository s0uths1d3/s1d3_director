use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::models::*;
use crate::{
    emotion_analyzer, script_generator, causal_graph, relation_network, co_pilot,
};
use co_pilot::{CoPilotChatRequest, CoPilotSuggestResponse, CoPilotSuggestRequest};

// ==================== 应用状态（共享） ====================

#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 数据库连接池（None 表示无数据库模式）
    pub db: Option<sqlx::PgPool>,
    /// DeepSeek API Key（从 .env 读取，空则使用模拟模式）
    pub deepseek_api_key: String,
    /// DeepSeek Base URL（从 .env 读取）
    pub deepseek_base_url: String,
}

// ==================== CORS 中间件 ====================

pub async fn cors_middleware(request: Request, next: middleware::Next) -> Response {
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        "*".parse().unwrap(),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Content-Type, Authorization, X-API-Key, X-Base-URL".parse().unwrap(),
    );

    response
}

// ==================== 辅助函数：提取请求头中的 API 配置 =================/// 从请求头提取 API Key（已弃用，保留兼容性）
fn extract_api_key(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

/// 从请求头提取 Base URL（已弃用，保留兼容性）
fn extract_base_url(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("X-Base-URL")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

/// 从 State 获取 API Key（优先级：环境变量 > 请求头）
fn get_api_key(state: &AppState, headers: &axum::http::HeaderMap) -> String {
    if !state.deepseek_api_key.is_empty() {
        return state.deepseek_api_key.clone();
    }
    // 兼容旧方式：从请求头读取
    extract_api_key(headers)
}

/// 从 State 获取 Base URL（优先级：环境变量 > 请求头）
fn get_base_url(state: &AppState, headers: &axum::http::HeaderMap) -> String {
    if !state.deepseek_base_url.is_empty() {
        return state.deepseek_base_url.clone();
    }
    // 兼容旧方式
    let header_val = extract_base_url(headers);
    if !header_val.is_empty() {
        return header_val;
    }
    "https://api.deepseek.com/v1".to_string()
}

/// 统一错误响应
fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(json!({
            "error": message,
            "detail": None::<String>,
        })),
    )
        .into_response()
}

// ==================== API Handlers =================/// 健康检查
pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

/// 分析小说文本
pub async fn analyze_text(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AnalyzeRequest>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    // 1. 情感分析
    let emotional_curve = match emotion_analyzer::analyze_emotions(&body.text, &api_key, &base_url).await {
        Ok(curve) => curve,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };

    // 2. 角色抽取（使用 LLM 或模拟）
    let characters_draft = match extract_characters_mock_or_llm(&body.text, &api_key, &base_url).await {
        Ok(chars) => chars,
        Err(e) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };

    // 3. 因果图谱初稿
    let causal_graph = match body.config.as_ref().map(|c| c.include_causal_graph).unwrap_or(false) {
        true => {
            match causal_graph::build_causal_graph("", &api_key, &base_url).await {
                Ok(cg) => causal_graph_data_to_response(&cg),
                Err(_) => CausalGraphResponse { events: vec![], edges: vec![] },
            }
        }
        _ => CausalGraphResponse { events: vec![], edges: vec![] },
    };

    // 4. 关系网络初稿
    let relation_network = match body.config.as_ref().map(|c| c.include_relation_network).unwrap_or(false) {
        true => {
            match relation_network::build_relation_network("", &api_key, &base_url).await {
                Ok(rn) => relation_network::data_to_response(&rn),
                Err(_) => RelationNetworkResponse { matrix: vec![] },
            }
        }
        _ => RelationNetworkResponse { matrix: vec![] },
    };

    // 5. 反套路探测（特殊叙事手法）
    let special_techniques = detect_special_techniques(&body.text);

    Json(AnalyzeResponse {
        emotional_curve,
        characters_draft,
        causal_graph,
        relation_network,
        special_techniques,
    }).into_response()
}

/// 生成完整剧本
pub async fn generate_script_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<GenerateScriptRequest>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    tracing::info!(api_key_len = api_key.len(), text_len = body.text.len(), "generate_script_handler called");

    match script_generator::generate_script(&body.text, &body.config, &api_key, &base_url).await {
        Ok(yaml_content) => {
            // 使用 Response Builder 返回 YAML 文本
            axum::http::Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/x-yaml; charset=utf-8")
                .body(yaml_content.into())
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to build response: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "构建响应失败")
                })
        }
        Err(e) => {
            tracing::error!(error = %e, "generate_script failed");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("生成失败: {}", e))
        }
    }
}

/// 查询因果影响
pub async fn causal_impact(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    let script_yaml = body.get("script_yaml")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let event_id = body.get("event_id")
        .and_then(|v| v.as_str());

    if let Some(eid) = event_id {
        match causal_graph::build_causal_graph(script_yaml, &api_key, &base_url).await {
            Ok(graph_data) => {
                let affected = causal_graph::find_downstream_events(&graph_data, eid);
                Json(json!({ "affected_events": affected })).into_response()
            }
            Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
        }
    } else {
        // 如果没有 event_id，返回空结果
        Json(json!({ "affected_events": [] })).into_response()
    }
}

/// 更新关系网络
pub async fn update_relation(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    let script_yaml = body.get("script_yaml")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let scene_ids: Vec<i32> = body
        .get("changed_scene_ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect()
        })
        .unwrap_or_default();

    match relation_network::update_with_scenes(&scene_ids, script_yaml, &api_key, &base_url).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

/// 副编剧聊天
pub async fn copilot_chat_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CoPilotChatRequest>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    let context = body.context.unwrap_or_default();

    match co_pilot::chat(
        &body.messages,
        &context,
        body.full_script_yaml.as_deref(),
        &api_key,
        &base_url,
    )
    .await
    {
        Ok(response) => Json(response).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

/// 副编剧建议
pub async fn copilot_suggest_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CoPilotSuggestRequest>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    match co_pilot::suggest(&body.script_yaml, &api_key, &base_url).await {
        Ok(suggestions) => Json(CoPilotSuggestResponse { suggestions }).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

// ==================== 内部辅助函数 =================/// 提取角色（LLM 或模拟模式）
async fn extract_characters_mock_or_llm(
    text: &str,
    api_key: &str,
    base_url: &str,
) -> Result<Vec<CharacterDraft>, String> {
    if !api_key.is_empty() && api_key != "mock" {
        let prompt = format!(
            "请从以下小说中提取主要角色。\n\
            返回JSON数组：[{{\"id\", \"name\", \"traits\": [...], \"voice\": \"...\"}}]\n\n\
            小说文本（前3000字）：\n{}",
            &text[..text.len().min(3000)]
        );

        let response = crate::llm_client::call_deepseek(&prompt, api_key, base_url, None, true).await?;
        serde_json::from_str::<Vec<CharacterDraft>>(&response)
            .map_err(|e| format!("解析角色数据失败: {}", e))
    } else {
        // 模拟模式
        Ok(vec![
            CharacterDraft {
                id: "char_001".to_string(),
                name: "林曦".to_string(),
                traits: vec!["敏感".to_string(), "执着".to_string(), "善良".to_string()],
                voice: Some("轻柔，慢速".to_string()),
            },
            CharacterDraft {
                id: "char_002".to_string(),
                name: "顾言".to_string(),
                traits: vec!["冷静".to_string(), "理性".to_string(), "内敛".to_string()],
                voice: Some("简洁，低沉".to_string()),
            },
            CharacterDraft {
                id: "char_003".to_string(),
                name: "苏晴".to_string(),
                traits: vec!["活泼".to_string(), "直率".to_string(), "热心".to_string()],
                voice: Some("语速快，音调高".to_string()),
            },
        ])
    }
}

/// 探测特殊叙事手法（反套路探测器）
fn detect_special_techniques(text: &str) -> Vec<SpecialTechnique> {
    let mut techniques = Vec::new();
    let text_lower = text.to_lowercase();

    // 不可靠叙述者检测
    if text_lower.contains("我记得") || text_lower.contains("似乎") || text_lower.contains("也许是我记错了") {
        techniques.push(SpecialTechnique {
            technique_type: "不可靠叙述者".to_string(),
            description: "检测到可能的不可靠叙述者手法".to_string(),
            suggestion: "可考虑在影视化中使用主观镜头或闪回来呈现叙述者的记忆不确定性。".to_string(),
        });
    }

    // 时间跳跃检测
    if text_lower.contains("三年后") || text_lower.contains("那年冬天") || text_lower.contains("回到") {
        techniques.push(SpecialTechnique {
            technique_type: "时间跳跃/非线性叙事".to_string(),
            description: "检测到非线性时间结构".to_string(),
            suggestion: "建议使用转场特效或字幕标注时间变化，帮助观众理解时间线。".to_string(),
        });
    }

    // 意识流检测
    if text_lower.matches("......").count() > 2 || text_lower.matches("——").count() > 5 {
        techniques.push(SpecialTechnique {
            technique_type: "意识流/内心独白".to_string(),
            description: "检测到大量内心活动描写".to_string(),
            suggestion: "可通过画外音(V.O.)或视觉隐喻将内心活动外化。".to_string(),
        });
    }

    // 如果没有检测到特殊手法，提供一个默认提示
    if techniques.is_empty() {
        techniques.push(SpecialTechnique {
            technique_type: "常规线性叙事".to_string(),
            description: "未检测到特殊叙事手法，故事采用常规线性结构。".to_string(),
            suggestion: "可考虑在某些关键场景加入闪回或预叙来增加层次感。".to_string(),
        });
    }

    techniques
}

/// 将内部因果图数据转为响应格式
fn causal_graph_data_to_response(data: &CausalGraphData) -> CausalGraphResponse {
    let events: Vec<CausalEvent> = data
        .graph
        .node_weights()
        .cloned()
        .collect();

    let edges: Vec<CausalEdge> = data
        .graph
        .edge_weights()
        .cloned()
        .collect();

    CausalGraphResponse { events, edges }
}
