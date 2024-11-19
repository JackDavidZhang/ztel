use std::io::Read;
use std::net::{SocketAddr, TcpListener};
use ztel::config::load_client_config;
use ztel::socks5_handler::connect;

fn main() {
    let config = match load_client_config(){
        Ok(c) => c,
        Err(msg) => {
            eprintln!("ERROR: Cannot load config: {}",msg);
            return ;
        }
    };
    let full_address = SocketAddr::new(match config.listener.address.parse(){
        Ok(a) => a,
        Err(_) => {
            eprintln!("ERROR: Unavailable listener address, exiting.");
            return;}
    },config.listener.port);
    let listener = match TcpListener::bind(&full_address) {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("ERROR: Cannot listen on {}, exiting.",full_address);
            return;
        }
    };
    println!("Listening on {}", full_address);
    for stream in listener.incoming() {
        let mut read_buff: [u8; 4096] = [0; 4096];
        let mut stream = match stream{
            Ok(a) => a,
            Err(_) => continue
        };
        let len = match stream.read(&mut read_buff){
            Ok(a) => a,
            Err(_) => continue
        };
        if (len==3)&&(read_buff[0] == 5)&&(read_buff[1] == 1)&&(read_buff[2] == 1)&&(read_buff[3] == 0) {
            connect(stream, &config.node,&config);
        }
    }
}
