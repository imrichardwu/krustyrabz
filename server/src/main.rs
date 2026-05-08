// Poker Server Main Entry Point
//
// This is the main entry point for the poker game server.
// The server acts as the "house" and manages poker games.

#[macro_use]
extern crate rocket;

pub mod deck;
pub mod betting;
pub mod house;
pub mod player;
pub mod table;
pub mod protocol;
pub mod game;

use house::House;

/// Launches the server with the House state and all routes mounted.
#[launch]
fn rocket() -> _ {
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
                house::get_stats,
                house::register_viewer,
                house::get_rules,
                house::open_floor,
                house::game,
            ],
        )
}
