use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        let len = stream.read(&mut buffer).unwrap();
        for i in 0..len {
            print!("{}", &buffer[i]);
        }
        println!();
    }
}