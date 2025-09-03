use std::{sync::Mutex, thread};

use serde::Deserialize;
use tauri::{
    plugin::{Builder, TauriPlugin}, Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
pub mod models;
mod server;
pub mod user;
mod storage;

static DATA_DIR: Mutex<String> = Mutex::new(String::new());
// Development endpoints
//static DAEMON_ENDPOINT: &str = "http://localhost:8085/v1";
//static AUTHIUM_ENDPOINT: &str = "http://localhost:3000";
// Production endpoints
static DAEMON_ENDPOINT: &str = "https://api.authium.ezerium.com/v1";
static AUTHIUM_ENDPOINT: &str = "https://authium.ezerium.com";

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Authium;
#[cfg(mobile)]
use mobile::Authium;

use crate::storage::setup_storage;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the authium APIs.
pub trait AuthiumExt<R: Runtime> {
    fn authium(&self) -> &Authium<R>;
}

impl<R: Runtime, T: Manager<R>> crate::AuthiumExt<R> for T {
    fn authium(&self) -> &Authium<R> {
        self.state::<Authium<R>>().inner()
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthiumConfig {
    pub api_key: Option<String>,
    pub app_id: Option<String>,
    pub port: Option<u16>,
}

impl AuthiumConfig {
    pub fn new(api_key: String, app_id: String) -> Self {
        Self { api_key: Some(api_key), app_id: Some(app_id), port: None }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>(config: Option<AuthiumConfig>) -> TauriPlugin<R, Option<AuthiumConfig>> {
    Builder::<R, Option<AuthiumConfig>>::new("authium")
        .invoke_handler(tauri::generate_handler![
            commands::sign_in,
            commands::logout,
            commands::get_user,
            commands::is_logged_in,
            commands::refresh
        ])
        .setup(move |app, api| {
            *DATA_DIR.lock().unwrap() = app.path().app_data_dir().unwrap_or(std::path::PathBuf::new()).to_string_lossy().to_string();

            let c = config.clone().unwrap_or_else(|| api.config().clone().expect("no configuration found for authium"));
            app.manage(c.clone());

            #[cfg(mobile)]
            let authium = mobile::init(app, api)?;

            #[cfg(desktop)]
            let authium = desktop::init(app, api)?;
            app.manage(authium);

            thread::spawn(move || {
                setup_storage();
            });

            let handle = app.app_handle();
            let boxed_handle = Box::new(handle.clone());
            thread::spawn(move || {
                server::start_server(&*boxed_handle, &c)
                    .expect("Failed to start Authium server");
            });

            Ok(())
        })
        .build()
}
