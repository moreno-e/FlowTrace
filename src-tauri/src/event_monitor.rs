//! # Event Monitor Module - Spike Testing
//!
//! Basic event listener for testing and debugging `rdev` integration.
//! This module was used during initial spike testing to verify event
//! capture works before building the full recording system.
//!
//! ## Purpose
//! - **Testing**: Verify rdev works on macOS
//! - **Debugging**: See raw event data from rdev
//! - **Demonstration**: Show event capture without recording
//!
//! ## Usage
//! Available in UI under "Test Functions (Spike Testing)" collapsible section.
//! Invoked via `start_event_listener()` Tauri command.
//!
//! ## Production Alternative
//! In production, use `start_recording()` which integrates event listening
//! with session management, screenshots, and storage.

use rdev::{listen, Event};

/// Starts a raw event listener that prints all system events to console.
///
/// **Purpose**: Testing/debugging only. Runs indefinitely until application closes.
///
/// # What It Captures
/// - All mouse events (move, click, release)
/// - All keyboard events (press, release)
/// - Raw rdev event data
///
/// # Output
/// Prints to stdout:
/// ```text
/// 🎯 Starting event listener test...
/// Event captured: Event { event_type: MouseMove { x: 713.0, y: 395.0 }, ... }
/// Event captured: Event { event_type: ButtonPress(Left), ... }
/// ```
///
/// # Blocking Behavior
/// `rdev::listen()` blocks forever. This function will never return unless
/// an error occurs. Should only be called from a background thread.
///
/// # Permissions Required
/// - macOS: Accessibility permission for launching application
///
/// # Invocation
/// Called by `start_event_listener()` Tauri command, which spawns this
/// in a background thread:
/// ```rust
/// std::thread::spawn(|| {
///     event_monitor::test_listener();
/// });
/// ```
pub fn test_listener() {
    #[cfg(debug_assertions)]
    println!("🎯 Starting event listener test...");

    // Start listening (blocks forever)
    if let Err(error) = listen(callback) {
        eprintln!("❌ Event listener error: {:?}", error);
    }
}

/// Callback function invoked by `rdev::listen()` for each captured event.
///
/// Prints raw event data to console for debugging. In production code
/// (see `lib.rs::handle_event()`), this would filter events, capture
/// screenshots, and add to recording session.
///
/// # Arguments
/// * `event` - Raw event from rdev containing type, timestamp, and metadata
///
/// # Output
/// ```text
/// Event captured: Event {
///     time: SystemTime { ... },
///     unicode: None,
///     platform_code: 1,
///     event_type: MouseMove { x: 713.0, y: 395.0 }
/// }
/// ```
fn callback(event: Event) {
    println!("Event captured: {:?}", event);
}
