//! Microphone capture. The only platform-specific piece: swap this module for Web Audio on wasm.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct Mic {
    _stream: cpal::Stream,
    buf: Arc<Mutex<VecDeque<f32>>>,
    pub sample_rate: f32,
    len: usize,
}

impl Mic {
    /// Opens the default input device and keeps the last `window_len()` mono samples.
    pub fn open() -> Result<Self, String> {
        let device = cpal::default_host().default_input_device().ok_or("no input device")?;
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        // shortcut: f32 only (CoreAudio and WASAPI default), add i16/i32 if a device needs it.
        if config.sample_format() != cpal::SampleFormat::F32 {
            return Err(format!("unsupported sample format {}", config.sample_format()));
        }
        let sample_rate = config.sample_rate() as f32;
        let channels = config.channels() as usize;
        let len = crate::pitch::window_len();
        let buf = Arc::new(Mutex::new(VecDeque::with_capacity(len)));

        let writer = buf.clone();
        let stream = device
            .build_input_stream(
                config.config(),
                move |data: &[f32], _: &_| {
                    let mut b = writer.lock().unwrap();
                    b.extend(data.chunks(channels).map(|frame| frame[0]));
                    let excess = b.len().saturating_sub(len);
                    b.drain(..excess);
                },
                |e| eprintln!("audio: {e}"),
                None,
            )
            .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(Mic { _stream: stream, buf, sample_rate, len })
    }

    /// Latest full analysis window, or `None` until enough audio has arrived.
    pub fn window(&self) -> Option<Vec<f32>> {
        let b = self.buf.lock().unwrap();
        (b.len() == self.len).then(|| b.iter().copied().collect())
    }
}
