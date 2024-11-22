use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use tokio::io::{copy, split, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::spawn;
use ztel::config;

#[tokio::main]
async fn main() {
    let config = config::load_server_config().unwrap();
    let mut full_address = String::new();
    full_address.push_str(&config.listener.address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.listener.port.to_string());
    let listener = match TcpListener::bind(&full_address).await {
        Ok(listener) => listener,
        Err(_) => panic!("unavailable address {}", full_address),
    };
    println!("Listening on {}", full_address);
    loop {
        let (stream, listen_socket) = match listener.accept().await {
            Ok(a) => a,
            Err(_) => {
                eprintln!("ERROR: Cannot accept connection, exiting.");
                break;
            }
        };
        spawn(async move {
            let mut read_buffer = [0; 128];
            stream.readable().await.unwrap();
            let len = match stream.try_read(&mut read_buffer) {
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
                    return;
                }
                let mut diststream = match TcpStream::connect(addr).await {
                    Ok(stream) => stream,
                    Err(_) => {
                        eprintln!(
                            "Cannot connect to {} from {}.",
                            addr,
                            stream.peer_addr().unwrap()
                        );
                        let write_buff: [u8; 10] =
                            [5, 1, 0, 1, 127, 0, 0, 1, (0 / 256) as u8, (0 % 256) as u8];
                        stream.writable().await.unwrap();
                        stream.try_write(&write_buff).unwrap();
                        return;
                    }
                };
                println!(
                    "Connect from {} to {} established",
                    stream.peer_addr().unwrap(),
                    addr
                );
                let write_buff: [u8; 10] = [5, 0, 0, 1, 0, 0, 0, 0, 0, 0];
                stream.try_write(&write_buff).unwrap();

                let (mut readstream, mut writestream) = split(stream);
                let (mut serverreadstream, mut serverwritestream) = split(diststream);
                let read_to_server = read_to_write(&mut readstream, &mut serverwritestream);
                let write_to_client = read_to_write(&mut serverreadstream, &mut writestream);
                select! {
                    r1 = read_to_server => {},
                    r2 = write_to_client => {},
                }
                writestream.flush().await.unwrap();
                writestream.shutdown().await.unwrap();
                serverwritestream.flush().await.unwrap();
                serverwritestream.shutdown().await.unwrap();
            }
        });
    }
}
async fn read_to_write(
    readstream: &mut ReadHalf<TcpStream>,
    writestream: &mut WriteHalf<TcpStream>,
) -> Result<(), &'static str> {
    let mut buf = [0u8; 128];
    println!("Begin forwarding");
    loop {
        let len = match readstream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        match writestream.write(&buf[..len]).await {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    writestream.flush().await.unwrap();
    writestream.shutdown().await.unwrap();
    Ok(())
}
