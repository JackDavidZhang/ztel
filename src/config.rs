use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::hash::Hash;
use std::io::{ErrorKind, Read, Write};

#[derive(Deserialize, Clone, Serialize)]
pub struct ClientConfig {
    pub listeners: Vec<Listener>,
    pub node: Node,
    //pub key: String,
}
#[derive(Deserialize, Clone, Serialize)]
pub struct Listener {
    protocol: String,
    pub address: String,
    pub port: u16,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Node {
    pub(crate) address: String,
    pub(crate) port: u16,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct ServerConfig {
    pub listeners: Vec<Listener>,
    //pub key: String,
}

pub fn load_client_config() -> ClientConfig {
    let config_file = File::open("config_client.toml");
    let config_result = match config_file {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let config: ClientConfig = toml::from_str(&contents).unwrap();
            config
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("WARNING: Config file not found, create default config.");
                let mut config_file =
                    File::create("config_client.toml").expect("Failed to create config.toml");
                let default_config = ClientConfig {
                    listeners: vec![Listener {
                        protocol: "socks5".to_string(),
                        address: "127.0.0.1".to_string(),
                        port: 8080,
                    }],
                    node: Node {
                        address: "127.0.0.1".to_string(),
                        port: 10086,
                    },
                    //key: "114514".to_string(),
                };
                let default_config_str = toml::to_string(&default_config).unwrap();
                config_file.write(default_config_str.as_bytes()).unwrap();
                return default_config;
            }
            other_error => {
                panic!("An unexpected error occurred {:?}", other_error)
            }
        },
    };
    let mut vailed_config: ClientConfig = ClientConfig {
        listeners: vec![],
        node: config_result.node,
        //key: config_result.key,
    };
    let mut _socks5_flag = false;
    for i in config_result.listeners.iter() {
        if i.protocol != "socks5" {
            println!("Unsupported protocol {}, ignored.", i.protocol);
        } else {
            vailed_config.listeners.push(i.clone());
            _socks5_flag = true;
        }
    }
    if vailed_config.listeners.len() < 1 {
        panic!("No available config found!");
    }
    if vailed_config.listeners.len() > 1 {
        panic!("More than one available config found!");
    }
    vailed_config
}

pub fn load_server_config() -> ServerConfig {
    let config_file = File::open("config_server.toml");
    let config_result = match config_file {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let config: ServerConfig = toml::from_str(&contents).unwrap();
            config
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("WARNING: Config file not found, create default config.");
                let mut config_file =
                    File::create("config_server.toml").expect("Failed to create config.toml");
                let default_config = ServerConfig {
                    listeners: vec![Listener {
                        protocol: "socks5".to_string(),
                        address: "127.0.0.1".to_string(),
                        port: 10086,
                    }],
                };
                let default_config_str = toml::to_string(&default_config).unwrap();
                config_file.write(default_config_str.as_bytes()).unwrap();
                return default_config;
            }
            other_error => {
                panic!("An unexpected error occurred {:?}", other_error)
            }
        },
    };
    let mut vailed_config: ServerConfig = ServerConfig { listeners: vec![] };
    let mut _socks5_flag = false;
    for i in config_result.listeners.iter() {
        if i.protocol != "socks5" {
            println!("Unsupported protocol {}, ignored.", i.protocol);
        } else {
            vailed_config.listeners.push(i.clone());
            _socks5_flag = true;
        }
    }
    if vailed_config.listeners.len() < 1 {
        panic!("No available config found!");
    }
    if vailed_config.listeners.len() > 1 {
        panic!("More than one available config found!");
    }
    vailed_config
}

// fn key_string_to_bin(key:&String) -> [u8;16]{
//     key.hash();
// }