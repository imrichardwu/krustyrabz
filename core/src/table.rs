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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper to generate a fake Player for testing
    fn mock_player() -> Player {
        Player::new("TestUser".to_string(), 1000, Uuid::new_v4())
    }

    #[test]
    fn test_new_table_is_empty() {
        let table = Table::new(); 
        assert_eq!(table.get_player_count(), 0, "A new table must have 0 players");
    }

    #[test]
    fn test_seat_and_remove_player() {
        let mut table = Table::new();
        let p1 = mock_player();
        let p1_id = p1.id; // Save the ID before seat_player takes ownership of p1

        // Add the player
        let join_result = table.seat_player(p1);
        assert!(join_result.is_ok(), "Should be able to seat a player at an empty table");
        assert_eq!(table.get_player_count(), 1, "Table should have exactly 1 player");

        // Remove the player 
        // create a dummy player with the exact same ID so the removal logic finds it
        let mut ghost_for_removal = mock_player();
        ghost_for_removal.id = p1_id;
        
        let leave_result = table.remove_player_from_table(&ghost_for_removal);
        assert!(leave_result.is_ok(), "Should be able to remove a seated player");
        assert_eq!(table.get_player_count(), 0, "Table should be empty after the only player leaves");
    }

    #[test]
    fn test_table_enforces_max_capacity() {
        let mut table = Table::new();
        let max_players = 5; 

        // fill the table to the maximum limit (5)
        for _ in 0..max_players {
            let res = table.seat_player(mock_player());
            assert!(res.is_ok(), "Should be able to add players up to the maximum capacity");
        }

        // try to add a 6th player
        let unlucky_player = mock_player();
        let overfill_result = table.seat_player(unlucky_player);

        assert!(overfill_result.is_err(), "Table MUST reject players when at maximum capacity");
        assert_eq!(overfill_result.unwrap_err(), "table_full", "Should return table_full error");
        assert_eq!(table.get_player_count(), max_players, "Player count must not exceed 5");
    }

    #[test]
    fn test_cannot_remove_from_empty_table() {
        let mut table = Table::new();
        let ghost_player = mock_player();

        // removing from a 0-player table returns a specific error
        let result = table.remove_player_from_table(&ghost_player);
        
        assert!(result.is_err(), "Should return an error when table is empty");
        assert_eq!(result.unwrap_err(), "table_empty", "Should return table_empty error");
    }
}