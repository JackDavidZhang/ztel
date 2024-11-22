use log::{debug, info, warn};
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
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

async fn write(buf: &[u8], stream: &mut TcpStream) -> Result<usize, Error> {
    stream.write(buf).await
}

async fn read(buf: &mut [u8], stream: &mut TcpStream) -> Result<usize, Error> {
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

pub async fn client_forward(connection: Connection, stream: TcpStream) -> Result<(), &'static str> {
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
    Ok(())
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

pub async fn server_forward() -> Result<(), &'static str> {
    Ok(())
}

pub async fn server_connection(mut stream: TcpStream) -> Option<()> {
    let mut buf = [0u8; 4096];
    let client_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => {
            debug!("Stop 0x0201");
            return None;
        }
    };
    let len = match read(&mut buf, &mut stream).await {
        Ok(len) => len,
        Err(_) => {
            debug!("Stop 0x0202");
            return None;
        }
    };
    let dist_addr: SocketAddr;
    let mut write_buf: [u8; 10] = [5, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    if (len > 6) && (buf[0] == 5) && (buf[1] == 1) && (buf[2] == 0) {
        if buf[3] == 1 {
            dist_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7])),
                buf[8] as u16 * 256 + buf[9] as u16,
            );
        } else if buf[3] == 4 {
            dist_addr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(
                    buf[4] as u16 * 256 + buf[5] as u16,
                    buf[6] as u16 * 256 + buf[7] as u16,
                    buf[8] as u16 * 256 + buf[9] as u16,
                    buf[10] as u16 * 256 + buf[11] as u16,
                    buf[12] as u16 * 256 + buf[13] as u16,
                    buf[14] as u16 * 256 + buf[15] as u16,
                    buf[16] as u16 * 256 + buf[17] as u16,
                    buf[18] as u16 * 256 + buf[19] as u16,
                )),
                buf[20] as u16 * 256 + buf[21] as u16,
            );
        } else {
            warn!(
                "Unknown address code {}, connection from {} refused.",
                buf[3], client_addr
            );
            write_buf[1] = 8;
            match write(&mut write_buf, &mut stream).await {
                Ok(_) => {}
                Err(_) => {
                    debug!("Stop 0x0203");
                }
            };
            return None;
        }
        let diststream = match TcpStream::connect(dist_addr).await {
            Ok(stream) => stream,
            Err(_) => {
                warn!("Cannot connect to {}.", dist_addr);
                write_buf[1] = 1;
                match stream.write(&write_buf).await {
                    Ok(_) => {}
                    Err(_) => {
                        debug!("Stop 0x0204");
                    }
                };
                return None;
            }
        };
        info!(
            "Connect from {} to {} established",
            stream.peer_addr().unwrap(),
            dist_addr
        );
        match stream.write(&write_buf).await {
            Ok(_) => {}
            Err(_) => {
                warn!("Connect with {} aborted.", client_addr);
                return None;
            }
        };

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
    Some(())
}
