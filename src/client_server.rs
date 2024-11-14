use crate::config::Node;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Instant;
use aes::{Aes128, Aes256};
use aes::cipher::generic_array::GenericArray;
use aes::cipher::KeyInit;

pub fn connect(
    node: &Node,
    request: &[u8],
    len: usize,
) -> Result<([u8; 128], usize, u128), &'static str> {
    if (len > 6)
        && (request[0] == 5)
        && (request[2] == 0)
        && (((request[3] == 1) && (len == 10))
            || ((request[3] == 4) && (len == 26) || (request[3] == 3)))
    {
        let address = node.address.clone() + ":" + &*node.port.to_string();
        let mut stream: TcpStream = match TcpStream::connect(&address) {
            Ok(tcpstream) => tcpstream,
            Err(_) => {
                return Err("Field to connect to node server.");
            }
        };
        write(request, &mut stream);
        match stream.write(request) {
            Ok(_) => {}
            Err(_) => {
                return Err("Error writing to node server.");
            }
        }
        let now = Instant::now();
        let mut read_buff: [u8; 128] = [0; 128];
        let len = match stream.read(&mut read_buff) {
            Ok(len) => len,
            Err(_) => {
                return Err("Error reading from node server.");
            }
        };
        let end = now.elapsed().as_millis();
        Ok((read_buff, len, end))
    } else {
        Err("Unaccepted connect request.")
    }
}

fn write(data : &[u8],stream:&mut TcpStream) -> Result<(), &'static str> {
    let key = GenericArray::from([7u8; 32]);
    let cipher = AesCbc::new(&key);
    Ok(())
}