# FlowTrace: Planning Phase (Hour 0)

**Date**: 2026-01-31
**Phase**: Strategic Planning & Technical Research
**Time Budget**: 4 hours total for assignment

---

## 📋 ASSIGNMENT CONTEXT

**Project**: Tauri desktop app for workflow recording
**Evaluation Focus**: AI tool fluency, prioritization under constraints, code quality at speed
**Key Constraint**: 4-hour strict time limit

### Requirements Summary

**MUST HAVE** (Core Features):
- Tauri desktop application structure
- Frontend button to start/stop workflow recording
- Rust backend monitoring global clicks and keyboard events
- Capture 3 screenshots per event (full screen, window crop, click crop)
- Store events locally

**SHOULD HAVE** (Value-Add):
- Parse actions into discernible steps
- Annotate steps with text titles/descriptions
- Classify action types (click, type, wait, assert)

**NICE TO HAVE** (Polish):
- Auto-group consecutive text inputs into single action
- Keyboard shortcut support for start/stop recording

---

## 🤖 AI COLLABORATION APPROACH

### Initial Prompts to AI

**Prompt 1: Technical Research (Pre-Planning)**
```
Question 1: Tech Stack for Event Monitoring
"For a Tauri desktop app that needs to capture global clicks
and keyboard events, what are the main Rust crates? Flag any known issues"

[Additional questions 2-4 about screenshots, window detection, prioritization]
```

**AI Response Summary**:
- **Event Monitoring**: Recommended `rdev` crate (RustDesk fork)
- **Critical Warning**: Official rdev has macOS crash bug, must use fork
- **Screenshot Capture**: `screenshots` crate for MVP, `xcap` for window crops
- **Window Detection**: `active-win-pos-rs` but requires macOS Accessibility permissions

**Prompt 2: Strategic Planning**
```
[Full assignment context + meta-instructions for documented collaboration]

Requesting:
1. Prioritized implementation plan
2. Technical architecture recommendations
3. Hour-by-hour timeline
4. Risk assessment
```

---

## 🎯 STRATEGIC DECISIONS MADE

### Decision 1: Descope to Realistic MVP

**Problem**: Original requirements require ~7 hours of work
**AI Analysis**: Must-haves alone = 5.5 hours (over budget)
**Decision**: Cut from MVP:
- ❌ Keyboard event monitoring (do clicks only first)
- ❌ Window crop screenshots (full screen only)
- ❌ Click crop screenshots (300x300px)
- ❌ Event log UI (console/file logging sufficient)

**Rationale**: Better to have working click recorder + good docs than half-finished everything.
**Time Savings**: 3.5 hours → achievable in 4-hour window with buffer for documentation

**AI Contribution**: Time estimates, dependency analysis, risk flagging
**Human Decision**: Final call on what to cut based on assignment evaluation criteria

---

### Decision 2: Use Battle-Tested rdev Fork

**Problem**: Official rdev crashes on macOS with key press events
**AI Discovery**: RustDesk maintains stable fork
**Decision**: Use `rdev = { git = "https://github.com/fufesou/rdev" }`

**Confidence Level**: HIGH (AI flagged this proactively)
**Verification Status**: Will test during Hour 0-1 spike
**Rollback Plan**: If fork doesn't work, pivot to mouse-only monitoring with official rdev

---

### Decision 3: JSON Storage Over SQLite

**Options Evaluated**:
- Option A: SQLite with `rusqlite` crate (+1 hour setup, queryable)
- Option B: JSON files with `serde_json` (+0.5 hour, simpler)

**Decision**: JSON files
**Rationale**: Assignment doesn't require querying, simpler debugging, faster implementation

**AI Contribution**: Time estimates for both approaches
**Human Decision**: Trade complexity for speed given time constraint

---

### Decision 4: Reserve Hour 3-4 for Documentation

**Problem**: AI tool usage documentation is "optional but strongly recommended"
**AI Analysis**: This is the differentiating factor in evaluation
**Decision**: Non-negotiable 1-hour block for:
- README with setup instructions (including macOS permissions!)
- `/prompts` folder with decision logs
- Code comments explaining AI-assisted vs human-refined sections
- Known limitations documentation

**Rationale**: "Code quality at speed" + "technical communication" are explicit criteria. A working demo with great docs beats buggy feature-complete submission.

---

## 🏗️ TECHNICAL ARCHITECTURE

### Stack (Pre-Scaffolded)
- **Frontend**: Vue 3 + TypeScript + Vite ✅
- **Backend**: Tauri 2 + Rust ✅
- **Storage**: JSON files (decided above)

### Critical Dependencies to Add

```toml
[dependencies]
# Event monitoring (HIGH PRIORITY)
rdev = { git = "https://github.com/fufesou/rdev" }

# Screenshot capture (HIGH PRIORITY)
screenshots = "0.8"

# Utilities (MEDIUM PRIORITY)
chrono = "0.4"          # Timestamps
uuid = { version = "1", features = ["v4", "serde"] }  # Event IDs
```

**AI Warning Noted**: Do NOT add `xcap` or `active-win-pos-rs` until MVP works. These are scope creep risks.

### File Structure Plan

```
src-tauri/src/
├── lib.rs              # Entry point (existing)
├── event_monitor.rs    # rdev integration [COMPLEX - main risk area]
├── screenshot.rs       # screenshots crate wrapper [MODERATE]
├── types.rs            # Event data structures [SIMPLE]
└── storage.rs          # JSON file I/O [SIMPLE]
```

**Dependency Map** (Identified Blockers):
```
Start Button → Event Monitor Thread → Click Event → Screenshot → Save JSON
                    ⚠️ CRITICAL PATH
```

**AI Prediction**: Event monitor thread communication with Tauri will be the hardest part. Plan 1+ hour for debugging.

---

## ⏰ HOUR-BY-HOUR TIMELINE

### Hour 0-1: Foundation (Setup + Spikes)
- Add Rust dependencies (Cargo.toml)
- Run `cargo build` (10+ min first time - work on frontend during this)
- Spike `event_monitor.rs` with println! statements
- Spike `screenshot.rs` with manual trigger
- **Checkpoint**: Can detect clicks? Can save screenshots?

### Hour 1-2: Core Integration
- Event → Screenshot pipeline
- Storage layer (events.json)
- Start/Stop recording Tauri commands
- **Checkpoint**: Can record 5+ clicks with screenshots?

### Hour 2-3: Stabilize or Extend (Decision Point)
- **If MVP shaky**: Debug threading, error handling
- **If MVP solid**: Add keyboard monitoring (stretch goal)
- **Checkpoint**: Demo-ready MVP?

### Hour 3-4: Documentation & Submission
- Create `/prompts` entries (this file + implementation log)
- Write comprehensive README
- Code cleanup + formatting
- Final testing
- **Checkpoint**: Submission package ready

---

## 🚨 IDENTIFIED RISKS

### High-Probability Issues (AI Predictions)

1. **macOS Accessibility Permissions** (Likelihood: 90%, Impact: 30 min)
   - App won't capture events until manual permission grant
   - **Mitigation**: Document in README prominently, add permission check button

2. **rdev Thread Blocking** (Likelihood: 70%, Impact: 45 min)
   - `rdev::listen()` blocks the calling thread
   - **Mitigation**: Spawn tokio thread, use channels for Tauri communication

3. **Cargo Build Times** (Likelihood: 100%, Impact: 15 min cumulative)
   - First build: 10-15 min
   - **Mitigation**: Start build immediately, context switch to frontend work

4. **Screenshot Timing Race Conditions** (Likelihood: 40%, Impact: 20 min)
   - Screenshot triggers before window update
   - **Mitigation**: Add 100ms delay or retry logic

**AI Contribution**: Risk identification based on common Tauri + rdev pitfalls
**Verification Plan**: Test each risk scenario during Hour 1 spikes

---

## 📊 SUCCESS METRICS (Self-Evaluation)

**Minimum Viable Success**:
- [ ] Tauri app runs on macOS
- [ ] "Start Recording" button works
- [ ] Records at least 5 click events with full-screen screenshots
- [ ] Events saved to JSON file with timestamps
- [ ] README explains setup (including permissions)
- [ ] `/prompts` folder documents AI collaboration process

**Stretch Success**:
- [ ] Keyboard events captured
- [ ] Window crop screenshots
- [ ] Action classification logic

**Documentation Success** (Key Differentiator):
- [ ] Clear explanation of what AI suggested vs what human decided
- [ ] Honest account of where AI led astray (if applicable)
- [ ] Time breakdown (what took longer than expected)
- [ ] Technical trade-offs documented with rationale

---

## 🔄 WHAT HAPPENS NEXT

**Immediate Next Steps** (To Be Logged in `01-implementation-log.md`):
1. Add dependencies to `Cargo.toml`
2. Start `cargo build` (background process)
3. Create `src-tauri/src/event_monitor.rs` stub
4. Test rdev basic event listening
5. Document any deviations from this plan with reasoning

**How This Document Will Be Used**:
- Reference point for time-boxing decisions
- Evidence of planning before coding
- Demonstration of AI-assisted prioritization
- Context for evaluators on what was cut and why

---

## 💭 META-NOTES (Human Reflections)

**On AI Collaboration**:
- AI provided time estimates (take with grain of salt - will verify in practice)
- AI flagged macOS permission issue proactively (HIGH VALUE)
- AI descoping recommendations aligned with my intuition (confidence builder)
- AI couldn't know my Rust proficiency → time estimates may be off

**On Assignment Strategy**:
- Treating this as "production under constraints" not "hackathon quality"
- Documentation time is non-negotiable (this IS the differentiator)
- Better to cut features than to submit messy code with no explanation

**Questions Still Unresolved**:
- Will rdev fork actually work on my macOS version? (Unknown until Hour 0 test)
- Is screenshots crate fast enough for real-time capture? (Unknown)
- Can I get Tauri thread communication right in < 1 hour? (Will find out)

---

**Status**: ✅ Planning Complete, Ready to Implement
**Next Document**: `01-implementation-log.md` (to be created during Hour 0-1)
