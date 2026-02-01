//! # Screenshot Capture Module
//!
//! Handles all screenshot capture operations including:
//! - Full screen capture
//! - Active window detection and cropping
//! - Click-region cropping (300x300px around click)
//!
//! ## Known Limitation: Retina Display Coordinate Scaling
//!
//! On Retina/HiDPI displays, coordinate system mismatch causes offset crops:
//! - **Problem**: Event coordinates are logical (e.g., 713, 395)
//! - **Reality**: Screenshots are physical pixels (e.g., 2880x1800 on 2x display)
//! - **Result**: Window and click crops appear ~2x offset from intended position
//! - **Status**: Full screen works perfectly, crops documented as known limitation

use crate::storage;
use active_win_pos_rs::get_active_window;
use image::DynamicImage;
use screenshots::Screen;
use std::fs;
use std::path::PathBuf;

/// Captures a full-screen screenshot for spike testing.
///
/// **Purpose**: Testing/debugging only. In production, use `capture_all_for_event()`
/// which integrates with session management.
///
/// # Returns
/// * `Ok(PathBuf)` - Path to saved screenshot
/// * `Err(String)` - Error message if capture or save failed
///
/// # File Location
/// Saves to: `recordings/screenshot_YYYYMMDD_HHMMSS.png`
///
/// # How It Works
/// 1. Get primary screen from `screenshots` crate
/// 2. Capture entire display as PNG
/// 3. Save with timestamp-based filename
///
/// # Permissions Required
/// - macOS: Screen Recording permission for launching application
///
/// # Example
/// ```rust
/// let path = capture_full_screen()?;
/// println!("Screenshot saved to: {}", path.display());
/// ```
pub fn capture_full_screen() -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    println!("📸 Attempting to capture screenshot...");

    // Get the primary screen (index 0)
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {:?}", e))?;

    let primary_screen = screens
        .first()
        .ok_or_else(|| "No screens found".to_string())?;

    #[cfg(debug_assertions)]
    println!(
        "📸 Found screen: {}x{}",
        primary_screen.display_info.width, primary_screen.display_info.height
    );

    // Capture the screen as raw image data
    let image = primary_screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {:?}", e))?;

    // Ensure recordings directory exists
    let recordings_dir = PathBuf::from("recordings");

    fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {:?}", e))?;

    // Generate filename with timestamp (avoids collisions)
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("screenshot_{}.png", timestamp);
    let filepath = recordings_dir.join(&filename);

    // Save to disk as PNG
    image
        .save(&filepath)
        .map_err(|e| format!("Failed to save screenshot: {:?}", e))?;

    #[cfg(debug_assertions)]
    println!("✅ Screenshot saved to: {:?}", filepath);

    Ok(filepath)
}

/// Captures all 3 screenshot types for a click event: full screen, window crop, and click crop.
///
/// This is the primary screenshot function used during recording sessions.
/// Captures a single full-screen image and derives two cropped variants from it.
///
/// # Arguments
/// * `session_id` - UUID of the recording session (for file path)
/// * `event_id` - UUID of the event (for filename)
/// * `click_x` - X coordinate of click in logical pixels
/// * `click_y` - Y coordinate of click in logical pixels
///
/// # Returns
/// * `Ok((full_path, window_path, click_path))` - Relative paths to saved screenshots
///   - `full_path` - Always `Some(String)` (full screen capture)
///   - `window_path` - `Option<String>` (may fail if window detection fails)
///   - `click_path` - `Option<String>` (may fail if crop out of bounds)
/// * `Err(String)` - Error message if full screen capture fails
///
/// # Screenshot Types
///
/// 1. **Full Screen** (~2.2MB each)
///    - Captures entire primary display
///    - Always succeeds (unless screen capture permission missing)
///    - File: `event_[id]_full.png`
///
/// 2. **Window Crop** (variable size)
///    - Detects active window via `active-win-pos-rs`
///    - Crops full screen to window bounds
///    - May fail if window detection fails
///    - File: `event_[id]_window.png`
///    - **Known Issue**: Offset on Retina displays
///
/// 3. **Click Crop** (300x300px)
///    - Crops 300x300px region centered on click position
///    - Bounded to screen edges (won't crop beyond display)
///    - May fail if position calculation errors
///    - File: `event_[id]_click.png`
///    - **Known Issue**: Offset on Retina displays
///
/// # Known Limitation: Retina Display Coordinate Scaling
///
/// On 2x Retina displays:
/// - Input coordinates are logical (713, 395)
/// - Screenshot is physical pixels (2880x1800)
/// - Crops apply logical coords to physical image → 2x offset
/// - **Fix**: Detect scale factor and multiply coordinates (not implemented)
///
/// # File Locations
/// All saved to: `recordings/[session_id]/event_[event_id]_[type].png`
///
/// # Error Handling
/// - Full screen failure → returns `Err` (critical)
/// - Window crop failure → logs warning, returns `None` for window_path
/// - Click crop failure → logs warning, returns `None` for click_path
///
/// # Example
/// ```rust
/// let (full, window, click) = capture_all_for_event(
///     "f2e904d2-286e-484c-83e8-5949bd8697f1",
///     "cece1f95-8a90-4fa5-8fcc-2995113918ab",
///     709,
///     328
/// )?;
/// ```
pub fn capture_all_for_event(
    session_id: &str,
    event_id: &str,
    click_x: i32,
    click_y: i32,
) -> Result<(String, Option<String>, Option<String>), String> {
    // STEP 1: Get primary screen and capture full screenshot
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {:?}", e))?;
    let primary_screen = screens
        .first()
        .ok_or_else(|| "No screens found".to_string())?;

    // Capture full screen as raw image data (this is the expensive operation)
    let full_image_raw = primary_screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {:?}", e))?;

    // STEP 2: Convert screenshots::Image to image::DynamicImage for manipulation
    // Why: DynamicImage provides crop_imm() and other image processing methods
    let width = full_image_raw.width();
    let height = full_image_raw.height();
    let dynamic_image = image::RgbaImage::from_raw(
        width,
        height,
        full_image_raw.as_raw().to_vec(), // Copy raw pixel data
    )
    .ok_or_else(|| "Failed to convert screenshot to image format".to_string())?;
    let dynamic_image = DynamicImage::ImageRgba8(dynamic_image);

    // STEP 3: Ensure session directory exists
    let session_dir = storage::get_session_dir(session_id);
    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session directory: {:?}", e))?;

    // STEP 4: Save full screen screenshot (always succeeds at this point)
    let full_filename = format!("event_{}_full.png", event_id);
    let full_filepath = session_dir.join(&full_filename);
    dynamic_image
        .save(&full_filepath)
        .map_err(|e| format!("Failed to save full screenshot: {:?}", e))?;
    let full_relative = format!("recordings/{}/{}", session_id, full_filename);

    // STEP 5: Try to capture window crop (graceful failure)
    // Non-fatal: If window detection fails, continue without window crop
    let window_relative =
        match capture_window_crop(&dynamic_image, session_id, event_id, &session_dir) {
            Ok(path) => {
                #[cfg(debug_assertions)]
                println!("✅ Window crop saved");
                Some(path)
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("⚠️  Window crop failed: {}", e);
                None // Continue recording without window crop
            }
        };

    // STEP 6: Try to capture click crop (graceful failure)
    // Non-fatal: If crop calculation fails, continue without click crop
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
            #[cfg(debug_assertions)]
            println!("✅ Click crop saved");
            Some(path)
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            println!("⚠️  Click crop failed: {}", e);
            None // Continue recording without click crop
        }
    };

    Ok((full_relative, window_relative, click_relative))
}

/// Captures a cropped screenshot of the active window.
///
/// Uses `active-win-pos-rs` to detect the currently focused window and
/// crops the full screen image to that window's boundaries.
///
/// # Arguments
/// * `dynamic_image` - Full screen capture to crop from
/// * `session_id` - UUID for file path
/// * `event_id` - UUID for filename
/// * `session_dir` - Directory to save crop
///
/// # Returns
/// * `Ok(String)` - Relative path to saved window crop
/// * `Err(String)` - Error if window detection or crop fails
///
/// # How It Works
/// 1. Get active window position/size via `active-win-pos-rs`
/// 2. Extract bounds: `x`, `y`, `width`, `height`
/// 3. Apply bounds checking (prevent crop beyond image edges)
/// 4. Crop full screen image to window rectangle
/// 5. Save as separate PNG file
///
/// # Known Issue: Retina Display Coordinate Scaling
///
/// **Problem**: Window bounds are in logical coordinates but image is physical pixels.
///
/// **Example on 2x Retina Display**:
/// - Window reports: `x=200, y=100, width=800, height=600` (logical)
/// - Screenshot is: 2880x1800 (physical pixels, 2x scale)
/// - Crop applies: (200, 100, 800, 600) to physical image
/// - **Expected crop**: (400, 200, 1600, 1200) in physical pixels
/// - **Result**: Crops wrong area, offset by 2x
///
/// **Fix** (not implemented):
/// ```rust
/// let scale = detect_display_scale_factor(); // e.g., 2.0
/// let physical_x = (logical_x * scale) as u32;
/// let physical_y = (logical_y * scale) as u32;
/// // ... use physical coordinates for crop
/// ```
///
/// # Bounds Checking
/// Prevents crop from extending beyond image edges:
/// - `x.max(0.0)` - Clamp negative x to 0
/// - `width.min(image_width - x)` - Clamp width to remaining space
fn capture_window_crop(
    dynamic_image: &DynamicImage,
    session_id: &str,
    event_id: &str,
    session_dir: &PathBuf,
) -> Result<String, String> {
    // Detect active window position and dimensions
    let window =
        get_active_window().map_err(|e| format!("Failed to get active window: {:?}", e))?;

    // Extract window bounds (in logical coordinates)
    // ISSUE: These are logical coords, but image is in physical pixels (Retina)
    let x = window.position.x.max(0.0) as u32;
    let y = window.position.y.max(0.0) as u32;
    let width = (window.position.width as u32).min(dynamic_image.width() - x);
    let height = (window.position.height as u32).min(dynamic_image.height() - y);

    // Crop full screen image to window bounds
    let cropped = dynamic_image.crop_imm(x, y, width, height);

    // Save window crop to disk
    let window_filename = format!("event_{}_window.png", event_id);
    let window_filepath = session_dir.join(&window_filename);
    cropped
        .save(&window_filepath)
        .map_err(|e| format!("Failed to save window crop: {:?}", e))?;

    Ok(format!("recordings/{}/{}", session_id, window_filename))
}

/// Captures a 300x300px crop centered on the click position.
///
/// Provides visual context of the UI element that was clicked, useful for
/// understanding what button/link/element the user interacted with.
///
/// # Arguments
/// * `dynamic_image` - Full screen capture to crop from
/// * `session_id` - UUID for file path
/// * `event_id` - UUID for filename
/// * `session_dir` - Directory to save crop
/// * `click_x` - X coordinate of click in logical pixels
/// * `click_y` - Y coordinate of click in logical pixels
/// * `screen_width` - Full screen width (for bounds checking)
/// * `screen_height` - Full screen height (for bounds checking)
///
/// # Returns
/// * `Ok(String)` - Relative path to saved click crop
/// * `Err(String)` - Error if crop calculation or save fails
///
/// # Crop Dimensions
/// - Target size: 300x300px square
/// - Centered on click position (±150px in each direction)
/// - Bounded to screen edges (won't extend beyond display)
///
/// # How It Works
/// 1. Calculate crop center: `(click_x, click_y)`
/// 2. Calculate crop bounds: `(x - 150, y - 150)` to `(x + 150, y + 150)`
/// 3. Apply bounds checking:
///    - Clamp to screen edges
///    - Ensure crop doesn't extend beyond 0 or max dimensions
/// 4. Crop full screen image to calculated rectangle
/// 5. Save as separate PNG file
///
/// # Known Issue: Retina Display Coordinate Scaling
///
/// **Problem**: Click coordinates are logical but image is physical pixels.
///
/// **Example on 2x Retina Display**:
/// - Click at: `(713, 395)` logical coordinates
/// - Screenshot: 2880x1800 physical pixels
/// - Expected crop center: `(1426, 790)` physical pixels (713×2, 395×2)
/// - Actual crop center: `(713, 395)` physical pixels
/// - **Result**: Crop centered on wrong location, offset by 2x
///
/// **Visual Impact**:
/// - If user clicks a button at logical (713, 395)
/// - Crop captures area around physical (713, 395) instead of (1426, 790)
/// - Button appears offset from center of crop image
///
/// **Fix** (not implemented):
/// ```rust
/// let scale = detect_display_scale_factor(); // e.g., 2.0
/// let physical_x = (click_x as f64 * scale) as i32;
/// let physical_y = (click_y as f64 * scale) as i32;
/// // Calculate crop bounds using physical coordinates
/// ```
///
/// # Bounds Checking Logic
/// - `(click_x - 150).max(0)` - Prevent negative x
/// - `.min(screen_width - 300)` - Prevent extending beyond right edge
/// - Similar logic for y-axis
/// - Final width/height clamped to remaining space if near edge
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
    /// Crop size: 300x300px provides good UI element context without being too large
    const CROP_SIZE: i32 = 300;
    /// Half of crop size used for centering calculation (±150px from click point)
    const HALF_SIZE: i32 = CROP_SIZE / 2;

    // Calculate crop bounds centered on click position
    // ISSUE: click_x and click_y are logical coords, but image is physical pixels (Retina)
    // Apply bounds checking to prevent cropping beyond screen edges
    let x = (click_x - HALF_SIZE).max(0).min(screen_width - CROP_SIZE) as u32;
    let y = (click_y - HALF_SIZE).max(0).min(screen_height - CROP_SIZE) as u32;

    // Final dimensions may be less than CROP_SIZE if near screen edge
    let width = CROP_SIZE.min(screen_width - x as i32) as u32;
    let height = CROP_SIZE.min(screen_height - y as i32) as u32;

    // Perform crop operation (non-mutating, returns new image)
    let cropped = dynamic_image.crop_imm(x, y, width, height);

    // Save click crop to disk
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
