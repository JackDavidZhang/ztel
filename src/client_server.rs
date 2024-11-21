use crate::config::Node;
use std::io::Error;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{copy, split};
use tokio::net::TcpStream;
use tokio::select;

pub struct Connection {
    pub(crate) stream: TcpStream,
    pub(crate) delay: Duration,
    pub(crate) reply: [u8; 4096],
    pub(crate) len: usize,
}

pub async fn connect(node: &Node, request: &[u8], len: usize) -> Result<Connection, &'static str> {
    let address = SocketAddr::new(node.address.parse().unwrap(), node.port);
    let mut stream: TcpStream = match TcpStream::connect(&address).await {
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
    stream.readable().await.unwrap();
    let len = match stream.try_read(&mut read_buff) {
        Ok(len) => len,
        Err(e) => {
            eprintln!("{}", e);
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
    stream.try_write(data)?;
    Ok(())
}

fn write_encrypted(data: &[u8], stream: &mut TcpStream) -> Result<(), &'static str> {
    Ok(())
}

pub async fn forward(connection: Connection, stream: TcpStream) -> Result<(), &'static str> {
    let (mut readstream, mut writestream) = split(stream);
    let (mut serverreadstream, mut serverwritestream) = split(connection.stream);
    let read_to_server = copy(&mut readstream, &mut serverwritestream);
    let write_to_client = copy(&mut serverreadstream, &mut writestream);
    select! {
        r1 = read_to_server => {},
        r2 = write_to_client => {},
    }
    Ok(())
}
