use anyhow::{anyhow, Result};
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::config::Config;

pub struct SSHTunnel {
    config: Arc<Config>,
    session: Option<Session>,
}

impl SSHTunnel {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            session: None,
        }
    }
    
    pub async fn connect(&mut self) -> Result<()> {
        log::info!("Connecting to SSH server: {}", self.config.get_ssh_url());
        
        let tcp = TcpStream::connect(self.config.get_ssh_url())?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        
        // Authenticate with private key
        self.authenticate_with_private_key(&mut session)?;
        
        // Check authentication
        if !session.authenticated() {
            return Err(anyhow!("SSH authentication failed"));
        }
        
        log::info!("SSH authentication successful");
        self.session = Some(session);
        Ok(())
    }
    
    fn authenticate_with_private_key(&self, session: &mut Session) -> Result<()> {
        let private_key_path = Path::new(&self.config.ssh_private_key_path);
        
        if let Some(passphrase) = &self.config.ssh_passphrase {
            if !passphrase.is_empty() {
                session.userauth_pubkey_file(
                    &self.config.ssh_username,
                    None,
                    private_key_path,
                    Some(passphrase),
                )?;
            } else {
                session.userauth_pubkey_file(
                    &self.config.ssh_username,
                    None,
                    private_key_path,
                    None,
                )?;
            }
        } else {
            session.userauth_pubkey_file(
                &self.config.ssh_username,
                None,
                private_key_path,
                None,
            )?;
        }
        
        Ok(())
    }
    
    pub fn create_reverse_tunnel(&self) -> Result<()> {
        let session = self.session.as_ref()
            .ok_or_else(|| anyhow!("SSH session not established"))?;
        
        log::info!(
            "Creating reverse tunnel: {} -> {}",
            self.config.get_remote_bind(),
            self.config.get_local_bind()
        );
        
        // Create reverse tunnel (remote port -> local port)
        let listener = session.channel_forward_listen(
            &self.config.remote_host,
            self.config.remote_port,
            None,
        )?;
        
        log::info!("Reverse tunnel established on {}", self.config.get_remote_bind());
        
        // In a real implementation, you'd handle incoming connections here
        // This is simplified for demonstration
        
        Ok(())
    }
    
    pub fn create_forward_tunnel(&self) -> Result<()> {
        let session = self.session.as_ref()
            .ok_or_else(|| anyhow!("SSH session not established"))?;
        
        log::info!(
            "Creating forward tunnel: {} -> {}",
            self.config.get_local_bind(),
            self.config.get_remote_bind()
        );
        
        // This would implement local port forwarding
        // Implementation depends on specific needs
        
        Ok(())
    }
    
    pub async fn health_check(&self) -> bool {
        match &self.session {
            Some(session) => {
                // Simple health check - try to execute a command
                match session.channel_session() {
                    Ok(mut channel) => {
                        if channel.exec("echo healthcheck").is_ok() {
                            channel.wait_close().is_ok()
                        } else {
                            false
                        }
                    }
                    Err(_) => false,
                }
            }
            None => false,
        }
    }
    
    pub fn disconnect(&mut self) {
        if let Some(session) = &self.session {
            session.disconnect(None, "Client disconnecting", None).ok();
        }
        self.session = None;
        log::info!("SSH connection disconnected");
    }
}
