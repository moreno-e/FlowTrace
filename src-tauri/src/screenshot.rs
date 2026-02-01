use screenshots::Screen;
use std::fs;
use std::path::PathBuf;
use crate::storage;

/// Capture full screen and save to recordings directory (for testing)
pub fn capture_full_screen() -> Result<PathBuf, String> {
    println!("📸 Attempting to capture screenshot...");

    // Get the primary screen
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {:?}", e))?;

    let primary_screen = screens
        .first()
        .ok_or_else(|| "No screens found".to_string())?;

    println!("📸 Found screen: {}x{}", primary_screen.display_info.width, primary_screen.display_info.height);

    // Capture the screen
    let image = primary_screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {:?}", e))?;

    // Create recordings directory if it doesn't exist
    let recordings_dir = PathBuf::from("recordings");

    fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {:?}", e))?;

    // Generate filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_{}.png", timestamp);
    let filepath = recordings_dir.join(&filename);

    // Save the image
    image
        .save(&filepath)
        .map_err(|e| format!("Failed to save screenshot: {:?}", e))?;

    println!("✅ Screenshot saved to: {:?}", filepath);

    Ok(filepath)
}

/// Capture screenshot for a specific event within a recording session
pub fn capture_for_event(session_id: &str, event_id: &str) -> Result<String, String> {
    // Get the primary screen
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {:?}", e))?;
    let primary_screen = screens
        .first()
        .ok_or_else(|| "No screens found".to_string())?;

    // Capture the screen
    let image = primary_screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {:?}", e))?;

    // Get session directory
    let session_dir = storage::get_session_dir(session_id);
    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session directory: {:?}", e))?;

    // Save with event ID
    let filename = format!("event_{}.png", event_id);
    let filepath = session_dir.join(&filename);

    image
        .save(&filepath)
        .map_err(|e| format!("Failed to save screenshot: {:?}", e))?;

    // Return relative path for JSON storage
    let relative_path = format!("recordings/{}/{}", session_id, filename);

    Ok(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_screen() {
        let result = capture_full_screen();
        assert!(result.is_ok());
        println!("Test screenshot saved to: {:?}", result.unwrap());
    }
}
