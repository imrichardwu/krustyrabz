use poker_core::{Player};

pub struct Table { 
   pub players vec<Player>  
}

impl Table { 
    pub fn new() -> Self {
        let mut players = vec![]; 
    }

    pub fn get_player_count(&self) -> u32 {
        &self.players.len(); 
    }

    pub fn seat_player(&self, &Player) -> Result<(), &'static str> {
        let count = &self.get_player_count(); 
        if (count < 5) {
                players.push(*Player); 
                Ok(());
        }
        else {
            Err("table_full"); 
        }
    }

    pub fn remove_player_from_table(&self, &Player) -> Result<(), &'static str> {
        let count = &self.get_player_count(); 
        if (count > 0) {
            players.retain(|&x| x.username != Player.username);
            Ok(()); 
        }
        else {
            Err("table_empty"); 
        }
    }
}


