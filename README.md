# FlowTrace

**A Desktop Application for Recording Workflow Events**

FlowTrace is a Tauri-based desktop application that captures global click and keyboard events with synchronized screenshots, storing them as structured JSON for workflow analysis and documentation.

---

## 📋 What Was Built

### ✅ MUST HAVE Features Implemented

- **Global Event Monitoring**
  - Mouse click events (left, right, middle buttons) with position tracking
  - Keyboard events (letters, numbers, special keys) with modifier filtering
  - Automatic wait/pause detection (gaps > 2 seconds between events)

- **Screenshot Capture**
  - **Full screen**: Complete display capture for each click event
  - **Window crop**: Active window detection and capture
  - **Click crop**: 300x300px crop centered on click position
  - All screenshots saved as PNG files (~2.2MB each)

- **Event Storage**
  - Session-based directory structure: `recordings/[session-id]/`
  - JSON format with pretty-printing for readability
  - Event metadata: timestamps, positions, screenshots, classifications

- **Recording Interface**
  - Start/Stop recording buttons in desktop UI
  - Real-time terminal logging for debugging
  - Session management with UUID-based identifiers

### ✅ SHOULD HAVE Features Implemented

- **Action Classification**
  - 8 event categories: `interaction`, `text_input`, `submit`, `navigation`, `correction`, `cancel`, `wait`, `special_key`
  - Human-readable descriptions for each event
  - Example: `"Clicked left button at position (709, 328)"` or `"Pressed Enter (submit)"`

### ❌ Descoped Features

- **Event log UI display**: Terminal logging only (no in-app visualization)
- **Step auto-grouping**: Each event stored individually (manual grouping possible post-processing)
- **Advanced screenshot options**: No OCR, no compression, no format selection

**Descoping Rationale**: With a 4-hour time constraint, these features would have compromised the quality of core functionality. Better to deliver polished essential features than buggy comprehensive implementation.

---

## 🚀 Setup Instructions

### Prerequisites

- **macOS** (tested on macOS Sonoma 25.2.0)
- **Rust** 1.70+ with Cargo
- **Node.js** 18+ with npm
- **Xcode Command Line Tools**

### Installation

1. **Clone the repository**
   ```bash
   cd FlowTrace
   ```

2. **Install dependencies**
   ```bash
   npm install
   cd src-tauri
   cargo build
   cd ..
   ```

3. **Configure macOS Permissions** ⚠️ **CRITICAL**

   macOS requires explicit permissions for event monitoring and screenshots. You must grant permissions to **your development tools**, not the app itself:

   **For Cursor (or VS Code):**
   - Open **System Settings** → **Privacy & Security**
   - Click **Accessibility** → Unlock with password
   - Add **Cursor.app** (or your code editor) and enable
   - Click **Screen Recording**
   - Add **Cursor.app** and enable

   **For iTerm2 (or Terminal):**
   - Open **System Settings** → **Privacy & Security**
   - Click **Accessibility** → Unlock with password
   - Add **iTerm2.app** (or Terminal.app) and enable
   - Click **Screen Recording**
   - Add **iTerm2.app** and enable

   **Why Both?** Tauri apps launched via `npm run tauri dev` inherit permissions from the launching process (editor + terminal), not from the app itself.

4. **Restart your editor and terminal** after granting permissions

### Running the Application

```bash
npm run tauri dev
```

The desktop window will open with **Start Recording** and **Stop Recording** buttons.

---

## 📖 How to Use

1. **Start Recording**: Click "Start Recording" button
   - Event monitoring begins immediately
   - All clicks and keypresses are captured
   - Pauses > 2 seconds automatically logged as Wait events

2. **Perform Actions**: Use your mouse and keyboard normally
   - Clicks trigger 3 screenshots: full screen, window crop, click crop
   - Keypresses are logged without screenshots (performance optimization)
   - Terminal shows real-time event logging

3. **Stop Recording**: Click "Stop Recording" button
   - Session is saved to `recordings/[session-id]/`
   - JSON file created with all events and metadata
   - Screenshots saved as PNG files in same directory

4. **Review Output**:
   ```bash
   ls recordings/
   # Output: [session-id]/

   ls recordings/[session-id]/
   # Output: session.json, event_[id]_full.png, event_[id]_window.png, event_[id]_click.png

   cat recordings/[session-id]/session.json
   # Pretty-printed JSON with all events
   ```

---

## 📄 Example Output

### Session Structure

```
recordings/
└── f2e904d2-286e-484c-83e8-5949bd8697f1/
    ├── session.json
    ├── event_cece1f95-8a90-4fa5-8fcc-2995113918ab_full.png
    ├── event_cece1f95-8a90-4fa5-8fcc-2995113918ab_window.png
    ├── event_cece1f95-8a90-4fa5-8fcc-2995113918ab_click.png
    └── [additional event screenshots...]
```

### Sample JSON Output

```json
{
  "session_id": "f2e904d2-286e-484c-83e8-5949bd8697f1",
  "started_at": "2026-02-01T15:43:08.646618Z",
  "stopped_at": "2026-02-01T15:43:18.855192Z",
  "events": [
    {
      "id": "cece1f95-8a90-4fa5-8fcc-2995113918ab",
      "event_type": {
        "type": "Click",
        "button": "Left"
      },
      "timestamp": "2026-02-01T15:43:11.627959Z",
      "position": {
        "x": 709,
        "y": 328
      },
      "screenshots": {
        "full_screen": "recordings/f2e904d2-286e-484c-83e8-5949bd8697f1/event_cece1f95-8a90-4fa5-8fcc-2995113918ab_full.png",
        "window_crop": "recordings/f2e904d2-286e-484c-83e8-5949bd8697f1/event_cece1f95-8a90-4fa5-8fcc-2995113918ab_window.png",
        "click_crop": "recordings/f2e904d2-286e-484c-83e8-5949bd8697f1/event_cece1f95-8a90-4fa5-8fcc-2995113918ab_click.png"
      },
      "action_category": "interaction",
      "description": "Clicked left button at position (709, 328)"
    },
    {
      "id": "e5f42293-3add-4fef-9e8c-df337f784814",
      "event_type": {
        "type": "Wait",
        "duration_seconds": 2.704
      },
      "timestamp": "2026-02-01T15:43:14.332838Z",
      "position": null,
      "screenshots": {
        "full_screen": null,
        "window_crop": null,
        "click_crop": null
      },
      "action_category": "wait",
      "description": "Paused for 2.7 seconds"
    }
  ]
}
```

---

## ⚠️ Known Limitations

### 1. Retina Display Coordinate Scaling Issue

**Problem**: On Retina/HiDPI displays (common on MacBooks), window crop and click crop screenshots are not accurately centered.

**Technical Cause**:
- **Event coordinates** (from `rdev` and `active-win-pos-rs`): Logical coordinates
  - Example: Click at (713, 395) on a 2x Retina display
- **Screenshot pixels** (from `screenshots` crate): Physical pixels
  - Example: Image is 2880x1800 (2x the logical 1440x900)
- **Mismatch**: Crop calculations apply logical coordinates to physical pixels
  - Window crop: Uses logical window bounds on physical pixel image → 2x offset
  - Click crop: Uses logical click position on physical pixel image → 2x offset

**Impact**:
- **Full screen**: ✅ Works perfectly (no coordinates needed)
- **Window crop**: ❌ Captures area offset from actual window
- **Click crop**: ❌ Captures area offset from actual click location

**Current Status**: Documented limitation, not fixed in this MVP.

**Potential Fix** (not implemented):
```rust
// Detect display scale factor
let scale_factor = get_display_scale_factor(); // e.g., 2.0 for Retina

// Adjust coordinates when cropping
let physical_x = (logical_x * scale_factor) as u32;
let physical_y = (logical_y * scale_factor) as u32;
let physical_width = (logical_width * scale_factor) as u32;
let physical_height = (logical_height * scale_factor) as u32;

// Use physical coordinates for crop
let cropped = dynamic_image.crop_imm(physical_x, physical_y, physical_width, physical_height);
```

**Affected Code**:
- `src-tauri/src/screenshot.rs:126-152` (capture_window_crop)
- `src-tauri/src/screenshot.rs:154-184` (capture_click_crop)

**Why Not Fixed?**: Discovered in final testing hour. Fixing would require:
- Research into display scale detection APIs
- Testing on multiple display configurations
- Risk of introducing new bugs close to deadline
- Decision: Document thoroughly rather than rush incomplete fix

### 2. Event Listener Graceful Shutdown

**Problem**: `rdev::listen()` blocks forever and cannot be gracefully stopped.

**Impact**: Must restart the entire application to start a new recording session after stopping.

**Workaround**: Close and reopen the app between recording sessions.

**Proper Fix** (estimated +30 minutes):
- Use tokio channels to communicate stop signal
- Add conditional event handling to exit listener loop
- Require significant refactoring of threading model

### 3. Click Position Accuracy

**Problem**: Click positions are tracked from `MouseMove` events, not `ButtonPress` events (rdev API limitation).

**Impact**: Position might be 1-5 pixels off if clicking while moving mouse rapidly.

**Acceptable for MVP**: This is a standard rdev pattern and has minimal impact on workflow documentation.

### 4. First Screenshot Delay

**Problem**: First screenshot in a session takes 2-3 seconds to capture.

**Cause**: Lazy initialization of `screenshots` crate internal state.

**Impact**: First event has noticeable delay; subsequent events are fast.

**Status**: Normal behavior for screenshots crate, no fix needed.

### 5. Keyboard Event Screenshots

**Design Decision**: Keyboard events do NOT capture screenshots (clicks do).

**Rationale**:
- Reduces storage requirements (each screenshot ~2.2MB)
- Improves performance (no screenshot overhead for rapid typing)
- Click screenshots provide sufficient visual context

**Trade-off**: Less visual evidence for what was typed, but action_category and description provide context.

---

## 🏗️ Architecture Overview

### Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.0
- **Event Monitoring**: `rdev` (RustDesk fork for macOS stability)
- **Screenshots**: `screenshots` crate + `image` crate for manipulation
- **Window Detection**: `active-win-pos-rs`
- **Storage**: JSON files with `serde` serialization

### Key Files

- **`src-tauri/src/lib.rs`**: Main integration, event handling, global state management
- **`src-tauri/src/types.rs`**: Data structures (Event, EventType, RecordingSession)
- **`src-tauri/src/screenshot.rs`**: Screenshot capture logic (full, window, click crops)
- **`src-tauri/src/storage.rs`**: JSON file I/O and session directory management
- **`src-tauri/src/event_monitor.rs`**: Basic rdev event listener (spike testing)
- **`src/App.vue`**: Recording UI (Start/Stop buttons)

### Threading Model

```
Main Thread (Tauri)
├── UI event handlers (Start/Stop Recording)
└── Shared State: Arc<Mutex<Option<RecordingSession>>>

Background Thread (rdev listener)
├── handle_event() - processes clicks and keypresses
├── check_and_insert_wait_event() - detects pauses
├── screenshot::capture_all_for_event() - takes 3 screenshots
└── Updates shared state via Arc<Mutex<>>
```

### Event Flow

```
1. User Action (click/keypress)
   ↓
2. rdev::listen() captures event
   ↓
3. handle_event() classifies and creates Event struct
   ↓
4. [If click] capture_all_for_event() takes 3 screenshots
   ↓
5. Event added to RecordingSession.events
   ↓
6. [On stop] save_session() writes JSON to disk
```

---

## 🤖 AI Tool Usage Summary

### Development Process

**Tool Used**: Claude Code (Sonnet 4.5)

**Total Development Time**: 3 hours 15 minutes active work
- Hour 0-1: Foundation & spike testing
- Hour 1-2: Integration pipeline
- Hour 2-3: Keyboard monitoring + action classification
- Hour 3-4: 3 screenshots implementation + documentation

### How AI Was Used

1. **Technical Research & Planning** (Hour 0)
   - Identified macOS-compatible crates (`rdev` fork vs official)
   - Warned about permission requirements
   - Provided time estimates for descoping decisions
   - **Impact**: Avoided 30+ minutes of debugging known issues

2. **Implementation Assistance** (Hours 1-3)
   - Generated boilerplate code for modules
   - Provided architectural patterns (Arc<Mutex<>>, session directories)
   - Quick bug diagnosis (chrono serde feature, rdev position tracking)
   - **Impact**: Accelerated implementation, minimized time lost to bugs

3. **Code Review & Safety** (Throughout)
   - Caught potential deadlock (lock held during screenshot)
   - Suggested modifier key filtering for keyboard events
   - Recommended performance optimizations (no screenshots for keypresses)
   - **Impact**: Prevented runtime issues, improved user experience

4. **Documentation** (Hour 4)
   - Comprehensive implementation log with git timeline analysis
   - Technical documentation for retina display limitation
   - README with setup instructions and examples
   - **Impact**: Clear communication of decisions and trade-offs

### What AI Did Well

✅ **Proactive Issue Flagging**: Identified rdev macOS crash bug BEFORE encountering it
✅ **Quick Bug Fixes**: Instantly diagnosed missing chrono serde feature
✅ **Descoping Recommendations**: Suggested realistic time estimates that kept project on track
✅ **Code Review**: Caught concurrency issues and performance bottlenecks

### What AI Struggled With

⚠️ **API Assumptions**: Assumed rdev had unified `event.position` field without verifying (cost: 10 min)
⚠️ **Time Estimates**: Some estimates were optimistic (e.g., "10 min" actually took 20 min)
⚠️ **Platform Nuances**: Didn't explicitly mention parent process permissions for macOS

### Collaboration Patterns

**Most Effective**:
- Start with strategic planning and descoping decisions
- Use AI for technical research and crate compatibility
- Verify AI assumptions with compiler feedback
- Let AI handle boilerplate, developer handles architecture decisions

**Lessons Learned**:
- Always verify external crate APIs, especially forks
- Treat AI time estimates as lower bounds (~1.5x multiplier)
- Use AI proactively for code review, not just implementation

---

## 📊 What Was Delivered vs What Was Descoped

| Requirement | Status | Notes |
|-------------|--------|-------|
| **Global click monitoring** | ✅ Complete | Left, right, middle buttons with position tracking |
| **Global keyboard monitoring** | ✅ Complete | Letters, numbers, special keys with modifier filtering |
| **3 screenshots per click** | ✅ Complete | Full screen, window crop, click crop (with retina caveat) |
| **JSON storage** | ✅ Complete | Session-based directories, pretty-printed format |
| **Start/Stop recording UI** | ✅ Complete | Simple button interface |
| **Action classification** | ✅ Complete | 8 categories with human-readable descriptions |
| **Wait detection** | ✅ Complete | Automatic pause detection (> 2 seconds) |
| **Event log UI** | ❌ Descoped | Terminal logging only, no in-app visualization |
| **Step auto-grouping** | ❌ Descoped | Each event stored individually |
| **Advanced screenshot options** | ❌ Descoped | No OCR, compression, or format selection |

**Descoping Strategy**: Focus on core functionality depth rather than feature breadth. Better to deliver polished essential features than buggy comprehensive implementation within 4-hour constraint.

---

## 🧪 Testing

### Manual Test Cases

1. **Basic Recording** ✅
   - Start recording → Click 5 times → Stop recording
   - Verify: 5 events in JSON, 15 PNG files (3 per click)

2. **Keyboard Events** ✅
   - Start recording → Type "hello world" → Stop recording
   - Verify: 11 KeyPress events in JSON (no screenshots)

3. **Wait Detection** ✅
   - Start recording → Click → Wait 3 seconds → Click → Stop recording
   - Verify: 2 Click events + 1 Wait event with ~3s duration

4. **Action Classification** ✅
   - Start recording → Click + Type + Press Enter → Stop recording
   - Verify: "interaction", "text_input", "submit" categories in JSON

5. **Session Management** ✅
   - Multiple recording sessions → Stop recording
   - Verify: Separate directories with unique session IDs

### Test Recordings Included

- `recordings/f2e904d2-286e-484c-83e8-5949bd8697f1/` - Example session with 4 events
- See `session.json` for full event structure

---

## 📚 Additional Documentation

For detailed implementation timeline, bug reports, and AI collaboration analysis, see:

- **`prompts/00-planning-phase.md`** - Strategic planning and descoping decisions
- **`prompts/01-implementation-log.md`** - Hour-by-hour development log with git timeline
- **`prompts/QUICK-REFERENCE.md`** - Troubleshooting guide and common issues

---

## 🙏 Acknowledgments

**Assignment**: Take-home coding challenge (4-hour time limit)

**Development Environment**:
- macOS Sonoma 25.2.0
- Rust 1.70+
- Cursor (VS Code fork) with Claude Code integration
- iTerm2 terminal

**Key Dependencies**:
- `rdev` (RustDesk fork) - Cross-platform input monitoring
- `screenshots` - Screen capture library
- `active-win-pos-rs` - Active window detection
- `tauri` - Desktop app framework

---

## 📝 License

This is a take-home assignment project. All rights reserved.

---

**Built with Claude Code in 3 hours 15 minutes** 🤖
