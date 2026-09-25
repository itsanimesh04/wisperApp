use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::thread;
use std::time::Duration;

/// Copy `text` to the system clipboard, then simulate Ctrl+V to paste
/// into whatever application is currently focused.
pub fn paste_text(text: &str) -> Result<(), String> {
    // Write text to clipboard
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    // Small delay to let the clipboard update propagate on Windows
    thread::sleep(Duration::from_millis(80));

    // Simulate Ctrl+V
    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Failed to init enigo: {}", e))?;

    enigo
        .key(Key::Control, Press)
        .map_err(|e| format!("Key press error: {}", e))?;
    enigo
        .key(Key::Unicode('v'), Click)
        .map_err(|e| format!("Key click error: {}", e))?;
    enigo
        .key(Key::Control, Release)
        .map_err(|e| format!("Key release error: {}", e))?;

    Ok(())
}
