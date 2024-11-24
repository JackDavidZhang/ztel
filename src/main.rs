use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::{Aes256Gcm, KeyInit};
use log::{debug, error, info, warn};
use sha2::Digest;
use sha2::Sha256;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::spawn;
use ztel::{config, poxy, socks5};

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
    let mut hasher = Sha256::new();
    hasher.update(config.passwd.as_bytes());
    let result = hasher.finalize().to_vec();
    let cipher = Aes256Gcm::new(GenericArray::from_slice(result.as_slice()));
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
        let (mut stream, _listen_socket) = match listener.accept().await {
            Ok(a) => a,
            Err(_) => {
                warn!("Failed to accept connection.");
                continue;
            }
        };
        let mut buf: [u8; 4096] = [0; 4096];
        let len = match poxy::read_decrypt(&mut buf, &mut stream, &cipher).await {
            Err(_) | Ok(0) => {
                debug!("Stop 0x0001");
                continue;
            }
            Ok(n) => n,
        };
        if (len >= 3) && (buf[0] == 5) && (buf[1] == 1) {
            spawn(socks5::server_connect(stream, buf, len, cipher.clone()));
        } else {
            debug!("Stop 0x0002");
        }
    }
}
