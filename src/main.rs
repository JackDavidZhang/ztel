use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use ztel::config;

fn main() {
    let config = config::load_server_config();
    let mut full_address = String::new();
    full_address.push_str(&config.listeners[0].address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.listeners[0].port.to_string());
    let listener = match TcpListener::bind(&full_address) {
        Ok(listener) => listener,
        Err(_) => panic!("unavailable address {}", full_address),
    };
    println!("Listening on {}", full_address);
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut read_buffer = [0; 128];
        let len = match stream.read(&mut read_buffer) {
            Ok(len) => len,
            Err(_) => panic!("read failed"),
        };
        if (len > 6)
            && (read_buffer[0] == 5)
            && (read_buffer[2] == 0)
            && (((read_buffer[3] == 1) && (len == 10))
                || ((read_buffer[3] == 4) && (len == 12))
                || (read_buffer[3] == 3))
        {
            if read_buffer[3] == 1 {
                println!(
                    "Connect from {} to {}.{}.{}.{}:{} established",
                    stream.peer_addr().unwrap(),
                    read_buffer[4],
                    read_buffer[5],
                    read_buffer[6],
                    read_buffer[7],
                    read_buffer[8] as i32 * 256 + read_buffer[9] as i32
                );
            } else if read_buffer[3] == 3 {
                print!("Connect from {} to ", stream.peer_addr().unwrap());
                for i in 4..len - 2 {
                    print!("{}", read_buffer[i] as char);
                }
                println!(
                    ":{} established",
                    read_buffer[len - 2] as i32 * 256 + read_buffer[len - 1] as i32
                );
            } else if read_buffer[3] == 4 {
                println!(
                    "Connect from {} to {}:{}:{}:{}:{}:{}::{} established",
                    stream.peer_addr().unwrap(),
                    read_buffer[4],
                    read_buffer[5],
                    read_buffer[6],
                    read_buffer[7],
                    read_buffer[8],
                    read_buffer[9],
                    read_buffer[10] as i32 * 256 + read_buffer[11] as i32
                );
            } else {
                println!("Unknown Address {}", read_buffer[3]);
            }
            let write_buff: [u8; 10] = [
                5,
                0,
                0,
                1,
                127,
                0,
                0,
                1,
                (0 / 256) as u8,
                (0 % 256) as u8,
            ];
            stream.write(&write_buff).unwrap();
        } else {
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
}
