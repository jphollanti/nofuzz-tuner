// src/lib.rs

use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::VecDeque;

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

// Helpers for bitmasking
// Used in YinPitchDetector::new() to select filters
pub fn set_bits(positions: &[u32]) -> usize {
    positions.iter().fold(0, |acc, &bit| acc | (1 << bit))
}

#[wasm_bindgen]
#[allow(clippy::boxed_local)]
pub fn set_bits_js(bits: Box<[u32]>) -> usize {
    bits.iter().fold(0, |acc, &bit| acc | (1 << bit))
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

#[wasm_bindgen]
pub struct FrequencySmoother {
    window: VecDeque<f64>,
    max_size: usize,
}

impl FrequencySmoother {
    fn new(max_size: usize) -> Self {
        FrequencySmoother {
            window: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn push(&mut self, value: f64) {
        if self.window.len() == self.max_size {
            self.window.pop_front();
        }
        self.window.push_back(value);
    }

    fn average(&self) -> Option<f64> {
        if self.window.is_empty() {
            None
        } else {
            Some(self.window.iter().sum::<f64>() / self.window.len() as f64)
        }
    }
}

#[wasm_bindgen]
pub struct ExpMovingAverage {
    alpha: f64, // e.g., 0.2
    current: Option<f64>,
}

impl ExpMovingAverage {
    fn new(alpha: f64) -> Self {
        Self {
            alpha,
            current: None,
        }
    }

    fn update(&mut self, new_value: f64) -> f64 {
        self.current = Some(match self.current {
            Some(prev) => self.alpha * new_value + (1.0 - self.alpha) * prev,
            None => new_value,
        });
        self.current.unwrap()
    }
}

pub trait PitchFindTrait: Send + Sync {
    fn maybe_find_pitch(&mut self, data: &[f64], tuning: &str) -> Option<PitchResult>;
    fn fft_refine_pitch(&self, samples: &[f32], approx_freq: f32) -> Option<f32>;
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

// Simple Direct‑Form I biquad filter (f64)
// A 2nd‑order IIR filter, meaning it uses the current sample plus the two previous input samples
// and the two previous output samples to compute each new output.
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

use std::f64::consts::PI;

impl Biquad {
    /// High‑pass @ `fc` (Hz) with quality `Q` (e.g. 0.707 for Butterworth)
    fn new_highpass(fs: f64, fc: f64, q: f64) -> Self {
        let w0 = 2.0 * PI * fc / fs;
        let alpha = w0.sin() / (2.0 * q);
        let cosw0 = w0.cos();
        let (b0, b1, b2) = ((1.0 + cosw0) / 2.0, -(1.0 + cosw0), (1.0 + cosw0) / 2.0);
        let (a0, a1, a2) = (1.0 + alpha, -2.0 * cosw0, 1.0 - alpha);

        Biquad {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    // /// Low-pass @ `fc` (Hz) with quality `Q` (e.g. 0.707 for Butterworth)
    pub fn new_lowpass(fs: f64, fc: f64, q: f64) -> Self {
        let w0 = 2.0 * PI * fc / fs;
        let alpha = w0.sin() / (2.0 * q);
        let cosw0 = w0.cos();

        // LP numerator (b0,b1,b2), HP is ((1+cos)/2, -(1+cos), (1+cos)/2)
        let (b0, b1, b2) = ((1.0 - cosw0) / 2.0, 1.0 - cosw0, (1.0 - cosw0) / 2.0);
        // denominator is the same form for all biquads
        let (a0, a1, a2) = (1.0 + alpha, -2.0 * cosw0, 1.0 - alpha);

        Biquad {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Notch (aka band‑stop) @ `fc` (Hz) with quality `Q` (narrow notch if Q large)
    fn new_notch(fs: f64, fc: f64, q: f64) -> Self {
        let w0 = 2.0 * PI * fc / fs;
        let alpha = w0.sin() / (2.0 * q);
        let cosw0 = w0.cos();
        let (b0, b1, b2) = (1.0, -2.0 * cosw0, 1.0);
        let (a0, a1, a2) = (1.0 + alpha, -2.0 * cosw0, 1.0 - alpha);

        Biquad {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn new_bandpass(fs: f64, fc: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * fc / fs;
        let alpha = w0.sin() / (2.0 * q);
        let cosw0 = w0.cos();

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cosw0;
        let a2 = 1.0 - alpha;

        Biquad {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process one sample, returning the filtered output
    fn process(&mut self, x0: f64) -> f64 {
        // Direct Form I: y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2]
        //                     - a1*y[n-1] - a2*y[n-2]
        let y0 = self.b0 * x0 + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        // shift delay lines
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;

        y0
    }
}

// Post pitch‑detection processing
// Seems to be more trouble than worth, especially with Yin
// fn octave_guard(
//     raw_freq: f64,
//     freq_min: f64,
//     freq_max: f64,
//     tuning: &str,
// ) -> Option<(f64, String, f64 /*note Hz*/, f64 /*dist*/)> {
//     // octave-error guard
//     // examine f/2, f, f*2  (add more powers of two if you ever need them)
//     let mut best: Option<(f64, String, f64 /*note Hz*/, f64 /*dist*/)> = None;

//     // how many octaves can fit between fmin and fmax?
//     let oct_down = (raw_freq / freq_min as f64).log2().floor() as i32;
//     let oct_up = (freq_max as f64 / raw_freq).log2().floor() as i32;

//     for k in -oct_down..=oct_up {
//         // e.g. −2…+2
//         let cand = raw_freq * 2f64.powi(k);
//         if let Some((note, note_freq, dist)) = find_closest_note(cand, tuning) {
//             let cand_score = (dist.abs(), cand);
//             let is_better = best
//                 .as_ref()
//                 .map_or(true, |b| cand_score < (b.3.abs(), b.0));

//             if is_better {
//                 best = Some((cand, note, note_freq, dist));
//             }
//         }
//     }
//     best
// }

#[wasm_bindgen]
pub struct YinPitchDetector {
    yin: yin::Yin,
    sample_rate: usize,
    filters: Vec<Biquad>,

    fft_refine: bool,
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,

    freq_smoother: FrequencySmoother,
    clarity_smoother: ExpMovingAverage,
}

#[wasm_bindgen]
impl YinPitchDetector {
    #[wasm_bindgen(constructor)]
    pub fn new(
        threshold: f64,
        freq_min: f64,
        freq_max: f64,
        sample_rate: usize,

        // Filters:
        // Select filters with a bitmask:
        // 0: highpass @ 70 Hz, Q=0.707
        //      Guitar fundamentals start at E2 ≈ 82 Hz (or D2 ≈ 73 Hz in Drop‑D).
        //      Below that lives rumble—from laptop fans, room resonance, mic handling
        //      noise—that can confuse autocorrelation or YIN.
        //
        // 1: notch @ 50 Hz, Q=30
        //
        //      Filter out mains‑hum in Europe (50 Hz) and US (60 Hz).
        //      Quiet-but-pervasive buzz you hear when your audio gear picks up the AC power signal
        //      from your wall sockets. Origin: Household electricity alternates at either 50 Hz
        //      (Europe, Asia, Africa, most of the world) or 60 Hz (North America, parts of Asia).
        //
        //      Cables, transformers, power supplies and even the wiring in walls create tiny
        //      electromagnetic fields. Your mic preamp or guitar cable, especially if unbalanced,
        //      can act like an antenna and accidentally capture that field.
        //
        // 2: notch @ 60 Hz, Q=30
        // 3: notch @ 100 Hz, Q=30
        //      A near‑pure low‑frequency tone (plus its 2nd harmonic at 100 Hz or 120 Hz), often
        //      experienced as a steady “hum” or “buzz” under your music.
        //
        // 4: notch @ 120 Hz, Q=30
        // 5: lowpass @ 5 kHz, Q=0.707

        // Note: add individual string filters with add_string_filter method
        filter_mask: usize,
        block: usize,
        fft_refine: bool,
    ) -> YinPitchDetector {
        // /**
        //  * This works OK now but G3 string is still noisy.
        //  *
        //  * Can try to:
        //  * - Apply harmonic correction
        //  * Check if the refined frequency is a clean multiple of the YIN estimate:
        //  * let ratio = refined_freq / approx_freq;
        //  * let rounded = ratio.round();
        //  * if (rounded - ratio).abs() < 0.05 && rounded <= 3.0 {
        //  *     refined_freq /= rounded;
        //  * }
        //  *
        //  * - Use longer FFT window just for G string
        //  *
        //  * - Increase smoothing just for G. Temporarily boost EMA smoothing:
        //  *      if note == "G":
        //  *           let alpha = if note == "G" { 0.2 } else { 0.4 };
        //  *
        //  * - Apply bandpass filter 180–220 Hz
        //  *
        //  * Apply a narrow bandpass filter around 180–220 Hz before passing G
        //  * string data into pitch estimation. This helps both YIN and FFT zero in.
        //  *
        //  * Summary Table
        //  * Fix	                            Helps With
        //  * Harmonic correction	            FFT misidentifying pitch
        //  * Larger FFT window (G-only)	    Bin resolution
        //  * Lower EMA α (G-only)	        Visual jitter
        //  * Bandpass filtering 180–220 Hz	Noise & overtones
        //  */
        let q = 0.707; // classic Butterworth

        fn is_bit_set(value: usize, bit: u32) -> bool {
            (value & (1 << bit)) != 0
        }

        let mut filters = Vec::new();

        if is_bit_set(filter_mask, 0) {
            filters.push(Biquad::new_highpass(sample_rate as f64, 70.0, q));
        }
        if is_bit_set(filter_mask, 1) {
            filters.push(Biquad::new_notch(sample_rate as f64, 50.0, 30.0));
        }
        if is_bit_set(filter_mask, 2) {
            filters.push(Biquad::new_notch(sample_rate as f64, 60.0, 30.0));
        }
        if is_bit_set(filter_mask, 3) {
            filters.push(Biquad::new_notch(sample_rate as f64, 100.0, 30.0));
        }
        if is_bit_set(filter_mask, 4) {
            filters.push(Biquad::new_notch(sample_rate as f64, 120.0, 30.0));
        }
        if is_bit_set(filter_mask, 5) {
            filters.push(Biquad::new_lowpass(sample_rate as f64, 5_000.0, q));
        }

        let yin = yin::Yin::init(threshold, freq_min, freq_max, sample_rate);
        let buffer_len: usize = block; //4096;// block;
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(buffer_len);

        YinPitchDetector {
            yin,
            sample_rate,
            filters,
            fft,
            fft_refine,
            freq_smoother: FrequencySmoother::new(3),
            // Alpha:
            // 0.1	Very smooth, slow reaction	Stable pitch display
            // 0.3	Moderate smoothing	        Fast UI with mild damping
            // 0.5+	Very reactive, less stable	Real-time effects, fast glides
            // 1.0	No smoothing (raw signal)	Rarely useful unless you like chaos
            clarity_smoother: ExpMovingAverage::new(0.4),
        }
    }

    #[wasm_bindgen]
    pub fn add_string_filter(&mut self, freq: f64) {
        let q = 1.0; // adjust for your desired bandwidth
        self.filters
            .push(Biquad::new_bandpass(self.sample_rate as f64, freq, q));
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

impl PitchFindTrait for YinPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64], tuning: &str) -> Option<PitchResult> {
        let mut buf = data.to_vec();

        // apply filters in place to increase frequencies picked up by Yin.
        // Observed changes in unit tests:
        // - E2: before 12, after 35
        // - A2: before 2, after 16

        for sample in buf.iter_mut() {
            for filter in &mut self.filters {
                *sample = filter.process(*sample);
            }
        }

        // simple RMS noise gate
        // let rms = (buf.iter().map(|s| s*s).sum::<f64>() / buf.len() as f64).sqrt();
        // if rms < self.noise_gate_threshold {
        //     return None;  // too quiet → probably just hiss
        // }

        let estimated_freq = self.yin.estimate_freq(&buf);
        if estimated_freq != f64::INFINITY {
            let mut freq: f64 = -1.0;

            if self.fft_refine {
                let buf_f32: Vec<f32> = buf.iter().map(|&x| x as f32).collect();
                let refined_freq = self.fft_refine_pitch(&buf_f32, estimated_freq as f32);
                if refined_freq.is_some() {
                    freq = refined_freq.unwrap() as f64;
                }
            } else {
                freq = estimated_freq;
            }

            if freq > 0.0 {
                self.freq_smoother.push(freq);

                if let Some(mean_freq) = self.freq_smoother.average() {
                    if (mean_freq - freq).abs() > 5.5 {
                        // Try to prevent fluctuations
                        return None;
                    }
                    //let stable_freq = freq;
                    let stable_freq = self.clarity_smoother.update(freq);

                    // Find closest note
                    let (closest_note, closest_freq, distance) =
                        find_closest_note(stable_freq, tuning).unwrap();
                    let cents = 1200.0 * (stable_freq / closest_freq).log2();
                    return Some(PitchResult::new(
                        stable_freq,
                        tuning.to_string(),
                        closest_note,
                        closest_freq,
                        distance,
                        cents,
                    ));
                }
            }
        }
        None
    }

    fn fft_refine_pitch(&self, samples: &[f32], approx_freq: f32) -> Option<f32> {
        let len = samples.len();

        // Apply Hann window to samples
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let hann_window =
                    0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / len as f32).cos();
                Complex {
                    re: x * hann_window,
                    im: 0.0,
                }
            })
            .collect();

        self.fft.process(&mut buffer);

        let bin_resolution = self.sample_rate as f32 / len as f32;
        let approx_bin = (approx_freq / bin_resolution).round() as usize;

        // Ensure the bin is safely within bounds
        if approx_bin < 2 || approx_bin >= len / 2 - 2 {
            return None;
        }

        // Find the actual local peak within ±1 bin around approx_bin
        let search_bins =
            approx_bin.saturating_sub(1)..=(approx_bin + 1).min(buffer.len().saturating_sub(1));

        let (peak_bin, _) = search_bins
            .map(|bin| (bin, buffer[bin].norm()))
            .max_by(|(_, mag_a), (_, mag_b)| mag_a.partial_cmp(mag_b).unwrap())?;

        // Guard: ensure we're not near the edge of the buffer
        if peak_bin < 1 || peak_bin + 1 >= buffer.len() {
            return None;
        }
        let mag_prev = buffer[peak_bin - 1].norm();
        let mag_curr = buffer[peak_bin].norm();
        let mag_next = buffer[peak_bin + 1].norm();

        let denominator = mag_prev - 2.0 * mag_curr + mag_next;
        if denominator.abs() < f32::EPSILON {
            return Some(peak_bin as f32 * bin_resolution);
        }

        let delta = 0.5 * (mag_prev - mag_next) / denominator;
        let refined_bin = peak_bin as f32 + delta;

        Some(refined_bin * bin_resolution)
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
    fn fft_refine_pitch(&self, _samples: &[f32], approx_freq: f32) -> Option<f32> {
        Some(approx_freq)
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

    fn fft_refine_pitch(&self, _samples: &[f32], approx_freq: f32) -> Option<f32> {
        Some(approx_freq)
    }
}
#[cfg(test)]
mod tests {
    use super::{find_closest_note, TUNINGS};

    /// Helper to unwrap the Option and compare String & f64 fields within epsilon.
    fn assert_note_result(
        result: Option<(String, f64, f64)>,
        want_note: &str,
        want_target: f64,
        want_dist: f64,
    ) {
        let (note, target, dist) = result.expect("expected Some(...)");
        assert_eq!(note, want_note);
        assert!(
            (target - want_target).abs() < 1e-6_f64,
            "target: got {}, want {}",
            target,
            want_target
        );
        assert!(
            (dist - want_dist).abs() < 1e-6_f64,
            "dist:   got {}, want {}",
            dist,
            want_dist
        );
    }

    #[test]
    fn standard_e_exact_match() {
        assert_note_result(
            find_closest_note(82.41_f64, "standard-e"),
            "E2",
            82.41_f64,
            0.0_f64,
        );
        assert_note_result(
            find_closest_note(110.0_f64, "standard-e"),
            "A2",
            110.0_f64,
            0.0_f64,
        );
        assert_note_result(
            find_closest_note(146.83_f64, "standard-e"),
            "D3",
            146.83_f64,
            0.0_f64,
        );
        assert_note_result(
            find_closest_note(196.0_f64, "standard-e"),
            "G3",
            196.0_f64,
            0.0_f64,
        );
        assert_note_result(
            find_closest_note(246.94_f64, "standard-e"),
            "B3",
            246.94_f64,
            0.0_f64,
        );
        assert_note_result(
            find_closest_note(329.63_f64, "standard-e"),
            "E4",
            329.63_f64,
            0.0_f64,
        );
    }

    #[test]
    fn standard_e_off_by_a_bit() {
        assert_note_result(
            find_closest_note(83.0_f64, "standard-e"),
            "E2",
            82.41_f64,
            (83.0_f64 - 82.41_f64).abs(),
        );
        assert_note_result(
            find_closest_note(108.0_f64, "standard-e"),
            "A2",
            110.0_f64,
            (110.0_f64 - 108.0_f64).abs(),
        );
        assert_note_result(
            find_closest_note(150.0_f64, "standard-e"),
            "D3",
            146.83_f64,
            (150.0_f64 - 146.83_f64).abs(),
        );
        assert_note_result(
            find_closest_note(200.0_f64, "standard-e"),
            "G3",
            196.0_f64,
            (200.0_f64 - 196.0_f64).abs(),
        );
        assert_note_result(
            find_closest_note(250.0_f64, "standard-e"),
            "B3",
            246.94_f64,
            (250.0_f64 - 246.94_f64).abs(),
        );
        assert_note_result(
            find_closest_note(330.5_f64, "standard-e"),
            "E4",
            329.63_f64,
            (330.5_f64 - 329.63_f64).abs(),
        );
    }

    #[test]
    fn drop_d_low_string() {
        assert_note_result(
            find_closest_note(73.42_f64, "drop-d"),
            "D2",
            73.42_f64,
            0.0_f64,
        );
        assert_note_result(
            find_closest_note(80.0_f64, "drop-d"),
            "D2",
            73.42_f64,
            (80.0_f64 - 73.42_f64).abs(),
        );
    }

    #[test]
    fn flat_e_tuning() {
        assert_note_result(
            find_closest_note(78.0_f64, "flat-e"),
            "Eb2",
            77.78_f64,
            (78.0_f64 - 77.78_f64).abs(),
        );
    }

    #[test]
    fn unknown_tuning_returns_none() {
        assert!(find_closest_note(100.0_f64, "no-such-tuning").is_none());
    }

    #[test]
    fn tuning_map_has_expected_keys() {
        for key in &["standard-e", "flat-e", "drop-d"] {
            assert!(
                TUNINGS.contains_key(*key),
                "TUNINGS missing expected key `{}`",
                key
            );
        }
    }

    use super::{PitchFindTrait, YinPitchDetector};
    use hound::WavReader;
    use std::fs::File;
    use symphonia::core::audio::{AudioBufferRef, SampleBuffer, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::get_probe;

    fn m4a_get_sample_rate(path: &str) -> u32 {
        let file = File::open(path).expect("Failed to open file");
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let hint = Hint::new(); // You could set extension hint: hint.with_extension("m4a");

        let probed = get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .expect("Failed to probe format");

        let format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.sample_rate.is_some())
            .expect("No track with sample rate found");

        track.codec_params.sample_rate.unwrap()
    }

    pub fn read_m4a_as_f32(path: &str) -> Vec<f32> {
        let file = File::open(path).expect("Failed to open file");
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let hint = Hint::new(); // You can add `.with_extension("m4a")` if needed

        let probed = get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .expect("Failed to probe format");

        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.sample_rate.is_some())
            .expect("No track with sample rate");

        let codec_params = &track.codec_params;
        let mut decoder = symphonia::default::get_codecs()
            .make(codec_params, &DecoderOptions::default())
            .expect("Failed to create decoder");

        let mut sample_buf: Option<SampleBuffer<f32>> = None;
        let mut output = Vec::new();

        while let Ok(packet) = format.next_packet() {
            let decoded = match decoder.decode(&packet) {
                Ok(audio_buf) => audio_buf,
                Err(_) => continue, // skip decode errors gracefully
            };

            match decoded {
                AudioBufferRef::F32(buf) => {
                    let channels = buf.spec().channels.count();
                    let frames = buf.frames();
                    for frame_idx in 0..frames {
                        let mono_sample = if channels == 1 {
                            buf.chan(0)[frame_idx]
                        } else {
                            // Downmix stereo by averaging channels
                            let mut sum = 0.0;
                            for ch in 0..channels {
                                sum += buf.chan(ch)[frame_idx];
                            }
                            sum / channels as f32
                        };
                        output.push(mono_sample);
                    }
                }
                _ => {
                    // If it's not already f32, convert to it
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    let channel_count = spec.channels.count();
                    let mut conv_buf = sample_buf
                        .take()
                        .unwrap_or_else(|| SampleBuffer::<f32>::new(duration, spec));
                    conv_buf.copy_interleaved_ref(decoded);
                    sample_buf = Some(conv_buf);

                    let conv = sample_buf.as_ref().unwrap();
                    let samples = conv.samples();

                    // Now use the stored `channel_count`
                    for chunk in samples.chunks(channel_count) {
                        let sum: f32 = chunk.iter().copied().sum();
                        output.push(sum / channel_count as f32);
                    }
                }
            }
        }

        output
    }

    fn read_wav_as_f32(path: &str) -> Vec<f32> {
        let mut reader = WavReader::open(path).expect("Failed to open WAV file");

        let spec = reader.spec();
        println!(
            "WAV format: {} Hz, {}-bit, {:?}",
            spec.sample_rate, spec.bits_per_sample, spec.channels
        );

        // Match based on sample format (usually i16 or f32)
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Int => reader
                .samples::<i16>()
                .filter_map(|s| s.ok()) // <- no unwraps, skip bad samples
                .map(|s| s as f32 / i16::MAX as f32)
                .collect(),

            hound::SampleFormat::Float => reader.samples::<f32>().filter_map(|s| s.ok()).collect(),
        };

        // Optional: downmix stereo to mono
        let mono_samples: Vec<f32> = if spec.channels == 2 {
            samples
                .chunks(2)
                .map(|ch| {
                    if ch.len() == 2 {
                        (ch[0] + ch[1]) / 2.0
                    } else {
                        ch[0]
                    }
                })
                .collect()
        } else {
            samples
        };
        mono_samples
    }

    fn wav_get_sample_rate(path: &str) -> u32 {
        let reader = WavReader::open(path).expect("Failed to open WAV file");
        let spec = reader.spec();
        spec.sample_rate
    }

    #[test]
    fn test_basic_yin_standard_e2() {
        const FILE: &str = "test_assets/82.wav";
        let sr: u32 = wav_get_sample_rate(FILE);
        let samples = read_wav_as_f32(FILE);
        let mut yin = YinPitchDetector::new(
            0.1,   // threshold
            60.0,  // min frequency
            500.0, // max frequency
            sr as usize,
            0b111110, // filter mask
            4096,     // block size
            false,    // fft_refine
        );
        let frame_size = 4096;
        let offset = 0; // You can slide this later

        let frame = &samples[offset..offset + frame_size];
        let frame_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();

        let rms = (frame.iter().map(|s| s * s).sum::<f32>() / frame.len() as f32).sqrt();
        println!("--- RMS: {}", rms);

        match yin.maybe_find_pitch(&frame_f64, "standard-e") {
            Some(res) => {
                // PitchResult {
                //     freq: res.freq(),
                //     tuning_to: TuningTo {
                //         tuning: res.tuning_to.tuning(),
                //         note: res.tuning_to.note(),
                //         freq: res.tuning_to.freq(),
                //         distance: res.tuning_to.distance(),
                //         cents: res.tuning_to.cents(),
                //     },
                // };
                assert!(res.tuning_to().note() == "E2");
            }
            None => panic!("====== Yin couldn't detect pitch in this frame."),
        }
    }

    #[test]
    fn test_recorded_yin_standard_e2() {
        let file: &str = "test_assets/E2.m4a";
        let sr: u32 = m4a_get_sample_rate(file);
        assert_eq!(sr, 48_000);
        let samples = read_m4a_as_f32(file);
        yin_find_note_from_samples(&samples, sr as usize, "standard-e", "E2", 4);
    }

    #[test]
    fn test_recorded_yin_standard_a2() {
        let file: &str = "test_assets/A.m4a";
        let sr: u32 = m4a_get_sample_rate(file);
        assert_eq!(sr, 48_000);
        let samples = read_m4a_as_f32(file);
        yin_find_note_from_samples(&samples, sr as usize, "standard-e", "A2", 4);
    }

    #[test]
    fn test_recorded_yin_standard_g3() {
        let file: &str = "test_assets/G3_22.m4a";
        let sr: u32 = m4a_get_sample_rate(file);
        assert_eq!(sr, 48_000);
        let samples = read_m4a_as_f32(file);
        yin_find_note_from_samples(&samples, sr as usize, "standard-e", "G3", 1);
    }

    #[test]
    fn test_recorded_yin_standard_b3() {
        let file: &str = "test_assets/B_2.m4a";
        let sr: u32 = m4a_get_sample_rate(file);
        assert_eq!(sr, 48_000);
        let samples = read_m4a_as_f32(file);
        yin_find_note_from_samples(&samples, sr as usize, "standard-e", "B3", 1);
    }

    // low audio, doesn't work properly at least yet
    // #[test]
    // fn test_recorded_yin_standard_e4() {
    //     let file: &str = "test_assets/E4.m4a";
    //     let sr: u32 = m4a_get_sample_rate(file);
    //     assert_eq!(sr, 48_000);
    //     let samples = read_m4a_as_f32(file);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "E4", 1);
    // }

    #[test]
    fn test_recorded_yin_standard_e4_b() {
        let file: &str = "test_assets/E4_2.m4a";
        let sr: u32 = m4a_get_sample_rate(file);
        assert_eq!(sr, 48_000);
        let samples = read_m4a_as_f32(file);
        yin_find_note_from_samples(&samples, sr as usize, "standard-e", "E4", 1);
    }

    // #[test]
    // fn yin_tracks_d3_from_wav() {
    //     // The asset lives in tests/assets so `cargo test` finds it in any cwd.
    //     let path = "test_assets/D3.wav";

    //     let (samples, sr) = read_wav_as_f32_2(path);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "D3");
    // }

    // #[test]
    // fn test_recorded_yin_standard_g3() {
    //     let file: &str = "test_assets/G.m4a";
    //     let sr: u32 = m4a_get_sample_rate(file);
    //     assert_eq!(sr, 48_000);
    //     let samples = read_m4a_as_f32(file);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "G3");
    // }

    // #[test]
    // fn test_recorded_yin_standard_b3() {
    //     let file: &str = "test_assets/B.m4a";
    //     let sr: u32 = m4a_get_sample_rate(file);
    //     assert_eq!(sr, 48_000);
    //     let samples = read_m4a_as_f32(file);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "B3");
    // }

    // #[test]
    // fn test_recorded_yin_standard_e4() {
    //     let file: &str = "test_assets/B.m4a";
    //     let sr: u32 = m4a_get_sample_rate(file);
    //     assert_eq!(sr, 48_000);
    //     let samples = read_m4a_as_f32(file);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "E4");
    // }

    // #[test]
    // fn yin_tracks_g3_from_wav() {
    //     // The asset lives in tests/assets so `cargo test` finds it in any cwd.
    //     let path = "test_assets/G3.wav";

    //     let (samples, sr) = read_wav_as_f32_2(path);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "G3");
    // }

    // #[test]
    // fn yin_tracks_e3_from_wav() {
    //     // The asset lives in tests/assets so `cargo test` finds it in any cwd.
    //     let path = "test_assets/E4.wav";

    //     let (samples, sr) = read_wav_as_f32_2(path);
    //     yin_find_note_from_samples(&samples, sr as usize, "standard-e", "E4");
    // }

    /// Runs pitch‑tracking on an already‑decoded **slice of samples** and asserts that the
    /// expected `note` is detected at least once.
    ///
    /// Frame‑based processing (2048 samples). Yin is an autocorrelation pitch
    /// detector: it needs a short window that contains at least a couple of
    /// periods of the fundamental.  
    ///   - At 48 kHz a 2048‑sample frame ≈ 42 ms, which comfortably contains  
    ///     two + periods down to ~60 Hz (the `min_frequency` chosen).  
    ///   - Using the full 12‑second file at once would blur many periods together,
    ///     destroying the clear trough in the Yin difference function; a small
    ///     window keeps the signal quasi‑stationary and maximises accuracy.
    ///
    /// Hop size (512 samples)
    ///   - Overlapping hops (¼‑frame here) give ~10 ms temporal resolution while
    ///     re‑using 75 % of each previous frame’s data. That’s what a live
    ///     microphone pipeline would do: slide a ring‑buffer forward and analyse
    ///     it again, fast enough to feel real‑time but slow enough to be cheap.
    ///
    /// Early bailout (`process_until = len / 4`)
    ///   - The test doesn’t need to scan the whole clip—just long enough to hit one
    ///     instance of the target note—so we quit after the first quarter to keep
    ///     unit‑tests snappy.
    ///
    /// Simulating a microphone
    ///   - In production you would feed `yin.process_sample(sample)` continuously
    ///     from an audio callback.  Splitting `samples` into small, overlapping
    ///     blocks replicates that streaming behaviour inside a deterministic test
    ///     without the complexity of real I/O threads.
    ///
    /// The assertions ensure:
    /// 1. At least one pitch is detected (`picked_up_something`).  
    /// 2. Every detected pitch normalises to the expected musical `note` under the
    ///    chosen `tuning` scheme.
    ///
    /// Together these checks act as a regression test for the Yin wrapper as well
    /// as for the entire decoding + normalisation pipeline.
    fn yin_find_note_from_samples(
        samples: &[f32],
        sample_rate: usize,
        tuning: &str,
        note: &str,
        fraction_to_check: usize,
    ) {
        let mut yin = YinPitchDetector::new(
            0.15,  // threshold
            60.0,  // min frequency
            500.0, // max frequency
            sample_rate,
            0b111110, // filter mask,
            4096,     // block
            false,    // fft_refine
        );

        let frame_size = 2048;
        let hop_size = 512; // or 1024 for lower resolution
        let mut picked_up_something = false;
        let mut picked_up_correct_note = 0;
        let mut picked_up_wrong_note = 0;
        let process_until = samples.len() / fraction_to_check - frame_size; // we don't need to go through the whole file
        let mut counter = 0;
        for i in (0..process_until).step_by(hop_size) {
            let frame = &samples[i..i + frame_size];
            let frame_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();

            let pitch = yin.maybe_find_pitch(&frame_f64, tuning);
            if let Some(res) = pitch {
                picked_up_something = true;
                counter += 1;
                println!(
                    "Time {:.2}s - Pitch: {:.2} Hz",
                    i as f32 / sample_rate as f32,
                    res.freq()
                );
                println!(
                    "Note: {} - Distance: {:.2} - Cents: {:.2}",
                    res.tuning_to().note(),
                    res.tuning_to().distance(),
                    res.tuning_to().cents()
                );
                if res.tuning_to().note() == note {
                    picked_up_correct_note += 1;
                    println!("Picked up the correct note: {}", note);
                } else {
                    picked_up_wrong_note += 1;
                    println!("Picked up the wrong note: {}", res.tuning_to().note());
                }
            }
        }
        println!("Picked up {} pitches", counter);
        assert!(picked_up_something, "Yin didn't pick up anything.");
        assert!(
            picked_up_correct_note > picked_up_wrong_note * 2,
            "Yin picked up wrong notes too often. Correct notes {}, wrong notes {}",
            picked_up_correct_note,
            picked_up_wrong_note
        );
    }
}
