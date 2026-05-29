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

/// Launches the Rocket server with the House state and all routes mounted.
#[launch]
fn rocket() -> _ {
    println!("Starting Poker Server...");
    println!("Server will be available at http://127.0.0.1:8000");

    rocket::build()
        .manage(House::new())
        .mount(
            "/",
            routes![
                house::index,
                house::list_games,
                house::create_game,
                house::join_game,
                house::get_game,
                house::perform_action,
                house::get_stats,
                house::add_chips,
                house::register_viewer,
                house::get_rules,
            ],
        )
}
