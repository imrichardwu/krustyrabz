pub mod repository;
pub mod entities;
pub mod migration;

// Re-export commonly used items
pub use repository::Repository;
pub use sea_orm::DatabaseConnection;

use sea_orm::{Database, DbErr};
use dotenv::dotenv;
use std::env;

/// Establish a connection to the Supabase PostgreSQL database using SeaORM
/// 
/// Reads DATABASE_URL from .env file or environment variables.
/// The DATABASE_URL should be in the format:
/// postgresql://postgres:[password]@[project-ref].supabase.co:5432/postgres
/// 
/// # Example
/// ```no_run
/// use storage::establish_connection;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let conn = establish_connection().await?;
///     // Use conn for database operations
///     Ok(())
/// }
/// ```
pub async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    
    Database::connect(&database_url).await
}
