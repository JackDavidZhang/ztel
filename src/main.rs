mod config;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

fn main() {
    let config = config::load_config();
    let mut full_address = String::new();
    full_address.push_str(&config.servers[0].address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.servers[0].port.to_string());
    let listener = match TcpListener::bind(&full_address) {
        Ok(listener) => listener,
        Err(_) => panic!("unavailable address {}", full_address),
    };
    println!("Listening on {}", full_address);
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buff = [0; 2048];
        let len = stream.read(&mut buff).unwrap();
        if buff[0] == 5 {
            if (len > 2) && (len == (2 + buff[1]) as usize) {
                print!(
                    "SOCKS5 Receive 1 from {}, ACCEPT {} METHOD(s): ",
                    stream.peer_addr().unwrap().to_string(),
                    buff[1]
                );
                for i in 2..len {
                    print!("{} ", buff[i]);
                }
                println!();
                let write_buff: [u8; 2] = [5, 0];
                stream.write(&write_buff).unwrap();
                println!("SOCKS5 Reply to {}", stream.peer_addr().unwrap().to_string());
            }
        }
    }
}
