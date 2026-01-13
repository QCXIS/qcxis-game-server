mod auth;
mod game;
mod handlers;
mod http_server;
mod metrics;
mod models;
mod state;

use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get configuration from environment
    let ws_host = env::var("GAME_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let ws_port = env::var("GAME_SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("GAME_SERVER_PORT must be a valid port number");

    let http_port = env::var("HTTP_SERVER_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()
        .expect("HTTP_SERVER_PORT must be a valid port number");

    let ws_addr = format!("{}:{}", ws_host, ws_port);
    let ws_socket_addr: SocketAddr = ws_addr.parse().expect("Unable to parse WebSocket socket address");

    let http_addr = format!("{}:{}", ws_host, http_port);
    let http_socket_addr: SocketAddr = http_addr.parse().expect("Unable to parse HTTP socket address");

    // Initialize global state
    let state = state::AppState::new();

    info!("🎮 QCXIS Game Server starting...");
    info!("📡 WebSocket listening on: ws://{}", ws_socket_addr);
    info!("📊 HTTP Status listening on: http://{}", http_socket_addr);

    // Start HTTP server for metrics/status
    let http_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = http_server::start_http_server(http_socket_addr, http_state).await {
            error!("HTTP server error: {}", e);
        }
    });

    // Start WebSocket TCP listener
    let listener = TcpListener::bind(&ws_socket_addr)
        .await
        .expect("Failed to bind WebSocket listener");

    info!("✅ Game server ready for connections!");

    // Accept WebSocket connections
    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            info!("New connection from: {}", addr);
            if let Err(e) = handlers::handle_connection(stream, state).await {
                error!("Error handling connection: {}", e);
            }
        });
    }
}
