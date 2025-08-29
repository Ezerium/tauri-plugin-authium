use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use actix_web::rt::net::TcpListener;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use rand::Rng as _;
use serde::Deserialize;
use tauri::{AppHandle, Emitter, Runtime, Url};

use crate::{AuthiumConfig, AUTHIUM_ENDPOINT};
use crate::user::{is_logged_in, login as user_login};

lazy_static! {
    static ref CSRF_TOKEN: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    static ref AUTH_CONFIG: Arc<Mutex<Option<AuthiumConfig>>> = Arc::new(Mutex::new(None));
}

#[actix_web::main]
pub async fn start_server<R: Runtime + 'static>(handle: &AppHandle<R>, config: &AuthiumConfig) -> std::io::Result<()> {
    AUTH_CONFIG.lock().unwrap().replace(config.clone());
    let port = config.port.unwrap_or(6483);

    let handle_data = web::Data::new(Arc::new(handle.clone()));
    match TcpListener::bind(format!("localhost:{}", port)).await {
        Ok(_) => {}
        Err(_) => return Ok(()),
    }

    HttpServer::new(move || {
        App::new()
            .app_data(handle_data.clone())
            .service(login)
            .service(callback)
    })
    .bind(format!("localhost:{}", port))?
    .run()
    .await
}

#[derive(Deserialize)]
struct LoginQuery {
    expiry: Option<u64>,
}

#[get("/login")]
async fn login(query: web::Query<LoginQuery>) -> impl Responder {
    let state = rand_str(32);
    CSRF_TOKEN.lock().unwrap().replace(state.clone());

    if is_logged_in() {
        return HttpResponse::Ok()
            .body("You are already logged in. You can close this window.");
    }

    let config = AUTH_CONFIG.lock().unwrap().as_ref().unwrap().clone();
    let mut api_key = config.api_key.clone();
    let mut app_id = config.app_id.clone();
    if api_key.to_lowercase().starts_with("env:") {
        api_key = std::env::var(&api_key[4..]).unwrap_or_default();
    }

    if app_id.to_lowercase().starts_with("env:") {
        app_id = std::env::var(&app_id[4..]).unwrap_or_default();
    }

    let expiry = query.expiry.clone();
    let mut url = Url::parse(format!("{}/authorize", AUTHIUM_ENDPOINT).as_str()).unwrap();
    url.query_pairs_mut()
        .append_pair("apiKey", &config.api_key)
        .append_pair("appId", &config.app_id)
        .append_pair("state", &state);
    if let Some(expiry) = expiry {
        url.query_pairs_mut().append_pair("exp", &expiry.to_string());
    }

    HttpResponse::Found()
        .append_header(("Location", url.to_string()))
        .finish()
}

#[derive(Deserialize)]
struct CallbackQuery {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    state: String,
}

#[get("/callback")]
async fn callback(
    query: web::Query<CallbackQuery>,
    handle: web::Data<Arc<AppHandle>>,
) -> impl Responder {
    let access_token = query.access_token.clone();
    let refresh_token = query.refresh_token.clone();
    let expires_in = query.expires_in;
    let state = query.state.clone();

    if state != *CSRF_TOKEN.lock().unwrap().as_ref().unwrap() {
        return HttpResponse::BadRequest().finish();
    }

    let expiry = SystemTime::now() + Duration::from_secs(expires_in);
    if let Ok(u) = user_login(access_token, refresh_token, expiry).await {
        handle.emit("authium:login-success", u).expect("Failed to emit login event");
    }

    CSRF_TOKEN.lock().unwrap().take();

    HttpResponse::Ok()
        .body("Authentication successful! You can close this window.")
}

fn rand_str(len: usize) -> String {
    let mut rng = rand::rng();
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let result: String = (0..len)
        .map(|_| {
            let i = rng.random_range(0..chars.len());
            chars.chars().nth(i).unwrap()
        })
        .collect();
    result
}