use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::spawn;
use ztel::config::load_client_config;
use ztel::socks5_handler::connect;

#[tokio::main]
async fn main() {
    let config = match load_client_config() {
        Ok(c) => c,
        Err(msg) => {
            eprintln!("ERROR: Cannot load config: {}", msg);
            return;
        }
    };
    let full_address = SocketAddr::new(
        match config.listener.address.parse() {
            Ok(a) => a,
            Err(_) => {
                eprintln!("ERROR: Unavailable listener address, exiting.");
                return;
            }
        },
        config.listener.port,
    );
    let listener = match TcpListener::bind(&full_address).await {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("ERROR: Cannot listen on {}, exiting.", full_address);
            return;
        }
    };
    println!("Listening on {}", full_address);
    loop {
        println!("DEBUG: Waiting for connections...");
        let (stream, listen_socket) = match listener.accept().await {
            Ok(a) => a,
            Err(_) => {
                eprintln!("ERROR: Cannot accept connection, exiting.");
                break;
            }
        };
        println!("DEBUG: Accepted!");
        let mut read_buff: [u8; 4096] = [0; 4096];
        let len = match stream.try_read(&mut read_buff) {
            Ok(n) => n,
            Err(_) => continue,
        };
        if (len >= 3) && (read_buff[0] == 5) && (read_buff[1] == 1) {
            let config_copy = config.clone();
            let config_node_copy = config.node.clone();
            spawn(connect(stream, config_node_copy, config_copy));
        }
    }
}
