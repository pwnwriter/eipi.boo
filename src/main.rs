mod db;
mod helper;
mod model;
mod server;
mod tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    server::run().await
}
