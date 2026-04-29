#[derive(Debug, Clone)]
pub struct GameUpdate {
    pub table_id: u32,
    pub update_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TableState {
    pub table_id: u32,
    pub players: Vec<String>,
    pub pot_size: u64,
}

impl TableState {
    pub fn new() -> Self {
        TableState {
            table_id: 0,
            players: Vec::new(),
            pot_size: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub player_id: u32,
    pub rounds_played: u32,
    pub games_won: u32,
}

impl PlayerStats {
    pub fn new() -> Self {
        PlayerStats {
            player_id: 0,
            rounds_played: 0,
            games_won: 0,
        }
    }
}

pub struct Viewer {
    username: String,
    watched_table_id: u32,
    connected: bool,
}

impl Viewer {
    pub fn new(username: String) -> Self { 
        Viewer {
            username,
            watched_table_id: 0,
            connected: false,
        }
    }
    
    // Connect to a specific table
    pub fn join_table(&mut self, table_id: u32) -> Result<(), String> { 
        self.watched_table_id = table_id;
        self.connected = true;
        Ok(())
     }
    
    // Receive game updates from server (read-only)
    pub fn receive_update(&mut self, update: GameUpdate) {
        println!("Received game update: {:?}", update);
     }
    
    // Request current game state
    pub fn get_table_state(&self) -> Result<TableState, String> {
        println!("Requested table state for table: {}", self.watched_table_id);
        Ok(TableState::new())
     }
    
    // Request player statistics
    pub fn get_statistics(&self, player_id: u32) -> Result<PlayerStats, String> {
        println!("Requested statistics for player: {}", player_id);
        Ok(PlayerStats::new())
    }
    
    // Disconnect from table
    pub fn leave_table(&mut self) -> Result<(), String> { 
        self.watched_table_id = 0;
        self.connected = false;
        Ok(())
     }
}