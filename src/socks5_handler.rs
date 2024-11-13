use std::io::{Read, Write};
use std::net::TcpStream;
use crate::{client_server, config};

pub fn connect(mut stream: TcpStream,node : & config::Node){
    let mut read_buffer:[u8;128] = [0; 128];
    let mut write_buffer:[u8;2] = [0;2];
    write_buffer[0] = 5;
    match stream.write(&write_buffer){
        Ok(_) => {}
        Err(_) => {
            println!("ERROR: Connect with {} failed", stream.peer_addr().unwrap());
            return;
        }
    }
    let len = match stream.read(&mut read_buffer){
        Ok(n) => n,
        Err(_) => {
            println!("ERROR: Connect with {} failed", stream.peer_addr().unwrap());
            return;
        }
    };
    let connect_result = match client_server::connect(&node,&read_buffer[0..len],len){
        Ok(n) => n,
        Err(msg) => {
            println!("ERROR: Connect with {} failed: {}", stream.peer_addr().unwrap(), msg);
            return;
        }
    };
}