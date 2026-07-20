pub mod entities;
pub mod error;
pub mod migration;
pub mod repository;

// Re-export commonly used items
pub use error::RepositoryError;
pub use repository::Repository;
pub use sea_orm::DatabaseConnection;

use dotenv::dotenv;
use sea_orm::{Database, DbErr};
use std::env;

/// Establish a connection to the Supabase PostgreSQL database using SeaORM.
/// Reads DATABASE_URL (format:
/// postgresql://postgres:[password]@[project-ref].supabase.co:5432/postgres)
/// from the .env file or environment variables.
pub async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    Database::connect(&database_url).await
}
