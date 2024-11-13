use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize,Clone,Serialize)]
pub struct ClientConfig {
    pub servers: Vec<Server>,
    pub(crate) node: Node
}
#[derive(Deserialize,Clone,Serialize)]
pub struct Server {
    protocol: String,
    pub address: String,
    pub port: u16,
}

#[derive(Deserialize,Serialize,Clone)]
pub struct Node{
    pub(crate) address: String,
    pub(crate) port: u16
}

pub enum Character{
    Server,Client
}

pub fn load_config(ch:&Character) -> ClientConfig {
    match ch{
        Character::Client => {
            let config_file = File::open("config.toml");
            let config_result = match config_file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    let config: ClientConfig = toml::from_str(&contents).unwrap();
                    config
                }
                Err(error) => match error.kind() {
                    ErrorKind::NotFound => {
                        let mut config_file = File::create("config.toml").expect("Failed to create config.toml");
                        let default_config = ClientConfig {
                            servers : vec![Server{
                                protocol: "socks5".to_string(),
                                address : "127.0.0.1".to_string(),
                                port: 8080,
                            }],
                            node: Node {
                                address : "127.0.0.1".to_string(),
                                port: 10808
                            },
                        };
                        let default_config_str = toml::to_string(&default_config).unwrap();
                        config_file.write(default_config_str.as_bytes()).unwrap();
                        return default_config;
                    }
                    other_error => {panic!("An unexpected error occurred {:?}", other_error)}
                }
            };
            let mut vailed_config: ClientConfig = ClientConfig { servers: vec![], node: config_result.node };
            let mut _socks5_flag = false;
            for i in config_result.servers.iter() {
                if i.protocol != "socks5" {
                    println!("Unsupported protocol {}, ignored.", i.protocol);
                }else {
                    vailed_config.servers.push(i.clone());
                    _socks5_flag = true;
                }
            }
            if vailed_config.servers.len() < 1 {
                panic!("No available config found!");
            }
            if vailed_config.servers.len() > 1 {
                panic!("More than one available config found!");
            }
            vailed_config
        }
        Character::Server => {

            let config_file = File::open("config.toml");
            let config_result = match config_file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();
                    let config: ClientConfig = toml::from_str(&contents).unwrap();
                    config
                }
                Err(error) => match error.kind() {
                    ErrorKind::NotFound => {
                        let mut config_file = File::create("config.toml").expect("Failed to create config.toml");
                        let default_config = ClientConfig {
                            servers : vec![Server{
                                protocol: "socks5".to_string(),
                                address : "127.0.0.1".to_string(),
                                port: 8080,
                            }],
                            node: Node {
                                address : "127.0.0.1".to_string(),
                                port: 10808
                            },
                        };
                        let default_config_str = toml::to_string(&default_config).unwrap();
                        config_file.write(default_config_str.as_bytes()).unwrap();
                        return default_config;
                    }
                    other_error => {panic!("An unexpected error occurred {:?}", other_error)}
                }
            };
            let mut vailed_config: ClientConfig = ClientConfig { servers: vec![], node: config_result.node };
            let mut _socks5_flag = false;
            for i in config_result.servers.iter() {
                if i.protocol != "socks5" {
                    println!("Unsupported protocol {}, ignored.", i.protocol);
                }else {
                    vailed_config.servers.push(i.clone());
                    _socks5_flag = true;
                }
            }
            if vailed_config.servers.len() < 1 {
                panic!("No available config found!");
            }
            if vailed_config.servers.len() > 1 {
                panic!("More than one available config found!");
            }
            vailed_config
        }
    }
}