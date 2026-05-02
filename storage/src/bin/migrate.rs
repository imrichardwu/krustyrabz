use sea_orm_migration::prelude::*;
use storage::migration::Migrator;
use dotenv::dotenv;
use std::path::PathBuf;
use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    // Try loading from project root (one level up from storage) first
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up one level to project root
    path.push(".env");
    
    // Try loading from project root first
    if path.exists() {
        dotenv::from_path(&path).ok();
    } else {
        // Fallback to current directory
        dotenv().ok();
    }
    
    // Debug: Check if DATABASE_URL is loaded (without showing password)
    if let Ok(db_url) = env::var("DATABASE_URL") {
        // Mask password in output
        let masked = db_url.split('@').next()
            .map(|s| format!("{}@***", s.split(':').take(2).collect::<Vec<_>>().join(":")))
            .unwrap_or_else(|| "***".to_string());
        eprintln!("DATABASE_URL loaded: {}", masked);
    } else {
        eprintln!("ERROR: DATABASE_URL not found in environment!");
        eprintln!("Make sure your .env file is in the project root and contains DATABASE_URL");
        std::process::exit(1);
    }
    
    cli::run_cli(Migrator).await;
}
