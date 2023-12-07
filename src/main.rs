use core::panic;
use std::{io::Write, sync::Arc};

use axum::{
    body::BoxBody,
    extract::State,
    http::{Request, StatusCode, Method},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use helper::db_helper::DbHelper;
use rand::Rng;
use router::{post_log_in, post_log_out};
use tokio::sync::Mutex;
use totp_rs::{Algorithm, TOTP};
use tower_http::cors::{CorsLayer, Any};
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
pub struct AppState {
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

        // if envvar $GENERATE_TOTP_QR is set, generate a QR code for TOTP and save to ./qr.png
        if let Ok(_) = std::env::var("GENERATE_TOTP_QR") {
            event!(Level::INFO, "Generating QR code for TOTP");
            // remove the previous qr.png
            std::fs::remove_file("./qr.png").unwrap_or_default();
            let qr = totp.get_qr_png().unwrap();
            let mut file = std::fs::File::create("./qr.png").unwrap();
            file.write_all(&qr).unwrap();
            event!(Level::INFO, "QR code generated");
        }

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

    pub fn verify(&self, code: &str) -> Result<bool, ()> {
        // if MOCK_TOTP is set, return true
        if let Ok(_) = std::env::var("MOCK_TOTP") {
            return Ok(true);
        }
        self.totp.check_current(code).map_err(|_| ())
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
        token.swap_remove(found as usize);
    }
}

fn init_totp() -> TOTP {
    let totp = TOTP::new(
        Algorithm::SHA256,
        8,
        1,
        30,
        "cewiugvnrb948gn9ffNIS".to_owned().into_bytes(),
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
        String::from_utf8(totp.secret.to_vec()).unwrap()
    );
    totp
}

async fn auth_middleware<B>(
    State(app_state): State<AppState>,
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let token = req
        .headers()
        .get("Authorization");
    if token.is_none() {
        return Response::builder()
            .status(401)
            .body(BoxBody::default())
            .unwrap();
    }
    let token = token.unwrap().to_str().unwrap();
    let token = token.split(" ").collect::<Vec<&str>>();
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
            return next.run(req).await;
        }
        AuthStatus::AuthNotValid => {
            event!(Level::INFO, "Auth not valid");
            return status!(UNAUTHORIZED, "AuthNotValid").into_response();
        }
        AuthStatus::AuthExpired => {
            event!(Level::INFO, "Auth expired");
            return status!(UNAUTHORIZED, "AuthExpired").into_response();
        }
        AuthStatus::NotLoggedIn => {
            event!(Level::INFO, "Not logged in");
            return status!(UNAUTHORIZED, "NotLoggedIn").into_response();
        }
    }
}

#[tokio::main]
async fn main() {
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
        .allow_methods([Method::GET,Method::POST])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .nest(
            router::anime_router::PATH,
            router::anime_router::create_router(&state),
        )
        .route("/auth", post(post_log_in))
        .route("/logout", post(post_log_out))
        .with_state(state)
        .layer(cors)
}