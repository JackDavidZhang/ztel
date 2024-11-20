use crate::config::Node;
use std::io::{BufReader, Error, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

pub struct Connection {
    pub(crate) stream: TcpStream,
    pub(crate) delay: Duration,
    pub(crate) reply: [u8; 4096],
    pub(crate) len: usize,
}

pub fn connect(node: &Node, request: &[u8], len: usize) -> Result<Connection, &'static str> {
    let address = SocketAddr::new(node.address.parse().unwrap(), node.port);
    let mut stream: TcpStream = match TcpStream::connect(&address) {
        Ok(tcpstream) => tcpstream,
        Err(_) => {
            return Err("Failed to connect to node server.");
        }
    };
    match write(request, &mut stream) {
        Ok(_) => {}
        Err(e) => {
            return Err("Failed write to node server.");
        }
    }
    let now = Instant::now();
    let mut read_buff = [0u8; 4096];
    let len = match stream.read(&mut read_buff) {
        Ok(len) => len,
        Err(_) => {
            return Err("Failed to read from node server.");
        }
    };
    Ok(Connection {
        stream,
        delay: now.elapsed(),
        reply: read_buff,
        len,
    })
}

fn write(data: &[u8], stream: &mut TcpStream) -> Result<(), Error> {
    // let mut encryptor =
    //     aes::cbc_encryptor(KeySize::KeySize128, &[73u8; 16], &[48u8; 16], PkcsPadding);
    // let stream_len;
    // let mut final_result = [0u8; 4096];
    // let mut stream = [0u8; 4096];
    // {
    //     let mut read_buff = RefReadBuffer::new(data);
    //     let mut write_buff = RefWriteBuffer::new(&mut stream);
    //     loop {
    //         match encryptor
    //             .encrypt(&mut read_buff, &mut write_buff, true)
    //             .unwrap()
    //         {
    //             BufferResult::BufferUnderflow => break,
    //             _ => continue,
    //         }
    //     }
    //     stream_len = write_buff.position() as usize;
    // }
    // let mut decryptor_read_buff = RefReadBuffer::new(stream[0..stream_len].as_ref());
    // let mut decryptor_write_buff = RefWriteBuffer::new(&mut final_result);
    // let mut decryptor = aes::cbc_decryptor(KeySize128, &[73u8; 16], &[48u8; 16], PkcsPadding);
    // loop {
    //     match decryptor
    //         .decrypt(&mut decryptor_read_buff, &mut decryptor_write_buff, true)
    //         .unwrap()
    //     {
    //         BufferResult::BufferUnderflow | BufferResult::BufferOverflow => break,
    //     }
    // }
    stream.write(data)?;
    Ok(())
}

fn write_encrypted(data: &[u8], stream: &mut TcpStream) -> Result<(), &'static str> {
    Ok(())
}

pub fn forward(connection: &mut Connection, stream: &mut TcpStream) -> Result<(), &'static str> {
    let mut read_buff = [0u8; 1024 * 100];
    let mut write_buff = [0u8; 1024 * 100];
    let mut len = 0usize;
    let mut read_buffer = BufReader::new(&mut *stream);
    match read_buffer.read(&mut read_buff) {
        Ok(0) => {}
        Ok(n) => {
            println!("DEBUG: Read {} bytes.", n);
            match write(&read_buff[0..n], &mut connection.stream) {
                Ok(_) => {
                    println!("DEBUG: Sent {} bytes.", n);
                }
                Err(_) => {}
            };
        }
        Err(_) => {}
    }
    let mut read_buffer = BufReader::new(&mut connection.stream);
    match read_buffer.read(&mut write_buff) {
        Ok(0) => {}
        Ok(n) => {
            println!("DEBUG: Read {} bytes.", n);
            match stream.write(&write_buff[0..n]) {
                Ok(_) => {
                    println!("DEBUG: Sent {} bytes.", n);
                }
                Err(_) => {}
            };
        }
        Err(_) => {}
    }
    Ok(())
}
