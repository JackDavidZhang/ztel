use crate::poxy;
use crate::poxy::client_forward;
use log::{debug, info, warn};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn connect(mut stream: TcpStream, node_addr: SocketAddr) {
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
    let remote_addr: SocketAddr;
    if (len > 6) && (read_buffer[0] == 5) && (read_buffer[1] == 1) && (read_buffer[2] == 0) {
        if (read_buffer[3] == 1) && (len == 10) {
            remote_addr = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(
                    read_buffer[4],
                    read_buffer[5],
                    read_buffer[6],
                    read_buffer[7],
                )),
                read_buffer[8] as u16 * 256 + read_buffer[9] as u16,
            );
        } else if (read_buffer[3] == 4) && (len == 22) {
            remote_addr = SocketAddr::new(
                IpAddr::V6(Ipv6Addr::new(
                    read_buffer[4] as u16 * 256 + read_buffer[5] as u16,
                    read_buffer[6] as u16 * 256 + read_buffer[7] as u16,
                    read_buffer[8] as u16 * 256 + read_buffer[9] as u16,
                    read_buffer[10] as u16 * 256 + read_buffer[11] as u16,
                    read_buffer[12] as u16 * 256 + read_buffer[13] as u16,
                    read_buffer[14] as u16 * 256 + read_buffer[15] as u16,
                    read_buffer[16] as u16 * 256 + read_buffer[17] as u16,
                    read_buffer[18] as u16 * 256 + read_buffer[19] as u16,
                )),
                read_buffer[20] as u16 * 256 + read_buffer[21] as u16,
            );
        } else {
            warn!(
                "Connect with {} failed: wrong request address.",
                source_addr
            );
            return;
        }
    } else {
        warn!(
            "Connect with {} failed: wrong {} bytes request.",
            source_addr, len
        );
        return;
    }
    let connect_result = match poxy::client_connect(&node_addr, &read_buffer[0..len]).await {
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
        match client_forward(connect_result, stream).await {
            Ok(_) => {}
            Err(msg) => {
                warn!(
                    "Forwarding {} to {} failed: {}",
                    source_addr, node_addr, msg
                );
            }
        };
    } else {
        let len = connect_result.len;
        warn!(
            "Connect with {} failed: wrong {} bytes reply from node server.",
            source_addr, len
        );
    }
}
