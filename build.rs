const COMMANDS: &[&str] = &["sign_in", "logout", "is_logged_in", "get_user"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
