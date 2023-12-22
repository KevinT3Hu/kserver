use axum::{extract::State, http::StatusCode, Json, routing::post, middleware::from_fn_with_state};
use tracing::event;

use crate::{
    model::request::{LogInRequest, LogOutRequest},
    AppState, auth_middleware
};

pub mod anime_router;

pub type ComplexResponse = (StatusCode, String);
pub type Result<T> = std::result::Result<T, ComplexResponse>;

pub fn create_router(state: &AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/validate", post(post_validate_login))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
        .route("/login", post(post_log_in))
        .route("/logout", post(post_log_out))
}

#[macro_export]
macro_rules! status {
    ($status:ident) => {
        (StatusCode::$status, String::new())
    };
    ($status:ident, $($msg:expr),+) => {
        (StatusCode::$status, format!($($msg),+))
    };
}

#[macro_export]
macro_rules! internal_error {
    ($($e:expr),+) => {
        tracing::error!($($e),+);
        return Err(status!(INTERNAL_SERVER_ERROR));
    };

    () => {
        return Err(status!(INTERNAL_SERVER_ERROR));
    }
}

async fn post_validate_login() -> Result<ComplexResponse> {
    Ok(status!(NO_CONTENT))
}

async fn post_log_in(
    State(app_state): State<AppState>,
    Json(req): Json<LogInRequest>,
) -> Result<String> {
    event!(
        tracing::Level::INFO,
        "Received request to log in, OTP: {}",
        req.otp
    );
    let ret = app_state.verify(&req.otp);
    if let Err(()) = &ret {
        internal_error!("Error Verifying OTP");
    }
    let ret = ret.unwrap();
    if ret {
        return Ok(app_state.gen_token().await);
    }
    Err(status!(UNAUTHORIZED, "OtpNotValid"))
}

async fn post_log_out(
    State(app_state): State<AppState>,
    Json(req): Json<LogOutRequest>,
) -> Result<String> {
    event!(tracing::Level::INFO, "Received request to log out");
    app_state.clear_token(&req.token).await;
    Ok(String::new())
}
