use log::{debug, error, info, warn};
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::spawn;
use ztel::config::load_client_config;
use ztel::socks5::connect;

#[tokio::main]
async fn main() {
    env_logger::init();
    let config = match load_client_config() {
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
    let node_addres = SocketAddr::new(
        match config.node.address.parse() {
            Ok(a) => a,
            Err(_) => {
                error!("Unavailable node address, exiting.");
                return;
            }
        },
        config.node.port,
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
        let (mut stream, _listen_socket) = match listener.accept().await {
            Ok(a) => a,
            Err(_) => {
                warn!("Failed to accept connection.");
                continue;
            }
        };
        let mut read_buff: [u8; 4096] = [0; 4096];
        let len = match stream.read(&mut read_buff).await {
            Ok(n) => n,
            Err(_) => {
                debug!("Stop 0x0001");
                continue;
            }
        };
        if (len >= 3) && (read_buff[0] == 5) && (read_buff[1] == 1) {
            let node_copy = node_addres.clone();
            spawn(connect(stream, node_copy));
        }
    }
}
