use audioviz::io::{Input, Device};
use audioviz::spectrum::{config::{StreamConfig, ProcessorConfig, VolumeNormalisation, PositionNormalisation, Interpolation}, stream::Stream};
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

fn main() {
    // captures audio from system using cpal
    let mut audio_input = Input::new();
    let (_, _, input_controller) = audio_input.init(&Device::DefaultInput).unwrap();

    // initialize console output
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();

    // spectrum visualizer stream
    let mut stream: Stream = Stream::new(StreamConfig {
        channel_count: 2,
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

    loop {
        if let Some(data) = input_controller.pull_data() {
            stream.push_data(data);
            stream.update();
        }

        let mut hvol :f32 = 0.0;
        let mut highest :f32 = 0.0;

        let frequencies = stream.get_frequencies();
        for (_, frequency) in frequencies.iter().enumerate() {
            //println!("{}: {:?}", i, frequency);    
            for item in frequency {
                if item.volume > hvol {
                    hvol = item.volume;
                    highest = item.freq;
                }
            }
        }

        // Guitar string frequencies cheat-sheet:
        // E2: 82.41 Hz
        // A2: 110.00 Hz
        // D3: 146.83 Hz
        // G3: 196.00 Hz
        // B3: 246.94 Hz
        // E4: 329.63 Hz
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(format!("Frequency: {} ", highest).as_bytes()).unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
        
        //println!("Frequencies: {:?}", frequencies);
        //break;
    }
    
    //stdout.execute(cursor::Show).unwrap();
    //println!("Done!");
}
