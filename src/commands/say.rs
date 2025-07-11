use pitch_tts::{synth_with_voice_config, PitchArg};
use rodio::buffer::SamplesBuffer;

pub fn handle_say(voice: &str, text: &str, pitch: &PitchArg, lipsync: bool) {
    let pitch_factor = pitch.as_factor();
    println!("Playing voice: {} (pitch: {})", voice, pitch_factor);
    match synth_with_voice_config(text.to_string(), voice) {
        Ok(samples) => {
            let processed_samples = pitch_tts::pitch_shift(&samples, pitch_factor);
            let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&handle).unwrap();
            let buf = SamplesBuffer::new(1, 22050, processed_samples.as_slice());
            sink.append(buf);
            sink.sleep_until_end();

            if lipsync {
                // Save to a temp WAV file
                let temp_wav = "temp_lipsync_say.wav";
                let spec = hound::WavSpec {
                    channels: 1,
                    sample_rate: 22050,
                    bits_per_sample: 16,
                    sample_format: hound::SampleFormat::Int,
                };
                let mut writer = hound::WavWriter::create(temp_wav, spec).unwrap();
                for sample in &processed_samples {
                    let sample_i16 = (*sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
                    writer.write_sample(sample_i16).unwrap();
                }
                writer.finalize().unwrap();
                crate::run_whisperx_on_wav(temp_wav, None);
                let _ = std::fs::remove_file(temp_wav);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
} 