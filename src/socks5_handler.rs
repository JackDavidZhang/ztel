use crate::{client_server, config};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

pub fn connect(mut stream: TcpStream, node: &config::Node) {
    let source_addr = stream.peer_addr().unwrap();
    let node_addr = SocketAddr::new(node.address.parse().unwrap(), node.port);
    let mut read_buffer: [u8; 128] = [0; 128];
    let mut write_buffer: [u8; 2] = [0; 2];
    write_buffer[0] = 5;
    match stream.write(&write_buffer) {
        Ok(_) => {}
        Err(_) => {
            println!("ERROR: Connect with {} failed", source_addr);
            return;
        }
    }
    let len = match stream.read(&mut read_buffer) {
        Ok(n) => n,
        Err(_) => {
            println!("ERROR: Connect with {} failed", source_addr);
            return;
        }
    };
    let connect_result = match client_server::connect(&node, &read_buffer[0..len], len) {
        Ok(n) => n,
        Err(msg) => {
            println!(
                "ERROR: Connect from {} to {} failed: {}",
                source_addr, node_addr, msg
            );
            return;
        }
    };
    match stream.write(&connect_result.0[0..connect_result.1]) {
        Ok(_) => {
            if (connect_result.1 > 6)
                && (connect_result.0[0] == 5)
                && (connect_result.0[1] == 0)
                && (connect_result.0[2] == 0)
                && (((connect_result.0[3] == 1) && (connect_result.1 == 10))
                    || ((connect_result.0[3] == 4) && (connect_result.1 == 22)
                        || (connect_result.0[3] == 3)))
            {
                print!(
                    "Connect from {} to {} success in {} ms. (",
                    source_addr, node_addr, connect_result.2
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
                }
                print!(" -> ");
                if connect_result.0[3] == 1 {
                    println!(
                        "{}.{}.{}.{}:{})",
                        connect_result.0[4],
                        connect_result.0[5],
                        connect_result.0[6],
                        connect_result.0[7],
                        (connect_result.0[8] as u16) * 256 + connect_result.0[9] as u16
                    );
                } else if connect_result.0[3] == 4 {
                    println!(
                        "[{}:{}:{}:{}:{}:{}:{}:{}]:{})",
                        connect_result.0[4] as u16 * 256 + connect_result.0[5] as u16,
                        connect_result.0[6] as u16 * 256 + connect_result.0[7] as u16,
                        connect_result.0[8] as u16 * 256 + connect_result.0[9] as u16,
                        connect_result.0[10] as u16 * 256 + connect_result.0[11] as u16,
                        connect_result.0[12] as u16 * 256 + connect_result.0[13] as u16,
                        connect_result.0[14] as u16 * 256 + connect_result.0[15] as u16,
                        connect_result.0[16] as u16 * 256 + connect_result.0[17] as u16,
                        connect_result.0[18] as u16 * 256 + connect_result.0[19] as u16,
                        connect_result.0[20] as u16 * 256 + connect_result.0[21] as u16
                    );
                }
            } else {
                println!("ERROR: Connect with {} failed", source_addr);
            }
        }
        Err(_) => {
            println!("ERROR: Connect with {} failed", source_addr);
        }
    }
}
