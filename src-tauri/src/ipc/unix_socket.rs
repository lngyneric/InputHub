use crate::{config::Config, db::Db};
use tracing::info;

pub async fn serve(path: String, _db: Db, _cfg: Config) -> anyhow::Result<()> {
    info!(%path, "unix socket starting (mock)");
    // Keep it alive
    std::future::pending::<()>().await;
    Ok(())
}
