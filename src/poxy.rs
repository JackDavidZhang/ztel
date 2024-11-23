use log::{debug, warn};
use std::io::Error;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use aes_gcm::{AeadCore, Aes128Gcm, AesGcm, KeyInit};
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, Nonce, OsRng};
use aes_gcm::aead::consts::U12;
use aes_gcm::aes::Aes128;
use lazy_static::lazy_static;
use tokio::io::{split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::select;

static KEY:[u8;16] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
static NONCE:[u8;12] = [16,17,18,19,20,21,22,23,24,25,26,27];

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
    let cipher = Aes128Gcm::new(GenericArray::from_slice(&KEY));
    let nonce = GenericArray::from_slice(&NONCE);
    let ciphertext = cipher.encrypt(&nonce,buf).unwrap();
    stream.write(ciphertext.as_slice()).await
}

pub async fn read(buf: &mut [u8], stream: &mut TcpStream) -> Result<usize, Error> {
    let cipher = Aes128Gcm::new(GenericArray::from_slice(&KEY));
    let nonce= GenericArray::from_slice(&NONCE);
    let mut read_buf = vec![0u8; 128];
    let len = match stream.read(&mut read_buf).await{
        Ok(0)=>{return Ok(0)},
        Ok(len) => len,
        Err(e) => return Err(e),
    };
    let plaintext = cipher.decrypt(&nonce,&read_buf[..len]).unwrap();
    buf[..plaintext.len()].copy_from_slice(&plaintext);
    Ok(plaintext.len())
}

pub async fn client_forward(connection: Connection, stream: TcpStream) {
    let (mut readstream, mut writestream) = split(stream);
    let (mut serverreadstream, mut serverwritestream) = split(connection.stream);
    let read_to_server = read_to_write(&mut readstream, &mut serverwritestream);
    let write_to_client = read_to_write(&mut serverreadstream, &mut writestream);
    select! {
        _r1  =read_to_server => {},
        _r2= write_to_client => {},
    }
    writestream.flush().await.unwrap_or(());
    writestream.shutdown().await.unwrap_or(());
    serverwritestream.flush().await.unwrap_or(());
    serverwritestream.shutdown().await.unwrap_or(());
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
    writestream.flush().await.unwrap_or(());
    writestream.shutdown().await.unwrap_or(());
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
    writestream.flush().await.unwrap_or(());
    writestream.shutdown().await.unwrap_or(());
    serverwritestream.flush().await.unwrap_or(());
    serverwritestream.shutdown().await.unwrap_or(());
}
