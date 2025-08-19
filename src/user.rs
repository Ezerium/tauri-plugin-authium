use lazy_static::lazy_static;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use std::{sync::{Arc, Mutex}, time::{Duration, SystemTime}};
use reqwest::{Client, Response, Url};

use crate::{storage::{clear_user_data, save_user_data}, User, DAEMON_ENDPOINT};

pub static USER: Lazy<Mutex<Option<User>>> = Lazy::new(|| Mutex::new(None));
static EXPIRY: Lazy<Mutex<SystemTime>> = Lazy::new(|| Mutex::new(SystemTime::now()));

lazy_static! {
    static ref ACCESS_TOKEN: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    static ref REFRESH_TOKEN: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

pub async fn login(mut access_token: String, refresh_token: String, mut expiry: SystemTime) -> Result<User, String> {
    *EXPIRY.lock().unwrap() = expiry;
    ACCESS_TOKEN.lock().unwrap().replace(access_token.clone());
    REFRESH_TOKEN.lock().unwrap().replace(refresh_token.clone());

    if let Err(e) = refresh(refresh_token.clone()).await {
        return Err("Failed to refresh token".into());
    }
    
    access_token = ACCESS_TOKEN.lock().unwrap().clone().unwrap();
    expiry = *EXPIRY.lock().unwrap();
    if let Err(_) = save_user_data(access_token.as_str(), refresh_token.as_str(), expiry) {
        return Err("Failed to save user data".into());
    }

    let user = fetch_user_data(access_token).await;
    if let Ok(u) = user {
        *USER.lock().unwrap() = Some(u.clone());
        Ok(u)
    } else {
        eprintln!("Failed to fetch user data: {}", user.err().unwrap());
        Err("Failed to fetch user data".into())
    }
}

async fn fetch_user_data(token: String) -> Result<User, String> {
    let client = Client::new();
    let _res = client.get(format!("{}/app/user", DAEMON_ENDPOINT))
        .bearer_auth(token)
        .send()
        .await;
    if let Ok(response) = _res {
        if response.status().is_success() {
            let body = response.json::<User>().await.unwrap();
            return Ok(body);
        } else {
            clear();
            return Err(format!("Failed to fetch user data: {}", response.status()).to_string());
        }
    }

    clear();
    Err("Failed to fetch user data".into())
}

#[derive(Debug, Deserialize)]
struct RefreshData {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Serialize)]
struct RefreshRequest {
    refresh_token: String,
}

async fn refresh(refresh_token: String) -> Result<(), String> {
    let expiry = *EXPIRY.lock().unwrap();
    let diff = expiry.duration_since(SystemTime::now()).unwrap_or_default();
    if diff > Duration::from_secs(0) {
        return Ok(());
    }

    let body = &RefreshRequest {
        refresh_token
    };
    let client = Client::new();
    let _res = client.post(format!("{}/token/refresh", DAEMON_ENDPOINT))
        .json(body)
        .send()
        .await;

    if let Ok(response) = _res {
        let status = response.status();
        if status.is_success() {
            let body = response.json::<RefreshData>().await.unwrap();

            ACCESS_TOKEN.lock().unwrap().replace(body.access_token);
            *EXPIRY.lock().unwrap() = SystemTime::now() + Duration::from_secs(body.expires_in);
            return Ok(());
        } else {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            clear();
            return Err(format!("{} - {}", status, body).into());
        }
    } else {
        clear();
        return Err("Failed to refresh token".into());
    }
}

pub fn get_user() -> Option<User> {
    let _ = refresh(REFRESH_TOKEN.lock().unwrap().clone().unwrap_or_default());

    let user_lock = USER.lock().unwrap();
    user_lock.clone()
}

fn clear() {
    *USER.lock().unwrap() = None;
    *ACCESS_TOKEN.lock().unwrap() = None;
    *REFRESH_TOKEN.lock().unwrap() = None;
    *EXPIRY.lock().unwrap() = SystemTime::now();
    clear_user_data();
}

pub fn logout() {
    let token = REFRESH_TOKEN.lock().unwrap().clone();
    if let Some(token) = token {
        let client = Client::new();
        let _ = client
            .post(format!("{}/user/logout", DAEMON_ENDPOINT))
            .bearer_auth(token)
            .send();
    }

    clear();
}

pub fn is_logged_in() -> bool {
    let user = get_user();
    user.is_some() && ACCESS_TOKEN.lock().unwrap().is_some()
}