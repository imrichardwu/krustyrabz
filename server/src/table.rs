use crate::player::Player;
use uuid::Uuid;

// Table Structure

/// Represents a poker table with players and viewers.
#[derive(Debug, Clone)]
pub struct Table {
    pub players: Vec<Player>,
    pub viewers: Vec<Uuid>,
    pub max_players: usize,
}

impl Table {
    // Defaults to 5 for standard Draw games if not specified
    pub fn new() -> Self {
        Self::with_max_players(5)
    }

    pub fn with_max_players(max_players: usize) -> Self {
        Self {
            players: Vec::with_capacity(max_players),
            viewers: Vec::new(),
            max_players,
        }
    }

    pub fn get_player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    pub fn seat_player(&mut self, player: Player) -> Result<(), &'static str> {
        if self.players.len() >= self.max_players {
            return Err("table_full");
        }
        if self.players.iter().any(|p| p.id == player.id) {
            return Err("player_already_seated");
        }
        self.players.push(player);
        Ok(())
    }

    pub fn remove_player_from_table(&mut self, player_id: Uuid) -> Result<(), &'static str> {
        let initial_len = self.players.len();
        self.players.retain(|p| p.id != player_id);
        if self.players.len() == initial_len {
            return Err("player_not_found");
        }
        Ok(())
    }

    pub fn get_player(&self, player_id: Uuid) -> Option<&Player> {
        self.players.iter().find(|p| p.id == player_id)
    }

    pub fn get_player_mut(&mut self, player_id: Uuid) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == player_id)
    }

    pub fn add_viewer(&mut self, viewer_id: Uuid) {
        if !self.viewers.contains(&viewer_id) {
            self.viewers.push(viewer_id);
        }
    }

    pub fn remove_viewer(&mut self, viewer_id: Uuid) {
        self.viewers.retain(|&v| v != viewer_id);
    }

    pub fn get_viewer_count(&self) -> usize {
        self.viewers.len()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}
