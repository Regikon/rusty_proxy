use std::collections::HashMap;
use std::env;
use thiserror::Error;

pub struct Config {
    proxy_host: String,
    proxy_port: u16,
    ssl_certificate: String,
    ssl_key: String,
    mongodb_uri: String,
}

mod rusty_env {
    pub const PROXY_HOST: &str = "RUSTY_PROXY_HOST";
    pub const PROXY_PORT: &str = "RUSTY_PROXY_PORT";
    pub const SSL_CERTIFICATE: &str = "RUSTY_PROXY_SSL_CERTIFICATE";
    pub const SSL_PRIVATE_KEY: &str = "RUSTY_PROXY_SSL_PRIVATE_KEY";
    pub const MONGO_DB_CONNECTION_URL: &str = "MONGO_DB_CONNECTION_URL";

    pub const ALL_PARAMS: [&str; 5] = [
        PROXY_HOST,
        PROXY_PORT,
        SSL_CERTIFICATE,
        SSL_PRIVATE_KEY,
        MONGO_DB_CONNECTION_URL,
    ];
}

impl Config {
    pub fn proxy_host(&self) -> &String {
        &self.proxy_host
    }

    pub fn proxy_port(&self) -> u16 {
        self.proxy_port
    }

    pub fn ssl_certificate(&self) -> &String {
        &self.ssl_certificate
    }

    pub fn ssl_key(&self) -> &String {
        &self.ssl_key
    }

    pub fn mongodb_uri(&self) -> &String {
        &self.mongodb_uri
    }

    pub fn from_env() -> Result<Self, ConfigParsingError> {
        let mut raw_config = HashMap::new();
        for &param_name in rusty_env::ALL_PARAMS.iter() {
            let param_value = env::var(param_name)
                .map_err(|_| ConfigParsingError::MissingParameter(param_name.to_string()))?;
            raw_config.insert(param_name, param_value);
        }
        return Ok(Config {
            proxy_host: raw_config.get(rusty_env::PROXY_HOST).unwrap().clone(),
            proxy_port: raw_config
                .get(rusty_env::PROXY_PORT)
                .unwrap()
                .parse()
                .map_err(|_| ConfigParsingError::InvalidParameterType {
                    param_name: rusty_env::PROXY_PORT.to_string(),
                    expected: "u16".to_string(),
                })?,
            ssl_certificate: raw_config.get(rusty_env::SSL_CERTIFICATE).unwrap().clone(),
            ssl_key: raw_config.get(rusty_env::SSL_PRIVATE_KEY).unwrap().clone(),
            mongodb_uri: raw_config
                .get(rusty_env::MONGO_DB_CONNECTION_URL)
                .unwrap()
                .clone(),
        });
    }
}

#[derive(Error, Debug)]
pub enum ConfigParsingError {
    #[error("invalid type of parameter {param_name:?}, expected {expected:?}")]
    InvalidParameterType {
        param_name: String,
        expected: String,
    },

    #[error("missing parameter {0:?}")]
    MissingParameter(String),
}
