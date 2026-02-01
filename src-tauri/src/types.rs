//! # Type Definitions for FlowTrace Recording System
//!
//! Core data structures representing:
//! - **Recording sessions**: Container for captured events with timestamps
//! - **Events**: Individual user actions (clicks, keypresses, waits)
//! - **Event classification**: Automatic categorization and description generation
//!
//! ## Data Flow
//! ```text
//! User Action → rdev::Event → EventType → Event → RecordingSession → JSON
//! ```
//!
//! ## Serialization
//! All types derive `Serialize` + `Deserialize` for JSON persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A complete recording session containing all captured user interactions.
///
/// # Structure
/// - **session_id**: UUIDv4 identifier (e.g., `f2e904d2-286e-484c-83e8-5949bd8697f1`)
/// - **started_at**: UTC timestamp when recording began
/// - **stopped_at**: UTC timestamp when recording ended (None if still recording)
/// - **events**: Ordered list of all captured events
///
/// # Lifecycle
/// 1. Created via `RecordingSession::new()` when user starts recording
/// 2. Events added via `add_event()` as user interacts
/// 3. Finalized via `stop()` when user stops recording
/// 4. Serialized to JSON via `storage::save_session()`
///
/// # Example JSON Output
/// ```json
/// {
///   "session_id": "f2e904d2-286e-484c-83e8-5949bd8697f1",
///   "started_at": "2026-02-01T15:43:08.646618Z",
///   "stopped_at": "2026-02-01T15:43:18.855192Z",
///   "events": [...]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecordingSession {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub events: Vec<Event>,
}

impl RecordingSession {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            started_at: Utc::now(),
            stopped_at: None,
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn stop(&mut self) {
        self.stopped_at = Some(Utc::now());
    }
}

/// Screenshot file paths for a single event.
///
/// Each event can have up to 3 associated screenshots.
/// Paths are relative to project root: `recordings/[session-id]/[filename]`
///
/// # Fields
/// - **full_screen**: Always `Some(String)` for click events, `None` for keyboard/wait
/// - **window_crop**: `Some(String)` if window detection succeeded, `None` otherwise
/// - **click_crop**: `Some(String)` if crop succeeded, `None` otherwise
///
/// # Known Limitation
/// Window and click crops may be offset on Retina displays due to
/// logical vs physical coordinate mismatch. Full screen always works.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Screenshots {
    pub full_screen: Option<String>,
    pub window_crop: Option<String>,
    pub click_crop: Option<String>,
}

/// A single captured user action with metadata, classification, and screenshots.
///
/// Events are the fundamental unit of recording. Each represents one user action:
/// - Mouse click (with position and screenshots)
/// - Keyboard press (with key name)
/// - Wait/pause (with duration)
///
/// # Automatic Classification
/// Events are automatically classified into 8 categories:
/// - `interaction` - Click events
/// - `text_input` - Letter and number keys
/// - `submit` - Enter/Return key
/// - `navigation` - Tab key
/// - `correction` - Backspace/Delete keys
/// - `cancel` - Escape key
/// - `wait` - Automatic pause detection
/// - `special_key` - Other special keys
///
/// # Fields
/// - **id**: UUIDv4 unique identifier
/// - **event_type**: Discriminated union (Click | KeyPress | Wait)
/// - **timestamp**: UTC timestamp when event occurred
/// - **position**: Screen coordinates (Some for clicks, None for keyboard/wait)
/// - **screenshots**: Paths to associated screenshot files
/// - **action_category**: One of 8 classification categories
/// - **description**: Human-readable description (e.g., "Clicked left button at (709, 328)")
///
/// # Example JSON
/// ```json
/// {
///   "id": "cece1f95-8a90-4fa5-8fcc-2995113918ab",
///   "event_type": {"type": "Click", "button": "Left"},
///   "timestamp": "2026-02-01T15:43:11.627959Z",
///   "position": {"x": 709, "y": 328},
///   "screenshots": {
///     "full_screen": "recordings/.../event_..._full.png",
///     "window_crop": "recordings/.../event_..._window.png",
///     "click_crop": "recordings/.../event_..._click.png"
///   },
///   "action_category": "interaction",
///   "description": "Clicked left button at position (709, 328)"
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub position: Option<Position>,
    pub screenshots: Screenshots,
    pub action_category: String,
    pub description: String,
}

impl Event {
    pub fn new(event_type: EventType, position: Option<Position>) -> Self {
        let (action_category, description) = Self::classify_and_describe(&event_type, &position);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            position,
            screenshots: Screenshots {
                full_screen: None,
                window_crop: None,
                click_crop: None,
            },
            action_category,
            description,
        }
    }

    pub fn with_screenshots(
        mut self,
        full: Option<String>,
        window: Option<String>,
        click: Option<String>,
    ) -> Self {
        self.screenshots = Screenshots {
            full_screen: full,
            window_crop: window,
            click_crop: click,
        };
        self
    }

    /// Automatically classifies an event and generates a human-readable description.
    ///
    /// This is the core classification engine that analyzes event types and
    /// assigns semantic meaning for workflow analysis.
    ///
    /// # Classification Categories
    ///
    /// | Category | Events | Purpose |
    /// |----------|--------|---------|
    /// | `interaction` | All click events | User interacting with UI elements |
    /// | `text_input` | Letters (KeyA-KeyZ), Numbers (Num0-Num9), Space | User typing content |
    /// | `submit` | Enter, Return keys | User submitting forms/commands |
    /// | `navigation` | Tab key | User navigating between fields |
    /// | `correction` | Backspace, Delete keys | User fixing mistakes |
    /// | `cancel` | Escape key | User canceling operations |
    /// | `wait` | Synthetic events (gap > 2s) | User pausing/thinking |
    /// | `special_key` | Other keys (arrows, function keys, etc.) | Other keyboard actions |
    ///
    /// # Description Format
    ///
    /// **Clicks**: `"Clicked {button} button at position ({x}, {y})"`
    /// - Example: `"Clicked left button at position (709, 328)"`
    ///
    /// **Text Input**: `"Typed: {key}"`
    /// - Example: `"Typed: A"`, `"Typed: 5"`, `"Typed: Space"`
    ///
    /// **Special Actions**: `"Pressed {key} ({category})"`
    /// - Example: `"Pressed Enter (submit)"`, `"Pressed Tab (navigate)"`
    ///
    /// **Wait Events**: `"Paused for {duration} seconds"`
    /// - Example: `"Paused for 2.7 seconds"`
    ///
    /// # Algorithm
    /// 1. Match on `EventType` (Click | KeyPress | Wait)
    /// 2. For clicks: Return "interaction" category with position
    /// 3. For keypresses: Analyze key name to determine category
    /// 4. For waits: Return "wait" category with duration
    ///
    /// # Key Classification Logic
    /// - Starts with "Key" → Letter key (KeyA, KeyB, ...) → `text_input`
    /// - Starts with "Num" → Number key (Num1, Num2, ...) → `text_input`
    /// - "Space" → `text_input`
    /// - "Return"/"Enter" → `submit`
    /// - "Tab" → `navigation`
    /// - "Backspace"/"Delete" → `correction`
    /// - "Escape" → `cancel`
    /// - Other → `special_key`
    ///
    /// # Arguments
    /// * `event_type` - The type of event to classify
    /// * `position` - Optional screen position (used for click descriptions)
    ///
    /// # Returns
    /// `(category: String, description: String)` tuple
    fn classify_and_describe(
        event_type: &EventType,
        position: &Option<Position>,
    ) -> (String, String) {
        match event_type {
            // CLICKS: Always classified as "interaction"
            EventType::Click { button } => {
                let category = "interaction".to_string();
                let button_name = match button {
                    MouseButton::Left => "left",
                    MouseButton::Right => "right",
                    MouseButton::Middle => "middle",
                };
                let description = match position {
                    Some(pos) => format!(
                        "Clicked {} button at position ({}, {})",
                        button_name, pos.x, pos.y
                    ),
                    None => format!("Clicked {} button", button_name),
                };
                (category, description)
            }
            // KEYBOARD: Classify based on key type and purpose
            EventType::KeyPress { key } => {
                // Pattern match on key name to determine intent
                let (category, description) = if key.starts_with("Key") {
                    // Letter keys: KeyA, KeyB, KeyC, ... KeyZ
                    // Purpose: User typing text content
                    let letter = key.strip_prefix("Key").unwrap_or(key);
                    ("text_input".to_string(), format!("Typed: {}", letter))
                } else if key.starts_with("Num") {
                    // Number keys: Num0, Num1, Num2, ... Num9
                    // Purpose: User typing numeric content
                    let num = key.strip_prefix("Num").unwrap_or(key);
                    ("text_input".to_string(), format!("Typed: {}", num))
                } else if key == "Space" {
                    // Space bar: User typing whitespace
                    ("text_input".to_string(), "Typed: Space".to_string())
                } else if key == "Return" || key == "Enter" {
                    // Submit action: User confirming/submitting form or command
                    ("submit".to_string(), "Pressed Enter (submit)".to_string())
                } else if key == "Tab" {
                    // Navigation: User moving between fields or UI elements
                    (
                        "navigation".to_string(),
                        "Pressed Tab (navigate)".to_string(),
                    )
                } else if key == "Backspace" || key == "Delete" {
                    // Correction: User fixing typos or removing content
                    (
                        "correction".to_string(),
                        format!("Pressed {} (correction)", key),
                    )
                } else if key == "Escape" {
                    // Cancel: User aborting operation or closing modal
                    ("cancel".to_string(), "Pressed Escape (cancel)".to_string())
                } else {
                    // Other special keys: Arrows, Function keys, etc.
                    // Catch-all for keys that don't fit other categories
                    ("special_key".to_string(), format!("Pressed: {}", key))
                };
                (category, description)
            }
            // WAIT: Synthetic event for pauses > 2 seconds
            EventType::Wait { duration_seconds } => {
                // Purpose: Capture user thinking time, page loads, or natural workflow pauses
                (
                    "wait".to_string(),
                    format!("Paused for {:.1} seconds", duration_seconds),
                )
            }
        }
    }
}

/// Discriminated union representing the type of captured event.
///
/// Uses `#[serde(tag = "type")]` for tagged union serialization.
/// Each variant carries its specific data.
///
/// # Variants
///
/// **Click** - Mouse button press
/// - `button: MouseButton` - Which button was pressed
/// - Has position (tracked from MouseMove events)
/// - Triggers 3 screenshots
///
/// **KeyPress** - Keyboard key press
/// - `key: String` - Key name (e.g., "KeyA", "Return", "Space")
/// - No position (keyboard events aren't location-based)
/// - No screenshots (design decision to reduce storage)
///
/// **Wait** - Synthetic pause event (auto-generated)
/// - `duration_seconds: f64` - Length of pause
/// - No position
/// - No screenshots
///
/// # JSON Serialization
/// ```json
/// {"type": "Click", "button": "Left"}
/// {"type": "KeyPress", "key": "KeyA"}
/// {"type": "Wait", "duration_seconds": 2.704}
/// ```
///
/// # Descoped
/// - `MouseMove` - Too noisy (100+ events/second), only used for position tracking
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventType {
    Click { button: MouseButton },
    KeyPress { key: String },
    Wait { duration_seconds: f64 },
    // MouseMove, // Descoped for MVP (too noisy)
}

/// Mouse button types that can be captured.
///
/// # Supported Buttons
/// - **Left** - Primary button (most common)
/// - **Right** - Context menu button
/// - **Middle** - Middle button / scroll wheel click
///
/// # Filtered Out
/// Other buttons (forward/back, trackpad gestures) are ignored by the event handler.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Screen coordinates for event position.
///
/// # Coordinate System
/// - Origin: Top-left corner of primary display (0, 0)
/// - X-axis: Increases right
/// - Y-axis: Increases down
/// - Units: **Logical pixels** (not physical pixels)
///
/// # Known Limitation: Retina Displays
/// On HiDPI displays (e.g., 2x Retina), these are logical coordinates:
/// - Logical position: (713, 395)
/// - Physical pixels: (1426, 790) on 2x display
/// - This mismatch causes offset crops in screenshot module
///
/// # Example
/// ```json
/// {"x": 709, "y": 328}
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: x as i32,
            y: y as i32,
        }
    }
}
