use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::*;
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};
use std::io::{Write, stdout};
use serde_yaml;

use nofuzz_tuner::Config;
use nofuzz_tuner::PitchFindTrait;
use nofuzz_tuner::YinPitchDetector;
use nofuzz_tuner::McleodPitchDetector;
use nofuzz_tuner::FftPitchDetector;
use nofuzz_tuner::find_string_and_distance;

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

fn output(freq:f64, string_freq:f64, distance:f64, string_key:String) {
    let mut corr = "".to_string();
    if distance.abs() > 0.9 {
        let dir = if distance < 0.0 {">"} else {"<"};
        corr = format!(" --- Correction: {} {:.1}", dir, distance);
    }

    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.queue(cursor::SavePosition).unwrap();
    stdout.write_all(format!("Detected frequency: {:.1} --- Closest to string {}:{} {}", freq, string_key, string_freq, corr).as_bytes()).unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
}