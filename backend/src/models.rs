use serde::{Deserialize, Serialize};
use petgraph::graph::{DiGraph, NodeIndex};

// ==================== 通用响应结构 ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

// ==================== 分析请求/响应 ====================

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub text: String,
    #[serde(default)]
    pub config: Option<AnalyzeConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnalyzeConfig {
    #[serde(default)]
    pub include_causal_graph: bool,
    #[serde(default)]
    pub include_relation_network: bool,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub emotional_curve: EmotionalCurve,
    pub characters_draft: Vec<CharacterDraft>,
    pub causal_graph: CausalGraphResponse,
    pub relation_network: RelationNetworkResponse,
    pub special_techniques: Vec<SpecialTechnique>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmotionalCurve {
    pub chapters: Vec<i32>,
    pub intensities: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterDraft {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub voice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecialTechnique {
    #[serde(rename = "type")]
    pub technique_type: String,
    pub description: String,
    pub suggestion: String,
}

// ==================== 剧本生成请求 ====================

#[derive(Debug, Deserialize)]
pub struct GenerateScriptRequest {
    pub text: String,
    pub config: GenConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GenConfig {
    pub style: String,
    #[serde(default)]
    pub blind_acting_mode: bool,
    #[serde(default = "default_max_alternatives")]
    pub max_alternatives: usize,
    #[serde(default)]
    pub include_media_hints: bool,
    #[serde(default)]
    pub include_causal_graph: bool,
    #[serde(default)]
    pub include_relation_network: bool,
}

fn default_max_alternatives() -> usize { 2 }

// ==================== 因果图谱 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CausalGraphResponse {
    pub events: Vec<CausalEvent>,
    pub edges: Vec<CausalEdge>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CausalEvent {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub scene_ids: Vec<i32>,
    pub chapter: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CausalEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub strength: f64,
    /// 具体的因果关系描述（如"导致""触发""引发"等），用于图谱连线上显示
    #[serde(default)]
    pub description: Option<String>,
}

/// 内部因果图结构，使用 petgraph 存储
#[derive(Debug, Clone)]
pub struct CausalGraphData {
    pub graph: DiGraph<CausalEvent, CausalEdge>,
    pub id_map: std::collections::HashMap<String, NodeIndex>,
}

impl CausalGraphData {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            id_map: std::collections::HashMap::new(),
        }
    }

    /// 查找某事件的所有下游事件（DFS）
    pub fn find_downstream(&self, event_id: &str) -> Vec<String> {
        let mut downstream = Vec::new();
        if let Some(&idx) = self.id_map.get(event_id) {
            let mut visited = std::collections::HashSet::new();
            self.dfs_collect(idx, &mut visited, &mut downstream);
        }
        downstream
    }

    fn dfs_collect(&self, idx: NodeIndex, visited: &mut std::collections::HashSet<NodeIndex>, result: &mut Vec<String>) {
        if !visited.insert(idx) {
            return;
        }
        if let Some(event) = self.graph.node_weight(idx) {
            result.push(event.id.clone());
        }
        for neighbor in self.graph.neighbors(idx) {
            self.dfs_collect(neighbor, visited, result);
        }
    }
}

// ==================== 关系网络 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationNetworkResponse {
    pub matrix: Vec<RelationEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationEntry {
    pub from: String,
    pub to: String,
    pub intimacy: f64,
    pub power_gap: f64,
    pub trust: f64,
    /// 关系类型标签（如"爱情""仇人""师徒"等），用于关系网络连线上显示
    #[serde(default)]
    pub relation_type: Option<String>,
    /// 关系描述文本
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub history: Vec<RelationHistoryItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelationHistoryItem {
    pub scene_id: i32,
    pub delta_intimacy: f64,
    pub delta_power: f64,
    pub delta_trust: f64,
}

// ==================== 完整剧本 YAML 结构 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptYaml {
    pub metadata: ScriptMetadata,
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub causal_graph: Option<CausalGraphResponse>,
    #[serde(default)]
    pub relation_network: Option<RelationNetworkResponse>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptMetadata {
    pub title: String,
    pub source_novel: String,
    pub adaptation_date: String,
    pub style: String,
    #[serde(default)]
    pub emotional_curve: Option<EmotionalCurve>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default)]
    pub voice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scene {
    pub id: i32,
    pub location: String,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub emotion_intensity: f64,
    #[serde(default)]
    pub beats: Vec<Beat>,
    #[serde(default)]
    pub media_hints: Option<MediaHints>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Beat {
    #[serde(rename = "type")]
    pub beat_type: String,
    pub content: String,
    #[serde(default)]
    pub alternatives: Vec<BeatAlternative>,
    #[serde(default = "default_selected")]
    pub selected: usize,
    #[serde(default)]
    pub speaker: Option<String>,
    #[serde(default)]
    pub emotion: Option<String>,
}

fn default_selected() -> usize { 0 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaHints {
    pub camera: String,
    pub music: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BeatAlternative {
    pub content: String,
    /// 方案名称（用户自定义或系统自动生成）
    #[serde(default)]
    pub name: Option<String>,
    /// 情绪/语气标签
    #[serde(default)]
    pub tone: Option<String>,
    /// 方案关联的情绪值，选中时同步到 beat.emotion
    #[serde(default)]
    pub emotion: Option<String>,
}

// ==================== 项目管理 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub style: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub owner: String,
    pub novel_preview: Option<String>,
    pub script_id: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub style: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub style: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub script_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
