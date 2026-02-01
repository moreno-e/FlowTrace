// Declare modules
mod event_monitor;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_event_listener() -> Result<String, String> {
    println!("🚀 Start event listener command called!");

    // Spawn listener in background thread (rdev::listen blocks)
    std::thread::spawn(|| {
        event_monitor::test_listener();
    });

    Ok("Event listener started in background thread".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_event_listener])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
