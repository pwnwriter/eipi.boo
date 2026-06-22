mod blocked_items;
mod canvas;
mod confession;
mod consts;
mod db;
mod handler;
mod input;
mod reply;
mod server;
mod tui;
mod web;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    server::run().await
}
