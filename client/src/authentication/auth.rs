use serde::{Deserialize, Serialize};
use std::io;
use dotenv::dotenv;
use std::env;
use storage::Repository;


/// Authentication session containing user info and access token
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
    pub username: String,
}

/// Supabase Auth API response structures
#[derive(Debug, Serialize, Deserialize)]
struct SignUpRequest {
    email: String,
    password: String,
    data: Option<serde_json::Value>,
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
    error: Option<String>,
}

/// Register a new user with Supabase Auth
/// 
/// # Returns
/// - `Ok(AuthSession)` on success
/// - `Err(String)` with error message on failure
pub async fn register() -> Result<AuthSession, String> {
    dotenv().ok();
    
    // Get Supabase URL and anon key from environment
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in .env file")?;
    
    let supabase_key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in .env file")?;
    
    // Get user input
    println!("Enter email:");
    let mut email = String::new();
    io::stdin().read_line(&mut email).expect("Failed to read line");
    let email = email.trim().to_string();
    
    println!("Enter password (min 6 characters):");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Failed to read line");
    let password = password.trim().to_string();
    
    if password.len() < 6 {
        return Err("Password must be at least 6 characters long".to_string());
    }
    
    println!("Enter username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read line");
    let username = username.trim().to_string();
    
    // Use regular signup endpoint (email confirmation disabled in Supabase settings)
    let signup_url = format!("{}/auth/v1/signup", supabase_url);
    
    let request_body = serde_json::json!({
        "email": email.clone(),
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

        let session_username = auth_response.user.user_metadata.and_then(|m| m.get("username").and_then(|v| v.as_str().map(|s| s.to_string()))) .unwrap_or_else(|| username.clone());

        println!("User '{}' registered successfully!", session_username);

        // create user in database
        let repository = Repository::new().await
            .map_err(|e| format!("Failed to create repository: {}", e))?;
        let _user = repository.create_user(session_username.clone())
            .await
            .map_err(|e| format!("Failed to create user in database: {}", e))?;

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
                error: None,
            });
        Err(format!("Login failed: {}", error.message))
    }
}

/// Login with existing credentials
/// 
/// # Returns
/// - `Ok(AuthSession)` on success
/// - `Err(String)` with error message on failure
pub async fn login() -> Result<AuthSession, String> {
    dotenv().ok();
    
    // Get Supabase URL and anon key from environment
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in .env file")?;
    
    let supabase_key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in .env file")?;
    
    // Get user input
    println!("Enter email:");
    let mut email = String::new();
    io::stdin().read_line(&mut email).expect("Failed to read line");
    let email = email.trim().to_string();
    
    println!("Enter password:");
    let mut password = String::new();
    io::stdin().read_line(&mut password).expect("Failed to read line");
    let password = password.trim().to_string();
    
    // Use the helper function
    let session = login_with_credentials(&email, &password).await?;
    println!("User '{}' logged in successfully!", session.username);
    Ok(session)
}

/// Verify if a session token is still valid
pub async fn verify_session(access_token: &str) -> Result<bool, String> {
    dotenv().ok();
    
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in .env file")?;
    
    let supabase_key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in .env file")?;
    
    let verify_url = format!("{}/auth/v1/user", supabase_url);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&verify_url)
        .header("apikey", &supabase_key)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    Ok(response.status().is_success())
}

/// Refresh an access token using a refresh token
pub async fn refresh_token(refresh_token: &str) -> Result<AuthSession, String> {
    dotenv().ok();
    
    let supabase_url = env::var("SUPABASE_URL")
        .map_err(|_| "SUPABASE_URL not found in .env file")?;
    
    let supabase_key = env::var("SUPABASE_KEY")
        .map_err(|_| "SUPABASE_KEY not found in .env file")?;
    
    let refresh_url = format!("{}/auth/v1/token?grant_type=refresh_token", supabase_url);
    
    let client = reqwest::Client::new();
    let response = client
        .post(&refresh_url)
        .header("apikey", &supabase_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status().is_success() {
        let auth_response: AuthResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let access_token = auth_response.access_token
            .ok_or_else(|| "Refresh response missing access token".to_string())?;
        let refresh_token = auth_response.refresh_token
            .ok_or_else(|| "Refresh response missing refresh token".to_string())?;
        
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
        Err("Failed to refresh token".to_string())
    }
}
