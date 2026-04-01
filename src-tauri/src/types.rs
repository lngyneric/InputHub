use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LaunchRequest {
    pub raw: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub app: String,
}

#[derive(Debug, Serialize)]
pub struct LaunchResponse {
    pub task_id: Uuid,
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Received,
    Running,
    Succeeded,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskRecord {
    pub id: Uuid,
    pub raw: String,
    pub status: TaskStatus,
    pub source: String,
    pub app: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: time::OffsetDateTime,
    pub error: Option<String>,
}
