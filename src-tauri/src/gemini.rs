use base64::Engine;
use serde::Deserialize;

/// Polish style for transcription.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PolishStyle {
    Formal,
    Casual,
    Raw,
}

impl Default for PolishStyle {
    fn default() -> Self {
        Self::Casual
    }
}

impl PolishStyle {
    pub fn system_prompt(&self) -> &str {
        match self {
            PolishStyle::Formal => {
                "You are a transcription assistant. Transcribe the following audio. \
                 Remove all filler words (um, uh, like, you know), false starts, \
                 and repetitions. Fix grammar, punctuation, and capitalization. \
                 Use a formal, professional tone. \
                 Return ONLY the final cleaned text. No commentary, no explanations, \
                 no quotation marks wrapping the output."
            }
            PolishStyle::Casual => {
                "You are a transcription assistant. Transcribe the following audio. \
                 Remove filler words (um, uh, like, you know), false starts, \
                 and repetitions. Fix grammar and punctuation but keep the tone \
                 casual and conversational. \
                 Return ONLY the final cleaned text. No commentary, no explanations, \
                 no quotation marks wrapping the output."
            }
            PolishStyle::Raw => {
                "You are a transcription assistant. Transcribe the following audio \
                 exactly as spoken, preserving all words including fillers. \
                 Add punctuation and capitalization only. \
                 Return ONLY the transcription text. No commentary, no explanations, \
                 no quotation marks wrapping the output."
            }
        }
    }
}

// --- Gemini API response types ---

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Option<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    parts: Option<Vec<Part>>,
}

#[derive(Debug, Deserialize)]
struct Part {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeminiError {
    message: String,
    // code: Option<i32>,
}

/// Send audio to Gemini API for transcription.
///
/// - `api_key`: Google Gemini API key
/// - `wav_bytes`: raw WAV file bytes
/// - `style`: polishing style
///
/// Returns the transcribed (and optionally polished) text.
pub async fn transcribe(
    api_key: &str,
    wav_bytes: &[u8],
    style: &PolishStyle,
) -> Result<String, String> {
    let audio_base64 = base64::engine::general_purpose::STANDARD.encode(wav_bytes);

    let payload = serde_json::json!({
        "system_instruction": {
            "parts": [{ "text": style.system_prompt() }]
        },
        "contents": [{
            "parts": [
                { "text": "Transcribe this audio:" },
                {
                    "inlineData": {
                        "mimeType": "audio/wav",
                        "data": audio_base64
                    }
                }
            ]
        }],
        "generationConfig": {
            "temperature": 0.1,
            "maxOutputTokens": 4096
        }
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Gemini API error ({}): {}", status, body));
    }

    let gemini_resp: GeminiResponse =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(err) = gemini_resp.error {
        return Err(format!("Gemini API error: {}", err.message));
    }

    let text = gemini_resp
        .candidates
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.content)
        .and_then(|c| c.parts)
        .and_then(|p| p.into_iter().next())
        .and_then(|p| p.text)
        .ok_or_else(|| "No transcription text in Gemini response".to_string())?;

    Ok(text.trim().to_string())
}
