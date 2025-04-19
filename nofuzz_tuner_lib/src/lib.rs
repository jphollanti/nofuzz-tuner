// src/lib.rs

use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

use audioviz::spectrum::{
    config::{
        Interpolation, PositionNormalisation, ProcessorConfig, StreamConfig as StreamConfig2,
        VolumeNormalisation,
    },
    stream::Stream,
};

use lazy_static::lazy_static;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use js_sys::Float64Array;
use std::cmp::Ordering;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    // Set the panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();
}

// Guitar string frequencies cheat-sheet:
lazy_static! {
    pub static ref TUNINGS: HashMap<&'static str, HashMap<&'static str, f64>> = {
        let mut tunings = HashMap::new();

        // ── 1. Standard E ─────────────────────────────
        tunings.insert("standard-e", HashMap::from([
            ("E2", 82.41),
            ("A2", 110.00),
            ("D3", 146.83),
            ("G3", 196.00),
            ("B3", 246.94),
            ("E4", 329.63),
        ]));

        // ── 2. Standard Eb / “Half‑step‑down” ─────────
        tunings.insert("flat-e", HashMap::from([
            ("Eb2", 77.78),
            ("Ab2", 103.83),
            ("Db3", 138.59),
            ("Gb3", 185.00),
            ("Bb3", 233.08),
            ("Eb4", 311.13),
        ]));

        // ── 3. Drop‑D ────────────────────────────────
        tunings.insert("drop-d", HashMap::from([
            ("D2", 73.42),
            ("A2", 110.00),
            ("D3", 146.83),
            ("G3", 196.00),
            ("B3", 246.94),
            ("E4", 329.63),
        ]));

        tunings
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub device_id: usize,
    pub pitch_detection: String,
    // Yin parameters
    pub threshold: f64,
    pub freq_min: f64,
    pub freq_max: f64,
    // Mcleod parameters
    pub power_threshold: f64,
    pub clarity_threshold: f64,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct TuningTo {
    tuning: String,
    note: String,
    freq: f64,
    distance: f64,
    cents: f64,
}

#[wasm_bindgen]
impl TuningTo {
    #[wasm_bindgen(constructor)]
    pub fn new(tuning: String, note: String, freq: f64, distance: f64, cents: f64) -> TuningTo {
        Self {
            tuning,
            note,
            freq,
            distance,
            cents,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn tuning(&self) -> String {
        self.tuning.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn note(&self) -> String {
        self.note.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn freq(&self) -> f64 {
        self.freq
    }
    #[wasm_bindgen(getter)]
    pub fn distance(&self) -> f64 {
        self.distance
    }
    #[wasm_bindgen(getter)]
    pub fn cents(&self) -> f64 {
        self.cents
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PitchResult {
    freq: f64,
    tuning_to: TuningTo,
}

#[wasm_bindgen]
impl PitchResult {
    #[wasm_bindgen(constructor)]
    pub fn new(
        freq: f64,
        tuning: String,
        closest_note: String,
        closest_freq: f64,
        distance: f64,
        cents: f64,
    ) -> PitchResult {
        let tuning_to = TuningTo::new(tuning, closest_note, closest_freq, distance, cents);
        Self { freq, tuning_to }
    }

    #[wasm_bindgen(getter)]
    pub fn freq(&self) -> f64 {
        self.freq
    }

    // custom JS property name: result.tuningTo
    #[wasm_bindgen(getter = tuningTo)]
    pub fn tuning_to(&self) -> TuningTo {
        self.tuning_to.clone()
    }
}

pub trait PitchFindTrait: Send + Sync {
    fn maybe_find_pitch(&mut self, data: &[f64], tuning: &str) -> Option<PitchResult>;
}

#[wasm_bindgen]
pub struct YinPitchDetector {
    yin: yin::Yin,
}

#[wasm_bindgen]
impl YinPitchDetector {
    #[wasm_bindgen(constructor)]
    pub fn new(
        threshold: f64,
        freq_min: f64,
        freq_max: f64,
        sample_rate: usize,
    ) -> YinPitchDetector {
        let yin = yin::Yin::init(threshold, freq_min, freq_max, sample_rate);
        YinPitchDetector { yin }
    }

    #[wasm_bindgen]
    pub fn maybe_find_pitch_js(
        &mut self,
        data: &Float64Array,
        tuning: &str,
    ) -> Option<PitchResult> {
        // Convert the Float64Array from JavaScript to a Rust slice
        let data_vec = data.to_vec(); // Convert the Float64Array to Vec<f64>

        self.maybe_find_pitch(&data_vec, tuning)
    }
}

fn find_closest_note(freq: f64, tuning: &str) -> Option<(String, f64, f64)> {
    let strings = TUNINGS.get(tuning)?;

    let (note, target_freq) = strings.iter().min_by(|a, b| {
        let da = (a.1 - freq).abs();
        let db = (b.1 - freq).abs();
        // unwrap_or(Ordering::Equal), because in the context of min_by,
        // prevents a stray NaN (which YIN can produce if something weird
        // slips through)
        da.partial_cmp(&db).unwrap_or(Ordering::Equal)
    })?;

    Some((
        (*note).to_string(),
        *target_freq,
        (*target_freq - freq).abs(),
    ))
}

impl PitchFindTrait for YinPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64], tuning: &str) -> Option<PitchResult> {
        let freq = self.yin.estimate_freq(data);
        if freq != f64::INFINITY {
            // Find closest note
            let (closest_note, closest_freq, distance) = find_closest_note(freq, tuning).unwrap();
            let cents = 1200.0 * (freq / closest_freq).log2();
            return Some(PitchResult::new(
                freq,
                tuning.to_string(),
                closest_note,
                closest_freq,
                distance,
                cents,
            ));
        }
        None
    }
}

pub struct McleodPitchDetector {
    sample_rate: usize,
    power_threshold: f64,
    clarity_threshold: f64,

    size: usize,
    padding: usize,
}
impl McleodPitchDetector {
    pub fn new(
        size: usize,
        padding: usize,
        sample_rate: usize,
        power_threshold: f64,
        clarity_threshold: f64,
    ) -> McleodPitchDetector {
        McleodPitchDetector {
            sample_rate,
            power_threshold,
            clarity_threshold,
            size,
            padding,
        }
    }
}

impl PitchFindTrait for McleodPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64], tuning: &str) -> Option<PitchResult> {
        let mut mcleod = McLeodDetector::new(self.size, self.padding);
        let pitch = mcleod.get_pitch(
            data,
            self.sample_rate,
            self.power_threshold,
            self.clarity_threshold,
        );
        if let Some(p) = pitch {
            let (closest_note, closest_freq, distance) =
                find_closest_note(p.frequency, tuning).unwrap();
            let cents = 1200.0 * (p.frequency / closest_freq).log2();
            return Some(PitchResult::new(
                p.frequency,
                tuning.to_string(),
                closest_note,
                closest_freq,
                distance,
                cents,
            ));
            //return Some(p.frequency);
        }
        None
    }
}

pub struct FftPitchDetector {
    stream: Stream,
}

impl Default for FftPitchDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FftPitchDetector {
    pub fn new() -> FftPitchDetector {
        // spectrum visualizer stream
        let stream: Stream = Stream::new(StreamConfig2 {
            channel_count: 1,
            processor: ProcessorConfig {
                sampling_rate: 8192,
                frequency_bounds: [0, 1000],
                resolution: None,
                volume: 1.0,
                volume_normalisation: VolumeNormalisation::Mixture,
                position_normalisation: PositionNormalisation::Harmonic,
                manual_position_distribution: None,
                interpolation: Interpolation::Cubic,
            },
            fft_resolution: 1024,
            refresh_rate: 30,
            gravity: Some(5.0),
        });

        FftPitchDetector { stream }
    }
}

impl PitchFindTrait for FftPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64], tuning: &str) -> Option<PitchResult> {
        let vec: Vec<f32> = data.iter().map(|&x| x as f32).collect();

        self.stream.push_data(vec);
        self.stream.update();

        let mut hvol: f32 = 0.0;
        let mut highest: f32 = 0.0;

        let frequencies = self.stream.get_frequencies();
        for frequency in frequencies.iter() {
            for item in frequency {
                if item.volume > hvol {
                    hvol = item.volume;
                    highest = item.freq;
                }
            }
        }
        let freq = highest as f64;
        if freq == 0.0 {
            return None;
        }
        let (closest_note, closest_freq, distance) = find_closest_note(freq, tuning).unwrap();
        let cents = 1200.0 * (freq / closest_freq).log2();
        Some(PitchResult::new(
            freq,
            tuning.to_string(),
            closest_note,
            closest_freq,
            distance,
            cents,
        ))
    }
}
