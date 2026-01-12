use crate::models::{GameStatus, Player, ServerMessage};
use crate::state::AppState;
use tracing::info;

pub async fn handle_player_join(
    state: &AppState,
    game_id: &str,
    player: Player,
) -> Result<ServerMessage, String> {
    let game = state.update_game(game_id, |game| {
        if !game.add_player(player.clone()) {
            return;
        }
    });

    match game {
        Some(game) => {
            info!("Player {} joined game {}", player.username, game_id);
            
            // Broadcast to all players except the new one
            let message = serde_json::to_string(&ServerMessage::PlayerJoined {
                player: player.clone(),
            })
            .unwrap();
            state.broadcast_to_game(game_id, &message, Some(&player.id));
            
            Ok(ServerMessage::GameState { game })
        }
        None => Err("Game not found".to_string()),
    }
}

pub async fn handle_player_leave(state: &AppState, game_id: &str, player_id: &str) {
    state.update_game(game_id, |game| {
        game.remove_player(player_id);
    });

    let message = serde_json::to_string(&ServerMessage::PlayerLeft {
        player_id: player_id.to_string(),
    })
    .unwrap();
    
    state.broadcast_to_game(game_id, &message, None);
    info!("Player {} left game {}", player_id, game_id);
}

pub async fn handle_start_game(state: &AppState, game_id: &str, player_id: &str) -> Result<(), String> {
    let game = state.get_game(game_id).ok_or("Game not found")?;
    
    // Only host can start the game
    if game.host_id != player_id {
        return Err("Only the host can start the game".to_string());
    }
    
    if game.status != GameStatus::Waiting {
        return Err("Game already started".to_string());
    }
    
    let started_at = chrono::Utc::now().timestamp();
    
    state.update_game(game_id, |game| {
        game.start_game();
    });
    
    let message = serde_json::to_string(&ServerMessage::GameStarted { started_at }).unwrap();
    state.broadcast_to_game(game_id, &message, None);
    
    info!("Game {} started by {}", game_id, player_id);
    Ok(())
}

pub async fn handle_update_progress(
    state: &AppState,
    game_id: &str,
    player_id: &str,
    progress: u32,
    wpm: u32,
    accuracy: f32,
) -> Result<(), String> {
    state.update_game(game_id, |game| {
        if let Some(player) = game.get_player_mut(player_id) {
            player.progress = progress;
            player.wpm = wpm;
            player.accuracy = accuracy;
        }
    });
    
    let message = serde_json::to_string(&ServerMessage::PlayerProgress {
        player_id: player_id.to_string(),
        progress,
        wpm,
        accuracy,
    })
    .unwrap();
    
    state.broadcast_to_game(game_id, &message, Some(player_id));
    Ok(())
}

pub async fn handle_finish_game(
    state: &AppState,
    game_id: &str,
    player_id: &str,
    wpm: u32,
    accuracy: f32,
) -> Result<(), String> {
    let finished_at = chrono::Utc::now().timestamp();
    
    let all_finished = state.update_game(game_id, |game| {
        if let Some(player) = game.get_player_mut(player_id) {
            player.finished = true;
            player.finished_at = Some(finished_at);
            player.wpm = wpm;
            player.accuracy = accuracy;
        }
    })
    .map(|game| game.check_all_finished())
    .unwrap_or(false);
    
    // Broadcast player finished
    let message = serde_json::to_string(&ServerMessage::PlayerFinished {
        player_id: player_id.to_string(),
        wpm,
        accuracy,
        finished_at,
    })
    .unwrap();
    state.broadcast_to_game(game_id, &message, None);
    
    // If all players finished, end the game
    if all_finished {
        if let Some(game) = state.get_game(game_id) {
            state.update_game(game_id, |g| {
                g.status = GameStatus::Finished;
            });
            
            let winner_id = game.get_winner();
            let mut final_standings = game.players.clone();
            final_standings.sort_by_key(|p| p.finished_at.unwrap_or(i64::MAX));
            
            let message = serde_json::to_string(&ServerMessage::GameFinished {
                winner_id,
                final_standings,
            })
            .unwrap();
            state.broadcast_to_game(game_id, &message, None);
            
            info!("Game {} finished", game_id);
        }
    }
    
    Ok(())
}
