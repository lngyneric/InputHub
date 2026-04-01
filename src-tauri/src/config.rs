use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: PathBuf,
    pub http_addr: String,
    pub socket_path: String,
    pub obsidian_vault_path: Option<PathBuf>,
    pub doubao_api_key: Option<String>,
    pub doubao_base_url: String,
    pub doubao_model: String,
    pub claude_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
        let db_path = env::var("AGENT_HUB_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".agent-hub/agent_hub.sqlite"));

        let http_addr = env::var("AGENT_HUB_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:9527".into());
        let socket_path =
            env::var("AGENT_HUB_SOCKET_PATH").unwrap_or_else(|_| "/tmp/agent_hub.sock".into());
        let obsidian_vault_path = env::var("OBSIDIAN_VAULT_PATH").ok().map(PathBuf::from);

        let doubao_api_key = env::var("DOUBAO_API_KEY").ok();
        let doubao_base_url = env::var("DOUBAO_BASE_URL")
            .unwrap_or_else(|_| "https://ark.cn-beijing.volces.com".into());
        let doubao_model =
            env::var("DOUBAO_MODEL").unwrap_or_else(|_| "ep-20241225134106-kpsps".into());
        let claude_api_key = env::var("CLAUDE_API_KEY").ok();

        Ok(Self {
            db_path,
            http_addr,
            socket_path,
            obsidian_vault_path,
            doubao_api_key,
            doubao_base_url,
            doubao_model,
            claude_api_key,
        })
    }
}
