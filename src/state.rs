use crate::models::{GameState, Player};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub type Tx = mpsc::UnboundedSender<String>;
pub type GameId = String;
pub type PlayerId = String;

#[derive(Clone)]
pub struct AppState {
    pub games: Arc<DashMap<GameId, GameState>>,
    pub player_connections: Arc<DashMap<PlayerId, PlayerConnection>>,
    pub game_players: Arc<DashMap<GameId, Vec<PlayerId>>>,
}

pub struct PlayerConnection {
    pub player_id: String,
    pub user_id: String,
    pub game_id: String,
    pub tx: Tx,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            games: Arc::new(DashMap::new()),
            player_connections: Arc::new(DashMap::new()),
            game_players: Arc::new(DashMap::new()),
        }
    }

    pub fn add_game(&self, game: GameState) {
        self.games.insert(game.id.clone(), game);
    }

    pub fn get_game(&self, game_id: &str) -> Option<GameState> {
        self.games.get(game_id).map(|g| g.clone())
    }

    pub fn update_game<F>(&self, game_id: &str, f: F) -> Option<GameState>
    where
        F: FnOnce(&mut GameState),
    {
        self.games.get_mut(game_id).map(|mut game| {
            f(&mut game);
            game.clone()
        })
    }

    pub fn add_player_connection(&self, player_id: String, connection: PlayerConnection) {
        let game_id = connection.game_id.clone();
        self.player_connections.insert(player_id.clone(), connection);
        
        self.game_players
            .entry(game_id)
            .or_insert_with(Vec::new)
            .push(player_id);
    }

    pub fn remove_player_connection(&self, player_id: &str) -> Option<String> {
        if let Some((_, conn)) = self.player_connections.remove(player_id) {
            let game_id = conn.game_id.clone();
            
            // Remove from game_players
            if let Some(mut players) = self.game_players.get_mut(&game_id) {
                players.retain(|id| id != player_id);
            }
            
            return Some(game_id);
        }
        None
    }

    pub fn broadcast_to_game(&self, game_id: &str, message: &str, exclude_player: Option<&str>) {
        if let Some(player_ids) = self.game_players.get(game_id) {
            for player_id in player_ids.iter() {
                if let Some(exclude) = exclude_player {
                    if player_id == exclude {
                        continue;
                    }
                }
                
                if let Some(conn) = self.player_connections.get(player_id) {
                    let _ = conn.tx.send(message.to_string());
                }
            }
        }
    }

    pub fn send_to_player(&self, player_id: &str, message: &str) {
        if let Some(conn) = self.player_connections.get(player_id) {
            let _ = conn.tx.send(message.to_string());
        }
    }
}
