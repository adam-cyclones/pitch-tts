use pitch_tts::{PitchArg, synthesize_and_handle};

pub fn handle_say(voice: &str, text: &str, pitch: &PitchArg, tempo: f32, lipsync: bool) {
    println!("Playing voice: {} (pitch: {})", voice, pitch.as_factor());
    synthesize_and_handle(
        text,
        voice,
        pitch,
        tempo,
        None, // No output WAV
        true, // Play audio
        lipsync,
        None, // Print lipsync JSON to terminal if lipsync is true
    );
} 