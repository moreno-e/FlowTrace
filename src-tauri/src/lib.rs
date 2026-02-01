//! # FlowTrace - Workflow Event Recording Library
//!
//! This module provides the core functionality for capturing and recording
//! user interactions (clicks and keyboard events) with synchronized screenshots.
//!
//! ## Architecture
//!
//! - **Global State Management**: Uses `Arc<Mutex<>>` for thread-safe shared state
//! - **Background Event Listener**: `rdev::listen()` runs in a separate thread
//! - **Screenshot Integration**: Captures 3 screenshots per click (full, window, click crop)
//! - **Automatic Wait Detection**: Inserts pause events for gaps > 2 seconds
//!
//! ## Threading Model
//!
//! ```text
//! Main Thread (Tauri)          Background Thread (rdev)
//! ├─ start_recording()         ├─ handle_event()
//! ├─ stop_recording()          ├─ check_and_insert_wait_event()
//! └─ Shared: CURRENT_SESSION   └─ screenshot::capture_all_for_event()
//! ```

// Declare modules
mod event_monitor;
mod screenshot;
mod storage;
mod types;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use types::{Event, EventType, MouseButton, Position, RecordingSession};

/// Global state for the current recording session.
///
/// Thread-safe access via `Arc<Mutex<>>`. Shared between:
/// - Main thread: start/stop commands
/// - Background thread: event handler adding events
///
/// **Note**: Only one recording session can be active at a time.
static CURRENT_SESSION: Lazy<Arc<Mutex<Option<RecordingSession>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Tracks the last known mouse position from `MouseMove` events.
///
/// **Why needed**: The `rdev` library doesn't provide position data in `ButtonPress` events.
/// We must track position from continuous `MouseMove` events and use it when a click occurs.
///
/// **Accuracy**: Position may be 1-5 pixels off if the user clicks while moving the mouse rapidly.
static LAST_MOUSE_POSITION: Lazy<Arc<Mutex<(f64, f64)>>> =
    Lazy::new(|| Arc::new(Mutex::new((0.0, 0.0))));

/// Tracks the timestamp of the last processed event for automatic wait detection.
///
/// Used by `check_and_insert_wait_event()` to detect pauses > 2 seconds.
/// When a significant gap is detected, a synthetic `Wait` event is inserted.
static LAST_EVENT_TIME: Lazy<Arc<Mutex<Option<chrono::DateTime<chrono::Utc>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Demo greeting command (from Tauri template).
///
/// # Arguments
/// * `name` - The name to greet
///
/// # Returns
/// A greeting string
///
/// **Note**: Not used in production, kept for template reference.
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Starts a standalone event listener for spike testing.
///
/// **Purpose**: Testing/debugging only - starts raw `rdev` event listener without recording.
///
/// # Returns
/// * `Ok(String)` - Success message
/// * `Err(String)` - Error message if failed to start
///
/// # Note
/// This spawns a separate thread that runs indefinitely. In production, use
/// `start_recording()` instead which integrates event listening with session management.
///
/// **UI Location**: Available under "Test Functions" collapsible section.
#[tauri::command]
fn start_event_listener() -> Result<String, String> {
    #[cfg(debug_assertions)]
    println!("🚀 Start event listener command called!");

    // Spawn listener in background thread (rdev::listen blocks forever)
    std::thread::spawn(|| {
        event_monitor::test_listener();
    });

    Ok("Event listener started in background thread".to_string())
}

/// Captures a single full-screen screenshot for spike testing.
///
/// **Purpose**: Testing/debugging only - captures screenshot without event context.
///
/// # Returns
/// * `Ok(String)` - Success message with file path
/// * `Err(String)` - Error message if capture failed
///
/// # Screenshots
/// Saved to: `recordings/screenshot_YYYYMMDD_HHMMSS.png`
///
/// # Permissions Required
/// - macOS: Screen Recording permission for launching application (Cursor/iTerm2)
///
/// **UI Location**: Available under "Test Functions" collapsible section.
#[tauri::command]
fn capture_screenshot() -> Result<String, String> {
    #[cfg(debug_assertions)]
    println!("📸 Capture screenshot command called!");

    match screenshot::capture_full_screen() {
        Ok(path) => Ok(format!("Screenshot saved to: {}", path.display())),
        Err(e) => Err(format!("Failed to capture screenshot: {}", e)),
    }
}

/// Starts a new recording session with integrated event monitoring.
///
/// Creates a new session with a unique UUID and spawns a background thread
/// that captures all mouse clicks and keyboard events until `stop_recording()` is called.
///
/// # Returns
/// * `Ok(String)` - Success message with session ID
/// * `Err(String)` - Error if recording already in progress
///
/// # What Gets Captured
/// - **Mouse clicks**: Left, right, middle button presses with positions
/// - **Keyboard events**: All key presses except modifier-only keys
/// - **Wait events**: Automatic detection of pauses > 2 seconds
/// - **Screenshots**: 3 per click (full screen, window crop, click crop)
///
/// # Session Management
/// - Session ID: UUIDv4 (e.g., `f2e904d2-286e-484c-83e8-5949bd8697f1`)
/// - Storage: `recordings/[session-id]/`
/// - Only one session active at a time (enforced)
///
/// # Threading
/// The event listener runs in a separate thread because `rdev::listen()` blocks.
/// Events are added to the shared `CURRENT_SESSION` via `Arc<Mutex<>>`.
///
/// # Known Limitation
/// The listener cannot be gracefully stopped. Workaround: restart the application
/// to start a new recording session.
///
/// # Permissions Required
/// - macOS: Accessibility + Screen Recording for launching application
#[tauri::command]
fn start_recording() -> Result<String, String> {
    #[cfg(debug_assertions)]
    println!("🎬 Start recording command called!");

    let mut session_lock = CURRENT_SESSION.lock().unwrap();

    // Enforce single active session
    if session_lock.is_some() {
        return Err("Recording already in progress".to_string());
    }

    // Create new session with unique identifier
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = RecordingSession::new(session_id.clone());

    #[cfg(debug_assertions)]
    println!("📝 Created recording session: {}", session_id);

    *session_lock = Some(session);
    drop(session_lock); // CRITICAL: Release lock before spawning thread to prevent deadlock

    // Start event listener in background thread (rdev::listen blocks forever)
    std::thread::spawn(move || {
        #[cfg(debug_assertions)]
        println!("👂 Starting integrated event listener...");

        if let Err(e) = rdev::listen(move |event| {
            handle_event(event);
        }) {
            eprintln!("❌ Event listener error: {:?}", e);
        }
    });

    Ok(format!("Recording started with session ID: {}", session_id))
}

/// Stops the current recording session and saves events to disk.
///
/// Finalizes the recording session, sets the `stopped_at` timestamp, and
/// persists all captured events to a JSON file.
///
/// # Returns
/// * `Ok(String)` - Success message with event count and file path
/// * `Err(String)` - Error if no recording in progress or save failed
///
/// # Saves To
/// - File: `recordings/[session-id]/session.json`
/// - Format: Pretty-printed JSON
/// - Screenshots: `recordings/[session-id]/event_[id]_*.png`
///
/// # Session Structure
/// ```json
/// {
///   "session_id": "uuid",
///   "started_at": "2026-02-01T15:43:08.646618Z",
///   "stopped_at": "2026-02-01T15:43:18.855192Z",
///   "events": [...]
/// }
/// ```
///
/// # Note
/// The background event listener continues running (known limitation).
/// To start a new recording, restart the application.
#[tauri::command]
fn stop_recording() -> Result<String, String> {
    #[cfg(debug_assertions)]
    println!("⏹️  Stop recording command called!");

    let mut session_lock = CURRENT_SESSION.lock().unwrap();

    match session_lock.take() {
        Some(mut session) => {
            session.stop();
            let event_count = session.events.len();

            // Save to disk as JSON
            match storage::save_session(&session) {
                Ok(path) => {
                    #[cfg(debug_assertions)]
                    println!(
                        "✅ Recording stopped. {} events saved to: {:?}",
                        event_count, path
                    );
                    Ok(format!(
                        "Recording stopped. {} events captured. Saved to: {}",
                        event_count,
                        path.display()
                    ))
                }
                Err(e) => Err(format!("Failed to save recording: {}", e)),
            }
        }
        None => Err("No recording in progress".to_string()),
    }
}

/// Detects significant pauses between user actions and inserts synthetic Wait events.
///
/// Called before processing each new event to check if enough time has elapsed
/// since the last event. If the gap exceeds the threshold, a `Wait` event is
/// automatically inserted into the recording.
///
/// # Algorithm
/// 1. Calculate time difference between now and last event
/// 2. If gap >= 2.0 seconds → create synthetic Wait event
/// 3. Add Wait event to current session
/// 4. Update last event timestamp to now
///
/// # Why This Matters
/// Wait events provide context about user behavior:
/// - Reading content
/// - Thinking/deciding
/// - Waiting for page load
/// - Natural workflow pauses
///
/// # Example Timeline
/// ```text
/// 10:00:00 - Click at (100, 200)
/// 10:00:03 - Check: 3 seconds elapsed → Insert Wait(3.0s)
/// 10:00:03 - Click at (150, 250)
/// ```
///
/// # Thread Safety
/// Uses `Arc<Mutex<>>` for thread-safe access to:
/// - `LAST_EVENT_TIME` - Tracks timestamp
/// - `CURRENT_SESSION` - Adds Wait event to recording
///
/// # Tuning
/// - Threshold: 2.0 seconds (configurable via `WAIT_THRESHOLD_SECONDS`)
/// - Too low: Noisy with many short waits
/// - Too high: Miss meaningful pauses
fn check_and_insert_wait_event() {
    /// Minimum pause duration (seconds) to trigger a Wait event.
    /// Tuned to capture meaningful pauses without noise.
    const WAIT_THRESHOLD_SECONDS: f64 = 2.0;

    let now = chrono::Utc::now();

    // Acquire lock and check last event time
    if let Ok(mut last_time_lock) = LAST_EVENT_TIME.lock() {
        if let Some(last_time) = *last_time_lock {
            // Calculate time gap in seconds (convert from milliseconds)
            let duration = (now - last_time).num_milliseconds() as f64 / 1000.0;

            // Only insert Wait event if gap is significant
            if duration >= WAIT_THRESHOLD_SECONDS {
                #[cfg(debug_assertions)]
                println!("⏸️  Wait detected: {:.1}s pause", duration);

                // Create synthetic Wait event with calculated duration
                let wait_event = Event::new(
                    EventType::Wait {
                        duration_seconds: duration,
                    },
                    None, // No position for Wait events
                );

                // Add to current recording session
                if let Ok(mut session_lock) = CURRENT_SESSION.lock() {
                    if let Some(session) = session_lock.as_mut() {
                        session.add_event(wait_event);
                        #[cfg(debug_assertions)]
                        println!(
                            "✅ Wait event added to session (total: {})",
                            session.events.len()
                        );
                    }
                }
            }
        }

        // Always update timestamp to mark this call as "last event"
        // (Even if no Wait event was inserted, prevents duplicate Waits)
        *last_time_lock = Some(now);
    }
}

/// Main event handler for all captured system events (clicks, keyboard, mouse moves).
///
/// This function is called by the `rdev` event listener for **every** system event.
/// It filters, processes, and records relevant events to the current session.
///
/// # Event Flow
/// ```text
/// rdev::listen() → handle_event() → [Filter] → [Capture] → Add to Session
/// ```
///
/// # What Gets Processed
/// - **MouseMove**: Track position (don't record event itself - too noisy)
/// - **ButtonPress**: Record clicks with screenshots (left, right, middle)
/// - **KeyPress**: Record keyboard input (filter out modifier-only keys)
/// - **Other events**: Ignored (button release, wheel, etc.)
///
/// # Wait Detection
/// Before processing each event, checks for pauses > 2 seconds and inserts
/// synthetic Wait events automatically.
///
/// # Thread Context
/// Runs in background thread spawned by `start_recording()`.
/// Uses `Arc<Mutex<>>` for thread-safe access to global state.
///
/// # Performance Considerations
/// - **MouseMove**: Not recorded (would generate 100+ events/second)
/// - **Keyboard screenshots**: Skipped (reduces storage by ~2.2MB per keystroke)
/// - **Lock ordering**: Drop locks before expensive operations (screenshot capture)
///
/// # Arguments
/// * `event` - Raw event from `rdev::listen()` containing event type and metadata
fn handle_event(event: rdev::Event) {
    // STEP 1: Check for significant time gaps and insert Wait events
    check_and_insert_wait_event();

    match event.event_type {
        // STEP 2: Track mouse position (required for clicks, but don't record moves)
        rdev::EventType::MouseMove { x, y } => {
            if let Ok(mut pos) = LAST_MOUSE_POSITION.lock() {
                *pos = (x, y);
            }
            // Early return: MouseMove events are too noisy to record
            // (Would generate 100+ events per second of mouse movement)
            return;
        }

        // STEP 3: Process and record mouse button clicks
        rdev::EventType::ButtonPress(button) => {
            // Filter: Only capture left, right, middle buttons
            let mouse_button = match button {
                rdev::Button::Left => MouseButton::Left,
                rdev::Button::Right => MouseButton::Right,
                rdev::Button::Middle => MouseButton::Middle,
                _ => return, // Ignore trackpad gestures, forward/back buttons, etc.
            };

            // Retrieve last known mouse position from global tracker
            // (rdev doesn't provide position in ButtonPress events)
            let (x, y) = {
                let pos = LAST_MOUSE_POSITION.lock().unwrap();
                *pos
            };

            let position = Position::new(x, y);

            #[cfg(debug_assertions)]
            println!("🖱️  Click detected at ({}, {})", position.x, position.y);

            // CRITICAL: Extract coordinates BEFORE moving position into Event
            // (Position is not Copy, and we need these values for screenshot cropping)
            let click_x = position.x;
            let click_y = position.y;

            // Create event with position (will be moved/consumed)
            let mut new_event = Event::new(
                EventType::Click {
                    button: mouse_button,
                },
                Some(position),
            );

            // Screenshot capture and session update
            if let Ok(session_lock) = CURRENT_SESSION.lock() {
                if let Some(session) = session_lock.as_ref() {
                    let session_id = session.session_id.clone();
                    let event_id = new_event.id.clone();

                    // CRITICAL: Drop lock BEFORE screenshot capture
                    // Screenshot can take 100-500ms, holding the lock would block other events
                    drop(session_lock);

                    // Capture 3 screenshots: full screen, window crop, click crop
                    // Note: Window and click crops may be offset on Retina displays (known issue)
                    match screenshot::capture_all_for_event(
                        &session_id,
                        &event_id,
                        click_x,
                        click_y,
                    ) {
                        Ok((full, window, click)) => {
                            new_event = new_event.with_screenshots(Some(full), window, click);
                            #[cfg(debug_assertions)]
                            println!(
                                "📸 Screenshots captured for event {} (full + window + click)",
                                event_id
                            );
                        }
                        Err(e) => {
                            // Non-fatal: Continue recording even if screenshot fails
                            eprintln!("⚠️  Failed to capture screenshots: {}", e);
                        }
                    }

                    // Re-acquire lock and add event to session
                    if let Ok(mut session_lock) = CURRENT_SESSION.lock() {
                        if let Some(session) = session_lock.as_mut() {
                            session.add_event(new_event);
                            #[cfg(debug_assertions)]
                            println!(
                                "✅ Event added to session (total: {})",
                                session.events.len()
                            );
                        }
                    }
                }
            }
        }

        // STEP 4: Process and record keyboard events
        rdev::EventType::KeyPress(key) => {
            // Early exit: Only capture if recording is active
            if let Ok(session_lock) = CURRENT_SESSION.lock() {
                if session_lock.is_none() {
                    return; // No active recording session
                }
            } else {
                return; // Failed to acquire lock
            }

            // Convert key enum to string representation (e.g., "KeyA", "Return", "Space")
            let key_str = format!("{:?}", key);

            // Filter: Skip modifier-only keys to reduce noise
            // Rationale: Modifier keys alone (Shift, Ctrl, Cmd) don't represent user intent
            // We only care about the final key combination (e.g., "KeyS" not "ShiftLeft + KeyS")
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

            #[cfg(debug_assertions)]
            println!("⌨️  Key pressed: {}", key_str);

            // Create event without position (keyboard events aren't location-based)
            let new_event = Event::new(EventType::KeyPress { key: key_str }, None);

            // Add to session WITHOUT screenshot capture
            // Design decision: Skip screenshots for keyboard events to:
            // - Reduce storage (each screenshot ~2.2MB)
            // - Improve performance (no capture overhead during typing)
            // - Rely on click screenshots for visual context
            if let Ok(mut session_lock) = CURRENT_SESSION.lock() {
                if let Some(session) = session_lock.as_mut() {
                    session.add_event(new_event);
                    #[cfg(debug_assertions)]
                    println!(
                        "✅ Key event added to session (total: {})",
                        session.events.len()
                    );
                }
            }
        }

        // STEP 5: Ignore all other event types
        _ => {
            // Explicitly ignored:
            // - ButtonRelease: We only care about press, not release
            // - Wheel: Mouse wheel events not relevant for workflow tracking
            // - Other: Any future event types from rdev
        }
    }
}

/// Initializes and runs the Tauri application.
///
/// Sets up the Tauri runtime with all registered commands and plugins.
/// This is the main entry point called from `main.rs`.
///
/// # Registered Commands
/// - `greet` - Demo command (template reference)
/// - `start_event_listener` - Spike testing command
/// - `capture_screenshot` - Spike testing command
/// - `start_recording` - **Main**: Start workflow recording
/// - `stop_recording` - **Main**: Stop and save recording
///
/// # Plugins
/// - `tauri_plugin_opener` - Handles file/URL opening
///
/// # Platform Support
/// - Primary: macOS (tested on Sonoma 25.2.0)
/// - Mobile: Conditional compilation via `#[cfg_attr]`
///
/// # Panics
/// Panics if Tauri application fails to initialize or run.
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
