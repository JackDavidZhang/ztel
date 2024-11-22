use log::{error, info, warn};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::spawn;
use ztel::config;
use ztel::poxy::server_connection;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = match config::load_server_config() {
        Ok(c) => c,
        Err(msg) => {
            error!("Cannot load config: {}", msg);
            return;
        }
    };
    let full_address = SocketAddr::new(
        match config.listener.address.parse() {
            Ok(a) => a,
            Err(_) => {
                error!("Unavailable listener address, exiting.");
                return;
            }
        },
        config.listener.port,
    );
    let listener = match TcpListener::bind(&full_address).await {
        Ok(listener) => listener,
        Err(_) => {
            error!("Cannot listen on {}, exiting.", full_address);
            return;
        }
    };
    info!("Listening on {}", full_address);
    loop {
        let (stream, _listen_socket) = match listener.accept().await {
            Ok(a) => a,
            Err(_) => {
                warn!("Failed to accept connection.");
                continue;
            }
        };
        spawn(server_connection(stream));
    }
}
