mod agents;
mod config;
mod db;
mod ipc;
mod llm;
mod types;

use anyhow::Context;
use config::Config;
use db::Db;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "agent_hub=info,tower_http=info".into()),
        )
        .init();

    let cfg = Config::from_env().context("load config")?;
    info!(?cfg, "agent hub starting");

    let db = Db::connect(&cfg.db_path).await.context("connect db")?;
    db.init().await.context("init db")?;

    // HTTP server (always available; easiest for Rime/Lua via curl)
    let http_db = db.clone();
    let http_cfg = cfg.clone();
    let http_handle = tokio::spawn(async move {
        if let Err(e) =
            ipc::http::serve(http_cfg.http_addr.clone(), http_db, http_cfg.clone()).await
        {
            warn!(error = %e, "http server exited");
        }
    });

    // Unix socket server (macOS/Linux); ignore on Windows
    #[cfg(unix)]
    {
        let sock_db = db.clone();
        let sock_path = cfg.socket_path.clone();
        let sock_cfg = cfg.clone();
        let sock_handle = tokio::spawn(async move {
            if let Err(e) = ipc::unix_socket::serve(sock_path, sock_db, sock_cfg).await {
                warn!(error = %e, "unix socket server exited");
            }
        });

        tokio::select! {
          _ = shutdown_signal() => { info!("shutdown requested"); }
          _ = http_handle => { warn!("http task finished"); }
          _ = sock_handle => { warn!("socket task finished"); }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::select! {
          _ = shutdown_signal() => { info!("shutdown requested"); }
          _ = http_handle => { warn!("http task finished"); }
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
