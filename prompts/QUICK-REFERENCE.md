# FlowTrace: Quick Reference Guide

**⏰ Time Check**: Record your actual start time here: `__________`

---

## 🚀 IMMEDIATE STARTUP CHECKLIST (First 15 Minutes)

### Step 1: Add Dependencies (DO THIS FIRST!)
```bash
cd src-tauri
```

Edit `src-tauri/Cargo.toml` and add to `[dependencies]`:
```toml
rdev = { git = "https://github.com/fufesou/rdev" }
screenshots = "0.8"
chrono = "0.4"
uuid = { version = "1", features = ["v4", "serde"] }
tokio = { version = "1", features = ["full"] }
```

### Step 2: Start Build (Background Process)
```bash
cargo build
# This will take 10-15 minutes. Work on Step 3-5 while waiting.
```

### Step 3: Create File Stubs
```bash
cd src-tauri/src
touch event_monitor.rs screenshot.rs types.rs storage.rs
```

### Step 4: Test rdev Works (Basic Spike)
Add to `event_monitor.rs`:
```rust
use rdev::{listen, Event};

pub fn test_listener() {
    println!("🎯 Starting event listener test...");

    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error);
    }
}

fn callback(event: Event) {
    println!("Event captured: {:?}", event);
}
```

### Step 5: Test macOS Permissions
```bash
cargo run
# If events don't print → macOS permissions issue (see troubleshooting below)
```

---

## 🚨 TROUBLESHOOTING (Copy-Paste Solutions)

### macOS Accessibility Permission Not Working
1. Open **System Settings** → **Privacy & Security** → **Accessibility**
2. Look for your app (might be "flowtrace" or "cargo")
3. Toggle ON
4. **CRITICAL**: Restart the app completely

**If still not working**: Build in release mode
```bash
cargo build --release
./target/release/flowtrace
```

### rdev Crashes on Keypress
- **Symptom**: App quits when you press keyboard
- **Cause**: Using official rdev instead of fork
- **Fix**: Double-check Cargo.toml has `git = "https://github.com/fufesou/rdev"`

### Cargo Build Hangs
- **Symptom**: Stuck on "Compiling..." for > 5 minutes
- **Fix**: Kill and restart with verbose output
```bash
cargo build -vv
```

### Screenshots Crate Not Found
- **Fix**: Clear cargo cache and rebuild
```bash
cargo clean
cargo build
```

---

## 📦 DATA STRUCTURE REFERENCE

### Event JSON Schema (Target Format)
```json
{
  "session_id": "uuid-v4",
  "events": [
    {
      "id": "uuid-v4",
      "type": "click",
      "timestamp": "2026-01-31T23:15:30Z",
      "position": {
        "x": 1024,
        "y": 768
      },
      "screenshot_path": "./recordings/session-id/event-id.png"
    }
  ]
}
```

### Rust Event Struct (Suggested)
```rust
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordingSession {
    pub session_id: String,
    pub started_at: DateTime<Utc>,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub position: Option<Position>,
    pub screenshot_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventType {
    Click,
    // KeyPress, // Add later if time permits
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
```

---

## ⏰ TIME CHECKPOINT QUESTIONS

### End of Hour 1 (Stop and Ask):
- [ ] Can I detect click events reliably?
- [ ] Can I capture a screenshot manually?
- [ ] Do I understand how rdev threading works?
- **If NO to any**: Spend Hour 2 fixing, not adding features.

### End of Hour 2 (Stop and Ask):
- [ ] Can I record 5+ clicks with screenshots?
- [ ] Is data saving to JSON?
- [ ] Can I start/stop recording from UI?
- **If NO to any**: Do NOT add keyboard or window crops. Debug first.

### End of Hour 3 (Stop and Ask):
- [ ] Would I be embarrassed to submit this code?
- [ ] Is the README clear about what works/doesn't?
- [ ] Have I documented my AI collaboration?
- **If NO to any**: Hour 4 is for docs, not more coding.

---

## 🎯 MVP ACCEPTANCE CRITERIA (The Bar)

**Can you demo this flow?**
1. Open app
2. Click "Start Recording" button
3. Click around screen 5 times
4. Click "Stop Recording"
5. Show generated `events.json` file with timestamps
6. Show 5 `.png` screenshot files in recordings folder

**If YES**: MVP achieved. Focus on documentation.
**If NO**: You have a bug, not a feature problem.

---

## 📝 DOCUMENTATION CHECKLIST (Hour 3-4)

### README.md Must Include:
- [ ] **Setup Instructions** (step-by-step, assume beginner)
- [ ] **macOS Permission Requirements** (with screenshots if possible)
- [ ] **How to Run** (exact commands)
- [ ] **What Was Built** (be specific: "Records clicks with full-screen screenshots")
- [ ] **What Was Descoped** (honest: "Keyboard events cut due to time")
- [ ] **Known Limitations** (e.g., "macOS only tested", "No error handling for...")
- [ ] **AI Tool Usage Summary** (2-3 sentences)

### /prompts Folder Must Include:
- [x] `00-planning-phase.md` (already created)
- [ ] `01-implementation-log.md` (create during Hour 0-2)
  - What worked immediately
  - What took longer than expected
  - Where AI suggestions were wrong
  - Where you deviated from the plan (and why)

### Code Comments Should Flag:
- [ ] "// AI-suggested approach: [brief description]"
- [ ] "// Deviated from AI suggestion because: [reason]"
- [ ] "// Known issue: [description] - descoped due to time"

---

## 🔄 IF THINGS GO WRONG (Pivot Decision Tree)

### Hour 1: Can't Get rdev Working
→ **Pivot**: Use tauri-plugin-global-shortcut for hotkey-only demo
→ **Trade-off**: Less impressive, but shows you can adapt

### Hour 2: Threading Issues with Tauri
→ **Pivot**: Make it a manual "Capture Now" button instead of auto-detection
→ **Trade-off**: Not true "monitoring", but proves screenshot integration works

### Hour 3: Nothing Works, Panic Mode
→ **Pivot**: Simplify to screenshot tool only (no event detection)
→ **Focus**: Make README exceptional, document what you learned

**Key Philosophy**: Broken code with great docs > buggy code with no explanation.

---

## 💡 AI COLLABORATION TIPS (For Implementation Phase)

### Good Prompts to Use During Coding:
- "I'm getting this Rust compiler error: [paste error]. What's wrong?"
- "How do I spawn a tokio thread in Tauri that communicates back to the main window?"
- "Review this code for common Tauri + rdev pitfalls: [paste code]"

### Prompts to AVOID (Time Wasters):
- "Write the complete implementation of..." (you won't understand the bugs)
- "Make this code production-ready" (scope creep)
- "Add error handling for all edge cases" (perfect is the enemy of done)

### When to STOP Using AI:
- When debugging a bug for > 20 minutes → rubber duck it yourself
- When AI keeps suggesting the same broken solution → try different approach
- When you're tempted to add "just one more feature" → you're off track

---

## 📊 FINAL SUBMISSION CHECKLIST

**Before you submit, verify:**
- [ ] Code compiles without warnings (`cargo build --release`)
- [ ] App runs and can record at least 3 clicks
- [ ] README has setup instructions (test by following your own steps)
- [ ] `/prompts` folder exists with at least 2 markdown files
- [ ] No sensitive data in recordings (don't accidentally include personal screenshots)
- [ ] Git history shows progression (commit after each major milestone)

**Commit Messages to Use:**
```bash
git commit -m "Add event monitoring with rdev"
git commit -m "Integrate screenshot capture on click events"
git commit -m "Implement JSON storage for recording sessions"
git commit -m "Add start/stop recording UI controls"
git commit -m "Document AI collaboration process and decisions"
```

---

**Good luck! Remember: Done is better than perfect. Document everything.**
