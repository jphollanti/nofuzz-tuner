use audioviz::io::{Input, Device};
use audioviz::spectrum::{Frequency, config::{StreamConfig, ProcessorConfig, Interpolation}, stream::Stream};
use std::{thread, time};
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};

fn main() {
    // captures audio from system using cpal
    let mut audio_input = Input::new();
    let (channel_count, sampling_rate, input_controller) = audio_input.init(&Device::DefaultInput).unwrap();

    // initialize console output
    let mut stdout = stdout();
    stdout.execute(cursor::Hide).unwrap();

    // spectrum visualizer stream
    let mut stream: Stream = Stream::new(StreamConfig::default()); 
    loop {
        if let Some(data) = input_controller.pull_data() {
            stream.push_data(data);
            stream.update();
        }

        let frequencies = stream.get_frequencies();
        for (i, frequency) in frequencies.iter().enumerate() {
            //println!("{}: {:?}", i, frequency);
            let mut hvol :f32 = 0.0;
            let mut highest :f32 = 0.0;
            for item in frequency {
                if item.volume > hvol {
                    hvol = item.volume;
                    highest = item.freq;
                }
            }
            
            stdout.queue(cursor::SavePosition).unwrap();
            stdout.write_all(format!("Frequency: {} ", highest).as_bytes()).unwrap();
            stdout.queue(cursor::RestorePosition).unwrap();
            stdout.flush().unwrap();
            stdout.queue(cursor::RestorePosition).unwrap();
            stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
        }
        
        //println!("Frequencies: {:?}", frequencies);

        //break;
    }
    
    stdout.execute(cursor::Show).unwrap();
    println!("Done!");

}
