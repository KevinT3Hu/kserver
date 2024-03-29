use core::panic;
use std::{io::Write, sync::Arc, time::SystemTimeError};

use axum::{
    body::BoxBody,
    extract::State,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Router,
};
use helper::db::DbHelper;
use rand::Rng;
use tokio::sync::Mutex;
use totp_rs::{Algorithm, TOTP};
use tower_http::cors::{Any, CorsLayer};
use tracing::{event, Level};

mod helper;
mod model;
mod router;

pub type AuthToken = String;

fn gen_token() -> AuthToken {
    let mut rng = rand::thread_rng();
    let mut token = vec![];
    for _ in 0..32 {
        token.push(rng.gen::<u8>());
    }
    hex::encode(token)
}

#[derive(Clone)]
struct AppState {
    pub db_helper: DbHelper,
    totp: TOTP,
    token: Arc<Mutex<Vec<AuthToken>>>,
}

pub enum AuthStatus {
    Authenticated,
    AuthNotValid,
    AuthExpired,
    NotLoggedIn,
}

impl AppState {
    pub async fn new() -> Self {
        event!(Level::INFO, "Start creating app state...");
        event!(Level::INFO, "Creating TOTP...");

        let totp = init_totp();

        event!(Level::INFO, "Creating database helper...");
        let db_helper = DbHelper::new().await;
        event!(Level::INFO, "Database helper created");

        let token = Arc::new(Mutex::new(vec![]));

        Self {
            db_helper,
            totp,
            token,
        }
    }

    pub fn verify(&self, code: &str) -> Result<bool, SystemTimeError> {
        // if MOCK_TOTP is set, return true
        if std::env::var("MOCK_TOTP").is_ok() {
            return Ok(true);
        }
        self.totp.check_current(code)
    }

    pub async fn auth(&self, in_token: &str) -> AuthStatus {
        let token = self.token.lock().await;

        event!(Level::INFO, "Checking token: {}", in_token);
        event!(Level::INFO, "Token list: {:?}", token);

        // check if in_token is in token list
        let mut found = false;
        for i in 0..token.len() {
            if token[i] == in_token {
                found = true;
                break;
            }
        }
        if !found {
            return AuthStatus::NotLoggedIn;
        }
        event!(Level::INFO, "Token found");
        AuthStatus::Authenticated
    }

    pub async fn gen_token(&self) -> String {
        let mut token = self.token.lock().await;
        let auth_token = gen_token();
        token.push(auth_token.clone());
        event!(Level::INFO, "Token generated: {}", auth_token);
        auth_token
    }

    pub async fn clear_token(&self, in_token: &str) {
        let mut token = self.token.lock().await;
        let mut found = false;
        for i in 0..token.len() {
            if token[i] == in_token {
                found = true;
                break;
            }
        }
        if !found {
            return;
        }
        token.swap_remove(usize::from(found));
    }
}

fn init_totp() -> TOTP {
    let totp = TOTP::new(
        Algorithm::SHA256,
        8,
        1,
        30,
        std::env::var("KSERVER_SECRET").unwrap().into_bytes(),
        Some("KServer".to_owned()),
        "SmilingPie".to_owned(),
    );
    if let Err(e) = totp {
        panic!("Error: {:?}", e);
    }
    let totp = totp.unwrap();
    event!(Level::INFO, "TOTP created");
    event!(
        Level::INFO,
        "totp secret: {}",
        String::from_utf8(totp.secret.clone()).unwrap()
    );
    totp
}

async fn auth_middleware<B>(
    State(app_state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let token = request.headers().get("Authorization");
    if token.is_none() {
        return Response::builder()
            .status(401)
            .body(BoxBody::default())
            .unwrap();
    }
    let token = token.unwrap().to_str().unwrap();
    let token = token.split(' ').collect::<Vec<&str>>();
    if token.len() != 2 {
        return Response::builder()
            .status(401)
            .body(BoxBody::default())
            .unwrap();
    }
    let token = token[1];
    let ret = app_state.auth(token).await;

    match ret {
        AuthStatus::Authenticated => {
            event!(Level::INFO, "Authenticated");
            next.run(request).await
        }
        AuthStatus::AuthNotValid => {
            event!(Level::INFO, "Auth not valid");
            status!(UNAUTHORIZED, "AuthNotValid").into_response()
        }
        AuthStatus::AuthExpired => {
            event!(Level::INFO, "Auth expired");
            status!(UNAUTHORIZED, "AuthExpired").into_response()
        }
        AuthStatus::NotLoggedIn => {
            event!(Level::INFO, "Not logged in");
            status!(UNAUTHORIZED, "NotLoggedIn").into_response()
        }
    }
}

#[tokio::main]
async fn main() {

    if std::env::var("GENERATE_TOTP_QR").is_ok() {
        let totp = init_totp();
        std::fs::remove_file("./qr.png").unwrap_or_default();
            let qr = totp.get_qr_png().unwrap();
            let mut file = std::fs::File::create("./qr.png").unwrap();
            file.write_all(&qr).unwrap();
        return;
    }

    let file_appender = tracing_appender::rolling::daily("/root/logs", "kserver.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(non_blocking)
        .init();

    let app = create_app().await;

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_app() -> Router {
    let state = AppState::new().await;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .nest(
            "/",
            router::create(&state)
        )
        .nest(
            router::anime::PATH,
            router::anime::create(&state),
        )
        .with_state(state)
        .layer(cors)
}