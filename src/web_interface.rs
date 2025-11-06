use axum::{
    routing::get,
    Router,
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::config::Config;

pub struct AppState {
    pub config: Arc<Config>,
    pub is_connected: bool,
}

pub async fn start_web_interface(config: Arc<Config>) -> Result<(), anyhow::Error> {
    let state = Arc::new(AppState {
        config,
        is_connected: false,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/status", get(status))
        .route("/config", get(get_config))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", &config.web_interface, config.web_port)
    ).await?;
    
    log::info!("Web interface running on http://{}:{}", config.web_interface, config.web_port);
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> &'static str {
    "SSH Forwarder is running"
}

async fn status(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "status": if state.is_connected { "connected" } else { "disconnected" },
        "ssh_host": state.config.ssh_host,
        "local_bind": state.config.get_local_bind(),
        "remote_bind": state.config.get_remote_bind(),
    }))
}

async fn get_config(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "ssh_host": state.config.ssh_host,
        "ssh_port": state.config.ssh_port,
        "local_host": state.config.local_host,
        "local_port": state.config.local_port,
        "remote_host": state.config.remote_host,
        "remote_port": state.config.remote_port,
    }))
}
