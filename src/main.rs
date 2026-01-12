mod auth;
mod game;
mod handlers;
mod models;
mod state;

use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, error};
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
    let host = env::var("GAME_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("GAME_SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("GAME_SERVER_PORT must be a valid port number");

    let addr = format!("{}:{}", host, port);
    let socket_addr: SocketAddr = addr.parse().expect("Unable to parse socket address");

    // Initialize global state
    let state = state::AppState::new();

    info!("🎮 QCXIS Game Server starting...");
    info!("📡 Listening on: ws://{}", socket_addr);

    // Start TCP listener
    let listener = TcpListener::bind(&socket_addr)
        .await
        .expect("Failed to bind");

    info!("✅ Game server ready for connections!");

    // Accept connections
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
