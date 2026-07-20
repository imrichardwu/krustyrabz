use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use storage::Repository;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while registering or logging a user in.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("{0} not found in .env file")]
    MissingEnv(&'static str),

    #[error("Password must be at least 6 characters long")]
    PasswordTooShort,

    #[error("Network error: {0}")]
    Network(#[source] reqwest::Error),

    #[error("Failed to read response: {0}")]
    ReadResponse(#[source] reqwest::Error),

    #[error("Failed to parse registration response: {0}")]
    ParseRegistration(#[source] serde_json::Error),

    #[error("Failed to parse response: {0}")]
    ParseResponse(#[source] reqwest::Error),

    #[error("Registration response missing access token")]
    RegistrationMissingAccessToken,

    #[error("Registration response missing refresh token")]
    RegistrationMissingRefreshToken,

    #[error("Login response missing access token")]
    LoginMissingAccessToken,

    #[error("Login response missing refresh token")]
    LoginMissingRefreshToken,

    #[error("Invalid user id from Supabase Auth")]
    InvalidUserId,

    #[error("Cannot connect to database. Set DATABASE_URL in .env (same as server). Error: {0}")]
    DatabaseConnect(String),

    #[error(
        "User created in Supabase Auth but failed to create in app database (UserAccount). Run migrations: cargo run -p storage --bin migrate -- up. Error: {0}"
    )]
    CreateUserInDb(String),

    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Registration failed with status {status}: {body}")]
    RegistrationFailedStatus {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("Login failed: {0}")]
    LoginFailed(String),
}

/// Load .env so DATABASE_URL is set for Repository (UserAccount table).
/// Tries current dir, then parent (project root when run from client/ or IDE).
fn ensure_dotenv_loaded() {
    dotenv().ok();
    if env::var("DATABASE_URL").is_ok() {
        return;
    }
    if let Ok(cwd) = env::current_dir() {
        for dir in [cwd.as_path(), cwd.join("..").as_path()] {
            let env_path = dir.join(".env");
            if env_path.exists() {
                dotenv::from_path(&env_path).ok();
                break;
            }
        }
    }
}

/// Authentication session containing user info and access token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SignInRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    user: UserInfo,
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    id: String,
    email: String,
    user_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AuthErrorResponse {
    message: String,
}

/// Internal helper to register a new user with Supabase Auth.
pub async fn register_helper(
    email: &str,
    username: &str,
    password: &str,
) -> Result<AuthSession, AuthError> {
    ensure_dotenv_loaded();

    // Get Supabase URL and anon key from environment
    let supabase_url =
        env::var("SUPABASE_URL").map_err(|_| AuthError::MissingEnv("SUPABASE_URL"))?;

    let supabase_key =
        env::var("SUPABASE_KEY").map_err(|_| AuthError::MissingEnv("SUPABASE_KEY"))?;

    if password.len() < 6 {
        return Err(AuthError::PasswordTooShort);
    }

    // Use regular signup endpoint (email confirmation disabled in Supabase settings)
    let signup_url = format!("{}/auth/v1/signup", supabase_url);

    let request_body = serde_json::json!({
        "email": email,
        "password": password,
        "data": {
            "username": username
        }
    });

    // Make HTTP POST request
    let client = reqwest::Client::new();
    let request = client
        .post(&signup_url)
        .header("Content-Type", "application/json")
        .header("apikey", &supabase_key);

    let response = request
        .json(&request_body)
        .send()
        .await
        .map_err(AuthError::Network)?;

    let status = response.status();
    let response_text = response.text().await.map_err(AuthError::ReadResponse)?;

    if status.is_success() {
        let auth_response: AuthResponse =
            serde_json::from_str(&response_text).map_err(AuthError::ParseRegistration)?;

        let access_token: String = auth_response
            .access_token
            .ok_or(AuthError::RegistrationMissingAccessToken)?;
        let refresh_token: String = auth_response
            .refresh_token
            .ok_or(AuthError::RegistrationMissingRefreshToken)?;

        let session_username = auth_response
            .user
            .user_metadata
            .and_then(|m| {
                m.get("username")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| username.to_string());

        // Create user in app database (UserAccount table) so the server can find you when creating/joining games
        let user_id = auth_response
            .user
            .id
            .parse::<Uuid>()
            .map_err(|_| AuthError::InvalidUserId)?;
        let repository = Repository::new()
            .await
            .map_err(|e| AuthError::DatabaseConnect(e.to_string()))?;
        repository
            .create_user(session_username.clone(), user_id)
            .await
            .map_err(|e| AuthError::CreateUserInDb(e.to_string()))?;

        println!("User '{}' registered successfully!", session_username);

        Ok(AuthSession {
            access_token,
            refresh_token,
            user_id: auth_response.user.id,
            email: auth_response.user.email,
            username: session_username,
        })
    } else {
        // Registration failed - get error message
        match serde_json::from_str::<AuthErrorResponse>(&response_text) {
            Ok(error) => Err(AuthError::RegistrationFailed(error.message)),
            Err(_) => Err(AuthError::RegistrationFailedStatus {
                status,
                body: response_text,
            }),
        }
    }
}

async fn login_with_credentials(email: &str, password: &str) -> Result<AuthSession, AuthError> {
    dotenv().ok();

    let supabase_url =
        env::var("SUPABASE_URL").map_err(|_| AuthError::MissingEnv("SUPABASE_URL"))?;

    let supabase_key =
        env::var("SUPABASE_KEY").map_err(|_| AuthError::MissingEnv("SUPABASE_KEY"))?;

    let signin_url = format!("{}/auth/v1/token?grant_type=password", supabase_url);
    let request_body = SignInRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&signin_url)
        .header("apikey", &supabase_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(AuthError::Network)?;

    if response.status().is_success() {
        let auth_response: AuthResponse =
            response.json().await.map_err(AuthError::ParseResponse)?;

        let access_token = auth_response
            .access_token
            .ok_or(AuthError::LoginMissingAccessToken)?;
        let refresh_token = auth_response
            .refresh_token
            .ok_or(AuthError::LoginMissingRefreshToken)?;

        let username = auth_response
            .user
            .user_metadata
            .and_then(|m| {
                m.get("username")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| "User".to_string());

        Ok(AuthSession {
            access_token,
            refresh_token,
            user_id: auth_response.user.id,
            email: auth_response.user.email,
            username,
        })
    } else {
        let error: AuthErrorResponse = response.json().await.unwrap_or(AuthErrorResponse {
            message: "Invalid email or password".to_string(),
        });
        Err(AuthError::LoginFailed(error.message))
    }
}

/// Login with existing credentials.
pub async fn login_helper(
    _username: &str,
    password: &str,
    email: &str,
) -> Result<AuthSession, AuthError> {
    ensure_dotenv_loaded();

    // Use the helper function
    let session = login_with_credentials(email, password).await?;

    // Ensure UserAccount row exists (so create_game / join_game can find the user)
    if let Ok(user_uuid) = Uuid::parse_str(&session.user_id) {
        let repo = match Repository::new().await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("!! LOGIN FALLBACK DB CONNECTION FAILED: {}", e);
                return Ok(session);
            }
        };

        if repo.get_user_by_id(user_uuid).await.is_err() {
            if let Err(e) = repo.create_user(session.username.clone(), user_uuid).await {
                eprintln!("!! LOGIN FALLBACK FAILED TO CREATE USER: {}", e);
            } else {
                println!("Recovered missing user account during login!");
            }
        }
    }

    Ok(session)
}
