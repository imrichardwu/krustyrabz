use supabase_rs::SupabaseClient;
use dotenv::dotenv;
use std::env;
use sea_orm::DatabaseConnection;
use crate::entities::UserAccount;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

/// Initialize and return a Supabase client using environment variables
/// 
/// Reads SUPABASE_URL and SUPABASE_KEY from .env file or environment variables
/// 
/// # Example
/// ```no_run
/// use storage::repository::create_supabase_client;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_supabase_client().await?;
///     // Use client for database operations
///     Ok(())
/// }
/// ```
pub async fn create_supabase_client() -> Result<SupabaseClient, Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Get Supabase URL and API key from environment
    let url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in environment variables")?;
    
    let key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in environment variables")?;
    
    // Create and return Supabase client
    let client = SupabaseClient::new(url, key)?;
    
    Ok(client)
}

/// Repository struct to hold both Supabase client and SeaORM database connection
pub struct Repository {
    supabase_client: SupabaseClient,
    db: DatabaseConnection,
}

impl Repository {
    /// Create a new repository instance with both Supabase client and SeaORM connection
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let supabase_client = create_supabase_client().await?;
        let db = crate::establish_connection().await?;
        Ok(Repository { 
            supabase_client,
            db,
        })
    }
    
    /// Get a reference to the Supabase client
    pub fn supabase_client(&self) -> &SupabaseClient {
        &self.supabase_client
    }
    
    /// Get a mutable reference to the Supabase client
    pub fn supabase_client_mut(&mut self) -> &mut SupabaseClient {
        &mut self.supabase_client
    }
    
    /// Get a reference to the SeaORM database connection
    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }
    
    /// Create a new user account with default values
    /// 
    /// # Arguments
    /// * `username` - The username for the new user
    /// 
    /// # Returns
    /// Returns the created user model or a database error
    pub async fn create_user(&self, username: String) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        use crate::entities::UserAccount;
        use sea_orm::{Set, EntityTrait, QueryFilter, ColumnTrait};
        
        let active_model = crate::entities::user_account::ActiveModel {
            username: Set(username.clone()),
            token_balance: Set(Some(0.0)),
            rounds_played: Set(Some(0)),
            pots_won: Set(Some(0)),
            number_folds: Set(Some(0)),
            ..Default::default()
        };
        
        UserAccount::insert(active_model)
            .exec(&self.db)
            .await?;
        
        // Fetch the created user by username (which is unique)
        UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound("User not found after creation".to_string()))
            .map_err(|e| e.into())
    }

    pub async fn get_user_by_username(&self, username: String) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        let username_clone = username.clone();
        
        let user = UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(|e| -> Box<dyn std::error::Error> { Box::from(e) })?;
        
        user.ok_or_else(move || -> Box<dyn std::error::Error> {
            Box::from(sea_orm::DbErr::RecordNotFound(format!("User with username '{}' not found", username_clone)))
        })
    }
}

// Example usage functions - adjust based on supabase_rs API documentation
// The supabase_rs crate uses a fluent API, so the exact syntax may vary
// Check https://docs.rs/supabase_rs for the exact API

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_client() {
        // This test will only work if .env file is present with valid credentials
        let result = create_supabase_client().await;
        assert!(result.is_ok());
    }
}
