# Wisper 🎙️

Personal voice dictation app for Windows 11. Hold a hotkey to record your voice, release to get polished text auto-pasted into any focused app — powered by Google Gemini.

**Free. Local. Private. No accounts, no telemetry.**

## How It Works

1. **Hold** the push-to-talk key (default: Right Ctrl)
2. **Speak** — audio is captured from your microphone
3. **Release** — audio is sent to the Gemini API
4. **Done** — cleaned text is pasted into your currently focused app

The app runs as a system tray icon with no visible window.

## Setup

### Prerequisites

- **Windows 11**
- **Node.js** 18+ — [nodejs.org](https://nodejs.org/)
- **Rust** (stable) — [rustup.rs](https://rustup.rs/)
- **WebView2** — pre-installed on Windows 11
- A **Google Gemini API key** (free tier) — [aistudio.google.com/apikey](https://aistudio.google.com/apikey)

### Install & Run

```bash
# Clone / extract the project, then:
cd wisper

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev
```

The first build will take a few minutes as Rust compiles all dependencies.

### Configure Your API Key

1. Once the app starts, look for the **Wisper icon** in the Windows system tray (bottom-right, may be in the overflow area ▲)
2. **Left-click** or **right-click → Settings** to open the settings window
3. Paste your **Gemini API key** and click **Save**
4. Close the settings window — you're ready to dictate!

### Windows Permissions

- **Microphone access**: Windows should prompt you to allow mic access the first time you record. If not, go to **Settings → Privacy → Microphone** and ensure access is enabled.
- **No admin privileges required** — the app runs as a normal user process.

## Features

| Feature | Details |
|---|---|
| **Push-to-Talk** | Hold Right Ctrl (configurable) to record |
| **Gemini AI** | Transcribes and polishes your speech |
| **Auto-Paste** | Result is pasted via Ctrl+V into the active app |
| **System Tray** | No taskbar clutter, lives in the tray |
| **Overlay** | Small floating indicator shows recording/processing state |
| **Polish Styles** | Formal, Casual, or Raw transcription |
| **Local Storage** | API key stored locally via tauri-plugin-store |

## Hotkey Options

You can change the push-to-talk key in Settings. Supported keys:

Right Ctrl (default), Left Ctrl, Right Alt, Left Alt, Right Shift, Left Shift,
Caps Lock, F1–F12, Scroll Lock, Pause, Insert

## Project Structure

```
wisper/
├── src/                    # React frontend
│   ├── main.tsx            # Entry point (routes to Settings or Overlay)
│   ├── SettingsApp.tsx     # Settings window UI
│   ├── OverlayApp.tsx      # Floating recording overlay
│   ├── index.css           # Settings styles
│   └── overlay.css         # Overlay styles
├── src-tauri/              # Rust backend
│   ├── Cargo.toml          # Rust dependencies
│   ├── tauri.conf.json     # Tauri configuration
│   ├── capabilities/       # Permission definitions
│   └── src/
│       ├── main.rs         # Windows entry point
│       ├── lib.rs          # App setup, tray, shortcuts, commands
│       ├── audio.rs        # Microphone capture via cpal
│       ├── gemini.rs       # Gemini API client
│       └── paste.rs        # Clipboard + Ctrl+V simulation
├── package.json
├── vite.config.ts
└── README.md
```

## Tech Stack

- **Tauri 2** — Rust backend + WebView frontend
- **React 18** — Settings & overlay UI
- **cpal** — Cross-platform audio capture
- **enigo** — Keystroke simulation (Ctrl+V paste)
- **arboard** — Clipboard access
- **hound** — WAV encoding
- **reqwest** — HTTP client for Gemini API
- **tauri-plugin-global-shortcut** — System-wide hotkey
- **tauri-plugin-store** — Local settings persistence

## Troubleshooting

### "API key not set"
Open Settings from the tray icon and paste your Gemini API key.

### Hotkey doesn't work
- Make sure no other app is using the same key globally
- Try a different key in Settings (e.g., F9, Right Alt)
- Restart the app after changing the hotkey

### No audio captured
- Check Windows mic permissions: Settings → Privacy → Microphone
- Ensure your mic is set as the default input device

### Paste doesn't work in some apps
Some apps (admin-elevated terminals, certain games) block simulated input.
Try targeting a regular text editor or browser.

## License

MIT — Personal use, no restrictions.
