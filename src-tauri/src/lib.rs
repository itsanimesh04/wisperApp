mod audio;
mod gemini;
mod paste;

use audio::AudioState;
use gemini::PolishStyle;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

/// Application settings persisted via tauri-plugin-store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub api_key: String,
    pub hotkey: String,
    pub polish_style: String, // "formal", "casual", "raw"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            hotkey: "RightControl".to_string(),
            polish_style: "casual".to_string(),
        }
    }
}

/// Shared app state accessible from commands and shortcuts.
pub struct AppState {
    pub audio: AudioState,
    pub is_recording: Arc<Mutex<bool>>,
}

// ----- Tauri Commands -----

#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    load_settings(&app)
}

#[tauri::command]
async fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    persist_settings(&app, &settings)?;

    // Re-register the hotkey with the new key
    register_hotkey(&app, &settings.hotkey)?;

    Ok(())
}

#[tauri::command]
async fn start_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    do_start_recording(&app, &state.audio, &state.is_recording)
}

#[tauri::command]
async fn stop_recording(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    do_stop_recording_and_transcribe(app, &state.audio, &state.is_recording).await
}

// ----- Core Recording Logic -----

fn do_start_recording(
    app: &tauri::AppHandle,
    audio: &AudioState,
    is_recording: &Arc<Mutex<bool>>,
) -> Result<(), String> {
    let mut is_rec = is_recording.lock().map_err(|e| e.to_string())?;
    if *is_rec {
        return Ok(());
    }
    audio.start_capture()?;
    *is_rec = true;

    // Show overlay
    show_overlay(app);
    let _ = app.emit("recording-started", ());
    Ok(())
}

async fn do_stop_recording_and_transcribe(
    app: tauri::AppHandle,
    audio: &AudioState,
    is_recording: &Arc<Mutex<bool>>,
) -> Result<String, String> {
    // Stop capture
    let wav_bytes = {
        let mut is_rec = is_recording.lock().map_err(|e| e.to_string())?;
        if !*is_rec {
            return Err("Not recording".into());
        }
        let bytes = audio.stop_capture()?;
        *is_rec = false;
        bytes
    };

    let _ = app.emit("transcription-started", ());

    let settings = load_settings(&app)?;
    if settings.api_key.is_empty() {
        let _ = app.emit("transcription-error", "API key not set");
        hide_overlay_delayed(&app);
        return Err("API key not set. Open Settings to configure.".into());
    }

    let style = match settings.polish_style.as_str() {
        "formal" => PolishStyle::Formal,
        "raw" => PolishStyle::Raw,
        _ => PolishStyle::Casual,
    };

    match gemini::transcribe(&settings.api_key, &wav_bytes, &style).await {
        Ok(text) => {
            if !text.is_empty() {
                if let Err(e) = paste::paste_text(&text) {
                    eprintln!("Paste failed: {}", e);
                    let _ = app.emit("transcription-error", e.to_string());
                }
            }
            let _ = app.emit("transcription-complete", &text);
            hide_overlay_delayed(&app);
            Ok(text)
        }
        Err(e) => {
            eprintln!("Transcription failed: {}", e);
            let _ = app.emit("transcription-error", e.clone());
            hide_overlay_delayed(&app);
            Err(e)
        }
    }
}

// ----- Helper Functions -----

fn load_settings(app: &tauri::AppHandle) -> Result<AppSettings, String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Store error: {}", e))?;

    let api_key = store
        .get("api_key")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    let hotkey = store
        .get("hotkey")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "RightControl".to_string());
    let polish_style = store
        .get("polish_style")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "casual".to_string());

    Ok(AppSettings {
        api_key,
        hotkey,
        polish_style,
    })
}

fn persist_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    let store = app
        .store("settings.json")
        .map_err(|e| format!("Store error: {}", e))?;

    store.set("api_key", serde_json::json!(&settings.api_key));
    store.set("hotkey", serde_json::json!(&settings.hotkey));
    store.set(
        "polish_style",
        serde_json::json!(&settings.polish_style),
    );
    store
        .save()
        .map_err(|e| format!("Store save error: {}", e))?;

    Ok(())
}

fn show_overlay(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.show();
        // Do NOT set_focus() — overlay must not steal focus from the user's active app
    }
}

fn hide_overlay_delayed(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        if let Some(win) = app_handle.get_webview_window("overlay") {
            let _ = win.hide();
        }
    });
}

fn parse_hotkey(key_name: &str) -> Option<Shortcut> {
    let code = match key_name {
        "RightControl" => Code::ControlRight,
        "LeftControl" => Code::ControlLeft,
        "RightAlt" => Code::AltRight,
        "LeftAlt" => Code::AltLeft,
        "RightShift" => Code::ShiftRight,
        "LeftShift" => Code::ShiftLeft,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "CapsLock" => Code::CapsLock,
        "ScrollLock" => Code::ScrollLock,
        "Pause" => Code::Pause,
        "Insert" => Code::Insert,
        _ => return None,
    };
    Some(Shortcut::new(None, code))
}

fn register_hotkey(app: &tauri::AppHandle, key_name: &str) -> Result<(), String> {
    let shortcut_manager = app.global_shortcut();

    // Unregister all existing shortcuts first
    let _ = shortcut_manager.unregister_all();

    let shortcut =
        parse_hotkey(key_name).ok_or_else(|| format!("Unknown hotkey: {}", key_name))?;

    shortcut_manager
        .register(shortcut)
        .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    Ok(())
}

// ----- Shortcut handler used by the plugin builder -----

fn handle_shortcut(app: &tauri::AppHandle, _shortcut: &Shortcut, event: ShortcutState) {
    let app_clone = app.clone();

    match event {
        ShortcutState::Pressed => {
            tauri::async_runtime::spawn(async move {
                let state = app_clone.state::<AppState>();
                if let Err(e) =
                    do_start_recording(&app_clone, &state.audio, &state.is_recording)
                {
                    eprintln!("Failed to start capture: {}", e);
                    let _ = app_clone.emit("transcription-error", e);
                }
            });
        }
        ShortcutState::Released => {
            tauri::async_runtime::spawn(async move {
                let state = app_clone.state::<AppState>();
                // Only proceed if actually recording
                {
                    let is_rec = state.is_recording.lock().unwrap();
                    if !*is_rec {
                        return;
                    }
                }
                if let Err(e) = do_stop_recording_and_transcribe(
                    app_clone.clone(),
                    &state.audio,
                    &state.is_recording,
                )
                .await
                {
                    eprintln!("Stop/transcribe failed: {}", e);
                    // Error events already emitted inside the function
                }
            });
        }
    }
}

// ----- Entry Point -----

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    handle_shortcut(app, shortcut, event.state);
                })
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            audio: AudioState::new(),
            is_recording: Arc::new(Mutex::new(false)),
        })
        .setup(|app| {
            // --- System Tray ---
            let settings_i =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("Wisper — Voice Dictation")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "settings" => {
                        if let Some(win) = app.get_webview_window("settings") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            let app = tray.app_handle();
                            if let Some(win) = app.get_webview_window("settings") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // --- Register default hotkey ---
            let settings = load_settings(&app.handle()).unwrap_or_default();
            if let Err(e) = register_hotkey(&app.handle(), &settings.hotkey) {
                eprintln!("Failed to register default hotkey: {}", e);
                // Try fallback
                let _ = register_hotkey(&app.handle(), "RightControl");
            }

            // --- Handle window close → hide instead ---
            let app_handle = app.handle().clone();
            if let Some(win) = app.get_webview_window("settings") {
                win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = app_handle.get_webview_window("settings") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            start_recording,
            stop_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running wisper");
}
