use std::{fs, path::Path, time::{Duration, SystemTime}};

use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit as _, Nonce};
use tauri::{async_runtime::spawn, AppHandle, Runtime, Emitter};

use crate::DATA_DIR;

const KEY_SLICE: &[u8; 32] = b"32-byte-key-authium4141234567890";
const NONCE_SLICE: &[u8; 12] = b"nonce-ezauth";

fn create_dir_if_not_exists(path: &str) {
    if !Path::new(path).exists() {
        fs::create_dir_all(path).expect("Failed to create directory");
    }
}

fn create_file_if_not_exists(file_path: &str) {
    if !Path::new(file_path).exists() {
        fs::File::create(file_path).expect("Failed to create file");
    }
}

pub fn setup_storage<R: Runtime + 'static>(handle: AppHandle<R>) {
    let data_dir = DATA_DIR.lock().unwrap().clone();

    create_dir_if_not_exists(&data_dir);

    let data = load_user_data();
    if let Ok((access_token, refresh_token, expiry)) = data {
        if !access_token.is_empty() && !refresh_token.is_empty() {
            spawn(async move {
                if let Ok(u) = crate::user::login(access_token, refresh_token, expiry).await {
                    handle.emit("authium:login-success", u).expect("Failed to emit login event");
                }
            });
        }
    } else {
        eprintln!("Failed to load user data: {}", data.err().unwrap());
    }
}

pub fn save_user_data(access_token: &str, refresh_token: &str, expiry: SystemTime) -> std::io::Result<()> {
    let data_dir = DATA_DIR.lock().unwrap().clone();
    let file_path = format!("{}/userdata.dat", data_dir);
    create_file_if_not_exists(&file_path);

    let key = Key::<Aes256Gcm>::from_slice(KEY_SLICE);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(NONCE_SLICE);

    let expiry_str = expiry.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let data = format!("{}\n{}\n{}", access_token, refresh_token, expiry_str);

    let encrypted_data = cipher.encrypt(nonce, data.as_bytes())
        .expect("Failed to encrypt user data");

    fs::write(file_path, encrypted_data).expect("Failed to write user data to file");
    Ok(())
}

pub fn load_user_data() -> std::io::Result<(String, String, SystemTime)> {
    let data_dir = DATA_DIR.lock().unwrap().clone();
    let file_path = format!("{}/userdata.dat", data_dir);
    if !Path::new(&file_path).exists() {
        return Ok((String::new(), String::new(), SystemTime::now()));
    }

    let encrypted_data = fs::read(file_path).expect("Failed to read user data from file");

    let key = Key::<Aes256Gcm>::from_slice(KEY_SLICE);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(NONCE_SLICE);

    let decrypted_data = cipher.decrypt(nonce, &*encrypted_data)
        .expect("Failed to decrypt user data");

    let data = String::from_utf8(decrypted_data).expect("Failed to convert user data to string");
    let mut lines = data.lines();

    let access_token = lines.next().unwrap_or_default().to_string();
    let refresh_token = lines.next().unwrap_or_default().to_string();
    let expires_in = lines.next().unwrap_or_default().parse().unwrap_or(0);

    let expiry = SystemTime::UNIX_EPOCH + Duration::from_secs(expires_in);

    Ok((access_token, refresh_token, expiry))
}

pub fn clear_user_data() {
    let data_dir = DATA_DIR.lock().unwrap().clone();
    let file_path = format!("{}/userdata.dat", data_dir);

    if Path::new(&file_path).exists() {
        fs::remove_file(file_path).expect("Failed to clear user data file");
    }
}