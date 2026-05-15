use crate::Player;

pub struct Table {
    pub players: Vec<Player>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            players: vec![],
        }
    }

    pub fn get_player_count(&self) -> u32 {
        self.players.len() as u32
    }

    pub fn seat_player(&mut self, player: Player) -> Result<(), &'static str> {
        if self.get_player_count() < 5 {
            self.players.push(player);
            Ok(())
        } else {
            Err("table_full")
        }
    }

    pub fn remove_player_from_table(&mut self, player: &Player) -> Result<(), &'static str> {
        if self.get_player_count() > 0 {
            self.players.retain(|p| p.id != player.id);
            Ok(())
        } else {
            Err("table_empty")
        }
    }
}


