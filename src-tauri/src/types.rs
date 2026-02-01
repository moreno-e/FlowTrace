use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a complete recording session with all captured events
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

/// Screenshots captured for an event
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Screenshots {
    pub full_screen: Option<String>,
    pub window_crop: Option<String>,
    pub click_crop: Option<String>,
}

/// Represents a single captured event (click, keypress, etc.)
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

    pub fn with_screenshots(mut self, full: Option<String>, window: Option<String>, click: Option<String>) -> Self {
        self.screenshots = Screenshots {
            full_screen: full,
            window_crop: window,
            click_crop: click,
        };
        self
    }

    /// Classify event and generate human-readable description
    fn classify_and_describe(event_type: &EventType, position: &Option<Position>) -> (String, String) {
        match event_type {
            EventType::Click { button } => {
                let category = "interaction".to_string();
                let button_name = match button {
                    MouseButton::Left => "left",
                    MouseButton::Right => "right",
                    MouseButton::Middle => "middle",
                };
                let description = match position {
                    Some(pos) => format!("Clicked {} button at position ({}, {})", button_name, pos.x, pos.y),
                    None => format!("Clicked {} button", button_name),
                };
                (category, description)
            }
            EventType::KeyPress { key } => {
                // Classify based on key type
                let (category, description) = if key.starts_with("Key") {
                    // Letter key (KeyA, KeyB, etc.)
                    let letter = key.strip_prefix("Key").unwrap_or(key);
                    ("text_input".to_string(), format!("Typed: {}", letter))
                } else if key.starts_with("Num") {
                    // Number key (Num1, Num2, etc.)
                    let num = key.strip_prefix("Num").unwrap_or(key);
                    ("text_input".to_string(), format!("Typed: {}", num))
                } else if key == "Space" {
                    ("text_input".to_string(), "Typed: Space".to_string())
                } else if key == "Return" || key == "Enter" {
                    ("submit".to_string(), "Pressed Enter (submit)".to_string())
                } else if key == "Tab" {
                    ("navigation".to_string(), "Pressed Tab (navigate)".to_string())
                } else if key == "Backspace" || key == "Delete" {
                    ("correction".to_string(), format!("Pressed {} (correction)", key))
                } else if key == "Escape" {
                    ("cancel".to_string(), "Pressed Escape (cancel)".to_string())
                } else {
                    // Other special keys
                    ("special_key".to_string(), format!("Pressed: {}", key))
                };
                (category, description)
            }
            EventType::Wait { duration_seconds } => {
                ("wait".to_string(), format!("Paused for {:.1} seconds", duration_seconds))
            }
        }
    }
}

/// Types of events we can capture
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventType {
    Click { button: MouseButton },
    KeyPress { key: String },
    Wait { duration_seconds: f64 },
    // MouseMove, // Descoped for MVP (too noisy)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Screen coordinates
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
