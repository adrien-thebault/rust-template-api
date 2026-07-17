use crate::AppState;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use rust_toolbox::axum_tools::{
    api_error::ApiError,
    auth::{Claims, Credential, User},
    controller::Controller,
};
use serde_derive::{Deserialize, Serialize};

/// admin session TTL
const SESSION_TTL: chrono::Duration = chrono::Duration::hours(12);

/// body for [`AuthController::login`]
#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

/// response body for [`AuthController::login`]
#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
}

/// owns `/api/auth/*`
pub struct AuthController;

impl Controller<AppState> for AuthController {
    fn router(&self) -> Router<AppState> {
        Router::new().nest(
            "/auth",
            Router::new()
                .route("/login", post(Self::login))
                .route("/me", get(Self::me)),
        )
    }
}

impl AuthController {
    /// `POST /api/auth/login` - public; issues a bearer session token from a
    /// [`Credential::Basic`] username/password pair
    async fn login(
        State(state): State<AppState>,
        Json(req): Json<LoginRequest>,
    ) -> Result<Json<LoginResponse>, ApiError> {
        let user = state.auth.authenticate(Credential::Basic {
            username: req.username,
            password: req.password,
        })?;

        let claims = Claims::for_user(&user, SESSION_TTL);
        let token = state.jwt.encode(&claims)?;
        Ok(Json(LoginResponse { token }))
    }

    /// `GET /api/auth/me` - requires a valid session (any role); returns the
    /// caller's identity, primarily so a frontend can check whether it's
    /// currently authenticated
    async fn me(user: User) -> Json<User> {
        Json(user)
    }
}
