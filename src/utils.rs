use std::io::{self, Read};
use std::fs::File;
use std::path::Path;
use hound::WavReader;

pub fn load_wav_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<f32>> {
    let mut reader = io::BufReader::new(File::open(path)?);
    let spec = hound::WavSpec::load(&mut reader)?;
    let samples = WavReader::new(&mut reader).unwrap().samples::<i16>();
    let sample_rate = spec.sample_rate as f32;
    let channel_count = spec.channels as usize;
    let mut output = Vec::new();

    for sample in samples {
        let sample = sample.unwrap();
        for channel in 0..channel_count {
            let index = channel as usize;
            if index >= output.len() {
                output.push(Vec::new());
            }
            output[index].push(sample[channel] as f32 / std::i16::MAX as f32);
        }
    }

    if output.is_empty() {
        output.push(Vec::new());
    }

    let channel_count = output.len();

    for channel in 0..channel_count {
        let samples = &mut output[channel];
        let mut i = 0;
        while i < samples.len() {
            let mut max = 0.0;
            for j in i..(i + spec.sample_rate as usize / 10) {
                if let Some(sample) = samples.get(j) {
                    let val = sample.abs();
                    if val > max {
                        max = val;
                    }
                }
            }
            for j in i..(i + spec.sample_rate as usize / 10) {
                if let Some(sample) = samples.get_mut(j) {
                    *sample = *sample / max;
                }
            }
            i += spec.sample_rate as usize / 10;
        }
    }

    Ok(output.into_iter().flatten().collect())
}
