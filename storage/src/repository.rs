use supabase_rs::SupabaseClient;
use dotenv::dotenv;
use std::env;
use sea_orm::DatabaseConnection;
use crate::entities::UserAccount;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use sea_orm::prelude::Expr;

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

    //pass negative to decrease and positive to increase 
    pub async fn update_user_token_balance(&self, username: String, amount: f64) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> { 
        use crate::entities::UserAccount;
        let username_clone = username.clone();
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::TokenBalance, Expr::col(crate::entities::user_account::Column::TokenBalance).add(amount))
            .filter(crate::entities::user_account::Column::Username.eq(username.clone()))
            .exec(&self.db)
            .await?;
        
        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update token_balance for username '{}'", username_clone).into());
        }
        
        // Fetch the updated user
        UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .ok_or_else(|| format!("User '{}' not found after update", username_clone).into())
    }
//could try to use generics to condense into single update method but it doesn't really matter for 4
//attributes
    pub async fn increase_rounds_played(&self, username: String, rounds: i32) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> { 
        if rounds < 0 {
            return Err(format!("Parameter error: rounds can only be >= 0, but '{}' passed instead", rounds).into());  
        }
        let username_clone = username.clone();
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::RoundsPlayed, Expr::col(crate::entities::user_account::Column::RoundsPlayed).add(rounds))
            .filter(crate::entities::user_account::Column::Username.eq(username.clone()))
            .exec(&self.db)
            .await?;
        
        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update rounds_played for username '{}'", username_clone).into());
        }
        
        // Fetch the updated user
        UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .ok_or_else(|| format!("User '{}' not found after update", username_clone).into())
    }

//could try to use generics to condense into single update method but it doesn't really matter for 4
//attributes
    pub async fn increase_pots_won(&self, username: String, pots: i32) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> { 
        use crate::entities::UserAccount;
        if pots < 0 {
            return Err(format!("Parameter error: pots can only be >= 0, but '{}' passed instead", pots).into());  
        }
        let username_clone = username.clone();
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::PotsWon, Expr::col(crate::entities::user_account::Column::PotsWon).add(pots))
            .filter(crate::entities::user_account::Column::Username.eq(username.clone()))
            .exec(&self.db)
            .await?;
        
        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update pots_won for username '{}'", username_clone).into());
        }
        
        // Fetch the updated user
        UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .ok_or_else(|| format!("User '{}' not found after update", username_clone).into())
    }

    pub async fn increase_number_folds(&self, username: String, folds: i32) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> { 
        use crate::entities::UserAccount;
        if folds < 0 {
            return Err(format!("Parameter error: folds can only be >= 0, but '{}' passed instead", folds).into());  
        }
        let username_clone = username.clone();
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::NumberFolds, Expr::col(crate::entities::user_account::Column::NumberFolds).add(folds))
            .filter(crate::entities::user_account::Column::Username.eq(username.clone()))
            .exec(&self.db)
            .await?;
        
        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update number_folds for username '{}'", username_clone).into());
        }
        
        // Fetch the updated user
        UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(username))
            .one(&self.db)
            .await?
            .ok_or_else(|| format!("User '{}' not found after update", username_clone).into())
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
