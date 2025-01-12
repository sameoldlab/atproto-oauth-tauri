use tauri::{command, AppHandle, Emitter, Window};
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[command]
fn start_server(app: AppHandle) -> Result<u16, String> {
    tauri_plugin_oauth::start( move |url| {
        println!("url: {}", url);
        let _ = app.emit("redirect_uri", url).unwrap();
    })
    .map_err(|err| err.to_string())
}

#[command]
fn stop_server(port: u16 ) -> Result<(), String> {
    println!("{}", port);
    tauri_plugin_oauth::cancel(port)
    .map_err(|err| err.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_server, stop_server])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
