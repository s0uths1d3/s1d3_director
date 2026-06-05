use crate::models::EmotionalCurve;
use crate::llm_client::call_deepseek;

/// 简易情感词典（正面/负面关键词及权重）
const POSITIVE_WORDS: &[(&str, f64)] = &[
    ("开心", 0.8), ("快乐", 0.8), ("幸福", 0.9), ("温暖", 0.6), ("希望", 0.7),
    ("爱", 0.8), ("笑", 0.6), ("感动", 0.7), ("美好", 0.7), ("甜蜜", 0.75),
    ("兴奋", 0.7), ("激动", 0.65), ("惊喜", 0.7), ("治愈", 0.6),
];

const NEGATIVE_WORDS: &[(&str, f64)] = &[
    ("悲伤", -0.8), ("痛苦", -0.85), ("愤怒", -0.7), ("恐惧", -0.75), ("绝望", -0.9),
    ("哭", -0.6), ("泪", -0.55), ("孤独", -0.6), ("失落", -0.65), ("心碎", -0.85),
    ("冰冷", -0.5), ("黑暗", -0.5), ("死亡", -0.9), ("分离", -0.7), ("背叛", -0.8),
    ("颤抖", -0.4), ("窒息", -0.7),
];

/// 分析文本情感，返回情感曲线
pub async fn analyze_emotions(
    text: &str,
    api_key: &str,
    base_url: &str,
) -> Result<EmotionalCurve, String> {
    // 如果有 API Key，使用 LLM 分析
    if !api_key.is_empty() && api_key != "mock" {
        let prompt = format!(
            "请分析以下小说文本的情感节奏。按章节划分，返回每章的平均情绪强度（0-1之间的浮点数）。\n\
            返回JSON格式：{{\"chapters\": [1,2,3,...], \"intensities\": [0.x, 0.y, ...]}}\n\n\
            文本内容（前5000字）：\n{}",
            &text[..text.len().min(5000)]
        );

        let response = call_deepseek(&prompt, api_key, base_url, None, true).await?;

        // 尝试从 LLM 响应中解析 JSON
        if let Ok(curve) = serde_json::from_str::<EmotionalCurve>(&response) {
            return Ok(curve);
        }
    }

    // 回退到基于词典的规则分析
    Ok(rule_based_emotion_analysis(text))
}

/// 基于情感词典的规则分析（模拟模式 / LLM 失败时的回退）
fn rule_based_emotion_analysis(text: &str) -> EmotionalCurve {
    // 简单按段落/章节拆分
    let chapters: Vec<&str> = text
        .split(|c| c == '\n' || c == '\r')
        .filter(|s| !s.trim().is_empty())
        .collect();

    // 将文本大致分为 3 个章节（或更少）
    let num_chapters = (chapters.len() / 10).max(1).min(5);
    let chunk_size = (chapters.len() + num_chapters - 1) / num_chapters;

    let mut intensities = Vec::new();

    for i in 0..num_chapters {
        let start = i * chunk_size;
        let end = ((i + 1) * chunk_size).min(chapters.len());
        let chunk: String = chapters[start..end].join(" ");

        let intensity = calculate_chunk_intensity(&chunk);
        intensities.push(intensity);
    }

    // 归一化到 0-1 范围
    let max_val = intensities.iter().cloned().fold(0.0_f64, |a, b| a.max(b)).max(1.0);
    let min_val = intensities.iter().cloned().fold(f64::INFINITY, |a, b| a.min(b));
    let range = (max_val - min_val).max(0.01);

    let intensities: Vec<f64> = intensities
        .iter()
        .map(|&v| ((v - min_val) / range).clamp(0.05, 1.0))
        .collect();

    let chapter_ids: Vec<i32> = (1..=num_chapters as i32).collect();

    EmotionalCurve {
        chapters: chapter_ids,
        intensities,
    }
}

/// 计算单个文本块的情感强度
fn calculate_chunk_intensity(text: &str) -> f64 {
    let text_lower = text.to_lowercase();

    let mut positive_score = 0.0;
    let mut negative_score = 0.0;

    for &(word, weight) in POSITIVE_WORDS {
        let count = text_lower.matches(word).count() as f64;
        positive_score += count * weight;
    }

    for &(word, weight) in NEGATIVE_WORDS {
        let count = text_lower.matches(word).count() as f64;
        negative_score += count * weight.abs();
    }

    // 综合得分：正向 + 反向绝对值，表示情绪强度（不区分正负）
    let total_intensity = positive_score + negative_score;

    // 加上标点强度因子（感叹号、问号等增加强度）
    let punctuation_boost = text.matches('！').count() as f64 * 0.15
        + text.matches('？').count() as f64 * 0.1
        + text.matches("...").count() as f64 * 0.1;

    total_intensity + punctuation_boost
}
