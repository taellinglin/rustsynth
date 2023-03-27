#[macro_use]
extern crate vst;
extern crate lazy_static;
extern crate hound;
extern crate walkdir;
extern crate log;
extern crate simple_logger;
use log::info;
use simple_logger::SimpleLogger;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::path::Path;
use walkdir::WalkDir;
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{Category, HostCallback, Info, Plugin};

const SAMPLES_FOLDER: &str = "/samples/";

lazy_static! {
    static ref SAMPLES_DATA: Mutex<Vec<Vec<f32>>> = Mutex::new(load_samples(SAMPLES_FOLDER));
}

fn load_samples<P: AsRef<Path>>(folder_path: P) -> Vec<Vec<f32>> {
    let mut samples = Vec::new();
    for entry in WalkDir::new(folder_path) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "wav" {
                    let mut reader = match hound::WavReader::open(path) {
                        Ok(reader) => reader,
                        Err(_) => continue,
                    };

                    let sample_data: Vec<f32> = reader
                        .samples::<i16>()
                        .filter_map(Result::ok)
                        .map(|s| s as f32 / i16::MAX as f32)
                        .collect();

                    samples.push(sample_data);
                }
            }
        }
    }

    samples
}

struct PlayingSample {
    sample_data: Vec<f32>,
    position: usize,
}

struct SampleVst {
    host: HostCallback,
    playing_samples: Vec<PlayingSample>,
}

impl Default for SampleVst {
    fn default() -> Self {
        SimpleLogger::new().init().unwrap();
        Self {
            host: HostCallback::default(),
            playing_samples: Vec::new(),
        }
    }
}

impl Plugin for SampleVst {
    fn get_info(&self) -> Info {
        Info {
            name: "SampleVst".to_string(),
            vendor: "MyVendor".to_string(),
            unique_id: 1357,
            inputs: 0,
            outputs: 2,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, _rate: f32) {
        // Update the sample rate if needed
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    if ev.data[0] == 144 {
                        let note = ev.data[1] as usize % 128;
                        info!("MIDI Note On: {}", note);
                        let samples = SAMPLES_DATA.lock().unwrap();
                        if note < samples.len() {
                            self.playing_samples.push(PlayingSample {
                                sample_data: samples[note].clone(),
                                position: 0,
                            });
                        }
                    }
                }
                _ => (),
            }
        }
    }
    

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut output_buffer) = buffer.split();

        for (output_channel, data) in output_buffer.into_iter().enumerate() {
            for output_sample in data.iter_mut() {
                *output_sample = 0.0;
                let mut i = 0;
                while i < self.playing_samples.len() {
                    let playing_sample = &mut self.playing_samples[i];
                    if playing_sample.position < playing_sample.sample_data.len() {
                        let sample = playing_sample.sample_data[playing_sample.position];
                        *output_sample += sample;
                        playing_sample.position += 1;
                        i += 1;
                    } else {
                        self.playing_samples.swap_remove(i);
                    }
                }
                
            }
        }
    }
}
plugin_main!(SampleVst);