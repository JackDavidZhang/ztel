use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use ztel::config;

fn main() {
    let config = config::load_server_config().unwrap();
    let mut full_address = String::new();
    full_address.push_str(&config.listener.address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.listener.port.to_string());
    let listener = match TcpListener::bind(&full_address) {
        Ok(listener) => listener,
        Err(_) => panic!("unavailable address {}", full_address),
    };
    println!("Listening on {}", full_address);
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut read_buffer = [0; 128];
        let len = match stream.read(&mut read_buffer) {
            Ok(len) => len,
            Err(_) => panic!("read failed"),
        };
        let addr: SocketAddr;
        if (len > 6)
            && (read_buffer[0] == 5)
            && (read_buffer[2] == 0)
            && (((read_buffer[3] == 1) && (len == 10))
                || ((read_buffer[3] == 4) && (len == 22))
                || (read_buffer[3] == 3))
        {
            if read_buffer[3] == 1 {
                addr = SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(
                        read_buffer[4],
                        read_buffer[5],
                        read_buffer[6],
                        read_buffer[7],
                    )),
                    read_buffer[8] as u16 * 256 + read_buffer[9] as u16,
                );
            } else if read_buffer[3] == 4 {
                addr = SocketAddr::new(
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
                println!("Unknown Address {}", read_buffer[3]);
                continue;
            }
            let mut diststream = match TcpStream::connect(addr) {
                Ok(stream) => stream,
                Err(_) => {
                    eprintln!(
                        "Cannot connect to {} from {}.",
                        addr,
                        stream.peer_addr().unwrap()
                    );
                    let write_buff: [u8; 10] =
                        [5, 1, 0, 1, 127, 0, 0, 1, (0 / 256) as u8, (0 % 256) as u8];
                    stream.write(&write_buff).unwrap();
                    continue;
                }
            };
            diststream
                .set_read_timeout(Some(std::time::Duration::new(1, 0)))
                .unwrap();
            diststream
                .set_write_timeout(Some(std::time::Duration::new(1, 0)))
                .unwrap();
            println!(
                "Connect from {} to {} established",
                stream.peer_addr().unwrap(),
                addr
            );
            let write_buff: [u8; 10] = [5, 0, 0, 1, 127, 0, 0, 1, (0 / 256) as u8, (0 % 256) as u8];
            stream.write(&write_buff).unwrap();
            let mut read_buff = [0u8; 1024 * 100];
            let mut len = match stream.read(&mut read_buff) {
                Ok(len) => len,
                Err(_) => continue,
            };
            if len == 0 {
                continue;
            }
            diststream.write(&read_buff[0..len]).unwrap();
            len = match diststream.read(&mut read_buff) {
                Ok(len) => len,
                Err(_) => continue,
            };
            if len == 0 {
                continue;
            }
            stream.write(&read_buff[0..len]).unwrap();
        } else {
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
}
