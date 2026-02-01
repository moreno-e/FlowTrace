use screenshots::Screen;
use std::fs;
use std::path::PathBuf;
use crate::storage;
use image::DynamicImage;
use active_win_pos_rs::get_active_window;

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

/// Capture all 3 screenshots for an event: full screen, window crop, and click crop
pub fn capture_all_for_event(
    session_id: &str,
    event_id: &str,
    click_x: i32,
    click_y: i32,
) -> Result<(String, Option<String>, Option<String>), String> {
    // Get the primary screen
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {:?}", e))?;
    let primary_screen = screens
        .first()
        .ok_or_else(|| "No screens found".to_string())?;

    // Capture full screen
    let full_image_raw = primary_screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {:?}", e))?;

    // Convert to DynamicImage for manipulation
    let width = full_image_raw.width();
    let height = full_image_raw.height();
    let dynamic_image = image::RgbaImage::from_raw(
        width,
        height,
        full_image_raw.as_raw().to_vec(),
    )
    .ok_or_else(|| "Failed to convert screenshot to image format".to_string())?;
    let dynamic_image = DynamicImage::ImageRgba8(dynamic_image);

    // Get session directory
    let session_dir = storage::get_session_dir(session_id);
    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session directory: {:?}", e))?;

    // 1. Save full screen screenshot
    let full_filename = format!("event_{}_full.png", event_id);
    let full_filepath = session_dir.join(&full_filename);
    dynamic_image
        .save(&full_filepath)
        .map_err(|e| format!("Failed to save full screenshot: {:?}", e))?;
    let full_relative = format!("recordings/{}/{}", session_id, full_filename);

    // 2. Try to capture window crop
    let window_relative = match capture_window_crop(&dynamic_image, session_id, event_id, &session_dir) {
        Ok(path) => {
            println!("✅ Window crop saved");
            Some(path)
        }
        Err(e) => {
            println!("⚠️  Window crop failed: {}", e);
            None
        }
    };

    // 3. Capture click crop (300x300 around click position)
    let click_relative = match capture_click_crop(
        &dynamic_image,
        session_id,
        event_id,
        &session_dir,
        click_x,
        click_y,
        width as i32,
        height as i32,
    ) {
        Ok(path) => {
            println!("✅ Click crop saved");
            Some(path)
        }
        Err(e) => {
            println!("⚠️  Click crop failed: {}", e);
            None
        }
    };

    Ok((full_relative, window_relative, click_relative))
}

/// Capture window crop using active window detection
fn capture_window_crop(
    dynamic_image: &DynamicImage,
    session_id: &str,
    event_id: &str,
    session_dir: &PathBuf,
) -> Result<String, String> {
    // Get active window info
    let window = get_active_window()
        .map_err(|e| format!("Failed to get active window: {:?}", e))?;

    // Crop to window bounds (with bounds checking)
    let x = window.position.x.max(0.0) as u32;
    let y = window.position.y.max(0.0) as u32;
    let width = (window.position.width as u32).min(dynamic_image.width() - x);
    let height = (window.position.height as u32).min(dynamic_image.height() - y);

    let cropped = dynamic_image.crop_imm(x, y, width, height);

    // Save window crop
    let window_filename = format!("event_{}_window.png", event_id);
    let window_filepath = session_dir.join(&window_filename);
    cropped
        .save(&window_filepath)
        .map_err(|e| format!("Failed to save window crop: {:?}", e))?;

    Ok(format!("recordings/{}/{}", session_id, window_filename))
}

/// Capture click crop (300x300 around click position)
fn capture_click_crop(
    dynamic_image: &DynamicImage,
    session_id: &str,
    event_id: &str,
    session_dir: &PathBuf,
    click_x: i32,
    click_y: i32,
    screen_width: i32,
    screen_height: i32,
) -> Result<String, String> {
    const CROP_SIZE: i32 = 300;
    const HALF_SIZE: i32 = CROP_SIZE / 2;

    // Calculate crop bounds (centered on click, but within screen bounds)
    let x = (click_x - HALF_SIZE).max(0).min(screen_width - CROP_SIZE) as u32;
    let y = (click_y - HALF_SIZE).max(0).min(screen_height - CROP_SIZE) as u32;
    let width = CROP_SIZE.min(screen_width - x as i32) as u32;
    let height = CROP_SIZE.min(screen_height - y as i32) as u32;

    let cropped = dynamic_image.crop_imm(x, y, width, height);

    // Save click crop
    let click_filename = format!("event_{}_click.png", event_id);
    let click_filepath = session_dir.join(&click_filename);
    cropped
        .save(&click_filepath)
        .map_err(|e| format!("Failed to save click crop: {:?}", e))?;

    Ok(format!("recordings/{}/{}", session_id, click_filename))
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
