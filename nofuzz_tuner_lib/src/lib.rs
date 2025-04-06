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
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    // Set the panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Guitar string frequencies cheat-sheet:
lazy_static! {
    static ref GUITAR_STRINGS: HashMap<String, f64> = {
        let mut m = HashMap::new();
        m.insert("E2".to_string(), 82.41);
        m.insert("A2".to_string(), 110.00);
        m.insert("D3".to_string(), 146.83);
        m.insert("G3".to_string(), 196.00);
        m.insert("B3".to_string(), 246.94);
        m.insert("E4".to_string(), 329.63);
        m
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

pub trait PitchFindTrait: Send + Sync {
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64>;
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
    pub fn maybe_find_pitch_js(&mut self, data: &Float64Array) -> Option<f64> {
        // Convert the Float64Array from JavaScript to a Rust slice
        let data_vec = data.to_vec(); // Convert the Float64Array to Vec<f64>

        self.maybe_find_pitch(&data_vec)
    }
}

impl PitchFindTrait for YinPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64> {
        let freq = self.yin.estimate_freq(data);
        if freq != f64::INFINITY {
            return Some(freq);
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
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64> {
        let mut mcleod = McLeodDetector::new(self.size, self.padding);
        let pitch = mcleod.get_pitch(
            data,
            self.sample_rate,
            self.power_threshold,
            self.clarity_threshold,
        );
        if let Some(p) = pitch {
            return Some(p.frequency);
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
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64> {
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
        Some(highest as f64)
    }
}

pub fn find_string_and_distance(freq: f64) -> (f64, f64, String) {
    let mut min_distance = f64::INFINITY;
    let mut string_freq = 0.0;
    let mut string_key = "".to_string();
    for (key, sf) in GUITAR_STRINGS.iter() {
        let distance = freq - sf;
        if distance.abs() < min_distance.abs() {
            min_distance = distance;
            string_freq = *sf;
            string_key = key.to_string();
        }
    }
    (string_freq, min_distance, string_key)
}
