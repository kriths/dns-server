use log::LevelFilter;

use crate::config::ServerConfig;
use crate::server::DNSServer;

mod config;
mod data;
mod handler;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::default();
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let server = DNSServer::new(config);
    server.listen().await
}
