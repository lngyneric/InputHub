use crate::config::Config;
use anyhow::Context;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::warn;
use uuid::Uuid;

/// Phase 2 MVP：用 ripgrep 搜索 Obsidian vault，返回 Markdown 列表。
pub async fn run(
    _task_id: Uuid,
    query: &str,
    cfg: &Config,
) -> anyhow::Result<(String, serde_json::Value)> {
    let vault = cfg
        .obsidian_vault_path
        .clone()
        .ok_or_else(|| anyhow::anyhow!("OBSIDIAN_VAULT_PATH 未设置"))?;

    if query.trim().is_empty() {
        return Ok((
            "## @obsidian\n\n请输入要搜索的关键词，例如：`@obsidian 供应链培训方案`。\n"
                .to_string(),
            serde_json::json!({ "agent": "obsidian", "empty_query": true }),
        ));
    }

    let (matches, truncated) = rg_search(&vault, query).await?;
    let md = render_markdown(query, &matches, truncated, &vault);

    Ok((
        md,
        serde_json::json!({
          "agent": "obsidian",
          "vault": vault.display().to_string(),
          "query": query,
          "truncated": truncated,
          "matches": matches,
        }),
    ))
}

#[derive(Debug, Clone, serde::Serialize)]
struct MatchLine {
    path: String,
    line: u32,
    text: String,
}

async fn rg_search(vault: &PathBuf, query: &str) -> anyhow::Result<(Vec<MatchLine>, bool)> {
    // 只搜 Markdown
    let output = Command::new("rg")
        .arg("--line-number")
        .arg("--no-heading")
        .arg("--color")
        .arg("never")
        .arg("--smart-case")
        .arg("-g")
        .arg("*.md")
        .arg(query)
        .arg(vault)
        .output()
        .await
        .context("运行 rg 失败（请先安装 ripgrep：rg）")?;

    if !output.status.success() && output.status.code() != Some(1) {
        // code=1 表示无匹配；其他非 0 为执行错误
        warn!(
          status=?output.status.code(),
          stderr=%String::from_utf8_lossy(&output.stderr),
          "rg non-zero"
        );
        return Err(anyhow::anyhow!(
            "rg 执行失败：{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut out = Vec::new();
    for line in stdout.lines() {
        // 格式：path:line:text（Windows 路径带 ':' 这里会更复杂；Phase 2 再兼容）
        let mut parts = line.splitn(3, ':');
        let path = parts.next().unwrap_or("").to_string();
        let line_no = parts.next().unwrap_or("0").parse::<u32>().unwrap_or(0);
        let text = parts.next().unwrap_or("").to_string();
        if !path.is_empty() {
            out.push(MatchLine {
                path,
                line: line_no,
                text,
            });
        }
        if out.len() >= 30 {
            return Ok((out, true));
        }
    }
    Ok((out, false))
}

fn render_markdown(query: &str, matches: &[MatchLine], truncated: bool, vault: &PathBuf) -> String {
    if matches.is_empty() {
        return format!(
            "## @obsidian 搜索结果\n\n关键词：`{}`\n\n未在 vault 中找到匹配。\n\nvault：`{}`\n",
            query,
            vault.display()
        );
    }

    let mut md = String::new();
    md.push_str("## @obsidian 搜索结果\n\n");
    md.push_str(&format!("关键词：`{}`\n\n", query));

    let mut last_path = "";
    for m in matches {
        if m.path != last_path {
            md.push_str(&format!("\n### {}\n", m.path));
            last_path = &m.path;
        }
        md.push_str(&format!("- L{}: {}\n", m.line, m.text.trim()));
    }

    if truncated {
        md.push_str("\n> 结果已截断（仅展示前 30 条匹配）。\n");
    }
    md
}
