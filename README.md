# FlowTrace

**Desktop Application for Defining Automation Workflow Steps**

FlowTrace is a Tauri-based desktop application that captures global click and keyboard events with synchronized screenshots, storing them as structured JSON to define automation workflow steps that could be consumed by automation tools.

**Tech Stack**: Tauri 2.0 + Rust backend + Vue 3 frontend + JSON storage

**Demo**
<div>
    <a href="https://www.loom.com/share/149ff48564434ddba68cb86d3f4b348c">
      <p>Automating Legacy Workflows with Flowtrace: A Desktop Application Demo - Watch Video</p>
    </a>
    <a href="https://www.loom.com/share/149ff48564434ddba68cb86d3f4b348c">
      <img style="max-width:300px;" src="https://cdn.loom.com/sessions/thumbnails/149ff48564434ddba68cb86d3f4b348c-456dbce584902d27-full-play.gif#t=0.1">
    </a>
</div>

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

The desktop window will open with **Start Recording** and **Stop Recording** buttons. Recorded sessions are saved to `recordings/[session-id]/` with JSON and PNG screenshots.

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

## 📋 What Was Completed vs What Was Descoped

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

## 🔧 Key Technical Decisions and Tradeoffs

### 1. JSON Storage vs SQLite
**Decision**: Use JSON files with session-based directories
**Rationale**: Simpler implementation (~30 min saved), easier debugging, no query requirements
**Trade-off**: No built-in querying, but sufficient for workflow recording use case

### 2. No Screenshots for Keyboard Events
**Decision**: Only capture screenshots for click events, not keypresses
**Rationale**: Performance optimization (each screenshot ~2.2MB), reduces storage by 70%+
**Trade-off**: Less visual evidence for typing, but action descriptions provide context

### 3. RustDesk rdev Fork vs Official
**Decision**: Use `rdev` fork from RustDesk repository
**Rationale**: Official rdev has known macOS crash bug with keyboard events
**Impact**: Stable keyboard monitoring without crashes (AI flagged this proactively)

### 4. Descope Event Log UI
**Decision**: Terminal logging only, no in-app event visualization UI
**Rationale**: 4-hour constraint required focus on core capture functionality
**Trade-off**: JSON file review instead of real-time UI, but sufficient for MVP

### 5. Session-Based Directory Structure
**Decision**: Each recording gets UUID directory with JSON + PNG files
**Rationale**: Co-located data, easy archival/deletion, no database overhead
**Benefit**: Simple file management, works well with version control

### 6. Known Limitation: Retina Display Coordinate Scaling
**Issue**: Window and click crops are offset on Retina displays (full screen works fine)
**Cause**: Event coordinates are logical pixels, screenshot pixels are physical (2x mismatch)
**Decision**: Document limitation rather than rush incomplete fix in final hour
**Fix Path**: Detect display scale factor and multiply coordinates before cropping

### 7. Event Listener Shutdown Limitation
**Issue**: `rdev::listen()` blocks forever, cannot gracefully stop
**Workaround**: Must restart app between recording sessions
**Proper Fix**: Tokio channels for stop signal (~30 min additional work)

---

## 🚀 What I'd Build Next with More Time

### High Priority (Next 2-4 hours)

1. **Fix Retina Display Coordinate Scaling** (~1 hour)
   - Detect display scale factor using macOS APIs
   - Multiply logical coordinates by scale factor before cropping
   - Test on multiple display configurations (1x, 2x, 3x scaling)

2. **In-App Event Review UI** (~2 hours)
   - Timeline view showing all captured events chronologically
   - Thumbnail previews of screenshots for each click
   - Edit event descriptions and action categories
   - Delete/merge events before saving session

3. **Graceful Event Listener Shutdown** (~1 hour)
   - Refactor to use tokio channels for stop signal
   - Allow multiple recording sessions without app restart
   - Better state management for listener lifecycle

### Medium Priority (4-8 hours)

4. **Auto-Group Text Input Actions** (~2 hours)
   - Detect consecutive KeyPress events and group into single "Type" action
   - Example: 5 keypresses → "Typed 'hello'" with character sequence
   - Configurable grouping timeout (e.g., 500ms between keypresses)

5. **Keyboard Shortcut Support** (~1 hour)
   - Global hotkey to start/stop recording (e.g., Cmd+Shift+R)
   - Eliminate need to click UI buttons during workflow capture

6. **Editable Action Annotations** (~3 hours)
   - UI for adding custom titles to events ("Login to dashboard", "Submit form")
   - Rich text descriptions with markdown support
   - Tag events for easier filtering and organization

### Nice to Have (8+ hours)

7. **Export to Automation Scripts** (~6 hours)
   - Generate Selenium/Playwright scripts from JSON
   - Map clicks to element selectors, text input to fill commands
   - Convert FlowTrace sessions into executable automation code

8. **OCR for Screenshot Text Extraction** (~4 hours)
   - Extract visible text from screenshots using Tesseract
   - Enhance event descriptions with detected UI text
   - Enable text-based search across recorded workflows

9. **Compression and Format Options** (~2 hours)
   - JPEG compression option to reduce file size (2.2MB → ~200KB per screenshot)
   - WebP format support for better compression ratios
   - Configurable screenshot quality settings

---

## 🤖 How I Used AI Tools

### Tool Used
**Claude Code (Sonnet 4.5)** via Claude API - Used extensively throughout planning, research, implementation, and documentation phases.

### AI Usage Throughout Development Process

#### **Hour 0: Planning & Research Phase**

**What I Prompted**:
- "For a Tauri app with the following objective and core requirements. Provide a brief technical review (pros & cons) of 3 top possible dependencies/crates needed for this application to complete the objective. Flag known issues. Object: 'objective placed in'. Core Requirement: 'requirements placed in'"
- "Compare rdev official vs RustDesk fork for stability"


**What AI Provided**:
- Provided a list of dependencies/crates needed for this application. Along with providing a realistic time estimates for descoping decisions.
- Identified `rdev` RustDesk fork to avoid known macOS crash bug
- Warned about macOS Accessibility permissions for event monitoring

**Impact**: Avoided 30+ minutes debugging known issues by using vetted fork upfront

#### **Hours 1-3: Implementation Phase**

**What I Prompted**:
- "Generate boilerplate for Rust module capturing screenshots with `screenshots` crate"
- "How to share state between Tauri main thread and rdev listener thread?"
- "Why am I getting 'chrono serde feature not enabled' compile error?"

**What AI Provided**:
- Generated initial code structure for screenshot.rs, types.rs, storage.rs
- Suggested `Arc<Mutex<Option<RecordingSession>>>` pattern for thread-safe shared state
- Instantly diagnosed missing Cargo.toml feature flag: `chrono = { version = "0.4", features = ["serde"] }`

**Impact**: Accelerated implementation by ~2 hours through boilerplate generation and instant bug diagnosis

#### **Throughout: Code Review & Refinement**

**What AI Caught**:
- **Deadlock Risk**: Warned that holding Mutex lock during screenshot capture (slow operation) could block UI thread
- **Performance Issue**: Suggested removing screenshots from keyboard events to reduce storage by 70%+
- **Event Filtering**: Recommended ignoring modifier keys (Shift, Ctrl, Alt) to avoid noise in recordings

**Impact**: Prevented runtime issues and improved UX before encountering problems

#### **Hour 4: Documentation Phase**

**What I Prompted**:
- "Analyze git commit history and create hour-by-hour implementation timeline"
- "Document the Retina display coordinate scaling issue technically"
- "Generate requirements compliance matrix from assignment PDF"

**What AI Provided**:
- Comprehensive implementation log with accurate git timeline
- Technical explanation of logical vs physical pixel coordinate mismatch
- Structured documentation matching assignment deliverable requirements

**Impact**: Professional-grade documentation in <1 hour that clearly communicates decisions and trade-offs

---

### Specific Examples

#### ✅ Where AI Accelerated Me

**Example**: Missing `chrono` serde feature causing compile error

```
Compiler Error: "the trait `Serialize` is not implemented for `DateTime<Utc>`"

My Prompt: "ISSUE: Getting a 'chrono serde error' in the terminal, review the following error message below. Review any logs, if needed, to determine the error and fix."

AI Response (instant): "Add serde feature to chrono in Cargo.toml:
chrono = { version = "0.4", features = ["serde"] }"

Time Saved: ~10 minutes of documentation searching
```

#### ⚠️ Where AI Led Me Astray

**Example**: Incorrect assumption about rdev API

```
AI Initial Code:
let position = event.position; // Assumed unified position field

Reality: rdev separates MouseMove (has position) from ButtonPress (no position)

Fix Required: Track last MouseMove position manually in global state

Time Lost: ~10 minutes debugging + refactoring
```

**Lesson**: Always verify AI assumptions about external crate APIs with actual documentation, especially for forks

#### ⚠️ Optimistic Time Estimates

**Example**: Screenshot implementation

```
AI Estimate: "Window crop with active-win-pos-rs: ~10 minutes"
Actual Time: ~20 minutes (needed to understand coordinate systems and test)

Pattern: AI time estimates were consistently 1.5x-2x lower than reality
```

**Adjustment**: Treat AI estimates as lower bounds, multiply by 1.5x for realistic planning

---

### Collaboration Patterns That Worked Best

1. **Strategic Planning First**: Start with AI-assisted research and descoping before coding
2. **Verify Assumptions**: Compile and test AI-generated code immediately, don't assume it works
3. **Proactive Code Review**: Ask AI to review code for concurrency issues, not just generate it
4. **Leverage for Boilerplate**: Let AI handle repetitive code, focus human effort on architecture decisions

### Key Takeaway

AI tools dramatically accelerate development when used strategically:
- ✅ Research, boilerplate generation, bug diagnosis, documentation
- ⚠️ Verify external API assumptions, multiply time estimates by 1.5x
- ❌ Don't blindly trust without compiler feedback loop

**Effective AI collaboration = Speed + Critical Thinking**

---

## 📚 Additional Documentation

For detailed implementation timeline, bug reports, and AI collaboration analysis:

- **`prompts/00-planning-phase.md`** - Strategic planning and descoping decisions
- **`prompts/01-implementation-log.md`** - Hour-by-hour development log with git timeline
