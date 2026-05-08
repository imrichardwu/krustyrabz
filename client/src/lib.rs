pub mod api;

pub use api::PokerClient;

pub fn banner(text: &str) {
    let width = text.len() + 6;
    println!("{}", "=".repeat(width));
    println!("== {} ==", text);
    println!("{}", "=".repeat(width));
}