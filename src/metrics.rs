use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Serialize, Deserialize)]
pub struct ServerMetrics {
    pub status: String,
    pub uptime_seconds: u64,
    pub timestamp: u64,
    pub system: SystemMetrics,
    pub games: GameMetrics,
}

#[derive(Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_used_percent: f32,
    pub process_memory_mb: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GameMetrics {
    pub total_games: usize,
    pub active_connections: usize,
    pub total_players_connected: usize,
}

pub struct MetricsCollector {
    system: System,
    start_time: SystemTime,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        system.refresh_all();

        Self {
            system,
            start_time: SystemTime::now(),
        }
    }

    pub fn collect(&mut self, state: &AppState) -> ServerMetrics {
        // Refresh system info
        self.system.refresh_cpu_all();
        self.system.refresh_memory();

        // Calculate uptime
        let uptime = self
            .start_time
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // CPU usage (global average)
        let cpu_usage = self.system.global_cpu_usage();

        // Memory metrics
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_percent = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };

        // Process memory (if available)
        let process_memory = self
            .system
            .processes()
            .values()
            .find(|p| p.pid() == sysinfo::get_current_pid().ok().unwrap())
            .map(|p| p.memory() / 1024 / 1024)
            .unwrap_or(0);

        // Game metrics
        let total_games = state.games.len();
        let active_connections = state.player_connections.len();
        let mut total_players = 0;
        for entry in state.game_players.iter() {
            total_players += entry.value().len();
        }

        ServerMetrics {
            status: "online".to_string(),
            uptime_seconds: uptime,
            timestamp,
            system: SystemMetrics {
                cpu_usage_percent: cpu_usage,
                memory_used_mb: used_memory / 1024 / 1024,
                memory_total_mb: total_memory / 1024 / 1024,
                memory_used_percent: memory_percent,
                process_memory_mb: process_memory,
            },
            games: GameMetrics {
                total_games,
                active_connections,
                total_players_connected: total_players,
            },
        }
    }
}
