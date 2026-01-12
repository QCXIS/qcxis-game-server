use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub wpm: u32,
    pub accuracy: f32,
    pub progress: u32,
    pub finished: bool,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: String,
    pub code: String,
    pub difficulty: String,
    pub text: String,
    pub host_id: String,
    pub players: Vec<Player>,
    pub status: GameStatus,
    pub started_at: Option<i64>,
    pub max_players: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GameStatus {
    Waiting,
    Playing,
    Finished,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Auth {
        token: String,
        game_id: String,
        game_code: String,
        difficulty: String,
        text: String,
        host_id: String,
    },
    StartGame,
    UpdateProgress {
        progress: u32,
        wpm: u32,
        accuracy: f32,
    },
    FinishGame {
        wpm: u32,
        accuracy: f32,
        time_taken: u32,
    },
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Connected {
        player_id: String,
    },
    GameState {
        game: GameState,
    },
    PlayerJoined {
        player: Player,
    },
    PlayerLeft {
        player_id: String,
    },
    GameStarted {
        started_at: i64,
    },
    PlayerProgress {
        player_id: String,
        progress: u32,
        wpm: u32,
        accuracy: f32,
    },
    PlayerFinished {
        player_id: String,
        wpm: u32,
        accuracy: f32,
        finished_at: i64,
    },
    GameFinished {
        winner_id: Option<String>,
        final_standings: Vec<Player>,
    },
    Error {
        message: String,
    },
    Pong,
}

impl GameState {
    pub fn new(id: String, code: String, difficulty: String, text: String, host_id: String) -> Self {
        Self {
            id,
            code,
            difficulty,
            text,
            host_id,
            players: Vec::new(),
            status: GameStatus::Waiting,
            started_at: None,
            max_players: 10,
        }
    }

    pub fn add_player(&mut self, player: Player) -> bool {
        if self.players.len() >= self.max_players as usize {
            return false;
        }
        self.players.push(player);
        true
    }

    pub fn remove_player(&mut self, player_id: &str) {
        self.players.retain(|p| p.id != player_id);
    }

    pub fn get_player_mut(&mut self, player_id: &str) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == player_id)
    }

    pub fn start_game(&mut self) {
        self.status = GameStatus::Playing;
        self.started_at = Some(chrono::Utc::now().timestamp());
    }

    pub fn check_all_finished(&self) -> bool {
        !self.players.is_empty() && self.players.iter().all(|p| p.finished)
    }

    pub fn get_winner(&self) -> Option<String> {
        self.players
            .iter()
            .filter(|p| p.finished)
            .min_by_key(|p| p.finished_at.unwrap_or(i64::MAX))
            .map(|p| p.id.clone())
    }
}

impl Player {
    pub fn new(id: String, user_id: String, username: String) -> Self {
        Self {
            id,
            user_id,
            username,
            wpm: 0,
            accuracy: 0.0,
            progress: 0,
            finished: false,
            finished_at: None,
        }
    }
}
