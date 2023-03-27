#[macro_use]
extern crate vst;

use std::f32::consts::PI;
use std::sync::Arc;
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{Category, Info, Plugin};

const MAX_VOICES: usize = 128;

struct TriangleSynth {
    voices: Vec<Voice>,
    sample_rate: f32,
}


impl Plugin for TriangleSynth {
    fn get_info(&self) -> Info {
        Info {
            name: "TriangleSynth".to_string(),
            unique_id: 135798642,
            inputs: 0,
            outputs: 2,
            category: Category::Synth,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        for voice in self.voices.iter_mut() {
            voice.sample_rate = rate;
        }
    }

    fn process_events(&mut self, events: &vst::api::Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => {
                    let status = ev.data[0] & 0xF0;
                    let note = ev.data[1] as f32;
                    let velocity = ev.data[2] as f32 / 127.0;
                    match status {
                        0x90 => {
                            if velocity > 0.0 {
                                if let Some(voice) = self.voices.iter_mut().find(|v| !v.active) {
                                    voice.note_on(note, velocity);
                                }
                            } else {
                                for voice in self.voices.iter_mut().filter(|v| v.active && v.note == note) {
                                    voice.note_off();
                                }
                            }
                        }
                        0x80 => {
                            for voice in self.voices.iter_mut().filter(|v| v.active && v.note == note) {
                                voice.note_off();
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut output_buffer) = buffer.split();
        let num_samples = output_buffer.len();

        // Clear the output buffer
        for output_channel in output_buffer.into_iter() {
            for output_sample in output_channel {
                *output_sample = 0.0;
            }
        }

        for voice in &mut self.voices {
            // Skip processing if the voice is not active
            if !voice.active {
                continue;
            }

            let mut voice_output = vec![0.0; num_samples];
            for (i, output) in voice_output.iter_mut().enumerate() {
                *output = voice.next_sample();
            }

            for (output_channel, output) in output_buffer.into_iter().enumerate() {
                for (output_sample, voice_sample) in output.iter_mut().zip(voice_output.iter()) {
                    *output_sample += *voice_sample / MAX_VOICES as f32;
                }
            }
        }
    }
}

plugin_main!(TriangleSynth);

struct Voice {
    active: bool,
    note: f32,
    phase: f32,
    velocity: f32,
    sample_rate: f32,
}

impl Voice {
    fn new(sample_rate: f32) -> Self {
        Voice {
            active: false,
            note: 0.0,
            phase: 0.0,
            velocity: 0.0,
            sample_rate,
        }
    }

    fn note_on(&mut self, note: f32, velocity: f32) {
        self.active = true;
        self.note = note;
        self.velocity = velocity;
        self.phase = 0.0;
    }

    fn note_off(&mut self) {
        self.active = false;
    }

    fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        let freq = 440.0 * (2.0f32).powf((self.note - 69.0) / 12.0);
        let value = 2.0 * self.phase - 1.0;

        self.phase += freq / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        value * self.velocity
    }
}

impl Default for TriangleSynth {
    fn default() -> Self {
        let sample_rate = 44100.0;
        TriangleSynth {
            voices: (0..MAX_VOICES).map(|_| Voice::new(sample_rate)).collect(),
            sample_rate,
        }
    }
}