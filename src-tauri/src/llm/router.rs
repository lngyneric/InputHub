use crate::config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Complexity {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "complex")]
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDecision {
    pub complexity: Complexity,
    pub reason: String,
    #[serde(default)]
    pub suggested_agent: Option<String>,
    #[serde(default)]
    pub need_clarify: bool,
    #[serde(default)]
    pub questions: Vec<String>,
}

impl IntentDecision {
    pub fn simple(reason: impl Into<String>) -> Self {
        Self {
            complexity: Complexity::Simple,
            reason: reason.into(),
            suggested_agent: None,
            need_clarify: false,
            questions: vec![],
        }
    }
}

/// Phase 3：意图识别入口。
///
/// - 优先：如果配置了豆包 key，则用 LLM 做判断（可扩展到 Claude）
/// - 兜底：简单规则（长度/关键词）
pub async fn decide(raw: &str, cfg: &Config) -> IntentDecision {
    if cfg.doubao_api_key.is_some() {
        if let Ok(d) = crate::llm::doubao::classify_intent(raw, cfg).await {
            return d;
        }
    }

    // 规则兜底：足够跑通 Phase 3 的 UI/流程
    let trimmed = raw.trim();
    let len = trimmed.chars().count();
    let complex_keywords = [
        "影响",
        "研究",
        "报告",
        "方案",
        "对比",
        "调研",
        "路线图",
        "可行性",
        "风险",
    ];
    let hit = complex_keywords.iter().any(|k| trimmed.contains(k));

    if len > 24 || hit {
        IntentDecision {
            complexity: Complexity::Complex,
            reason: "rule: long or contains research keyword".into(),
            suggested_agent: Some("deerflow".into()),
            need_clarify: true,
            questions: vec![
                "侧重：技术替代风险 / 市场机会 / 两者都要？".into(),
                "时间范围：近3年 / 5年 / 10年？".into(),
                "输出格式：详细报告 / 执行摘要？".into(),
            ],
        }
    } else {
        IntentDecision {
            complexity: Complexity::Simple,
            reason: "rule: short query".into(),
            suggested_agent: Some("obsidian".into()),
            need_clarify: false,
            questions: vec![],
        }
    }
}
