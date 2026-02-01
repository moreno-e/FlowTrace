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

/// Represents a single captured event (click, keypress, etc.)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub position: Option<Position>,
    pub screenshot_path: Option<String>,
}

impl Event {
    pub fn new(event_type: EventType, position: Option<Position>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            position,
            screenshot_path: None,
        }
    }

    pub fn with_screenshot(mut self, path: String) -> Self {
        self.screenshot_path = Some(path);
        self
    }
}

/// Types of events we can capture
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventType {
    Click { button: MouseButton },
    // KeyPress { key: String }, // Descoped for MVP
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
