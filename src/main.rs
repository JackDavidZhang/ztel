mod config;

use std::io::Read;
use std::net::{TcpListener,TcpStream};

fn main() {
    let config = config::load_config();
    let mut full_address = String::new();
    full_address.push_str(&config.servers[0].address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.servers[0].port.to_string());
    let listener = match TcpListener::bind(&full_address) {
        Ok(listener) => listener,
        Err(_) => panic!("unavailable address {}", full_address)
    };
    println!("Listening on {}", full_address);
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Connection from {} established!",stream.peer_addr().unwrap().to_string());
        let mut buff = [0; 2048];
        let len = stream.read(&mut buff).unwrap();
        for i in 0..len {
            print!("{} ", buff[i]);
        }
        println!();
    }
}
