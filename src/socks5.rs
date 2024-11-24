use crate::poxy;
use crate::poxy::{client_forward, server_forward};
use aes_gcm::aead::consts::U12;
use aes_gcm::aes::Aes256;
use aes_gcm::AesGcm;
use log::{debug, info, warn};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn client_connect(
    mut stream: TcpStream,
    node_addr: SocketAddr,
    cipher: AesGcm<Aes256, U12>,
) {
    let source_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => {
            debug!("Stop 0x0101");
            return;
        }
    };
    let mut read_buffer = [0u8; 4096];
    let mut write_buffer: [u8; 2] = [0; 2];
    write_buffer[0] = 5;
    match stream.write(&write_buffer).await {
        Ok(_) => {}
        Err(_) => {
            debug!("Stop 0x0102");
            return;
        }
    }
    let len = match stream.read(&mut read_buffer).await {
        Ok(n) => n,
        Err(_) => {
            debug!("Stop 0x0103");
            return;
        }
    };
    let remote_addr: SocketAddr = match get_addr(&read_buffer[0..len]) {
        Ok(addr) => addr,
        Err(_) => {
            warn!(
                "Connect with {} failed: wrong {} bytes request.",
                source_addr, len
            );
            return;
        }
    };
    let connect_result = match poxy::client_connect(&node_addr, &read_buffer[0..len], &cipher).await
    {
        Some(n) => n,
        None => {
            return;
        }
    };
    if (connect_result.len >= 3) && (connect_result.reply[0] == 5) && (connect_result.reply[2] == 0)
    {
        match stream
            .write(&connect_result.reply[0..connect_result.len])
            .await
        {
            Ok(_) => {}
            Err(_) => {
                warn!("Connect with {} aborted.", source_addr);
                return;
            }
        };
        info!(
            "Connect from {} to {} success in {} ms. ({})",
            source_addr,
            node_addr,
            connect_result.delay.as_millis(),
            remote_addr
        );
        client_forward(connect_result, stream, &cipher).await;
    } else {
        let len = connect_result.len;
        warn!(
            "Connect with {} failed: wrong {} bytes reply from node server.",
            source_addr, len
        );
    }
}

pub async fn server_connect(
    mut stream: TcpStream,
    request: [u8; 4096],
    len: usize,
    cipher: AesGcm<Aes256, U12>,
) {
    let client_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => {
            debug!("Stop 0x0201");
            return;
        }
    };
    let mut write_buf: [u8; 10] = [5, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    let dist_addr: SocketAddr = match get_addr(&request[0..len]) {
        Ok(addr) => addr,
        Err(_) => {
            warn!("Connect with {} failed: wrong request.", client_addr);
            write_buf[1] = 8;
            match poxy::write_encrypt(&mut write_buf, &mut stream, &cipher).await {
                Ok(_) => {}
                Err(_) => {
                    debug!("Stop 0x0202");
                }
            };
            return;
        }
    };
    let diststream = match TcpStream::connect(dist_addr).await {
        Ok(stream) => stream,
        Err(_) => {
            warn!("Cannot connect to {}.", dist_addr);
            write_buf[1] = 1;
            match poxy::write_encrypt(&write_buf, &mut stream, &cipher).await {
                Ok(_) => {}
                Err(_) => {
                    debug!("Stop 0x0204");
                }
            };
            return;
        }
    };
    match poxy::write_encrypt(&mut write_buf, &mut stream, &cipher).await {
        Ok(_) => {}
        Err(_) => {
            warn!("Connect with {} aborted.", client_addr);
            return;
        }
    };
    info!("Connect from {} to {} established", client_addr, dist_addr);
    server_forward(stream, diststream, &cipher).await;
}

fn get_addr(src: &[u8]) -> Result<SocketAddr, ()> {
    if (src.len() > 6) && (src[0] == 5) && (src[1] == 1) && (src[2] == 0) {
        if (src[3] == 1) && (src.len() == 10) {
            Ok(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(src[4], src[5], src[6], src[7])),
                src[8] as u16 * 256 + src[9] as u16,
            ))
        } else if (src[3] == 4) && (src.len() == 22) {
            Ok(SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(
                    src[4] as u16 * 256 + src[5] as u16,
                    src[6] as u16 * 256 + src[7] as u16,
                    src[8] as u16 * 256 + src[9] as u16,
                    src[10] as u16 * 256 + src[11] as u16,
                    src[12] as u16 * 256 + src[13] as u16,
                    src[14] as u16 * 256 + src[15] as u16,
                    src[16] as u16 * 256 + src[17] as u16,
                    src[18] as u16 * 256 + src[19] as u16,
                )),
                src[20] as u16 * 256 + src[21] as u16,
            ))
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}
