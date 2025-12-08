use crate::core::configure::app::get_static_dir;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct SecretConfig {
    pub private_access_key: PathBuf,
    pub public_access_key: PathBuf,
    pub private_refresh_key: PathBuf,
    pub public_refresh_key: PathBuf,
}

impl SecretConfig {
    pub fn read_private_access_key(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(get_static_dir().unwrap().join("secret_key/private_access_rsa_key.pem"))
    }

    pub fn read_public_access_key(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(get_static_dir().unwrap().join("secret_key/public_access_rsa_key.pem"))
    }

    pub fn read_private_refresh_key(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(get_static_dir().unwrap().join("secret_key/private_refresh_rsa_key.pem"))
    }

    pub fn read_public_refresh_key(&self) -> Result<String, std::io::Error> {
        fs::read_to_string(get_static_dir().unwrap().join("secret_key/public_refresh_rsa_key.pem"))
    }
}
