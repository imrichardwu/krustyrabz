mod games;
mod lib;
mod auth;

use lib::banner;
use games::{five_card_draw, seven_card_stud, texas_holdem};
use auth::{login, register};

fn main() {
    banner("Welcome to KrustyRabz Poker");
    let mut authenticated = false;
    while true {
        if !authenticated {
            println!("1. Register");
            println!("2. Login");
            println!("3. Exit");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect("Failed to read line");

            match choice.trim() {
                "1" => {
                    if register() {
                        authenticated = true;
                    }
                }
                "2" => {
                    if login() {
                        authenticated = true;
                    }
                }
                "3" => {
                    println!("Exiting the game. Goodbye!");
                    return;
                }
                _ => println!("Invalid choice, please try again."),
            }
        } else {

            println!("Select a game mode:");
            println!("1. Five Card Draw");
            println!("2. Seven Card Stud");
            println!("3. Texas Hold'em");
            println!("4. Exit");

            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect("Failed to read line");

            match choice.trim() {
                "1" => five_card_draw(),
                "2"=> seven_card_stud(),
                "3" => texas_holdem(),
                "4" => {
                    println!("Exiting the game. Goodbye!");
                    break;
                }
                _ => println!("Invalid choice, please try again."),
            }
        }
    }
}
