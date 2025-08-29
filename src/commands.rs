use tauri::{Manager, State, Url, WebviewUrl, WebviewWindowBuilder};
use tauri::{AppHandle, command, Runtime};

use crate::{models::*, AuthiumConfig};

#[command]
pub(crate) fn sign_in<R: Runtime>(app: AppHandle<R>, config: State<AuthiumConfig>, expiry: Option<u64>) {
    let port = config.port.unwrap_or(6483);
    let mut url_str = format!("http://localhost:{}/login", port);
    if let Some(expiry) = expiry {
        url_str.push_str(&format!("?expiry={}", expiry));
    }
    let url = Url::parse(&url_str).expect("invalid url");

    tauri::async_runtime::spawn(async move {
        let window = WebviewWindowBuilder::new(&app, "authium-auth-cb-signin".to_string(), WebviewUrl::External(url))
            .title("Authium | Sign In")
            .build()
            .expect("Failed to create window");

        window.show().expect("Failed to show window");
    });
}

#[command]
pub(crate) fn get_user() -> Option<User> {
    crate::user::get_user()
}

#[command]
pub(crate) fn logout() {
    crate::user::logout();
}

#[command]
pub(crate) fn is_logged_in() -> bool {
    crate::user::is_logged_in()
}

#[command]
pub(crate) fn refresh(refresh_data: bool) {
    let _ = crate::user::refresh_user(refresh_data);
}