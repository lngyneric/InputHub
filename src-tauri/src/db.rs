use crate::types::{TaskRecord, TaskStatus};
use anyhow::Context;
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::warn;
use uuid::Uuid;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn connect(db_path: &Path) -> anyhow::Result<Self> {
        let mut path = PathBuf::from(db_path);

        if let Some(parent) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("create db dir: {}", parent.display()))
            {
                let fallback = std::env::temp_dir().join("agent-hub/agent_hub.sqlite");
                warn!(
                  error = %e,
                  db_path = %path.display(),
                  fallback = %fallback.display(),
                  "db dir not writable; falling back to temp dir"
                );
                path = fallback;
            }
        }

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("create db dir: {}", parent.display()))?;
        }

        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(opts).await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        // sqlite doesn't have a real enum; store status as TEXT
        sqlx::query(
            r#"
      CREATE TABLE IF NOT EXISTS tasks (
        id TEXT PRIMARY KEY,
        raw TEXT NOT NULL,
        status TEXT NOT NULL,
        source TEXT NOT NULL,
        app TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        error TEXT,
        result_markdown TEXT,
        result_meta_json TEXT
      );
      "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_task(
        &self,
        id: Uuid,
        raw: &str,
        source: &str,
        app: &str,
        status: TaskStatus,
    ) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc();
        let now_s = now.format(&Rfc3339)?;
        sqlx::query(
            r#"
      INSERT INTO tasks (id, raw, status, source, app, created_at, updated_at)
      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
      "#,
        )
        .bind(id.to_string())
        .bind(raw)
        .bind(status.as_str())
        .bind(source)
        .bind(app)
        .bind(&now_s)
        .bind(&now_s)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: TaskStatus,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
        sqlx::query(
            r#"
      UPDATE tasks
      SET status = ?2, updated_at = ?3, error = ?4
      WHERE id = ?1
      "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(&now)
        .bind(error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_result(
        &self,
        id: Uuid,
        markdown: &str,
        meta_json: &str,
    ) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc().format(&Rfc3339)?;
        sqlx::query(
            r#"
      UPDATE tasks
      SET result_markdown = ?2, result_meta_json = ?3, updated_at = ?4
      WHERE id = ?1
      "#,
        )
        .bind(id.to_string())
        .bind(markdown)
        .bind(meta_json)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_tasks(&self, limit: i64) -> anyhow::Result<Vec<TaskRecord>> {
        let rows = sqlx::query(
            r#"
      SELECT id, raw, status, source, app, created_at, updated_at, error
      FROM tasks
      ORDER BY created_at DESC
      LIMIT ?1
      "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let id: String = r.try_get("id")?;
            let raw: String = r.try_get("raw")?;
            let status: String = r.try_get("status")?;
            let source: String = r.try_get("source")?;
            let app: String = r.try_get("app")?;
            let created_at: String = r.try_get("created_at")?;
            let updated_at: String = r.try_get("updated_at")?;
            let error: Option<String> = r.try_get("error")?;

            out.push(TaskRecord {
                id: Uuid::parse_str(&id)?,
                raw,
                status: match status.as_str() {
                    "received" => TaskStatus::Received,
                    "running" => TaskStatus::Running,
                    "succeeded" => TaskStatus::Succeeded,
                    "failed" => TaskStatus::Failed,
                    _ => TaskStatus::Failed,
                },
                source,
                app,
                created_at: OffsetDateTime::parse(&created_at, &Rfc3339)?,
                updated_at: OffsetDateTime::parse(&updated_at, &Rfc3339)?,
                error,
            });
        }
        Ok(out)
    }
}
