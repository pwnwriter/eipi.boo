mod handler;
pub(crate) mod input;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use parking_lot::Mutex;

use anyhow::Result;
use log::info;
use russh::server::{self, Server as _};

use crate::db;
use handler::ClientHandler;

pub(crate) struct AppState {
    pub(crate) db: Mutex<rusqlite::Connection>,
    pub(crate) online: AtomicUsize,
}

struct SshServer {
    state: Arc<AppState>,
}

impl server::Server for SshServer {
    type Handler = ClientHandler;

    fn new_client(&mut self, addr: Option<std::net::SocketAddr>) -> ClientHandler {
        info!("New connection from {:?}", addr);
        ClientHandler::new(self.state.clone())
    }
}

fn load_or_generate_host_key(path: &str) -> Result<russh::keys::PrivateKey> {
    if Path::new(path).exists() {
        info!("Loading host key from {}", path);
        let key = russh::keys::load_secret_key(path, None)?;
        Ok(key)
    } else {
        info!("Generating new Ed25519 host key at {}", path);
        let key =
            russh::keys::PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519)
                .map_err(|e| anyhow::anyhow!("Failed to generate key: {}", e))?;

        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        key.write_openssh_file(Path::new(path), russh::keys::ssh_key::LineEnding::LF)
            .map_err(|e| anyhow::anyhow!("Failed to write host key: {}", e))?;

        Ok(key)
    }
}

pub async fn run() -> Result<()> {
    let db_path = std::env::var("EIPI_DB_PATH").unwrap_or_else(|_| "eipi.db".to_string());
    let host_key_path =
        std::env::var("EIPI_HOST_KEY").unwrap_or_else(|_| "assets/host_key".to_string());
    let listen_addr = std::env::var("EIPI_LISTEN").unwrap_or_else(|_| "0.0.0.0:22".to_string());

    let conn = db::init(&db_path)?;
    let state = Arc::new(AppState {
        db: Mutex::new(conn),
        online: AtomicUsize::new(0),
    });

    let host_key = load_or_generate_host_key(&host_key_path)?;

    let config = Arc::new(server::Config {
        keys: vec![host_key],
        ..Default::default()
    });

    crate::helper::web::write_index();

    info!("Starting eipi.boo SSH server on {}", listen_addr);
    info!(
        "Connect with: ssh -p {} localhost",
        listen_addr.rsplit(':').next().unwrap_or("22")
    );

    let mut server = SshServer { state };
    server.run_on_address(config, &listen_addr).await?;

    Ok(())
}
