<template>
  <main class="container">
    <h1>🎥 FlowTrace</h1>
    <p style="color: #666;">Workflow Recording Application</p>

    <div style="margin: 2rem 0;">
      <button
        v-if="!isRecording"
        @click="startRecording"
        style="margin: 1rem; padding: 1em 2em; font-size: 1.2em; background-color: #4CAF50; color: white; border: none;"
      >
        ⏺️ Start Recording
      </button>

      <button
        v-else
        @click="stopRecording"
        style="margin: 1rem; padding: 1em 2em; font-size: 1.2em; background-color: #f44336; color: white; border: none;"
      >
        ⏹️ Stop Recording
      </button>
    </div>

    <p><strong>{{ recordingStatus }}</strong></p>

    <div v-if="isRecording" style="margin: 1rem; padding: 1rem; background-color: #fff3cd; border-radius: 8px;">
      <p style="margin: 0; color: #856404;">
        🔴 <strong>Recording in progress...</strong><br/>
        🖱️ Capturing clicks with screenshots<br/>
        ⌨️ Capturing keyboard events<br/>
        <small>(Check terminal for event logs)</small>
      </p>
    </div>

    <hr style="margin: 3rem 0; opacity: 0.3;" />

    <details style="margin: 2rem 0;">
      <summary style="cursor: pointer; font-weight: bold;">🧪 Test Functions (Spike Testing)</summary>

      <div style="margin-top: 1rem;">
        <h3>🎯 Event Listener Test</h3>
        <button @click="startListener" style="margin: 0.5rem;">
          Start Event Listener
        </button>
        <p><strong>{{ listenerStatus }}</strong></p>
        <p style="font-size: 0.9em; color: #666;">
          After clicking, check your terminal for event logs.
        </p>

        <hr style="margin: 2rem 0; opacity: 0.3;" />

        <h3>📸 Screenshot Test</h3>
        <button @click="captureScreenshot" style="margin: 0.5rem;">
          Capture Screenshot
        </button>
        <p><strong>{{ screenshotStatus }}</strong></p>
        <p style="font-size: 0.9em; color: #666;">
          Captures full screen and saves to ./recordings/ folder.
        </p>
      </div>
    </details>
  </main>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const listenerStatus = ref("");
const screenshotStatus = ref("");
const recordingStatus = ref("");
const isRecording = ref(false);

async function startListener() {
  try {
    const result = await invoke("start_event_listener");

    listenerStatus.value = `✅ ${result}`;

    console.log("Event listener started! Check your terminal for event logs.");
  } catch (error) {
    listenerStatus.value = `❌ Error: ${error}`;

    console.error("Failed to start listener:", error);
  }
}

async function captureScreenshot() {
  screenshotStatus.value = "📸 Capturing...";

  try {
    const result = await invoke("capture_screenshot");

    screenshotStatus.value = `✅ ${result}`;

    console.log("Screenshot captured:", result);
  } catch (error) {
    screenshotStatus.value = `❌ Error: ${error}`;

    console.error("Failed to capture screenshot:", error);
  }
}

async function startRecording() {
  recordingStatus.value = "🎬 Starting recording...";

  try {
    const result = await invoke("start_recording");

    recordingStatus.value = `✅ Recording started!`;
    isRecording.value = true;

    console.log("Recording started:", result);
  } catch (error) {
    recordingStatus.value = `❌ Error: ${error}`;
    isRecording.value = false;

    console.error("Failed to start recording:", error);
  }
}

async function stopRecording() {
  recordingStatus.value = "⏹️ Stopping recording...";

  try {
    const result = await invoke("stop_recording");

    recordingStatus.value = `✅ ${result}`;
    isRecording.value = false;

    console.log("Recording stopped:", result);
  } catch (error) {
    recordingStatus.value = `❌ Error: ${error}`;

    console.error("Failed to stop recording:", error);
  }
}
</script>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>