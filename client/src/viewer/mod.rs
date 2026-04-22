// viewer/mod.rs
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
    pub fn join_table(&mut self, table_id: u32) -> Result<()> { 
        self.watched_table_id = table_id;
        self.connected = true;
        Ok(())
     }
    
    // Receive game updates from server (read-only)
    pub fn receive_update(&mut self, update: GameUpdate) {
        println!("Received game update: {:?}", update);
     }
    
    // Request current game state
    pub fn get_table_state(&self) -> Result<TableState> {
        println!("Requested table state for table: {}", self.watched_table_id);
        Ok(TableState::new())
     }
    
    // Request player statistics
    pub fn get_statistics(&self, player_id: u32) -> Result<PlayerStats> {
        println!("Requested statistics for player: {}", player_id);
        Ok(PlayerStats::new())
    }
    
    // Disconnect from table
    pub fn leave_table(&mut self) -> Result<()> { 
        self.watched_table_id = 0;
        self.connected = false;
        Ok(())
     }
}