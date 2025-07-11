use pitch_tts::{synth_to_wav_with_pitch, PitchArg};

pub fn handle_export(voice: &str, output: &str, text: &str, pitch: &PitchArg, lipsync: bool, json_output: &str) {
    let pitch_factor = pitch.as_factor();
    println!("Exporting voice: {} to {} (pitch: {})", voice, output, pitch_factor);
    match synth_to_wav_with_pitch(text.to_string(), voice, output, pitch_factor) {
        Ok(_) => {
            println!("Successfully exported to {}", output);
            if lipsync {
                crate::run_whisperx_on_wav(output, Some(json_output));
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
} 