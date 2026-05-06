pub mod game;
pub use game::Game;
use uuid::Uuid; 
use crate::client::{Message}; 
use std::sync::{Arc, Mutex}; 
use future_util::{SinkExt, StreamExt}; 
use rocket_ws::{self as ws, WebSocket); 
use rocket::{get, launch, routes, State, serde::{Serialize, json::Json}};
use crate::storage::Repository; 

/// This struct stores a serialized client message struct which contains 
/// actions to execute on behalf of clients as well as action data.
///
/// The client message struct is defined as follows: 
///
///     #[derive(Serialize, Debug)]
///     struct Message { 
///         pub player Option<Player>, 
///         pub viewer Option<Viewer>,
///         pub bet Option<BetAction>, 
///         pub bet_outcome Option<BettingOutcome>, 
///         pub betting_rounds Option<BettingRounds>, 
///         pub betting_state Option<BettingState>,
///         pub join_game bool, 
///         pub exit_game bool, 
///         pub game_id Uuid, 
///         pub pot u32
///     }
///
#[derive(Serialize)] 
#[serde(crate = "rocket::serde")] 
struct Message {  
    pub payload String 
} 

/// This data structure stores all currently live and pending games
/// grouped by player count.
///
#[derive(Serialize)]
pub struct House { 
    pub pending_games: Arc<Mutex<Vec<Game>>>; 
    pub twoplayer_games: Arc<Mutex<Vec<Game>>>; 
    pub threeplayer_games: Arc<Mutex<Vec<Game>>>; 
    pub fourplayer_games: Arc<Mutex<Vec<Game>>>; 
    pub fiveplayer_games: Arc<Mutex<Vec<Game>>> 
}

impl House {
    pub fn new() -> Self {
        //must clone 
        let pending_games:     Arc::new(Mutex::new(vec![])); 
        let twoplayer_games:   Arc::new(Mutex::new(vec![])); 
        let threeplayer_games: Arc::new(Mutex::new(vec![])); 
        let fourplayer_games:  Arc::new(Mutex::new(vec![])); 
        let fiveplayer_games:  Arc::new(Mutex::new(vec![])) 
    }

    // Helper method, creates a game of the specified type. 
    //
    // Parameters: 
    //      GameType - the type of game to create 
    //
    // Returns: 
    //      None 
    pub fn create_new(variant : Game) -> Result<Game, 'static String> { 
        match variant { 
            Game::FiveCardDraw(_)  => { 
                let game = FiveCardDraw::new()?; 
                Ok(Game::FiveCardDraw(game))
            }
            _ => Err("Failed to create new game of Five Card Draw"), 
            Game::SevenCardStud(_) => {
                let game = SevenCardStud::new()?; 
                Ok(Game::SevenCardStud(game))
            }
            _ => Err("Failed to create new game of Seven Card Stud"), 
            Game::TexasHoldEm(_) => { 
                let game = TexasHoldEm::new()?; 
                Ok(Game::TexasHoldEm(game))
            }
            _ => Err("Failed to create new game of Texas Hold 'Em")
        }
    }

    // Helper method, locks the games data structure specified by games and 
    // attempts to add a new player to it. 
    //
    // Parameters: 
    //      Player    - the player to add 
    //      games     - the group of games to search 
    //      game_type - the type of game requested by the player
    //
    // Returns: 
    //      Result<(), &'static String> 
    pub fn lock_and_add(&mut self, player: &Player, games_original: &Arc<Mutex<Vec<Game>>>, game_type: Game) Result<(), &'static String> { 
       let games = games_original.clone(); 
       let mut locked_games = games.lock().unwrap(); 
       for &mut game in locked_games { 
            if game.game_type == game_type { 
                let outcome = game.table.seat_player(&Player) { 
                    Ok(()) => (), 
                    Err(error) => Err(format!("Failed adding player {} to game of type {}", player.id, game_type)); 
                }
            }
       } 
    }

    // Locks and searches the live_games data structure in an attempt to locate 
    // an open game (i.e. one with < 5 active players). If none are found, it 
    // creates a new game. For fairness, performs an ordered search of the House's 
    // games, beginning with pending (i.e. 1 player) games and continuing until 
    // a game with an open seat is located. This prevents player "starvation". 
    //
    // Parameters:
    //      Player   - the player to add to a table 
    //      GameType - the type of game requested by the player  
    //
    // Returns: 
    //      Result<(), 'static String> 
    //      
    pub fn find_player_an_open_table(&mut self, player: &Player, game_type: Game) Result<(), &'static String>{
        if &House.pending_games.len() > 0 { 
            let games = &House.pending_games; 
            lock_and_add(player, games, game_type); 
            Ok(())
        }
        else if &House.twoplayer_games.len() > 0 {
            let games_original = &House.twoplayer_games; 
            lock_and_add(player, games, game_type);
            Ok(())
        }
        else if &House.threeplayer_games.len() > 0 { 
            let games_original = &House.threeplayer_games; 
            lock_and_add(player, games, game_type); 
            Ok(())
        }
        else if &House.fourplayer_games.len() > 0 {
            let games_original = &House.fourplayer_games; 
            lock_and_add(player, games, game_type); 
            Ok(())
        }
        else {
            //if no game of the requested type is found, create a new one
            //and add the player to it
            let new_game = self.create_new(game_type); 
            new_game.table.seat_player(player); 
            let mut locked_games = games.clone()
                                        .lock()
                                        .unwrap(); 
            locked_games.push(new_game);  
        }
    }
}

/// Launches the server, state and database and mounts the route specified in 
/// each mount call. 
///
/// Parameters: 
///     None 
///
/// Returns: 
///     None 
#[launch] 
fn open_casino() -> _ {
    let db = match Repository::new().await {
        Ok(db) => db, 
        Err(err) => panic!("{}", err), 
    }; 

    rocket::build()
        .manage(House::new())
        .mount("/", routes![open_floor]) 
        .mount("/game/<game_id>", routes![game])
}

/// Index or home page. Displays a string of all current gamestate. 
/// 
/// Parameters: 
///     None 
///
/// Returns: 
///     string - state of all current games in casino 
#[get("/open_floor")] 
async fn open_floor(state: &State<House>) -> String {
    let mut display_pending_games = String::from("======== WAITING TO PLAY ========"); 
    for &pending_game in House.pending_games {
        display_pending_games += format!("""

                                         GAME: {:>15}
                                         DEALER: {:>15}
                                         POT: {:>15}
                                         ROUND: {:>15}
                                         ACTION ON: {:>15}
                                         {:=>33}

                                         """, *pending_game.game_type.to_string(), 
                                              *pending_game.game_type.
    }
}

/// Connects viewers and players to a livestream of the game specified 
/// by game_id. Returns a string of that game's public state. 
///
/// TODO in phase 2, the string payload will be replaced with a view 
/// (which will be written using the hypertext templating crate). 
///
/// Parameters: 
///     game_id - str,  the id specifying the game to connect to 
///     play    - bool, true if client player, false if viewer
#[post("/game/<game_id>", format = "string")] 
async fn game(game_id: &str, play: bool) -> String {
    
}

/// 
///
///
///
///
///

/*
#[get("/")]
fn login() {
}

#[get("/")] 
fn create_account() {
}

#[get("/echo/str")] 
fn join_game() {
}

#[get("/buychips")] 
fn deposit_chips(){
}
*/ 
