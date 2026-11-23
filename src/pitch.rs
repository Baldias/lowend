//! Pitch detection (pYIN via the `pyin` crate) and note math.
//! No I/O here: runs the same on desktop and wasm.

use pyin::{Framing, PYINExecutor};

/// Search range: below B0 (30.9 Hz, 5-string) up to above G4.
const MIN_HZ: f64 = 28.0;
const MAX_HZ: f64 = 420.0;
/// pYIN frame: must hold two periods of the lowest note.
const FRAME: usize = 4096;
/// Two overlapping frames per analysis so pYIN's voicing HMM has context.
const HOP: usize = 1024;

/// Samples per analysis window.
pub fn window_len() -> usize {
    FRAME + HOP
}

pub struct Detector {
    pyin: PYINExecutor<f32>,
}

impl Detector {
    pub fn new(sample_rate: f32) -> Self {
        // shortcut: FRAME is sized for 44.1/48 kHz; scale it if a device runs at 96 kHz+.
        let pyin = PYINExecutor::new(MIN_HZ, MAX_HZ, sample_rate as u32, FRAME, None, Some(HOP), None);
        Detector { pyin }
    }

    /// Fundamental frequency in Hz of the newest frame in `buf`, or `None` if unvoiced/silent.
    pub fn detect(&mut self, buf: &[f32]) -> Option<f32> {
        let (_, f0, voiced, _) = self.pyin.pyin(buf, f32::NAN, Framing::Valid);
        (*voiced.last()?).then(|| *f0.last().unwrap())
    }
}

// ASCII "#": Geist has no ♯ glyph.
const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    /// MIDI number of the nearest note (A4 = 69).
    pub midi: i32,
    /// Deviation from that note, −50…+50.
    pub cents: f32,
}

impl Note {
    pub fn from_hz(hz: f32, a4: f32) -> Self {
        let exact = 69.0 + 12.0 * (hz / a4).log2();
        let midi = exact.round() as i32;
        Note { midi, cents: (exact - midi as f32) * 100.0 }
    }
    pub fn name(self) -> &'static str {
        NAMES[self.midi.rem_euclid(12) as usize]
    }
    pub fn octave(self) -> i32 {
        self.midi.div_euclid(12) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;

    /// Bass-like tone: weak fundamental, strong 2nd/3rd harmonics (the classic octave-error trap).
    fn bass_tone(hz: f32) -> Vec<f32> {
        (0..window_len())
            .map(|i| {
                let t = i as f32 / SR * std::f32::consts::TAU * hz;
                0.3 * t.sin() + 0.5 * (2.0 * t).sin() + 0.3 * (3.0 * t).sin()
            })
            .collect()
    }

    #[test]
    fn detects_open_strings_within_two_cents() {
        let mut d = Detector::new(SR);
        for hz in [30.87, 41.20, 55.00, 73.42, 98.00] {
            let got = d.detect(&bass_tone(hz)).expect("no pitch");
            let cents = 1200.0 * (got / hz).log2();
            // pYIN reads ~1 ¢ sharp on harmonic-heavy tones; fine for a tuner.
            assert!(cents.abs() < 2.0, "{hz} Hz → {got} Hz ({cents:+.2} ¢)");
        }
    }

    #[test]
    fn silence_has_no_pitch() {
        assert_eq!(Detector::new(SR).detect(&vec![0.0; window_len()]), None);
    }

    #[test]
    fn note_math() {
        let e1 = Note::from_hz(41.25, 440.0);
        assert_eq!((e1.name(), e1.octave()), ("E", 1));
        assert!((e1.cents - 1.96).abs() < 0.05); // E1 = 41.2034 Hz
        let a = Note::from_hz(54.43, 440.0);
        assert_eq!((a.name(), a.octave()), ("A", 1));
        assert!((a.cents + 18.0).abs() < 0.2);
    }
}
