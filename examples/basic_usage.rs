clearuse pitch_tts::{synth_with_voice_config, synth_to_wav_with_pitch, get_available_voices};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Pitch TTS Library Example");
    println!("========================");
    
    // List available voices
    let voices = get_available_voices();
    println!("Available voices: {}", voices.len());
    for voice in voices.iter().take(5) {
        println!("  {} - {} ({})", voice.id, voice.display_name, voice.language);
    }
    println!("  ... and {} more", voices.len() - 5);
    println!();
    
    // Synthesize speech to memory
    println!("Synthesizing speech...");
    let samples = synth_with_voice_config(
        "Hello! This is a test of the pitch TTS library.".to_string(),
        "en_GB-alba-medium"
    )?;
    println!("Generated {} audio samples", samples.len());
    
    // Export to WAV with pitch shifting
    println!("Exporting to WAV with pitch shifting...");
    synth_to_wav_with_pitch(
        "This is a higher pitched version of the same text.".to_string(),
        "en_GB-alba-medium",
        "example_output.wav",
        1.3  // 30% higher pitch
    )?;
    
    println!("Example completed successfully!");
    Ok(())
} 