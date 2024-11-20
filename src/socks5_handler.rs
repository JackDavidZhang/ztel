use crate::client_server::{forward, Connection};
use crate::config::ClientConfig;
use crate::{client_server, config};
use std::io::{Read, Write};
use std::net::Shutdown::Both;
use std::net::{SocketAddr, TcpStream};

pub fn connect(mut stream: TcpStream, node: &config::Node, config: &ClientConfig) {
    let source_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return,
    };
    let node_addr = SocketAddr::new(node.address.parse().unwrap(), node.port);
    let mut read_buffer = [0u8; 4096];
    let mut write_buffer: [u8; 2] = [0; 2];
    write_buffer[0] = 5;
    match stream.write(&write_buffer) {
        Ok(_) => {}
        Err(_) => {
            close_stream(stream, &source_addr);
            return;
        }
    }
    let len = match stream.read(&mut read_buffer) {
        Ok(n) => n,
        Err(_) => {
            close_stream(stream, &source_addr);
            return;
        }
    };
    if !((len > 6)
        && (read_buffer[0] == 5)
        && (read_buffer[1] == 1)
        && (read_buffer[2] == 0)
        && (((read_buffer[3] == 1) && (len == 10)) || ((read_buffer[3] == 4) && (len == 22))))
    {
        eprintln!(
            "WARNING: Connect with {} failed: wrong {} bytes request.",
            source_addr, len
        );
        close_stream(stream, &source_addr);
        return;
    }
    let mut connect_result = match client_server::connect(&node, &read_buffer[0..len], len) {
        Ok(n) => n,
        Err(msg) => {
            eprintln!(
                "WARNING: Connect from {} to {} failed: {}",
                source_addr, node_addr, msg
            );
            close_stream(stream, &source_addr);
            return;
        }
    };
    if (connect_result.len >= 3)
        && (connect_result.reply[0] == 5)
        //&& (connect_result.reply[1] == 0)
        && (connect_result.reply[2] == 0)
    {
        match stream.write(&connect_result.reply[0..connect_result.len]) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("WARNING: Connect with {} aborted.", source_addr);
                close_stream(stream, &source_addr);
                close_connection(connect_result, &source_addr, &node_addr);
                return;
            }
        };
        print!(
            "INFO: Connect from {} to {} success in {} ms. (",
            source_addr,
            node_addr,
            connect_result.delay.as_millis()
        );
        if read_buffer[3] == 1 {
            print!(
                "{}.{}.{}.{}:{}",
                read_buffer[4],
                read_buffer[5],
                read_buffer[6],
                read_buffer[7],
                (read_buffer[8] as u16) * 256 + read_buffer[9] as u16
            );
        } else if read_buffer[3] == 3 {
            for i in 4..len - 2 {
                print!("{}", read_buffer[i] as char);
            }
            print!(
                ":{}",
                (read_buffer[len - 2] as u16) * 256 + read_buffer[len - 1] as u16
            );
        } else if read_buffer[3] == 4 {
            print!(
                "[{}:{}:{}:{}:{}:{}:{}:{}]:{}",
                read_buffer[4] as u16 * 256 + read_buffer[5] as u16,
                read_buffer[6] as u16 * 256 + read_buffer[7] as u16,
                read_buffer[8] as u16 * 256 + read_buffer[9] as u16,
                read_buffer[10] as u16 * 256 + read_buffer[11] as u16,
                read_buffer[12] as u16 * 256 + read_buffer[13] as u16,
                read_buffer[14] as u16 * 256 + read_buffer[15] as u16,
                read_buffer[16] as u16 * 256 + read_buffer[17] as u16,
                read_buffer[18] as u16 * 256 + read_buffer[19] as u16,
                read_buffer[20] as u16 * 256 + read_buffer[21] as u16
            );
        };
        println!(")");
        match forward(&mut connect_result, &mut stream) {
            Ok(_) => {}
            Err(msg) => {
                eprintln!(
                    "WARNING: Forwarding {} to {} failed: {}",
                    source_addr, node_addr, msg
                );
            }
        };
        close_stream(stream, &source_addr);
        close_connection(connect_result, &source_addr, &node_addr);
    } else {
        close_stream(stream, &source_addr);
        let len = connect_result.len;
        close_connection(connect_result, &source_addr, &node_addr);
        eprintln!(
            "WARNING: Connect with {} failed: wrong {} bytes reply from node server.",
            source_addr, len
        );
    }
}

fn close_stream(stream: TcpStream, source_addr: &SocketAddr) {
    match stream.shutdown(Both) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("WARNING: Failed to shutdown TCP stream to {}.", source_addr);
        }
    };
}
fn close_connection(connection: Connection, source_addr: &SocketAddr, node_addr: &SocketAddr) {
    match connection.stream.shutdown(Both) {
        Ok(_) => {}
        Err(_) => {
            eprintln!(
                "WARNING: Failed to shutdown TCP stream to {} (from {}).",
                node_addr, source_addr
            );
        }
    };
}
