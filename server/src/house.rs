use std::sync::{Arc, Mutex}; 
use threadpool::ThreadPool; 
//TODO need something like use std::
use std::thread;


//dealer-threads let players buy into games 
const n_floorstaff: usize = 20;

pub mod game;
pub use game::{Game}; 

pub struct House { 
    pub live_games: vec<Game>;
    pub floorstaff_pool: ThreadPool //in casinos, floorstaff seat players at open tables
}

impl House {
    pub fn new() -> Self {
        let mut live_games = vec![]; 
        let floorstaff_pool = ThreadPool::new(n_floorstaff); 
    }

    //get connections, add player 
    //whenever a player enters, a floorstaff/thread 
    //is tasked with admitting that player to a game
    //ThreadPool, Arc with mutex 
    pub fn open_doors() {
        while true {
         ;   
        }
    }

    pub fn find_player_an_open_table(&mut self) {
        ; 
    }

    //pit boss: opens new games  
}
