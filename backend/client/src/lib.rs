pub mod api;

pub use api::PokerClient;
use std::io::{self, Write};

pub fn banner(text: &str) {
    let width = text.len() + 6;
    println!("{}", "=".repeat(width));
    println!("== {} ==", text);
    println!("{}", "=".repeat(width));
}

pub fn read_input(prompt: &str) -> String {
    print!("{} ", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input
}