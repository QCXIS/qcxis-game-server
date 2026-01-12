use crate::auth;
use crate::game;
use crate::models::{ClientMessage, Player, ServerMessage};
use crate::state::{AppState, PlayerConnection};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn handle_connection(stream: TcpStream, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let mut player_id: Option<String> = None;
    let mut game_id: Option<String> = None;
    let mut authenticated = false;

    // Spawn task to handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            ClientMessage::Auth { token, game_id: gid, game_code, difficulty, text, host_id } => {
                                match auth::verify_token(&token) {
                                    Ok(claims) => {
                                        let pid = Uuid::new_v4().to_string();
                                        player_id = Some(pid.clone());
                                        game_id = Some(gid.clone());
                                        authenticated = true;

                                        // Create game if it doesn't exist
                                        if state.get_game(&gid).is_none() {
                                            let new_game = crate::models::GameState::new(
                                                gid.clone(),
                                                game_code,
                                                difficulty,
                                                text,
                                                host_id,
                                            );
                                            state.add_game(new_game);
                                            info!("Created game {} in state", gid);
                                        }

                                        let player = Player::new(
                                            pid.clone(),
                                            claims.user_id.clone(),
                                            claims.username.clone(),
                                        );

                                        // Add player connection
                                        let connection = PlayerConnection {
                                            player_id: pid.clone(),
                                            user_id: claims.user_id,
                                            game_id: gid.clone(),
                                            tx: tx.clone(),
                                        };
                                        state.add_player_connection(pid.clone(), connection);

                                        // Join game
                                        match game::handle_player_join(&state, &gid, player).await {
                                            Ok(response) => {
                                                let msg = serde_json::to_string(&response).unwrap();
                                                let _ = tx.send(msg);
                                            }
                                            Err(e) => {
                                                let error = ServerMessage::Error {
                                                    message: e,
                                                };
                                                let msg = serde_json::to_string(&error).unwrap();
                                                let _ = tx.send(msg);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Auth failed: {}", e);
                                        let error = ServerMessage::Error {
                                            message: "Authentication failed".to_string(),
                                        };
                                        let msg = serde_json::to_string(&error).unwrap();
                                        let _ = tx.send(msg);
                                        break;
                                    }
                                }
                            }
                            _ if !authenticated => {
                                let error = ServerMessage::Error {
                                    message: "Not authenticated".to_string(),
                                };
                                let msg = serde_json::to_string(&error).unwrap();
                                let _ = tx.send(msg);
                            }
                            ClientMessage::StartGame => {
                                if let (Some(ref pid), Some(ref gid)) = (&player_id, &game_id) {
                                    if let Err(e) = game::handle_start_game(&state, gid, pid).await {
                                        let error = ServerMessage::Error { message: e };
                                        let msg = serde_json::to_string(&error).unwrap();
                                        let _ = tx.send(msg);
                                    }
                                }
                            }
                            ClientMessage::UpdateProgress {
                                progress,
                                wpm,
                                accuracy,
                            } => {
                                if let (Some(ref pid), Some(ref gid)) = (&player_id, &game_id) {
                                    let _ = game::handle_update_progress(
                                        &state, gid, pid, progress, wpm, accuracy,
                                    )
                                    .await;
                                }
                            }
                            ClientMessage::FinishGame {
                                wpm,
                                accuracy,
                                time_taken: _,
                            } => {
                                if let (Some(ref pid), Some(ref gid)) = (&player_id, &game_id) {
                                    let _ = game::handle_finish_game(&state, gid, pid, wpm, accuracy).await;
                                }
                            }
                            ClientMessage::Ping => {
                                let pong = ServerMessage::Pong;
                                let msg = serde_json::to_string(&pong).unwrap();
                                let _ = tx.send(msg);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client closed connection");
                break;
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup on disconnect
    if let (Some(pid), Some(gid)) = (player_id, game_id) {
        state.remove_player_connection(&pid);
        game::handle_player_leave(&state, &gid, &pid).await;
    }

    send_task.abort();
    Ok(())
}
