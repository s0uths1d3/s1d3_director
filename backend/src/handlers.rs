use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::models::*;
use crate::utils::safe_truncate;
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
            // 异步保存剧本和项目到数据库（不阻塞响应）
            if let Some(ref db) = state.db {
                let db_clone = db.clone();
                let yaml_clone = yaml_content.clone();
                let style_clone = body.config.style.clone();
                let text_preview: String = body.text.chars().take(500).collect();

                tokio::spawn(async move {
                    // 保存剧本到 scripts 表
                    let title = format!("改编剧本 - {}", chrono::Local::now().format("%Y-%m-%d %H:%M"));
                    let script_result: Result<(sqlx::types::Uuid,), _> = sqlx::query_as(
                        "INSERT INTO scripts (title, source_novel, style, yaml_content) VALUES ($1, $2, $3, $4) RETURNING id"
                    )
                    .bind(&title)
                    .bind(&text_preview)
                    .bind(&style_clone)
                    .bind(&yaml_clone)
                    .fetch_one(&db_clone)
                    .await;

                    if let Ok((script_id,)) = script_result {
                        // 创建/更新关联的项目
                        let project_title = format!("{} - {}", style_to_label(&style_clone), chrono::Local::now().format("%m/%d %H:%M"));
                        sqlx::query(
                            "INSERT INTO projects (title, description, style, status, owner, novel_preview, script_id) VALUES ($1, $2, $3, 'completed', 'anonymous', $4, $5)"
                        )
                        .bind(&project_title)
                        .bind(format!("基于小说文本生成的{}风格剧本", style_to_label(&style_clone)))
                        .bind(&style_clone)
                        .bind(&text_preview)
                        .bind(script_id.to_string())
                        .execute(&db_clone)
                        .await.ok();
                        tracing::info!(script_id = %script_id, "剧本及项目已自动保存");
                    }
                });
            }

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

/// 风格代码转中文标签
fn style_to_label(style: &str) -> &'static str {
    match style {
        "film" => "电影",
        "stage" => "舞台剧",
        "anime" => "动漫",
        _ => "短剧",
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

/// AI 重生成（节拍/场景）
pub async fn regenerate_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<co_pilot::RegenerateRequest>,
) -> Response {
    let api_key = get_api_key(&state, &headers);
    let base_url = get_base_url(&state, &headers);

    tracing::info!(
        target = body.target,
        scene_id = body.scene_id,
        beat_index = ?body.beat_index,
        instruction_len = body.instruction.len(),
        "AI 重生成请求"
    );

    match co_pilot::regenerate(&body, &api_key, &base_url).await {
        Ok(response) => {
            tracing::info!(content_len = response.content.len(), "AI 重生成成功");
            Json(response).into_response()
        }
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
            safe_truncate(text, 3000)
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

// ==================== 剧本数据 API ====================

/// 根据 ID 获取剧本 YAML 内容
pub async fn get_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let uuid_id = match sqlx::types::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的剧本ID格式"),
    };

    let row: Option<(String,)> = sqlx::query_as("SELECT yaml_content FROM scripts WHERE id = $1")
        .bind(uuid_id)
        .fetch_optional(db)
        .await
        .unwrap_or(None);

    match row {
        Some((yaml_content,)) => {
            axum::http::Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/x-yaml; charset=utf-8")
                .body(yaml_content.into())
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to build script response: {}", e);
                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "构建响应失败")
                })
        }
        None => error_response(StatusCode::NOT_FOUND, "剧本不存在"),
    }
}

/// 更新剧本 YAML 内容
pub async fn update_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    body: String,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let uuid_id = match sqlx::types::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的剧本ID格式"),
    };

    let result = sqlx::query("UPDATE scripts SET yaml_content = $2, updated_at = NOW() WHERE id = $1")
        .bind(uuid_id)
        .bind(&body)
        .execute(db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            tracing::info!(script_id = %id, yaml_len = body.len(), "剧本已更新");
            Json(serde_json::json!({
                "success": true,
                "message": "保存成功",
                "script_id": id
            })).into_response()
        }
        Ok(_) => error_response(StatusCode::NOT_FOUND, "剧本不存在"),
        Err(e) => {
            tracing::error!(error = %e, script_id = %id, "更新剧本失败");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("保存失败: {}", e))
        }
    }
}

/// 创建新剧本记录（用于编辑器首次保存）
pub async fn create_script(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let title = body.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("未命名剧本")
        .to_string();
    let style = body.get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("short_drama")
        .to_string();
    let yaml_content = body.get("yaml_content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let row: Option<(sqlx::types::Uuid,)> = sqlx::query_as(
        "INSERT INTO scripts (title, style, yaml_content) VALUES ($1, $2, $3) RETURNING id"
    )
    .bind(&title)
    .bind(&style)
    .bind(&yaml_content)
    .fetch_optional(db)
    .await
    .unwrap_or(None);

    match row {
        Some((id,)) => {
            tracing::info!(script_id = %id, title = %title, yaml_len = yaml_content.len(), "新剧本已创建");
            Json(serde_json::json!({
                "id": id.to_string(),
                "title": title,
                "message": "创建成功"
            })).into_response()
        }
        None => error_response(StatusCode::INTERNAL_SERVER_ERROR, "创建剧本失败"),
    }
}

// ==================== 项目管理 API ====================

/// 获取项目列表（支持分页、搜索、筛选）
pub async fn list_projects(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return Json(ProjectListResponse { projects: vec![], total: 0, page: 1, page_size: 10 }).into_response(),
    };

    let page: i64 = params.get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    let page_size: i64 = params.get("page_size").and_then(|s| s.parse().ok()).unwrap_or(10).min(50);
    let search = params.get("search").cloned().unwrap_or_default();
    let status_filter = params.get("status").cloned();
    let offset = (page - 1) * page_size;

    // 构建查询
    let mut query_str = String::from(
        "SELECT id, title, description, style, status, owner, novel_preview, script_id, created_at, updated_at FROM projects WHERE 1=1"
    );
    let mut count_str = String::from("SELECT COUNT(*) FROM projects WHERE 1=1");

    if !search.is_empty() {
        query_str.push_str(&format!(" AND (title ILIKE '%{}%' OR description ILIKE '%{}%')", search.replace('\'', "''"), search.replace('\'', "''")));
        count_str.push_str(&format!(" AND (title ILIKE '%{}%' OR description ILIKE '%{}%')", search.replace('\'', "''"), search.replace('\'', "''")));
    }
    if let Some(ref s) = status_filter {
        query_str.push_str(&format!(" AND status = '{}'", s.replace('\'', "''")));
        count_str.push_str(&format!(" AND status = '{}'", s.replace('\'', "''")));
    }

    query_str.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", page_size, offset));

    let total: i64 = sqlx::query_as::<_, (i64,)>(&count_str)
        .fetch_one(db)
        .await
        .map(|r| r.0)
        .unwrap_or(0);

    let rows = sqlx::query_as::<_, (sqlx::types::Uuid, String, String, String, String, String, Option<String>, Option<sqlx::types::Uuid>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(&query_str)
        .fetch_all(db)
        .await
        .unwrap_or_default();

    let projects: Vec<Project> = rows.into_iter().map(|(id, title, description, style, status, owner, novel_preview, script_id, created_at, updated_at)| Project {
        id: id.to_string(), title, description, style, status, owner,
        novel_preview,
        script_id: script_id.map(|s| s.to_string()),
        created_at: created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        updated_at: updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    }).collect();

    Json(ProjectListResponse { projects, total, page, page_size }).into_response()
}

/// 创建新项目
pub async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let style = body.style.unwrap_or_else(|| "short_drama".to_string());
    let owner = body.owner.unwrap_or_else(|| "anonymous".to_string());

    let row: Result<(sqlx::types::Uuid,), _> = sqlx::query_as(
        "INSERT INTO projects (title, description, style, owner) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&body.title)
    .bind(&body.description)
    .bind(&style)
    .bind(&owner)
    .fetch_one(db)
    .await;

    match row {
        Ok((id,)) => {
            tracing::info!(project_id = %id, title = %body.title, "新项目已创建");
            // 返回新创建的项目
            get_project_by_id(db, &id.to_string()).await
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("创建项目失败: {}", e)),
    }
}

/// 获取单个项目详情
pub async fn get_project(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };
    get_project_by_id(db, &id).await
}

/// 更新项目信息
pub async fn update_project(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let uuid_id = match sqlx::types::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的项目ID格式"),
    };

    // 动态构建 UPDATE 语句
    let mut sets = Vec::new();
    let mut bind_idx = 2;
    if body.title.is_some() { sets.push(format!("title = ${}", bind_idx)); bind_idx += 1; }
    if body.description.is_some() { sets.push(format!("description = ${}", bind_idx)); bind_idx += 1; }
    if body.status.is_some() { sets.push(format!("status = ${}", bind_idx)); bind_idx += 1; }
    if body.style.is_some() { sets.push(format!("style = ${}", bind_idx)); bind_idx += 1; }
    if body.owner.is_some() { sets.push(format!("owner = ${}", bind_idx)); bind_idx += 1; }
    // script_id 需要转为 UUID 或 NULL
    if body.script_id.is_some() { sets.push(format!("script_id = ${}::uuid", bind_idx)); bind_idx += 1; }

    if sets.is_empty() {
        return get_project_by_id(db, &id).await;
    }

    sets.push("updated_at = NOW()".to_string());
    let set_clause = sets.join(", ");

    // 构建动态查询
    let query_str = format!("UPDATE projects SET {} WHERE id = $1", set_clause);
    let mut query = sqlx::query(&query_str).bind(uuid_id);

    if let Some(ref v) = body.title { query = query.bind(v); }
    if let Some(ref v) = body.description { query = query.bind(v); }
    if let Some(ref v) = body.status { query = query.bind(v); }
    if let Some(ref v) = body.style { query = query.bind(v); }
    if let Some(ref v) = body.owner { query = query.bind(v); }
    if let Some(ref v) = body.script_id {
        if v.is_empty() {
            query = query.bind::<Option<String>>(None);
        } else {
            query = query.bind(v.as_str());
        }
    }

    let result = query.execute(db).await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            tracing::info!(project_id = %id, "项目已更新");
            get_project_by_id(db, &id).await
        }
        _ => error_response(StatusCode::NOT_FOUND, "项目不存在"),
    }
}

/// 删除项目
pub async fn delete_project(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let uuid_id = match sqlx::types::Uuid::parse_str(&id) {
        Ok(u) => u,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的项目ID格式"),
    };

    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(uuid_id)
        .execute(db)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => {
            tracing::info!(project_id = %id, "项目已删除");
            Json(json!({"success": true, "message": "项目已删除"})).into_response()
        }
        _ => error_response(StatusCode::NOT_FOUND, "项目不存在"),
    }
}

/// 根据ID获取单个项目的内部辅助函数
async fn get_project_by_id(db: &sqlx::PgPool, id: &str) -> Response {
    let row: Option<(sqlx::types::Uuid, String, String, String, String, String, Option<String>, Option<sqlx::types::Uuid>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT id, title, description, style, status, owner, novel_preview, script_id, created_at, updated_at FROM projects WHERE id = $1"
        )
        .bind(sqlx::types::Uuid::parse_str(id).unwrap_or_else(|_| sqlx::types::Uuid::nil()))
        .fetch_optional(db)
        .await
        .unwrap_or(None);

    match row {
        Some((id, title, description, style, status, owner, novel_preview, script_id, created_at, updated_at)) => {
            Json(Project {
                id: id.to_string(), title, description, style, status, owner,
                novel_preview,
                script_id: script_id.map(|s| s.to_string()),
                created_at: created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                updated_at: updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            }).into_response()
        }
        None => error_response(StatusCode::NOT_FOUND, "项目不存在"),
    }
}

// ==================== 剧本快照（自动备份） ====================

/// 创建剧本快照（自动备份）
pub async fn create_snapshot(
    State(state): State<AppState>,
    axum::extract::Path(script_id): axum::extract::Path<String>,
    body: String,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let uuid_id = match sqlx::types::Uuid::parse_str(&script_id) {
        Ok(u) => u,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的剧本ID格式"),
    };

    // 限制每个剧本最多保留 20 个快照，超出则删除最旧的
    let count_result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM script_snapshots WHERE script_id = $1"
    )
    .bind(uuid_id)
    .fetch_one(db)
    .await;

    if let Ok(count) = count_result {
        if count >= 20 {
            sqlx::query(
                "DELETE FROM script_snapshots WHERE script_id = $1 AND id IN (SELECT id FROM script_snapshots WHERE script_id = $1 ORDER BY created_at ASC LIMIT 1)"
            )
            .bind(uuid_id)
            .execute(db)
            .await
            .ok();
        }
    }

    let result = sqlx::query(
        "INSERT INTO script_snapshots (script_id, yaml_content, label) VALUES ($1, $2, $3)"
    )
    .bind(uuid_id)
    .bind(&body)
    .bind("手动保存") // 默认标签
    .execute(db)
    .await;

    match result {
        Ok(_) => {
            tracing::info!(script_id = %script_id, yaml_len = body.len(), "已创建剧本快照");
            Json(serde_json::json!({
                "success": true,
                "message": "快照创建成功",
            })).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "创建快照失败");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("快照创建失败: {}", e))
        }
    }
}

/// 获取剧本的快照列表
pub async fn list_snapshots(
    State(state): State<AppState>,
    axum::extract::Path(script_id): axum::extract::Path<String>,
) -> Response {
    let db = match &state.db {
        Some(pool) => pool,
        None => return error_response(StatusCode::SERVICE_UNAVAILABLE, "数据库不可用"),
    };

    let uuid_id = match sqlx::types::Uuid::parse_str(&script_id) {
        Ok(u) => u,
        Err(_) => return error_response(StatusCode::BAD_REQUEST, "无效的剧本ID格式"),
    };

    let rows: Vec<(sqlx::types::Uuid, String, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as("SELECT id, label, created_at FROM script_snapshots WHERE script_id = $1 ORDER BY created_at DESC LIMIT 20")
            .bind(uuid_id)
            .fetch_all(db)
            .await
            .unwrap_or_default();

    let snapshots: Vec<serde_json::Value> = rows.into_iter().map(|(id, label, created_at)| {
        serde_json::json!({
            "id": id.to_string(),
            "label": label,
            "created_at": created_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })
    }).collect();

    Json(serde_json::json!({ "snapshots": snapshots, "total": snapshots.len() })).into_response()
}
