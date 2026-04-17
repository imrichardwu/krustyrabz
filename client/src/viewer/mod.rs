// viewer/mod.rs
pub struct Viewer {
    username: String,
    watched_table_id: u32,
    connected: bool,
}

impl Viewer {
    pub fn new(username: String) -> Self { }
    
    // Connect to a specific table
    pub fn join_table(&mut self, table_id: u32) -> Result<()> {  }
    
    // Receive game updates from server (read-only)
    pub fn receive_update(&mut self, update: GameUpdate) { }
    
    // Request current game state
    pub fn get_table_state(&self) -> Result<TableState> { }
    
    // Request player statistics
    pub fn get_statistics(&self, player_id: u32) -> Result<PlayerStats> { }
    
    // Disconnect from table
    pub fn leave_table(&mut self) -> Result<()> {  }
}