import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Settings {
  api_key: string;
  hotkey: string;
  polish_style: string;
}

const HOTKEY_OPTIONS = [
  { value: "RightControl", label: "Right Ctrl" },
  { value: "LeftControl", label: "Left Ctrl" },
  { value: "RightAlt", label: "Right Alt" },
  { value: "LeftAlt", label: "Left Alt" },
  { value: "RightShift", label: "Right Shift" },
  { value: "LeftShift", label: "Left Shift" },
  { value: "CapsLock", label: "Caps Lock" },
  { value: "F1", label: "F1" },
  { value: "F2", label: "F2" },
  { value: "F3", label: "F3" },
  { value: "F4", label: "F4" },
  { value: "F5", label: "F5" },
  { value: "F6", label: "F6" },
  { value: "F7", label: "F7" },
  { value: "F8", label: "F8" },
  { value: "F9", label: "F9" },
  { value: "F10", label: "F10" },
  { value: "F11", label: "F11" },
  { value: "F12", label: "F12" },
  { value: "ScrollLock", label: "Scroll Lock" },
  { value: "Pause", label: "Pause" },
  { value: "Insert", label: "Insert" },
];

export default function SettingsApp() {
  const [settings, setSettings] = useState<Settings>({
    api_key: "",
    hotkey: "RightControl",
    polish_style: "casual",
  });
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);
  const [showKey, setShowKey] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then((s) => {
        setSettings(s);
        setLoading(false);
      })
      .catch((e) => {
        console.error(e);
        setLoading(false);
      });
  }, []);

  const handleSave = async () => {
    setError("");
    setSaved(false);
    try {
      await invoke("save_settings", { settings });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e: any) {
      setError(String(e));
    }
  };

  if (loading) {
    return (
      <div className="app-container">
        <div className="loading">Loading...</div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <header className="app-header">
        <div className="logo-area">
          <div className="logo-icon">🎙️</div>
          <h1>Wisper</h1>
        </div>
        <p className="subtitle">Voice Dictation for Windows</p>
      </header>

      <main className="settings-form">
        {/* API Key */}
        <div className="form-group">
          <label htmlFor="api-key">
            <span className="label-icon">🔑</span>
            Gemini API Key
          </label>
          <div className="input-row">
            <input
              id="api-key"
              type={showKey ? "text" : "password"}
              value={settings.api_key}
              onChange={(e) =>
                setSettings({ ...settings, api_key: e.target.value })
              }
              placeholder="Paste your API key here"
              spellCheck={false}
              autoComplete="off"
            />
            <button
              className="toggle-btn"
              onClick={() => setShowKey(!showKey)}
              title={showKey ? "Hide" : "Show"}
              type="button"
            >
              {showKey ? "🙈" : "👁️"}
            </button>
          </div>
          <span className="help-text">
            Get a free key at{" "}
            <a
              href="https://aistudio.google.com/apikey"
              target="_blank"
              rel="noopener"
            >
              aistudio.google.com
            </a>
          </span>
        </div>

        {/* Hotkey */}
        <div className="form-group">
          <label htmlFor="hotkey">
            <span className="label-icon">⌨️</span>
            Push-to-Talk Key
          </label>
          <select
            id="hotkey"
            value={settings.hotkey}
            onChange={(e) =>
              setSettings({ ...settings, hotkey: e.target.value })
            }
          >
            {HOTKEY_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
          <span className="help-text">
            Hold this key to record, release to transcribe
          </span>
        </div>

        {/* Polish Style */}
        <div className="form-group">
          <label>
            <span className="label-icon">✨</span>
            Polish Style
          </label>
          <div className="radio-group">
            {[
              {
                value: "formal",
                label: "Formal",
                desc: "Professional tone, clean grammar",
              },
              {
                value: "casual",
                label: "Casual",
                desc: "Natural tone, cleaned up",
              },
              {
                value: "raw",
                label: "Raw",
                desc: "Exact transcription, fillers kept",
              },
            ].map((opt) => (
              <label
                key={opt.value}
                className={`radio-card ${
                  settings.polish_style === opt.value ? "selected" : ""
                }`}
              >
                <input
                  type="radio"
                  name="polish_style"
                  value={opt.value}
                  checked={settings.polish_style === opt.value}
                  onChange={(e) =>
                    setSettings({ ...settings, polish_style: e.target.value })
                  }
                />
                <div className="radio-content">
                  <span className="radio-label">{opt.label}</span>
                  <span className="radio-desc">{opt.desc}</span>
                </div>
              </label>
            ))}
          </div>
        </div>

        {/* Save Button */}
        <button className="save-btn" onClick={handleSave}>
          {saved ? "✅ Saved!" : "Save Settings"}
        </button>

        {error && <div className="error-msg">{error}</div>}
      </main>

      <footer className="app-footer">
        <span>Wisper v0.1.0 — Local & Private</span>
      </footer>
    </div>
  );
}
