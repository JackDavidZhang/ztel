use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use ztel::config;

fn main() {
    let config = config::load_server_config();
    let mut full_address = String::new();
    full_address.push_str(&config.listeners[0].address.as_str());
    full_address.push_str(":");
    full_address.push_str(&config.listeners[0].port.to_string());
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
        if (len > 6)
            && (read_buffer[0] == 5)
            && (read_buffer[2] == 0)
            && (((read_buffer[3] == 1) && (len == 10)) || ((read_buffer[3] == 4) && (len == 12)))
        {
            if read_buffer[3] == 1 {
                println!(
                    "Connect from {} to {}.{}.{}.{}:{} established",
                    stream.peer_addr().unwrap(),
                    read_buffer[4],
                    read_buffer[5],
                    read_buffer[6],
                    read_buffer[7],
                    read_buffer[8] as i32 * 256 + read_buffer[9] as i32
                );
            } else if read_buffer[3] == 3 {
                print!("Connect from {} to ", stream.peer_addr().unwrap());
                for i in 4..len - 2 {
                    print!("{}", read_buffer[i] as char);
                }
                println!(
                    ":{} established",
                    read_buffer[len - 2] as i32 * 256 + read_buffer[len - 1] as i32
                );
            } else if read_buffer[3] == 4 {
                println!(
                    "Connect from {} to {}:{}:{}:{}:{}:{}::{} established",
                    stream.peer_addr().unwrap(),
                    read_buffer[4],
                    read_buffer[5],
                    read_buffer[6],
                    read_buffer[7],
                    read_buffer[8],
                    read_buffer[9],
                    read_buffer[10] as i32 * 256 + read_buffer[11] as i32
                );
            } else {
                println!("Unknown Address {}", read_buffer[3]);
            }
            let write_buff: [u8; 10] = [
                5,
                0,
                0,
                1,
                127,
                0,
                0,
                1,
                (10086 / 126) as u8,
                (10086 % 126) as u8,
            ];
            stream.write(&write_buff).unwrap();
        } else {
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
    // for stream in listener.incoming() {
    //     let mut stream = stream.unwrap();
    //     let mut buff = [0; 2048];
    //     let mut len = stream.read(&mut buff).unwrap();
    //     if buff[0] == 5 {
    //         if (len > 2) && (len == (2 + buff[1]) as usize) {
    //             print!(
    //                 "SOCKS5 Receive 1 from {}, ACCEPT {} METHOD(s): ",
    //                 stream.peer_addr().unwrap().to_string(),
    //                 buff[1]
    //             );
    //             for i in 2..len {
    //                 print!("{} ", buff[i]);
    //             }
    //             println!();
    //             let write_buff: [u8; 2] = [5, 0];
    //             stream.write(&write_buff).unwrap();
    //             println!("SOCKS5 Reply to {}", stream.peer_addr().unwrap().to_string());
    //         }
    //         len = stream.read(&mut buff).unwrap();
    //         if len>6{
    //             print!("SOCKS5 Receive 2 from {}, DATA: ", stream.peer_addr().unwrap().to_string());
    //             if buff[1]==1{
    //                 print!("CONNECT ");
    //             }else if buff[1]==2{
    //                 print!("BIND ");
    //             }else if buff[1]==3{
    //                 print!("UDP ");
    //             }else{
    //                 print!("Unknown{} ", buff[1]);
    //             }
    //             if buff[3]==1
    //             {
    //                 print!("IPv4 Address {}.{}.{}.{} Port {}",buff[4],buff[5],buff[6],buff[7],buff[8] as i32*256+buff[9] as i32);
    //             }else if buff[3]==3{
    //                 print!("Host Address ");
    //                 for i in 4..len-2{
    //                     print!("{}", buff[i] as char);
    //                 }
    //                 print!(" Port {}",buff[len-2] as i32*256+buff[len-1] as i32);
    //             }else if buff[3]==4{
    //                 print!("IPv6 Address {}:{}:{}:{}:{}:{} Port {}",buff[4],buff[5],buff[6],buff[7],buff[8],buff[9],buff[10] as i32*256+buff[11] as i32);
    //             }else{
    //                 print!("Unknown Address {}", buff[3]);
    //             }
    //             println!();
    //             let write_buff:[u8;10] = [5,0,0,1,127,0,0,1,(8080/126) as u8,(8080%126) as u8];
    //             stream.write(&write_buff).unwrap();
    //             println!("SOCKS5 Reply to {}", stream.peer_addr().unwrap().to_string());
    //         }
    //         len = stream.read(&mut buff).unwrap();
    //         for i in 0..len-4{
    //             if buff[i]==13&&buff[i-1]==10{
    //                 continue;
    //             }
    //             print!("{}", buff[i] as char);
    //         }
    //         let write_buff = "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes();
    //         stream.write(&write_buff).unwrap();
    //     }
    // }
}
