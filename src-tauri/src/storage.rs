use crate::types::RecordingSession;
use std::fs;
use std::path::PathBuf;

/// Save a recording session to JSON file
pub fn save_session(session: &RecordingSession) -> Result<PathBuf, String> {
    // Create recordings directory if it doesn't exist
    let recordings_dir = PathBuf::from("recordings");
    fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {:?}", e))?;

    // Create session subdirectory
    let session_dir = recordings_dir.join(&session.session_id);
    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session directory: {:?}", e))?;

    // Save session metadata to JSON
    let json_path = session_dir.join("session.json");
    let json_data = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize session: {:?}", e))?;

    fs::write(&json_path, json_data)
        .map_err(|e| format!("Failed to write session file: {:?}", e))?;

    println!("💾 Session saved to: {:?}", json_path);

    Ok(json_path)
}

/// Get the session directory path for saving screenshots
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
