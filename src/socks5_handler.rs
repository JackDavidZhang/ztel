use crate::config::ClientConfig;
use crate::{client_server, config};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::net::Shutdown::Both;
use crate::client_server::forward;

pub fn connect(mut stream: TcpStream, node: &config::Node, config: &ClientConfig) {
    let source_addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => {return},
    };
    let node_addr = SocketAddr::new(node.address.parse().unwrap(), node.port);
    let mut read_buffer = [0u8; 4096];
    let mut write_buffer: [u8; 2] = [0; 2];
    write_buffer[0] = 5;
    match stream.write(&write_buffer) {
        Ok(_) => {}
        Err(_) => {
            match stream.shutdown(Both){
                Ok(_) => {},
                Err(_) => {
                    eprintln!("WARNING: Failed to shutdown TCP stream {}.",source_addr);
                }
            }
            return;
        }
    }
    let len = match stream.read(&mut read_buffer) {
        Ok(n) => n,
        Err(_) => {
            match stream.shutdown(Both){
                Ok(_) => {},
                Err(_) => {
                    eprintln!("WARNING: Failed to shutdown TCP stream {}.",source_addr);
                }
            }
            return;
        }
    };
    let mut connect_result = match client_server::connect(&node, &read_buffer[0..len], len) {
        Ok(n) => n,
        Err(msg) => {
            eprintln!(
                "WARNING: Connect from {} to {} failed: {}",
                source_addr, node_addr, msg
            );
            match stream.shutdown(Both){
                Ok(_) => {},
                Err(_) => {
                    eprintln!("WARNING: Failed to shutdown TCP stream {}.",source_addr);
                }
            }
            return;
        }
    };
    if (connect_result.len >= 3)
        && (connect_result.reply[0] == 5)
        && (connect_result.reply[1] == 0)
        && (connect_result.reply[2] == 0)
    {
        match stream.write(&connect_result.reply[0..connect_result.len]) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("WARNING: Connect with {} aborted.", source_addr);
                match stream.shutdown(Both){
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("WARNING: Failed to shutdown TCP stream to {}.",source_addr);
                    }
                };
                match connect_result.stream.shutdown(Both){
                    Ok(_) => {},
                    Err(_) => {
                        eprintln!("WARNING: Failed to shutdown TCP stream to {} (from {}).",node_addr,source_addr);
                    }
                };
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
        
        
        match stream.shutdown(Both){
            Ok(_) => {},
            Err(_) => {
                eprintln!("WARNING: Failed to shutdown TCP stream to {}.",source_addr);
            }
        };
        match connect_result.stream.shutdown(Both){
            Ok(_) => {},
            Err(_) => {
                eprintln!("WARNING: Failed to shutdown TCP stream to {} (from {}).",node_addr,source_addr);
            }
        };
    } else {
        match stream.shutdown(Both){
            Ok(_) => {},
            Err(_) => {
                eprintln!("WARNING: Failed to shutdown TCP stream to {}.",source_addr);
            }
        };
        match connect_result.stream.shutdown(Both){
            Ok(_) => {},
            Err(_) => {
                eprintln!("WARNING: Failed to shutdown TCP stream to {} (from {}).",node_addr,source_addr);
            }
        };
        eprintln!(
            "WARNING: Connect with {} failed: wrong {} bytes reply from node server.",
            source_addr, connect_result.len
        );
    }
}
