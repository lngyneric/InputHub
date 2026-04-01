use crate::{config::Config, db::Db, types::TaskStatus};
use tracing::info;
use uuid::Uuid;

pub mod deerflow;
pub mod obsidian;

pub async fn execute_task(task_id: Uuid, raw: String, db: Db, cfg: Config) -> anyhow::Result<()> {
    db.update_status(task_id, TaskStatus::Running, None).await?;

    // Simple routing based on prefix
    if raw.starts_with("@obsidian") {
        let query = raw.trim_start_matches("@obsidian").trim();
        match obsidian::run(task_id, query, &cfg).await {
            Ok((md, meta)) => {
                db.set_result(task_id, &md, &meta.to_string()).await?;
                db.update_status(task_id, TaskStatus::Succeeded, None)
                    .await?;
            }
            Err(e) => {
                db.update_status(task_id, TaskStatus::Failed, Some(&e.to_string()))
                    .await?;
            }
        }
    } else {
        // Fallback or other agents
        info!("No specific agent matched, using dummy logic");
        db.set_result(task_id, "Result markdown", "{}").await?;
        db.update_status(task_id, TaskStatus::Succeeded, None)
            .await?;
    }

    Ok(())
}
