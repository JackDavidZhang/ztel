use log::warn;
use std::io::Error;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::select;

pub struct Connection {
    pub(crate) stream: TcpStream,
    pub(crate) delay: Duration,
    pub(crate) reply: [u8; 4096],
    pub(crate) len: usize,
}

pub async fn client_connect(address: &SocketAddr, request: &[u8]) -> Option<Connection> {
    let mut stream: TcpStream = match TcpStream::connect(&address).await {
        Ok(tcpstream) => tcpstream,
        Err(_) => {
            warn!("Failed to connect to node server {}", address);
            return None;
        }
    };
    match write(request, &mut stream).await {
        Ok(0) | Err(_) => {
            warn!("Failed write to node server {}.", address);
            return None;
        }
        Ok(_) => {}
    }
    let now = Instant::now();
    let mut buf = [0u8; 4096];
    let len = match read(&mut buf, &mut stream).await {
        Ok(0) | Err(_) => {
            warn!("Failed to read from node server {}.", address);
            return None;
        }
        Ok(len) => len,
    };
    Some(Connection {
        stream,
        delay: now.elapsed(),
        reply: buf,
        len,
    })
}

pub(crate) async fn write(buf: &[u8], stream: &mut TcpStream) -> Result<usize, Error> {
    stream.write(buf).await
}

pub async fn read(buf: &mut [u8], stream: &mut TcpStream) -> Result<usize, Error> {
    stream.read(buf).await
}

// fn write_encrypted(data: &[u8], stream: &mut TcpStream) -> Result<(), &'static str> {
// let mut encryptor =
//     //     aes::cbc_encryptor(KeySize::KeySize128, &[73u8; 16], &[48u8; 16], PkcsPadding);
//     // let stream_len;
//     // let mut final_result = [0u8; 4096];
//     // let mut stream = [0u8; 4096];
//     // {
//     //     let mut read_buff = RefReadBuffer::new(data);
//     //     let mut write_buff = RefWriteBuffer::new(&mut stream);
//     //     loop {
//     //         match encryptor
//     //             .encrypt(&mut read_buff, &mut write_buff, true)
//     //             .unwrap()
//     //         {
//     //             BufferResult::BufferUnderflow => break,
//     //             _ => continue,
//     //         }
//     //     }
//     //     stream_len = write_buff.position() as usize;
//     // }
//     // let mut decryptor_read_buff = RefReadBuffer::new(stream[0..stream_len].as_ref());
//     // let mut decryptor_write_buff = RefWriteBuffer::new(&mut final_result);
//     // let mut decryptor = aes::cbc_decryptor(KeySize128, &[73u8; 16], &[48u8; 16], PkcsPadding);
//     // loop {
//     //     match decryptor
//     //         .decrypt(&mut decryptor_read_buff, &mut decryptor_write_buff, true)
//     //         .unwrap()
//     //     {
//     //         BufferResult::BufferUnderflow | BufferResult::BufferOverflow => break,
//     //     }
//     // }
//     Ok(())
// }

pub async fn client_forward(connection: Connection, stream: TcpStream) {
    let (mut readstream, mut writestream) = split(stream);
    let (mut serverreadstream, mut serverwritestream) = split(connection.stream);
    let read_to_server = read_to_write(&mut readstream, &mut serverwritestream);
    let write_to_client = read_to_write(&mut serverreadstream, &mut writestream);
    select! {
        _r1  =read_to_server => {},
        _r2= write_to_client => {},
    }
    writestream.flush().await.unwrap();
    writestream.shutdown().await.unwrap();
    serverwritestream.flush().await.unwrap();
    serverwritestream.shutdown().await.unwrap();
}

async fn read_to_write(
    readstream: &mut ReadHalf<TcpStream>,
    writestream: &mut WriteHalf<TcpStream>,
) -> Result<(), &'static str> {
    let mut buf = [0u8; 128];
    loop {
        let len = match readstream.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        match writestream.write(&buf[..len]).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
    writestream.flush().await.unwrap();
    writestream.shutdown().await.unwrap();
    Ok(())
}

pub async fn server_forward(stream: TcpStream, diststream: TcpStream) {
    let (mut readstream, mut writestream) = split(stream);
    let (mut serverreadstream, mut serverwritestream) = split(diststream);
    let read_to_server = read_to_write(&mut readstream, &mut serverwritestream);
    let write_to_client = read_to_write(&mut serverreadstream, &mut writestream);
    select! {
        _r1 = read_to_server => {},
        _r2 = write_to_client => {},
    }
    writestream.flush().await.unwrap();
    writestream.shutdown().await.unwrap();
    serverwritestream.flush().await.unwrap();
    serverwritestream.shutdown().await.unwrap();
}
