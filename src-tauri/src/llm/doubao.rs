use crate::{config::Config, llm::router::IntentDecision};
use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

/// 使用豆包（火山方舟 OpenAI-compatible）做意图识别。
///
/// 需要环境变量：
/// - DOUBAO_API_KEY
/// - DOUBAO_BASE_URL（默认 https://ark.cn-beijing.volces.com）
/// - DOUBAO_MODEL
pub async fn classify_intent(raw: &str, cfg: &Config) -> anyhow::Result<IntentDecision> {
    let key = cfg
        .doubao_api_key
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("missing DOUBAO_API_KEY"))?;

    let url = format!(
        "{}/api/v3/chat/completions",
        cfg.doubao_base_url.trim_end_matches('/')
    );
    let sys = r#"
你是一个任务路由器。你的职责：根据用户输入判断：
1) complexity: simple | complex
2) suggested_agent: 例如 "obsidian" / "deerflow" / null
3) need_clarify: true/false
4) questions: 需要澄清时给出 1-3 个问题（字符串数组）
只允许输出 JSON，禁止输出多余文本。
JSON schema:
{
  "complexity": "simple|complex",
  "reason": "string",
  "suggested_agent": "string|null",
  "need_clarify": true|false,
  "questions": ["..."]
}
"#;

    let user = format!(
        r#"用户输入：
{}
"#,
        raw
    );

    let req = ChatRequest {
        model: &cfg.doubao_model,
        messages: vec![
            Message {
                role: "system",
                content: sys,
            },
            Message {
                role: "user",
                content: &user,
            },
        ],
        temperature: 0.0,
    };

    let client = Client::new();
    let resp = client
        .post(url)
        .bearer_auth(key)
        .json(&req)
        .send()
        .await
        .context("doubao request failed")?;

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow::anyhow!("doubao http {}: {}", status, text));
    }

    let parsed: ChatResponse = serde_json::from_str(&text).context("parse doubao response")?;
    let content = parsed
        .choices
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("doubao empty choices"))?
        .message
        .content
        .trim()
        .to_string();

    // 理想情况：就是纯 JSON
    if let Ok(d) = serde_json::from_str::<IntentDecision>(&content) {
        return Ok(d);
    }

    // 次优：提取第一段 {...}
    if let (Some(l), Some(r)) = (content.find('{'), content.rfind('}')) {
        if l < r {
            let slice = &content[l..=r];
            if let Ok(d) = serde_json::from_str::<IntentDecision>(slice) {
                return Ok(d);
            }
        }
    }

    Err(anyhow::anyhow!("doubao returned non-json: {}", content))
}
