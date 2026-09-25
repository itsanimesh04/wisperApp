use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Holds the state for an active recording session.
pub struct RecordingSession {
    stream: cpal::Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
}

// SAFETY: cpal::Stream on Windows WASAPI is thread-safe but doesn't implement Send.
// We protect all access through a Mutex, so this is safe.
unsafe impl Send for RecordingSession {}
unsafe impl Sync for RecordingSession {}

/// Shared audio state managed by Tauri.
pub struct AudioState {
    session: Mutex<Option<RecordingSession>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }

    /// Start capturing audio from the default input device.
    /// Returns Ok(()) if capture started, Err if already recording or no device available.
    pub fn start_capture(&self) -> Result<(), String> {
        let mut session_lock = self.session.lock().map_err(|e| e.to_string())?;
        if session_lock.is_some() {
            return Err("Already recording".into());
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device available")?;

        let supported_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels();
        let sample_format = supported_config.sample_format();
        let config: cpal::StreamConfig = supported_config.into();

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = Arc::clone(&buffer);

        let err_fn = |err: cpal::StreamError| {
            eprintln!("Audio stream error: {}", err);
        };

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buf) = buffer_clone.lock() {
                            buf.extend_from_slice(data);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("Failed to build f32 stream: {}", e))?,
            cpal::SampleFormat::I16 => {
                let buffer_clone_i16 = Arc::clone(&buffer);
                device
                    .build_input_stream(
                        &config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            if let Ok(mut buf) = buffer_clone_i16.lock() {
                                for &sample in data {
                                    buf.push(sample as f32 / i16::MAX as f32);
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build i16 stream: {}", e))?
            }
            cpal::SampleFormat::U16 => {
                let buffer_clone_u16 = Arc::clone(&buffer);
                device
                    .build_input_stream(
                        &config,
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            if let Ok(mut buf) = buffer_clone_u16.lock() {
                                for &sample in data {
                                    let s = (sample as f32 / u16::MAX as f32) * 2.0 - 1.0;
                                    buf.push(s);
                                }
                            }
                        },
                        err_fn,
                        None,
                    )
                    .map_err(|e| format!("Failed to build u16 stream: {}", e))?
            }
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        *session_lock = Some(RecordingSession {
            stream,
            buffer,
            sample_rate,
            channels,
        });

        Ok(())
    }

    /// Stop capturing and return the recorded audio as WAV bytes.
    pub fn stop_capture(&self) -> Result<Vec<u8>, String> {
        let mut session_lock = self.session.lock().map_err(|e| e.to_string())?;
        let session = session_lock
            .take()
            .ok_or("Not currently recording")?;

        // Drop the stream to stop recording
        drop(session.stream);

        let samples = session
            .buffer
            .lock()
            .map_err(|e| e.to_string())?
            .clone();

        if samples.is_empty() {
            return Err("No audio captured".into());
        }

        // Downmix to mono if stereo+
        let mono_samples = if session.channels > 1 {
            downmix_to_mono(&samples, session.channels as usize)
        } else {
            samples
        };

        // Encode to WAV (16-bit PCM mono, 16kHz target for smaller payload)
        let target_rate = 16000u32;
        let resampled = if session.sample_rate != target_rate {
            simple_resample(&mono_samples, session.sample_rate, target_rate)
        } else {
            mono_samples
        };

        samples_to_wav(&resampled, target_rate, 1)
    }
}

/// Downmix interleaved multi-channel audio to mono by averaging channels.
fn downmix_to_mono(samples: &[f32], channels: usize) -> Vec<f32> {
    samples
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Simple linear interpolation resampler (adequate for speech).
fn simple_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let s0 = samples[idx.min(samples.len() - 1)];
        let s1 = samples[(idx + 1).min(samples.len() - 1)];
        output.push((s0 as f64 * (1.0 - frac) + s1 as f64 * frac) as f32);
    }

    output
}

/// Encode f32 mono samples to 16-bit PCM WAV in memory.
fn samples_to_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer =
            WavWriter::new(&mut cursor, spec).map_err(|e| format!("WAV writer error: {}", e))?;
        for &sample in samples {
            let clamped = sample.max(-1.0).min(1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(int_sample)
                .map_err(|e| format!("WAV write error: {}", e))?;
        }
        writer
            .finalize()
            .map_err(|e| format!("WAV finalize error: {}", e))?;
    }

    Ok(cursor.into_inner())
}
