use supabase_rs::SupabaseClient;
use dotenv::dotenv;
use std::env;
use sea_orm::DatabaseConnection;
use crate::entities::UserAccount;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use sea_orm::prelude::Expr;
use uuid::Uuid;

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
    
    /// Create a new user account with default values.
    /// Username must be unique; returns error if it is already taken.
    ///
    /// # Arguments
    /// * `username` - The username for the new user (must be unique)
    /// * `id` - The user id from Supabase Auth (used as primary key)
    ///
    /// # Returns
    /// Returns the created user model or a database error
    pub async fn create_user(&self, username: String, id: Uuid) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        use crate::entities::UserAccount;
        use sea_orm::{Set, EntityTrait, QueryFilter, ColumnTrait};

        // Ensure username is unique before creating
        let existing = UserAccount::find()
            .filter(crate::entities::user_account::Column::Username.eq(&username))
            .one(&self.db)
            .await?;
        if existing.is_some() {
            return Err(format!("Username '{}' is already taken", username).into());
        }

        let active_model = crate::entities::user_account::ActiveModel {
            username: Set(username),
            token_balance: Set(Some(0.0)),
            rounds_played: Set(Some(0)),
            pots_won: Set(Some(0)),
            number_folds: Set(Some(0)),
            game_id: Set(None),
            id: Set(id),
        };

        UserAccount::insert(active_model)
            .exec(&self.db)
            .await?;

        self.get_user_by_id(id).await
    }

    /// Get a user account by id (Supabase Auth id).
    pub async fn get_user_by_id(&self, id: Uuid) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        match UserAccount::find_by_id(id).one(&self.db).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => {
                Err(Box::from(sea_orm::DbErr::RecordNotFound(format!(
                    "User with id '{}' not found",
                    id
                ))))
            }
            Err(entity_err) => {
                Err(Box::from(entity_err))
            }
        }
    }

    //pass negative to decrease and positive to increase
    pub async fn update_user_token_balance(&self, id: Uuid, amount: f64) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        use crate::entities::UserAccount;
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::TokenBalance, Expr::col(crate::entities::user_account::Column::TokenBalance).add(amount))
            .filter(crate::entities::user_account::Column::Id.eq(id))
            .exec(&self.db)
            .await?;

        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update token_balance for user id '{}'", id).into());
        }

        self.get_user_by_id(id).await
    }
//could try to use generics to condense into single update method but it doesn't really matter for 4
//attributes
    pub async fn increase_rounds_played(&self, id: Uuid, rounds: i32) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        if rounds < 0 {
            return Err(format!("Parameter error: rounds can only be >= 0, but '{}' passed instead", rounds).into());
        }
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::RoundsPlayed, Expr::col(crate::entities::user_account::Column::RoundsPlayed).add(rounds))
            .filter(crate::entities::user_account::Column::Id.eq(id))
            .exec(&self.db)
            .await?;

        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update rounds_played for user id '{}'", id).into());
        }

        self.get_user_by_id(id).await
    }

//could try to use generics to condense into single update method but it doesn't really matter for 4
//attributes
    pub async fn increase_pots_won(&self, id: Uuid, pots: i32) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        use crate::entities::UserAccount;
        if pots < 0 {
            return Err(format!("Parameter error: pots can only be >= 0, but '{}' passed instead", pots).into());
        }
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::PotsWon, Expr::col(crate::entities::user_account::Column::PotsWon).add(pots))
            .filter(crate::entities::user_account::Column::Id.eq(id))
            .exec(&self.db)
            .await?;

        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update pots_won for user id '{}'", id).into());
        }

        self.get_user_by_id(id).await
    }

    pub async fn increase_number_folds(&self, id: Uuid, folds: i32) -> Result<crate::entities::user_account::Model, Box<dyn std::error::Error>> {
        use crate::entities::UserAccount;
        if folds < 0 {
            return Err(format!("Parameter error: folds can only be >= 0, but '{}' passed instead", folds).into());
        }
        let update_result = UserAccount::update_many()
            .col_expr(crate::entities::user_account::Column::NumberFolds, Expr::col(crate::entities::user_account::Column::NumberFolds).add(folds))
            .filter(crate::entities::user_account::Column::Id.eq(id))
            .exec(&self.db)
            .await?;

        if update_result.rows_affected == 0 {
            return Err(format!("Failed to update number_folds for user id '{}'", id).into());
        }

        self.get_user_by_id(id).await
    }
    #[cfg(test)]
    pub fn new_with_mock(db: DatabaseConnection) -> Self {
    Self {
        supabase_client: SupabaseClient::new("http://localhost".to_string(), "dummy".to_string()).unwrap(),
        db,
    }
}
}

// Example usage functions - adjust based on supabase_rs API documentation
// The supabase_rs crate uses a fluent API, so the exact syntax may vary
// Check https://docs.rs/supabase_rs for the exact API

#[cfg(test)]
mod mock_tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
    use uuid::Uuid;

    // Helper to generate a standardized mock user
    fn generate_mock_user(id: Uuid, username: &str) -> crate::entities::user_account::Model {
        crate::entities::user_account::Model {
            id,
            username: username.to_string(),
            token_balance: Some(100.0),
            rounds_played: Some(10),
            pots_won: Some(3),
            number_folds: Some(2),
            game_id: None,
        }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let test_id = Uuid::new_v4();
        let expected_user = generate_mock_user(test_id, "NewPlayer");

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            //SELECT: check for existing user (returns empty)
            .append_query_results(vec![Vec::<crate::entities::user_account::Model>::new()])
            //INSERT
            .append_query_results(vec![vec![expected_user.clone()]])
            //SELECT: fetch via get_user_by_id
            .append_query_results(vec![vec![expected_user.clone()]])
            .into_connection();

        let repo = Repository::new_with_mock(db);
        let result = repo.create_user("NewPlayer".to_string(), test_id).await;

        // Print the exact error if the mock queues are misaligned
        if let Err(ref e) = result {
            println!("DB Mock Error: {:?}", e);
        }

        assert!(result.is_ok());
        assert_eq!(result.unwrap().username, "NewPlayer");
    }

    #[tokio::test]
    async fn test_create_user_fails_on_duplicate_username() {
        let test_id = Uuid::new_v4();
        let existing_user = generate_mock_user(test_id, "ExistingPlayer");

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            //check for existing user (returns the user, triggering the failure)
            .append_query_results(vec![vec![existing_user]])
            .into_connection();

        let repo = Repository::new_with_mock(db);
        let result = repo.create_user("ExistingPlayer".to_string(), test_id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Username 'ExistingPlayer' is already taken");
    }

    #[tokio::test]
    async fn test_update_token_balance_success() {
        let test_id = Uuid::new_v4();
        let mut expected_user = generate_mock_user(test_id, "Player1");
        expected_user.token_balance = Some(150.0); // Simulated updated balance

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .append_query_results(vec![vec![expected_user]])
            .into_connection();

        let repo = Repository::new_with_mock(db);
        let result = repo.update_user_token_balance(test_id, 50.0).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().token_balance, Some(150.0));
    }

    #[tokio::test]
    async fn test_increase_rounds_played_validation() {
        let test_id = Uuid::new_v4();
        
        //don't even need a mock DB here because it should fail before the DB call
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let repo = Repository::new_with_mock(db);

        //attempting to pass a negative round count
        let result = repo.increase_rounds_played(test_id, -1).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parameter error: rounds can only be >= 0"));
    }

    #[tokio::test]
    async fn test_increase_pots_won_success() {
        let test_id = Uuid::new_v4();
        let mut expected_user = generate_mock_user(test_id, "Winner");
        expected_user.pots_won = Some(4);

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results(vec![MockExecResult { last_insert_id: 0, rows_affected: 1 }])
            .append_query_results(vec![vec![expected_user]])
            .into_connection();

        let repo = Repository::new_with_mock(db);
        let result = repo.increase_pots_won(test_id, 1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().pots_won, Some(4));
    }
}