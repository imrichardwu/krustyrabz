// Poker Server Main Entry Point
//
// This is the main entry point for the poker game server.
// The server acts as the "house" and manages poker games using Rocket HTTP.

#[macro_use]
extern crate rocket;

pub mod betting;
pub mod deck;
pub mod game;
pub mod house;
pub mod player;
pub mod table;

use house::House;
use storage::establish_connection;
use storage::repository::create_supabase_client;
use tokio::runtime::Runtime;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

/// CORS fairing — allows the web client (port 8001) to connect to the server (port 8000).
/// Required so the browser's EventSource API can subscribe to SSE game updates.
pub struct CorsFairing;

#[rocket::async_trait]
impl Fairing for CorsFairing {
    fn info(&self) -> Info {
        Info { name: "CORS", kind: Kind::Response }
    }
    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "http://127.0.0.1:8001"));
        res.set_header(Header::new("Access-Control-Allow-Methods", "GET, POST, OPTIONS"));
        res.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
    }
}

/// Launches the Rocket server with the House state and all routes mounted.
#[launch]
fn rocket() -> _ {
    println!("Starting Poker Server...");
    println!("Server will be available at http://127.0.0.1:8000");

    // We need a Tokio runtime here to block the async call to establish_connection, 
    // because [launch] functions cannot be async 
    let rt = Runtime::new().expect("Failed to create Tokio runtime"); 
    let db = rt
        .block_on(establish_connection())
        .expect("Failed to establish connection"); 
   
    // We need to pass a supabase connection as managed state 
    let client = rt
        .block_on(create_supabase_client())
        .expect("Failed to create client");

    let house = House::new();
    let house_games = house.live_games.clone();

    rocket::build()
        .attach(CorsFairing)
        .attach(rocket::fairing::AdHoc::on_liftoff("Timeout Checker", |_| Box::pin(async move {
            // Start timeout checker after Rocket runtime is ready
            House::start_timeout_checker(house_games);
            println!("⏰ Timeout checker started (30s inactivity limit)");
        })))
        .manage(house)
        .manage(db)
        .manage(client)
        .mount(
            "/",
            routes![
                house::index,
                house::list_games,
                house::create_game,
                house::join_game,
                house::start_hand,
                house::get_game,
                house::perform_action,
                house::get_stats,
                house::add_chips,
                house::withdraw_chips,
                house::sit_out,
                house::register_viewer,
                house::get_rules,
                house::game_events,
            ],
        )
}
