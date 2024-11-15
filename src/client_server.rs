use crate::config::Node;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use crypto::aes;
use crypto::aes::KeySize;
use crypto::aes::KeySize::KeySize128;
use crypto::blockmodes::PkcsPadding;
use crypto::buffer::{BufferResult, RefReadBuffer, RefWriteBuffer, WriteBuffer};

pub struct Connection {
    pub(crate) stream: TcpStream,
    pub(crate) delay: Duration,
    pub(crate) reply: [u8;256],
    pub(crate) len: usize,
}

pub fn connect(
    node: &Node,
    request: &[u8],
    len: usize,
) -> Result<Connection, &'static str> {
    if (len > 6)
        && (request[0] == 5)
        && (request[2] == 0)
        && (((request[3] == 1) && (len == 10))
            || ((request[3] == 4) && (len == 22) || (request[3] == 3)))
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
        let mut read_buff: [u8; 256] = [0; 256];
        let len = match stream.read(&mut read_buff) {
            Ok(len) => len,
            Err(_) => {
                return Err("Error reading from node server.");
            }
        };
        Ok(Connection{
            stream,
            delay: now.elapsed(),
            reply: read_buff,
            len,
        })
    } else {
        for i in 0..request.len()
        {
            print!("{} ",request[i]);
        }
        println!("");
        Err("Unaccepted connect request.")
    }
}

fn write(data : &[u8],stream:&mut TcpStream) -> Result<(), &'static str> {
    let mut encryptor = aes::cbc_encryptor(KeySize::KeySize128, &[73u8;16], &[48u8;16], PkcsPadding);
    let stream_len;
    let mut final_result = [0u8;4096];
    let mut stream = [0u8;4096];{
    let mut read_buff = RefReadBuffer::new(data);
    let mut write_buff= RefWriteBuffer::new(&mut stream);
    loop{
        match encryptor.encrypt(&mut read_buff, &mut write_buff, true).unwrap(){
            BufferResult::BufferUnderflow => break,
            _ => continue,
        }
    }
    stream_len = write_buff.position() as usize;
    }
    let mut decryptor_read_buff = RefReadBuffer::new(stream[0..stream_len].as_ref());
    let mut decryptor_write_buff = RefWriteBuffer::new(&mut final_result);
    let mut decryptor = aes::cbc_decryptor(KeySize128,&[73u8;16],&[48u8;16], PkcsPadding);
    loop{
        match decryptor.decrypt(&mut decryptor_read_buff, &mut decryptor_write_buff, true).unwrap() {
            BufferResult::BufferUnderflow | BufferResult::BufferOverflow => break,
            _ => continue,
        }
    }
    Ok(())
}