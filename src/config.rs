use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value = "./config/dev.toml")]
    pub config: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub general: General,
    pub database: Database,
    pub server: Server,
    pub oauth: OAuth,
}

impl Config {
    pub fn load(source_file: String) -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = fs::read_to_string(source_file)?;
        let app_config: Self = toml::from_str(&config_file)?;
        Ok(app_config)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct General {
    pub client_addr: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub connection: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub port: u16,
    pub address: String,
    pub tls_key: String,
    pub tls_cert: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth {
    pub github_client_id: String,
    pub github_secret_id: String,
}
