use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    // SSH Configuration
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_username: String,
    pub ssh_private_key_path: String,
    pub ssh_passphrase: Option<String>,
    
    // Forwarding Configuration
    pub local_host: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    
    // Application Configuration
    pub health_check_interval: u64,
    pub reconnect_delay: u64,
    pub log_level: String,
    
    // Web Interface Configuration
    pub web_interface: String,
    pub web_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();
        
        // Add environment source
        cfg = cfg.add_source(config::Environment::default());
        
        let cfg = cfg.build()?;
        
        cfg.try_deserialize()
    }
    
    pub fn get_ssh_url(&self) -> String {
        format!("{}:{}", self.ssh_host, self.ssh_port)
    }
    
    pub fn get_local_bind(&self) -> String {
        format!("{}:{}", self.local_host, self.local_port)
    }
    
    pub fn get_remote_bind(&self) -> String {
        format!("{}:{}", self.remote_host, self.remote_port)
    }
}
