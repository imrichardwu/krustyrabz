use poker_core::{
    ActionRequest, AddChipsRequest, AddChipsResponse, CreateGameRequest, DealerChoiceRequest,
    GameAction, GameListResponse, GameResponse, GameStateUpdate, GameType, HouseRules,
    JoinGameRequest, ServerResponse, SitOutRequest, ViewerRequest, WithdrawChipsRequest,
    WithdrawChipsResponse,
};
use reqwest::Client;
use thiserror::Error;

pub struct PokerClient {
    client: Client,
    base_url: String,
}

impl PokerClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub fn localhost() -> Self {
        Self::new("http://127.0.0.1:8000")
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn list_games(&self) -> Result<GameListResponse, ApiError> {
        let response = self
            .client
            .get(format!("{}/games", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn create_game(
        &self,
        player_id: &str,
        username: &str,
        game_type: GameType,
    ) -> Result<GameResponse, ApiError> {
        let request = CreateGameRequest {
            player_id: player_id.to_string(),
            username: username.to_string(),
            game_type,
        };

        let response = self
            .client
            .post(format!("{}/games", self.base_url))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn join_game(
        &self,
        player_id: &str,
        username: &str,
        game_id: &str,
    ) -> Result<GameResponse, ApiError> {
        let request = JoinGameRequest {
            player_id: player_id.to_string(),
            username: username.to_string(),
            game_id: game_id.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/games/{}/join", self.base_url, game_id))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn dealer_choice(
        &self,
        game_id: &str,
        game_type: &str,
    ) -> Result<GameResponse, ApiError> {
        let request = DealerChoiceRequest {
            game_id: game_id.to_string(),
            game_type: game_type.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/games/{}/dealer_choice", self.base_url, game_id))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn leave_game(
        &self,
        player_id: &str,
        game_id: &str,
    ) -> Result<ServerResponse, ApiError> {
        let response = self
            .client
            .post(format!("{}/games/{}/leave", self.base_url, game_id))
            .json(
                &serde_json::json!({ "game_id": game_id, "username": "", "player_id": player_id }),
            )
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn get_game(
        &self,
        game_id: &str,
        player_id: &str,
    ) -> Result<GameStateUpdate, ApiError> {
        let response = self
            .client
            .get(format!("{}/games/{}", self.base_url, game_id))
            .query(&[("player_id", player_id)])
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("Request failed with status {}", status));
            return Err(ApiError::Server(msg));
        }
        let state = response.json().await?;
        Ok(state)
    }

    pub async fn start_hand(
        &self,
        game_id: &str,
        player_id: &str,
    ) -> Result<GameResponse, ApiError> {
        let body = serde_json::json!({ "player_id": player_id });
        let response = self
            .client
            .post(format!("{}/games/{}/start", self.base_url, game_id))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn perform_action(
        &self,
        player_id: &str,
        game_id: &str,
        action: GameAction,
    ) -> Result<GameResponse, ApiError> {
        let request = ActionRequest {
            player_id: player_id.to_string(),
            game_id: game_id.to_string(),
            action,
        };

        let response = self
            .client
            .post(format!("{}/games/{}/action", self.base_url, game_id))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn fold(&self, player_id: &str, game_id: &str) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Fold)
            .await
    }

    pub async fn check(&self, player_id: &str, game_id: &str) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Check)
            .await
    }

    pub async fn pass(&self, player_id: &str, game_id: &str) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Pass)
            .await
    }

    pub async fn call(&self, player_id: &str, game_id: &str) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Call)
            .await
    }

    pub async fn bet(
        &self,
        player_id: &str,
        game_id: &str,
        amount: u32,
    ) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Bet { amount })
            .await
    }

    pub async fn raise(
        &self,
        player_id: &str,
        game_id: &str,
        amount: u32,
    ) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Raise { amount })
            .await
    }

    pub async fn draw(
        &self,
        player_id: &str,
        game_id: &str,
        discard_indices: Vec<usize>,
    ) -> Result<GameResponse, ApiError> {
        self.perform_action(player_id, game_id, GameAction::Draw { discard_indices })
            .await
    }

    pub async fn withdraw_chips(
        &self,
        player_id: &str,
        num: u32,
    ) -> Result<WithdrawChipsResponse, ApiError> {
        let request = WithdrawChipsRequest {
            player_id: player_id.to_string(),
            num_chips: num,
        };
        let response = self
            .client
            .post(format!(
                "{}/players/{}/withdrawchips",
                self.base_url, player_id
            ))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn add_chips(&self, player_id: &str, num: u32) -> Result<AddChipsResponse, ApiError> {
        let request = AddChipsRequest {
            player_id: player_id.to_string(),
            num_chips: num,
            credit_limit: 65535,
        };
        let response = self
            .client
            .post(format!("{}/players/{}/addchips", self.base_url, player_id))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn sit_out_hand(
        &self,
        player_id: &str,
        game_id: &str,
    ) -> Result<ServerResponse, ApiError> {
        let request = SitOutRequest {
            player_id: player_id.to_string(),
        };
        let response = self
            .client
            .post(format!("{}/games/{}/sitout", self.base_url, game_id))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn register_viewer(
        &self,
        viewer_id: &str,
        game_id: &str,
    ) -> Result<ServerResponse, ApiError> {
        let request = ViewerRequest {
            viewer_id: viewer_id.to_string(),
            game_id: game_id.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/games/{}/viewers", self.base_url, game_id))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn get_rules(&self) -> Result<HouseRules, ApiError> {
        let response = self
            .client
            .get(format!("{}/rules", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Server error: {0}")]
    Server(String),
}
