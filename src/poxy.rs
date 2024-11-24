use aes_gcm::aead::consts::U12;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::aes::Aes256;
use aes_gcm::{AeadCore, Aes256Gcm, AesGcm};
use log::warn;
use std::io::Error;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::select;

pub struct Connection {
    pub(crate) stream: TcpStream,
    pub(crate) delay: Duration,
    pub(crate) reply: [u8; 4096],
    pub(crate) len: usize,
}

pub async fn client_connect(
    address: &SocketAddr,
    request: &[u8],
    cipher: &AesGcm<Aes256, U12>,
) -> Option<Connection> {
    let mut stream: TcpStream = match TcpStream::connect(&address).await {
        Ok(tcpstream) => tcpstream,
        Err(_) => {
            warn!("Failed to connect to node server {}", address);
            return None;
        }
    };
    match write_encrypt(request, &mut stream, cipher).await {
        Ok(0) | Err(_) => {
            warn!("Failed write to node server {}.", address);
            return None;
        }
        Ok(_) => {}
    }
    let now = Instant::now();
    let mut buf = [0u8; 4096];
    let len = match read_decrypt(&mut buf, &mut stream, cipher).await {
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

pub async fn write_encrypt<T: AsyncWrite + Unpin>(
    buf: &[u8],
    stream: &mut T,
    cipher: &AesGcm<Aes256, U12>,
) -> Result<usize, Error> {
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let mut write_buf = Vec::from(nonce.as_slice());

    let encrypt_text = match cipher.encrypt(&nonce, buf) {
        Ok(b) => b,
        Err(e) => {
            warn!("Encrypt failed: {}", e);
            return Ok(0);
        }
    };
    let length = [encrypt_text.len() as u8];
    let encrypt_length = match cipher.encrypt(&nonce, length.as_slice()) {
        Ok(b) => b,
        Err(e) => {
            warn!("Encrypt failed: {}", e);
            return Ok(0);
        }
    };
    write_buf.extend(encrypt_length);
    write_buf.extend(encrypt_text);
    stream.write(write_buf.as_slice()).await
}

pub async fn read_decrypt<T: AsyncRead + Unpin>(
    buf: &mut [u8],
    stream: &mut T,
    cipher: &AesGcm<Aes256, U12>,
) -> Result<usize, Error> {
    let mut nonce_buf = [0u8; 12];
    match stream.read(&mut nonce_buf).await {
        Ok(12) => 12,
        Ok(_) => return Ok(0),
        Err(e) => return Err(e),
    };
    let nonce = GenericArray::from_slice(&nonce_buf);
    let mut length_buf = [0u8; 17];
    match stream.read(&mut length_buf).await {
        Ok(17) => 17,
        Ok(_) => return Ok(0),
        Err(e) => return Err(e),
    };
    let length = match cipher.decrypt(&nonce, length_buf.as_slice()) {
        Ok(a) => a[0],
        Err(e) => {
            warn!("Decrypt length failed: {}", e);
            return Ok(0);
        }
    };
    let mut read_buf = Vec::new();
    read_buf.resize(length as usize, 0);
    match stream.read(&mut read_buf).await {
        Ok(0) => return Ok(0),
        Ok(len) => len,
        Err(e) => return Err(e),
    };
    let plaintext = match cipher.decrypt(&nonce, read_buf.as_slice()) {
        Ok(a) => a,
        Err(e) => {
            warn!("Decrypt failed: {}", e);
            return Ok(0);
        }
    };
    buf[..plaintext.len()].copy_from_slice(&plaintext);
    Ok(plaintext.len())
}

pub async fn client_forward(
    connection: Connection,
    stream: TcpStream,
    cipher: &AesGcm<Aes256, U12>,
) {
    let (mut readstream, mut writestream) = split(stream);
    let (mut serverreadstream, mut serverwritestream) = split(connection.stream);
    let read_to_server = read_to_write_encrypt(&mut readstream, &mut serverwritestream, cipher);
    let write_to_client = read_decrypt_to_write(&mut serverreadstream, &mut writestream, cipher);
    select! {
        _r1 = read_to_server => {},
        _r2 = write_to_client => {},
    }
    writestream.flush().await.unwrap_or(());
    writestream.shutdown().await.unwrap_or(());
    serverwritestream.flush().await.unwrap_or(());
    serverwritestream.shutdown().await.unwrap_or(());
}
pub async fn server_forward(
    stream: TcpStream,
    diststream: TcpStream,
    cipher: &AesGcm<Aes256, U12>,
) {
    let (mut readstream, mut writestream) = split(stream);
    let (mut serverreadstream, mut serverwritestream) = split(diststream);
    let read_to_server = read_decrypt_to_write(&mut readstream, &mut serverwritestream, cipher);
    let write_to_client = read_to_write_encrypt(&mut serverreadstream, &mut writestream, cipher);
    select! {
        _r1 = read_to_server => {},
        _r2 = write_to_client => {},
    }
    writestream.flush().await.unwrap_or(());
    writestream.shutdown().await.unwrap_or(());
    serverwritestream.flush().await.unwrap_or(());
    serverwritestream.shutdown().await.unwrap_or(());
}

async fn read_to_write_encrypt(
    readstream: &mut ReadHalf<TcpStream>,
    writestream: &mut WriteHalf<TcpStream>,
    cipher: &AesGcm<Aes256, U12>,
) -> Result<(), &'static str> {
    let mut buf = [0; 128];
    loop {
        let len = match readstream.read(buf.as_mut_slice()).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        match write_encrypt(&buf[..len], writestream, cipher).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
    writestream.flush().await.unwrap_or(());
    writestream.shutdown().await.unwrap_or(());
    Ok(())
}

async fn read_decrypt_to_write(
    readstream: &mut ReadHalf<TcpStream>,
    writestream: &mut WriteHalf<TcpStream>,
    cipher: &AesGcm<Aes256, U12>,
) -> Result<(), &'static str> {
    let mut buf = [0; 128];
    loop {
        let len = match read_decrypt(&mut buf, readstream, cipher).await {
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
