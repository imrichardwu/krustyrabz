use serde::{Deserialize, Serialize};
use dotenv::dotenv;
use std::env;
use storage::Repository;
use uuid::Uuid; 

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
struct AuthError {
    message: String,
}

/// Internal helper to register a new user with Supabase Auth
///
/// # Returns
/// - `Ok(AuthSession)` on success
/// - `Err(String)` with error message on failure
pub async fn register_helper(email: &str, username: &str, password: &str) -> Result<AuthSession, String> {
    ensure_dotenv_loaded();

    // Get Supabase URL and anon key from environment
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in .env file")?;
    
    let supabase_key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in .env file")?;
            
    if password.len() < 6 {
        return Err("Password must be at least 6 characters long".to_string());
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
        .map_err(|e| format!("Network error: {}", e))?;
    
    let status = response.status();
    let response_text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    
    if status.is_success() {
        let auth_response: AuthResponse = serde_json::from_str(&response_text).map_err(|e| format!("Failed to parse registration response: {}", e))?;
      
        let access_token: String = auth_response.access_token.ok_or_else(|| "Registration response missing access token".to_string())?;
        let refresh_token: String = auth_response.refresh_token.ok_or_else(|| "Registration response missing refresh token".to_string())?;

        let session_username = auth_response.user.user_metadata.and_then(|m| m.get("username").and_then(|v| v.as_str().map(|s| s.to_string()))) .unwrap_or_else(|| username.to_string());

        // Create user in app database (UserAccount table) so the server can find you when creating/joining games
        let user_id = auth_response.user.id.parse::<Uuid>()
            .map_err(|_| "Invalid user id from Supabase Auth".to_string())?;
        let repository = Repository::new().await
            .map_err(|e| format!("Cannot connect to database. Set DATABASE_URL in .env (same as server). Error: {}", e))?;
        repository.create_user(session_username.clone(), user_id)
            .await
            .map_err(|e| format!("User created in Supabase Auth but failed to create in app database (UserAccount). Run migrations: cargo run -p storage --bin migrate -- up. Error: {}", e))?;

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
        match serde_json::from_str::<AuthError>(&response_text) {
            Ok(error) => {
                Err(format!("Registration failed: {}", error.message))
            }
            Err(_) => {
                Err(format!("Registration failed with status {}: {}", status, response_text))
            }
        }
    }
}


/// Internal helper to login with credentials (used after Admin API user creation)
async fn login_with_credentials(email: &str, password: &str) -> Result<AuthSession, String> {
    dotenv().ok();
    
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in .env file")?;
    
    let supabase_key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in .env file")?;
    
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
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let access_token = auth_response.access_token
            .ok_or_else(|| "Login response missing access token".to_string())?;
        let refresh_token = auth_response.refresh_token
            .ok_or_else(|| "Login response missing refresh token".to_string())?;
        
        let username = auth_response
            .user
            .user_metadata
            .and_then(|m| m.get("username").and_then(|v| v.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| "User".to_string());


        
        Ok(AuthSession {
            access_token,
            refresh_token,
            user_id: auth_response.user.id,
            email: auth_response.user.email,
            username,
        })
    } else {
        let error: AuthError = response
            .json()
            .await
            .unwrap_or(AuthError {
                message: "Invalid email or password".to_string(),
            });
        Err(format!("Login failed: {}", error.message))
    }
}

/// Login with existing credentials
/// 
/// # Returns
/// - `Ok(AuthSession)` on success
/// - `Err(String)` with error message on failure
pub async fn login_helper(_username: &str, password: &str, email: &str) -> Result<AuthSession, String> {
    ensure_dotenv_loaded();
    
    // Use the helper function
    let session = login_with_credentials(email, password).await?;

    // Ensure UserAccount row exists (so create_game / join_game can find the user)
    if let Ok(user_uuid) = Uuid::parse_str(&session.user_id) {
        if let Some(repo) = Repository::new().await.ok() {
            if repo.get_user_by_id(user_uuid).await.is_err() {
                let _ = repo.create_user(session.username.clone(), user_uuid).await;
            }
        }
    }

    Ok(session)
}
