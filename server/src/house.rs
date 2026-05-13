pub mod game;
pub use game::Game;
use uuid::Uuid; 
use crate::client::{Message}; 
use std::sync::{Arc, Mutex};
use std::collections::HashSet; 
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
#[derive(Debug, Serialize)] 
#[serde(crate = "rocket::serde")] 
struct Message {  
    pub payload String 
} 

/// This data structure stores all currently live and pending games
/// grouped by player count.
///
#[derive(Serialize)]
pub struct House { 
    pub pending_games:     Arc<Mutex<Vec<Game>>>; 
    pub twoplayer_games:   Arc<Mutex<Vec<Game>>>; 
    pub threeplayer_games: Arc<Mutex<Vec<Game>>>; 
    pub fourplayer_games:  Arc<Mutex<Vec<Game>>>; 
    pub fiveplayer_games:  Arc<Mutex<Vec<Game>>> 
}

impl House {
    pub fn new() -> Self {
        //must clone
        //consider factoring out into separate game_type discriminated groups  
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
            },
            _ => Err("Failed to create new game of Five Card Draw"), 
            Game::SevenCardStud(_) => {
                let game = SevenCardStud::new()?; 
                Ok(Game::SevenCardStud(game))
            },
            _ => Err("Failed to create new game of Seven Card Stud"), 
            Game::TexasHoldEm(_) => { 
                let game = TexasHoldEm::new()?; 
                Ok(Game::TexasHoldEm(game))
            },
            _ => Err("Failed to create new game of Texas Hold 'Em")
        }
    }

    // Helper method, locks the games data structure specified by search and 
    // attempts to locate a free game inside it. 
    //
    // If optional parameter game_id is supplied, attempts to add a player to the 
    // game specified by that id. On success, invalidates the search group, 
    // removing the game with incremented player count and relocating it to target. 
    //
    // If no optional parameter is supplied, finds the first free game and adds 
    // the player to it, then moves that game to the group with incremented player 
    // counts. 
    //
    // Parameters: 
    //      Player    - the player to add 
    //      search    - the group of games to search 
    //      target    - the group of games to target 
    //      game_type - the type of game requested by the player
    //      game_id   - optional, if adding the player to a specific game
    //
    // Returns: 
    //      Result<String, &'static String> 
    pub fn lock_add_and_move(&mut self, player: &Player, 
                                   search: &Arc<Mutex<Vec<Game>>>, 
                                   target: &Arc<Mutex<Vec<Game>>>, 
                                   game_type: String, 
                                   game_id Option<String>) Result<String, &'static String> {

       match game_id { 
           Some(value) => {
                    let search_group = search.clone(); 
                    let mut locked_search_group = search_group.lock.unwrap(); 
                    for game in locked_search_group.iter_mut() { 
                        if game.game_id == game_id { 
                            match game.table.seat_player(&player) { 
                                Ok(()) => { 
                                    let target_group = target.clone(); 
                                    let mut locked_target_group = target_group.lock.unwrap(); 
                                    locked_target_group.push(game); 
                                    locked_search_group.retain(|&g| g.game_id != game_id); 
                                    return Ok(game.game_Id) 
                                }, 
                                Err(_) => { continue } 
                            }
                        }
                    }
           }, 
           None => {
                    let search_group = search.clone(); 
                    let mut locked_search_group = search_group.lock.unwrap(); 
                    for game in locked_search_group.iter_mut() { 
                        if game.game_type.to_string() == game_type { 
                            match game.table.seat_player(&player) { 
                                Ok(()) => {
                                    let target_group = target.clone(); 
                                    let mut locked_target_group = target_group.lock.unwrap(); 
                                    locked_target_group.push(game); 
                                    locked_search_group.retain(|&g| g.game_id != game_id); 
                                    return Ok(game.game_Id) 
                                }
                                Err(_) => { continue } 
                        }
                    }
                }
            }
       } 
       return Err(format!("Failed adding player {} to game of type {}", player.id, game_type));
    }

    // Helper method, on client disconnect, removes the dropped player from the game 
    // and relocates that game to the group with decremented player counts. 
    //
    // Parameters: 
    //      player  - the player to remove from the table 
    //      game    - the game to remove the player from
    //      target  - the group to move the game into after reducing its player count
    pub fn remove_player_from(game: &mut Game, player: &Player, target: &Arc<Mutex<Vec<Game>>>) {
    }

    //TODO GIVE PLAYER OPTION TO SELECT A GAME FROM HOME SCREEN

    /// Locks and searches the live_games data structure in an attempt to locate 
    /// an open game (i.e. one with < 5 active players). If none are found, it 
    /// creates a new game. For fairness, performs an ordered search of the House's 
    /// games, beginning with pending (i.e. 1 player) games and continuing until 
    /// a game with an open seat is located. This prevents player "starvation". 
    ///
    /// Parameters:
    ///     player    - the player to add to a table 
    ///     game_type - the type of game requested by the player  
    ///
    /// Returns: 
    ///    Result<String, 'static String> 
    ///     
    pub fn find_player_an_open_table(&mut self, player: &Player, game_type: String) Result<Uuid, &'static String>{
        if &self.pending_games.len() > 0 { 
            let from_one_player  = &House.pending_games; 
            let into_two_players = &House.twoplayer_games; 
            lock_add_and_move(player, from_one_player, into_two_players, game_type); 
            Ok(())
        }
        else if &self.twoplayer_games.len() > 0 {
            let from_two_players     = &House.twoplayer_games; 
            let into_three_players   = &House.threeplayer_games; 
            lock_add_and_move(player, from_two_players, into_three_players, game_type);
            Ok(())
        }
        else if &self.threeplayer_games.len() > 0 { 
            let from_three_players = &House.threeplayer_games; 
            let into_four_players  = &House.fourplayer_games; 
            lock_add_and_move(player, from_three_players, into_four_players, game_type); 
            Ok(())
        }
        else if &self.fourplayer_games.len() > 0 {
            let from_four_players = &House.fourplayer_games; 
            let into_five_players = &House.fiveplayer_games; 
            lock_add_and_move(player, from_four_players, into_five_players, game_type); 
            Ok(())
        }
        else {
            //if no game of the requested type is found, create a new one
            //and add the player to it
            let mut new_game = self.create_new(game_type); 
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


/// Index or home page. Opens a WebSocket channel to a client, stores the 
/// client's connection data in server state, displays a view of all active 
/// and pending games, and accepts messages from the client. 
/// 
/// Parameters: 
///     username: the username of the connecting client
///     ws:       a request guard identifying a WebSocket requests (from docs) 
///     gametype: String, the type of game requested by the client
///
/// Returns: 
///      
#[get("/open_floor")] 
async fn open_floor(ws: ws::WebSocket, gametype: String, username: String, state: &State<House>) -> ws::Channel<'static> {
    
    ws.channel(move |mut stream| { 
        Box::pin(async move { 
            let id = Uuid::new_v4(); 

            //construct a new player for client
            {
                let mut player = Player::new();
                player.id = id;
                player.username = username; 

                stream.send()
                      .await 
                      .unwrap(); 

                let result = find_player_an_open_table(gametype); 
                match result { 
                    Ok() => stream.send(format!("Added you to game {}", result),  
                    Err() => stream.send("Failed to find you a new game.")
                                   .await
                                   .unwrap(); 
                }
            }
        })
    }); 
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
#[post("/games", format = "string")] 
async fn games() -> Json {
   format!("") 
}

///
///
///
///
///
///
#[post("/games")] 
async fn new_game() { 
}

///
///
///
///
///
///
#[post("/players/<player_id>/stats", format = "string")] 
async fn stats(player_id: &str) -> Json { 
} 

/// 
///
///
///
///
///
#[post("/games/<game_id>/viewer", format = "string")] 
async fn viewers(game_id: &str) { 
}

///
///
///
///
///
///
#[post("/rules")] 
pub async fn rules() { 
}

///
///
///
///
///
///
#[post("/games/<game_id>/action")]
async fn action(game_id: &str) { 
}

///
///
///
///
///
///
#[post("/")]
