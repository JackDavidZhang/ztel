use std::io::{Read, Write};
use std::net::TcpListener;
use ztel::config::load_client_config;
use ztel::socks5_handler::connect;

fn main() {
    let config = load_client_config();
    let mut full_address = String::new();
    full_address.push_str(&config.listeners[0].address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.listeners[0].port.to_string());
    let listener = match TcpListener::bind(&full_address) {
        Ok(listener) => listener,
        Err(_) => panic!("Cannot listen on {}", full_address),
    };
    println!("Listening on {}", full_address);
    for stream in listener.incoming() {
        let mut read_buff:[u8;128] = [0; 128];
        let mut stream = stream.unwrap();
        let len = stream.read(&mut read_buff).unwrap();
        if read_buff[0] == 5{
            connect(stream,&config.node);
        }
    }
}