use thiserror::Error;
use uuid::Uuid;

/// Errors returned by the storage layer (repository and connection helpers).
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// A required environment variable was missing.
    #[error("{0}")]
    Env(String),

    /// The Supabase client failed to initialize.
    #[error("{0}")]
    Supabase(String),

    /// An error surfaced by the underlying SeaORM database layer.
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),

    /// Attempted to create a user with a username that already exists.
    #[error("Username '{0}' is already taken")]
    UsernameTaken(String),

    /// No user account exists for the requested id.
    #[error("User with id '{0}' not found")]
    UserNotFound(Uuid),

    /// An update affected zero rows for the given field/user.
    #[error("Failed to update {field} for user id '{id}'")]
    UpdateFailed { field: &'static str, id: Uuid },

    /// A counter increment was called with a negative value.
    #[error("Parameter error: {name} can only be >= 0, but '{value}' passed instead")]
    NegativeParameter { name: &'static str, value: i32 },
}
