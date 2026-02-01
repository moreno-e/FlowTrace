// Declare modules
mod event_monitor;
mod screenshot;
mod types;
mod storage;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use types::{RecordingSession, Event, EventType, MouseButton, Position};

// Global state for current recording session
static CURRENT_SESSION: Lazy<Arc<Mutex<Option<RecordingSession>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

// Track last mouse position (updated by MouseMove events)
static LAST_MOUSE_POSITION: Lazy<Arc<Mutex<(f64, f64)>>> =
    Lazy::new(|| Arc::new(Mutex::new((0.0, 0.0))));

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

#[tauri::command]
fn capture_screenshot() -> Result<String, String> {
    println!("📸 Capture screenshot command called!");

    match screenshot::capture_full_screen() {
        Ok(path) => Ok(format!("Screenshot saved to: {}", path.display())),
        Err(e) => Err(format!("Failed to capture screenshot: {}", e)),
    }
}

#[tauri::command]
fn start_recording() -> Result<String, String> {
    println!("🎬 Start recording command called!");

    let mut session_lock = CURRENT_SESSION.lock().unwrap();

    if session_lock.is_some() {
        return Err("Recording already in progress".to_string());
    }

    // Create new session
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = RecordingSession::new(session_id.clone());

    println!("📝 Created recording session: {}", session_id);

    *session_lock = Some(session);
    drop(session_lock); // Release lock before spawning thread

    // Start event listener in background
    std::thread::spawn(move || {
        println!("👂 Starting integrated event listener...");

        if let Err(e) = rdev::listen(move |event| {
            handle_event(event);
        }) {
            eprintln!("❌ Event listener error: {:?}", e);
        }
    });

    Ok(format!("Recording started with session ID: {}", session_id))
}

#[tauri::command]
fn stop_recording() -> Result<String, String> {
    println!("⏹️  Stop recording command called!");

    let mut session_lock = CURRENT_SESSION.lock().unwrap();

    match session_lock.take() {
        Some(mut session) => {
            session.stop();
            let event_count = session.events.len();

            // Save to disk
            match storage::save_session(&session) {
                Ok(path) => {
                    println!("✅ Recording stopped. {} events saved to: {:?}", event_count, path);
                    Ok(format!("Recording stopped. {} events captured. Saved to: {}", event_count, path.display()))
                }
                Err(e) => Err(format!("Failed to save recording: {}", e)),
            }
        }
        None => Err("No recording in progress".to_string()),
    }
}

/// Handle incoming events from rdev listener
fn handle_event(event: rdev::Event) {
    match event.event_type {
        // Track mouse position from MouseMove events
        rdev::EventType::MouseMove { x, y } => {
            if let Ok(mut pos) = LAST_MOUSE_POSITION.lock() {
                *pos = (x, y);
            }
            // Don't process MouseMove events further (too noisy for MVP)
            return;
        }

        // Process button press events
        rdev::EventType::ButtonPress(button) => {
            let mouse_button = match button {
                rdev::Button::Left => MouseButton::Left,
                rdev::Button::Right => MouseButton::Right,
                rdev::Button::Middle => MouseButton::Middle,
                _ => return, // Ignore other buttons
            };

            // Get last known mouse position
            let (x, y) = {
                let pos = LAST_MOUSE_POSITION.lock().unwrap();
                *pos
            };

            let position = Position::new(x, y);

            println!("🖱️  Click detected at ({}, {})", position.x, position.y);

            // Create event
            let mut new_event = Event::new(
                EventType::Click {
                    button: mouse_button,
                },
                Some(position),
            );

            // Try to get session and capture screenshot
            if let Ok(session_lock) = CURRENT_SESSION.lock() {
                if let Some(session) = session_lock.as_ref() {
                    let session_id = session.session_id.clone();
                    let event_id = new_event.id.clone();

                    drop(session_lock); // Release lock before screenshot

                    // Capture screenshot
                    match screenshot::capture_for_event(&session_id, &event_id) {
                        Ok(screenshot_path) => {
                            new_event = new_event.with_screenshot(screenshot_path);
                            println!("📸 Screenshot captured for event {}", event_id);
                        }
                        Err(e) => {
                            eprintln!("⚠️  Failed to capture screenshot: {}", e);
                        }
                    }

                    // Add event to session
                    if let Ok(mut session_lock) = CURRENT_SESSION.lock() {
                        if let Some(session) = session_lock.as_mut() {
                            session.add_event(new_event);
                            println!("✅ Event added to session (total: {})", session.events.len());
                        }
                    }
                }
            }
        }

        // Process keyboard events
        rdev::EventType::KeyPress(key) => {
            // Only capture if we have an active session
            if let Ok(session_lock) = CURRENT_SESSION.lock() {
                if session_lock.is_none() {
                    return; // No active recording session
                }
            } else {
                return;
            }

            // Get key name as string
            let key_str = format!("{:?}", key);

            // Filter out modifier-only keys to reduce noise
            if matches!(
                key,
                rdev::Key::ShiftLeft
                    | rdev::Key::ShiftRight
                    | rdev::Key::ControlLeft
                    | rdev::Key::ControlRight
                    | rdev::Key::Alt
                    | rdev::Key::AltGr
                    | rdev::Key::MetaLeft
                    | rdev::Key::MetaRight
            ) {
                return; // Skip modifier-only presses
            }

            println!("⌨️  Key pressed: {}", key_str);

            // Create event (no position for keyboard events)
            let new_event = Event::new(EventType::KeyPress { key: key_str }, None);

            // Add event to session (skip screenshot for keyboard to reduce noise)
            if let Ok(mut session_lock) = CURRENT_SESSION.lock() {
                if let Some(session) = session_lock.as_mut() {
                    session.add_event(new_event);
                    println!("✅ Key event added to session (total: {})", session.events.len());
                }
            }
        }

        _ => {
            // Ignore other event types (mouse moves, button release, etc.)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_event_listener,
            capture_screenshot,
            start_recording,
            stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
