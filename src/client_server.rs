use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Instant;
use crate::config::Node;

pub fn connect(node:& Node,request: &[u8],len : usize)->Result<([u8;256],usize),&'static str>{
    if (len > 6)&&(request[0]==5)&&(request[2]==0)&&(((request[3]==1)&&(len==10))||((request[3]==4)&&(len==12)))
    {
        let address = node.address.clone()+":"+ &*node.port.to_string();
        let mut stream:TcpStream =
            match TcpStream::connect(&address){
                Ok(tcpstream)=> tcpstream ,
                Err(_)=>{return Err("Field to connect to node server.");}
            };
        match stream.write(request){
            Ok(_)=>{}
            Err(_)=>{return Err("Error writing to node server.");}
        }
        let now = Instant::now();
        let mut read_buff:[u8;256] = [0;256];
        let len =
            match stream.read(&mut read_buff){
                Ok(len)=>len,
                Err(_)=>{return Err("Error reading from node server.");}
            };
        let end = now.elapsed().as_millis();
        //if (len > 6)&&(read_buff[0]==5)&&(read_buff[1]==0)&&(read_buff[2]==0)&&(((read_buff[3]==1)&&(len==10))||((read_buff[3]==4)&&(len==12)))
        {
            println!("Connected to node server {address} in {end} ms.");
        }
        Ok((read_buff,len))
    }else{
        Err("Unaccepted connect request.")
    }
}