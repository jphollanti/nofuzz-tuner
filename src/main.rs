use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::*;
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::io::{stdout, Write};

use nofuzz_tuner_lib::Config;
use nofuzz_tuner_lib::FftPitchDetector;
use nofuzz_tuner_lib::McleodPitchDetector;
use nofuzz_tuner_lib::PitchFindTrait;
use nofuzz_tuner_lib::YinPitchDetector;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("config.yaml")?;
    let config: Config = serde_yaml::from_reader(f)?;
    println!("{:?}", config);

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("failed to find input device");
    let supported_config = device.default_input_config().unwrap();

    let buffer_size = 1024;
    let stream_config: StreamConfig = StreamConfig {
        channels: 1,
        sample_rate: supported_config.sample_rate(),
        buffer_size: cpal::BufferSize::Fixed(buffer_size),
    };

    let sample_rate = stream_config.sample_rate.0 as usize;
    let detector: Box<dyn PitchFindTrait> = match config.pitch_detection.as_str() {
        "yin" => {
            let yin = YinPitchDetector::new(
                config.threshold,
                config.freq_min,
                config.freq_max,
                sample_rate,
            );
            Box::new(yin)
        }
        "mcleod" => {
            let mcleod = McleodPitchDetector::new(
                buffer_size as usize,
                (buffer_size / 2) as usize,
                sample_rate,
                config.power_threshold,
                config.clarity_threshold,
            );
            Box::new(mcleod)
        }
        "fft" => {
            let fft = FftPitchDetector::new();
            Box::new(fft)
        }
        _ => panic!("Invalid pitch detection method"),
    };

    match supported_config.sample_format() {
        cpal::SampleFormat::F32 => {
            detect_from_input_stream::<f32>(&device, &stream_config, detector)
        }
        cpal::SampleFormat::I16 => {
            detect_from_input_stream::<i16>(&device, &stream_config, detector)
        }
        cpal::SampleFormat::U16 => {
            detect_from_input_stream::<u16>(&device, &stream_config, detector)
        }
    }

    Ok(())
}

fn detect_from_input_stream<T: Sample>(
    device: &Device,
    config: &StreamConfig,
    mut detector: Box<dyn PitchFindTrait>,
) {
    // const TUNING: &str = "standard-e";
    // const TUNING: &str = "flat-e";
    const TUNING: &str = "drop-d";
    let err_fn = |err| println!("{}", err);

    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let f64_vals: Vec<f64> = data.iter().map(|x| x.to_f32() as f64).collect();
                let freq = (*detector).maybe_find_pitch(&f64_vals, TUNING);
                if freq.is_some() {
                    let res = freq.unwrap();
                    let tt = res.tuning_to();
                    output(res.freq(), tt.cents(), tt.freq(), tt.distance(), tt.note());
                }
            },
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn output(freq: f64, cents: f64, string_freq: f64, distance: f64, string_key: String) {
    let mut corr = "".to_string();
    if distance.abs() > 0.9 {
        let dir = if distance < 0.0 { ">" } else { "<" };
        corr = format!(" --- Correction: {} {:.1}", dir, distance);
    }

    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();
    stdout.queue(cursor::SavePosition).unwrap();
    stdout
        .write_all(
            format!(
                "Detected frequency: {:.1}, cents: {:.2} --- Closest to string {}:{} {}",
                freq, cents, string_key, string_freq, corr
            )
            .as_bytes(),
        )
        .unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout.flush().unwrap();
    stdout.queue(cursor::RestorePosition).unwrap();
    stdout
        .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
        .unwrap();
}
