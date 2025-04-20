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

#[cfg(test)]
use hound::WavReader;
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, Signal};
#[cfg(test)]
use symphonia::core::codecs::DecoderOptions;
#[cfg(test)]
use symphonia::core::formats::FormatOptions;
#[cfg(test)]
use symphonia::core::io::MediaSourceStream;
#[cfg(test)]
use symphonia::core::meta::MetadataOptions;
#[cfg(test)]
use symphonia::core::probe::Hint;
#[cfg(test)]
use symphonia::default::get_probe;

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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
    return mono_samples;
}

#[cfg(test)]
fn wav_get_sample_rate(path: &str) -> u32 {
    let reader = WavReader::open(path).expect("Failed to open WAV file");
    let spec = reader.spec();
    spec.sample_rate
}

#[cfg(test)]
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

#[cfg(test)]
#[test]
fn test_recorded_yin_standard_a2() {
    let file: &str = "test_assets/A.m4a";
    let sr: u32 = m4a_get_sample_rate(file);
    assert_eq!(sr, 48_000);
    let samples = read_m4a_as_f32(file);
    yin_find_note_from_samples(&samples, sr as usize, "standard-e", "A2");
}

#[cfg(test)]
#[test]
fn test_recorded_yin_standard_e2() {
    let file: &str = "test_assets/E2.m4a";
    let sr: u32 = m4a_get_sample_rate(file);
    assert_eq!(sr, 48_000);
    let samples = read_m4a_as_f32(file);
    yin_find_note_from_samples(&samples, sr as usize, "standard-e", "E2");
}

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
#[cfg(test)]
fn yin_find_note_from_samples(samples: &[f32], sample_rate: usize, tuning: &str, note: &str) {
    let mut yin = YinPitchDetector::new(
        0.1,   // threshold
        60.0,  // min frequency
        500.0, // max frequency
        sample_rate,
    );

    let frame_size = 2048;
    let hop_size = 512; // or 1024 for lower resolution
    let mut picked_up_something = false;
    let process_until = samples.len() / 4 - frame_size; // we don't need to go through the whole file

    for i in (0..process_until - frame_size).step_by(hop_size) {
        let frame = &samples[i..i + frame_size];
        let frame_f64: Vec<f64> = frame.iter().map(|&s| s as f64).collect();

        let pitch = yin.maybe_find_pitch(&frame_f64, tuning);
        if let Some(res) = pitch {
            picked_up_something = true;
            println!(
                "Time {:.2}s - Pitch: {:.2} Hz",
                i as f32 / sample_rate as f32,
                res.freq()
            );
            assert!(res.tuning_to().note() == note);
        }
    }
    assert!(picked_up_something, "Yin didn't pick up anything.");
}
