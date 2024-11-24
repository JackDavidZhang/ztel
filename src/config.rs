use log::{error, warn};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::IpAddr;

#[derive(Deserialize, Clone, Serialize)]
pub struct ClientConfig {
    pub listener: Listener,
    pub node: Node,
    //pub key: String,
}
#[derive(Deserialize, Clone, Serialize)]
pub struct Listener {
    pub address: String,
    pub port: u16,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Node {
    pub address: String,
    pub port: u16,
    pub passwd: String,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct ServerConfig {
    pub listener: Listener,
    pub passwd: String,
}
pub fn load_client_config() -> Result<ClientConfig, &'static str> {
    let config_file = File::open("config_client.toml");
    let config_result = match config_file {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => (),
                Err(_) => return Err("Cannot read config_client.toml."),
            }
            let config: ClientConfig = match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    error!("Cannot resolve config_client.toml: {}", e.message());
                    return Err("Cannot read config_client.toml.");
                }
            };
            config
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                warn!("Config file not found, creating default config.");
                let mut config_file = match File::create("config_client.toml") {
                    Ok(file) => file,
                    Err(_) => return Err("Cannot create config_client.toml."),
                };
                let default_config = ClientConfig {
                    listener: Listener {
                        address: "127.0.0.1".to_string(),
                        port: 8080,
                    },
                    node: Node {
                        address: "127.0.0.1".to_string(),
                        port: 10086,
                        passwd: "password".to_string(),
                    },
                };
                let default_config_str = toml::to_string(&default_config).unwrap();
                match config_file.write(default_config_str.as_bytes()) {
                    Ok(_) => (),
                    Err(_) => return Err("Cannot write to config_client.toml."),
                };
                default_config
            }
            _ => {
                return Err("Cannot open config_client.toml.");
            }
        },
    };
    match config_result.listener.address.parse::<IpAddr>() {
        Ok(_) => {}
        Err(_) => {
            return Err("Illegal listener address.");
        }
    }
    match config_result.node.address.parse::<IpAddr>() {
        Ok(_) => Ok(config_result),
        Err(_) => Err("Illegal node address."),
    }
}

pub fn load_server_config() -> Result<ServerConfig, &'static str> {
    let config_file = File::open("config_server.toml");
    let config_result = match config_file {
        Ok(mut file) => {
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => (),
                Err(_) => return Err("Cannot read config_server.toml."),
            }
            let config: ServerConfig = match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    error!("Cannot resolve config_server.toml: {}", e.message());
                    return Err("Cannot read config_server.toml.");
                }
            };
            config
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                warn!("WARNING: Config file not found, creating default config.");
                let mut config_file = match File::create("config_server.toml") {
                    Ok(file) => file,
                    Err(_) => return Err("Cannot create config_server.toml."),
                };
                let default_config = ServerConfig {
                    listener: Listener {
                        address: "127.0.0.1".to_string(),
                        port: 10086,
                    },
                    passwd: "password".to_string(),
                };
                let default_config_str = toml::to_string(&default_config).unwrap();
                match config_file.write(default_config_str.as_bytes()) {
                    Ok(_) => (),
                    Err(_) => return Err("Cannot write to config_server.toml."),
                };
                default_config
            }
            _ => {
                return Err("Cannot open config_server.toml.");
            }
        },
    };
    match config_result.listener.address.parse::<IpAddr>() {
        Ok(_) => Ok(config_result),
        Err(_) => Err("Illegal listener address."),
    }
}
