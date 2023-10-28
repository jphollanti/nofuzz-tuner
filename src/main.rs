use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::*;
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};
use std::io::{Write, stdout};
use serde_yaml;

use serde::{Deserialize, Serialize};

use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;

use audioviz::spectrum::{config::{StreamConfig as StreamConfig2, ProcessorConfig, VolumeNormalisation, PositionNormalisation, Interpolation}, stream::Stream};

use std::collections::HashMap;
use lazy_static::lazy_static;


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
struct Config {
    device_id: usize,
    pitch_detection: String,
    // Yin parameters
    threshold: f64,
    freq_min: f64,
    freq_max: f64,
    // Mcleod parameters
    power_threshold: f64, 
    clarity_threshold: f64
}

trait PitchFindTrait: Send + Sync  {
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64>;
}

struct YinPitchDetector {
    yin: yin::Yin,
}
impl YinPitchDetector {
    fn new(threshold: f64, freq_min: f64, freq_max: f64, sample_rate: usize) -> YinPitchDetector {
        let yin = yin::Yin::init(threshold, freq_min, freq_max, sample_rate);
        YinPitchDetector { yin: yin }
    }
}

impl PitchFindTrait for YinPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64> {
        let freq = self.yin.estimate_freq(data);
        if freq != std::f64::INFINITY {
            return Some(freq);
        }
        return None;
    }
}

struct McleodPitchDetector {
    sample_rate: usize,
    power_threshold: f64,
    clarity_threshold: f64,

    size: usize,
    padding: usize,
}
impl McleodPitchDetector {
    fn new(size: usize, padding: usize, sample_rate: usize, power_threshold: f64, clarity_threshold: f64) -> McleodPitchDetector {
        McleodPitchDetector { sample_rate, power_threshold, clarity_threshold, size, padding }
    }
}

impl PitchFindTrait for McleodPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64> {
        let mut mcleod = McLeodDetector::new(self.size, self.padding);
        let pitch = mcleod.get_pitch(data, self.sample_rate, self.power_threshold, self.clarity_threshold);
        if pitch.is_some() {
            return Some(pitch.unwrap().frequency);
        }
        return None
    }
}

struct FftPitchDetector {
    stream: Stream,
}

impl FftPitchDetector {
    fn new() -> FftPitchDetector {
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

        FftPitchDetector {stream}
    }
}

impl PitchFindTrait for FftPitchDetector {
    fn maybe_find_pitch(&mut self, data: &[f64]) -> Option<f64> {
        let vec: Vec<f32> = data.iter().map(|&x| x as f32).collect();

        self.stream.push_data(vec);
        self.stream.update();
        
        let mut hvol :f32 = 0.0;
        let mut highest :f32 = 0.0;

        let frequencies = self.stream.get_frequencies();
        for (_, frequency) in frequencies.iter().enumerate() {
            for item in frequency {
                if item.volume > hvol {
                    hvol = item.volume;
                    highest = item.freq;
                }
            }
        }
        return Some(highest as f64);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read config.cfg
    let f = std::fs::File::open("config.yaml")?;
    let config: Config = serde_yaml::from_reader(f)?;
    println!("{:?}", config);
    
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("failed to find input device");
    let supported_config = device.default_input_config().unwrap();

    let buffer_size = 1024;
    let stream_config: StreamConfig = 
        StreamConfig {
            channels: 1,
            sample_rate: supported_config.sample_rate(),
            buffer_size: cpal::BufferSize::Fixed(buffer_size),
        };
    
    let sample_rate = stream_config.sample_rate.0 as usize;
    let detector: Box<dyn PitchFindTrait>;

    match config.pitch_detection.as_str() {
        "yin" => {
            let yin = YinPitchDetector::new(
                config.threshold, 
                config.freq_min, 
                config.freq_max, 
                sample_rate);
            detector = Box::new(yin);
        } 
        "mcleod" => {
            let mcleod = McleodPitchDetector::new(
                buffer_size as usize, 
                (buffer_size / 2) as usize, 
                sample_rate, 
                config.power_threshold, 
                config.clarity_threshold);
            detector = Box::new(mcleod);
        }
        "fft" => {
            let fft = FftPitchDetector::new();
            detector = Box::new(fft);
        }
        _ => panic!("Invalid pitch detection method"),
    };
    
    
    match supported_config.sample_format() {
        cpal::SampleFormat::F32 => detect_from_input_stream::<f32>(&device, &stream_config.into(), detector),
        cpal::SampleFormat::I16 => detect_from_input_stream::<i16>(&device, &stream_config.into(), detector),
        cpal::SampleFormat::U16 => detect_from_input_stream::<u16>(&device, &stream_config.into(), detector),
    }

    Ok(())
}

fn detect_from_input_stream<T: Sample>(device: &Device, config: &StreamConfig, mut detector: Box<dyn PitchFindTrait>) {
    let err_fn = |err| println!("{}", err);
    
    let stream = device
        .build_input_stream(
            &config,
            move |data: &[T], _| {
                let f64_vals: Vec<f64> = data.iter().map(|x| x.to_f32() as f64).collect();
                let freq = (*detector).maybe_find_pitch(&f64_vals);
                if freq != None {
                    let s_and_f = find_string_and_distance(freq.unwrap());
                    output(freq.unwrap(), s_and_f.0, s_and_f.1, s_and_f.2);
                }
            },
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();
    loop {}
}

fn find_string_and_distance(freq: f64) -> (f64, f64, String) {
    let mut min_distance = std::f64::INFINITY;
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
    return (string_freq, min_distance, string_key);
}

fn output(freq:f64, string_freq:f64, distance:f64, string_key:String) {
    let mut corr = "".to_string();
    if distance.abs() > 0.9 {
        let dir = if distance < 0.0 {">"} else {"<"};
        corr = format!(" --- Correction: {} {:.1}", dir, distance);
    }

    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.queue(cursor::SavePosition).unwrap();
    stdout.write_all(format!("Detected frequency: {:.1} --- Closes to string {}:{} {}", freq, string_key, string_freq, corr).as_bytes()).unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
}