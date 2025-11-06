mod config;
mod ssh_tunnel;

use anyhow::Result;
use config::Config;
use ssh_tunnel::SSHTunnel;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    log::info!("Starting SSH Forwarder...");
    
    // Load configuration
    let config = Arc::new(Config::from_env()?);
    log::debug!("Configuration loaded: {:?}", config);
    
    // Validate SSH private key exists
    if !std::path::Path::new(&config.ssh_private_key_path).exists() {
        return Err(anyhow::anyhow!(
            "SSH private key not found at: {}", 
            config.ssh_private_key_path
        ));
    }
    
    let mut tunnel_manager = SSHTunnel::new(config.clone());
    
    // Main connection loop
    loop {
        match tunnel_manager.connect().await {
            Ok(()) => {
                log::info!("SSH connection established successfully");
                
                // Create reverse tunnel
                if let Err(e) = tunnel_manager.create_reverse_tunnel() {
                    log::error!("Failed to create reverse tunnel: {}", e);
                    tunnel_manager.disconnect();
                    sleep(Duration::from_secs(config.reconnect_delay)).await;
                    continue;
                }
                
                // Health check loop
                let mut health_check_failures = 0;
                const MAX_HEALTH_CHECK_FAILURES: u32 = 3;
                
                while health_check_failures < MAX_HEALTH_CHECK_FAILURES {
                    sleep(Duration::from_secs(config.health_check_interval)).await;
                    
                    if tunnel_manager.health_check().await {
                        health_check_failures = 0;
                        log::debug!("Health check passed");
                    } else {
                        health_check_failures += 1;
                        log::warn!("Health check failed ({}/{})", health_check_failures, MAX_HEALTH_CHECK_FAILURES);
                    }
                }
                
                log::error!("Too many health check failures, reconnecting...");
                tunnel_manager.disconnect();
            }
            Err(e) => {
                log::error!("Failed to connect to SSH server: {}", e);
                log::info!("Reconnecting in {} seconds...", config.reconnect_delay);
                sleep(Duration::from_secs(config.reconnect_delay)).await;
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Shutdown signal received");
}
