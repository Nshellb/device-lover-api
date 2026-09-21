use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::num::ParseIntError;

#[derive(Clone, Debug)]
pub struct Settings {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid HOST value: {0}")]
    InvalidHost(#[from] AddrParseError),
    #[error("invalid PORT value: {0}")]
    InvalidPort(#[from] ParseIntError),
    #[error("DATABASE_URL not set")]
    DatabaseUrlNotSet,
    #[error("JWT_SECRET not set")]
    JwtSecretNotSet,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| ConfigError::DatabaseUrlNotSet)?;
        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| ConfigError::JwtSecretNotSet)?;

        let host = match std::env::var("HOST") {
            Ok(value) => value.parse()?,
            Err(_) => IpAddr::from([0, 0, 0, 0]),
        };

        let port = match std::env::var("PORT") {
            Ok(value) => value.parse()?,
            Err(_) => 3000,
        };

        Ok(Self {
            host,
            port,
            database_url,
            jwt_secret,
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}
