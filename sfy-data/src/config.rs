use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub address: SocketAddr,
    pub database: Option<PathBuf>,
    pub tokens: Vec<String>,
}

impl Config {
    pub fn default() -> Config {
        Config {
            address: "0.0.0.0:3000".parse().unwrap(),
            database: None,
            tokens: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn test_config() -> Config {
        Config {
            address: "0.0.0.0:3000".parse().unwrap(),
            database: None,
            tokens: vec!["token1".into()],
        }
    }

    pub fn from_path<P: AsRef<Path>>(p: P) -> Config {
        let p = p.as_ref();

        let f = fs::read_to_string(p).expect("could not read config file");
        toml::from_str(&f).expect("could not parse config file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_conf() {
        let c = fs::read_to_string("sfy-data.toml").unwrap();
        let c: Config = toml::from_str(&c).unwrap();
        println!("{:#?}", c);
    }

    #[test]
    fn load_test_conf() {
        let c = Config::test_config();
    }
}
