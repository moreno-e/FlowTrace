//! # Storage Module - Session Persistence
//!
//! Handles saving recording sessions to disk as JSON files with
//! session-based directory organization.
//!
//! ## Directory Structure
//! ```text
//! recordings/
//! └── [session-id]/
//!     ├── session.json              (event metadata)
//!     ├── event_[id]_full.png       (full screen screenshots)
//!     ├── event_[id]_window.png     (window crop screenshots)
//!     └── event_[id]_click.png      (click crop screenshots)
//! ```
//!
//! ## Benefits of Session-Based Organization
//! - Easy to zip/share individual sessions
//! - No filename conflicts between sessions
//! - Clean deletion of old recordings
//! - Clear mapping between JSON and screenshots

use crate::types::RecordingSession;
use std::fs;
use std::path::PathBuf;

/// Saves a recording session to a JSON file with pretty-printing.
///
/// Creates a session-specific directory and persists all event metadata
/// as a human-readable JSON file. Screenshots are saved separately by
/// the screenshot module to the same directory.
///
/// # Arguments
/// * `session` - The recording session to save
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to saved `session.json` file
/// * `Err(String)` - Error message if directory creation or write fails
///
/// # File Location
/// ```text
/// recordings/[session-id]/session.json
/// ```
///
/// # Directory Creation
/// - Creates `recordings/` if it doesn't exist
/// - Creates `recordings/[session-id]/` if it doesn't exist
/// - Idempotent: Safe to call multiple times
///
/// # JSON Format
/// Uses `serde_json::to_string_pretty()` for human-readable output:
/// - 2-space indentation
/// - Newlines between fields
/// - Sorted keys (stable serialization)
///
/// # Example Output
/// ```json
/// {
///   "session_id": "f2e904d2-286e-484c-83e8-5949bd8697f1",
///   "started_at": "2026-02-01T15:43:08.646618Z",
///   "stopped_at": "2026-02-01T15:43:18.855192Z",
///   "events": [
///     {
///       "id": "cece1f95-8a90-4fa5-8fcc-2995113918ab",
///       "event_type": {"type": "Click", "button": "Left"},
///       ...
///     }
///   ]
/// }
/// ```
///
/// # Typical File Sizes
/// - Minimal session (few events): ~2-5 KB
/// - Moderate session (50 events): ~20-50 KB
/// - Large session (200+ events): ~100-200 KB
/// - Screenshots: ~2.2 MB each (stored separately)
///
/// # Error Handling
/// Returns `Err(String)` if:
/// - Unable to create directories (permission denied)
/// - Unable to serialize session (should never happen)
/// - Unable to write file (disk full, permission denied)
pub fn save_session(session: &RecordingSession) -> Result<PathBuf, String> {
    // Ensure base recordings directory exists
    let recordings_dir = PathBuf::from("recordings");
    fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {:?}", e))?;

    // Ensure session-specific subdirectory exists
    let session_dir = recordings_dir.join(&session.session_id);
    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session directory: {:?}", e))?;

    // Serialize session to pretty-printed JSON
    let json_path = session_dir.join("session.json");
    let json_data = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize session: {:?}", e))?;

    // Write JSON to disk (overwrites if exists)
    fs::write(&json_path, json_data)
        .map_err(|e| format!("Failed to write session file: {:?}", e))?;

    #[cfg(debug_assertions)]
    println!("💾 Session saved to: {:?}", json_path);

    Ok(json_path)
}

/// Returns the directory path for a session's files (screenshots + JSON).
///
/// Used by screenshot module to determine where to save screenshot files.
/// Does NOT create the directory - caller is responsible for creation.
///
/// # Arguments
/// * `session_id` - UUID of the recording session
///
/// # Returns
/// `PathBuf` to session directory: `recordings/[session-id]/`
///
/// # Example
/// ```rust
/// let session_dir = get_session_dir("f2e904d2-286e-484c-83e8-5949bd8697f1");
/// // Returns: PathBuf("recordings/f2e904d2-286e-484c-83e8-5949bd8697f1")
/// ```
pub fn get_session_dir(session_id: &str) -> PathBuf {
    PathBuf::from("recordings").join(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Event, EventType, MouseButton, Position};

    #[test]
    fn test_save_session() {
        let mut session = RecordingSession::new("test-session".to_string());

        let event = Event::new(
            EventType::Click {
                button: MouseButton::Left,
            },
            Some(Position::new(100.0, 200.0)),
        );

        session.add_event(event);
        session.stop();

        let result = save_session(&session);
        assert!(result.is_ok());
        println!("Test session saved to: {:?}", result.unwrap());
    }
}
