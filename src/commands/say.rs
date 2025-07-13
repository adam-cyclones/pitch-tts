use text_to_face::{PitchArg, synthesize_and_handle};
use crate::LipsyncLevel;

pub fn handle_say(voice: &str, text: &str, pitch: &PitchArg, tempo: f32, lipsync: LipsyncLevel) {
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
        None, // lipsync_with_llm: not used in 'say' command
    );
} 