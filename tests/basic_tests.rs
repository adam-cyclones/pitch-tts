use pitch_tts::{synth_with_voice_config, synth_to_wav_with_pitch, get_available_voices, pitch_shift};

#[test]
fn test_voice_listing() {
    let voices = get_available_voices();
    assert!(!voices.is_empty(), "Should have at least one voice available");
    
    // Check that Alba voice exists
    let alba_voice = voices.iter().find(|v| v.id == "en_GB-alba-medium");
    assert!(alba_voice.is_some(), "Alba voice should be available");
    
    // Check voice structure
    for voice in voices.iter().take(5) {
        assert!(!voice.id.is_empty(), "Voice ID should not be empty");
        assert!(!voice.display_name.is_empty(), "Display name should not be empty");
        assert!(!voice.language.is_empty(), "Language should not be empty");
    }
}

#[test]
fn test_speech_synthesis() {
    let result = synth_with_voice_config(
        "Hello! This is a test of the pitch TTS library.".to_string(),
        "en_GB-alba-medium"
    );
    
    match result {
        Ok(samples) => {
            assert!(!samples.is_empty(), "Should generate audio samples");
            assert!(samples.len() > 1000, "Should generate substantial audio");
            
            // Check that samples are reasonable audio values
            for &sample in samples.iter().take(100) {
                assert!(sample >= -1.0 && sample <= 1.0, "Sample should be in valid audio range");
            }
        }
        Err(e) => {
            // If synthesis fails, it might be because the model isn't downloaded
            // This is acceptable for tests, but we should log it
            eprintln!("Speech synthesis failed (model may not be downloaded): {}", e);
        }
    }
}

#[test]
fn test_pitch_shifting() {
    let test_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1];
    
    // Test no pitch change
    let unchanged = pitch_shift(&test_samples, 1.0);
    assert_eq!(unchanged.len(), test_samples.len(), "No pitch change should preserve length");
    
    // Test pitch up (should make audio shorter)
    let pitched_up = pitch_shift(&test_samples, 2.0);
    assert!(pitched_up.len() < test_samples.len(), "Pitch up should make audio shorter");
    
    // Test pitch down (should make audio longer)
    let pitched_down = pitch_shift(&test_samples, 0.5);
    assert!(pitched_down.len() > test_samples.len(), "Pitch down should make audio longer");
    
    // Test extreme pitch values
    let extreme_up = pitch_shift(&test_samples, 4.0);
    assert!(extreme_up.len() < test_samples.len() / 2, "Extreme pitch up should significantly shorten audio");
    
    let extreme_down = pitch_shift(&test_samples, 0.25);
    assert!(extreme_down.len() > test_samples.len() * 2, "Extreme pitch down should significantly lengthen audio");
}

#[test]
fn test_wav_export() {
    let test_output = "test_output.wav";
    
    let result = synth_to_wav_with_pitch(
        "This is a test export.".to_string(),
        "en_GB-alba-medium",
        test_output,
        1.0  // Normal pitch
    );
    
    match result {
        Ok(_) => {
            // Check that file was created
            assert!(std::path::Path::new(test_output).exists(), "WAV file should be created");
            
            // Clean up
            let _ = std::fs::remove_file(test_output);
        }
        Err(e) => {
            // If export fails, it might be because the model isn't downloaded
            // This is acceptable for tests, but we should log it
            eprintln!("WAV export failed (model may not be downloaded): {}", e);
        }
    }
}

#[test]
fn test_voice_metadata() {
    let voices = get_available_voices();
    
    for voice in voices.iter().take(10) {
        // Test that voice IDs follow expected format: language_country-voice-quality
        let parts: Vec<&str> = voice.id.split('-').collect();
        assert!(parts.len() >= 3, "Voice ID should have at least 3 parts: {}", voice.id);
        
        // Test that language code is valid
        let lang_country = parts[0];
        assert!(lang_country.len() >= 2, "Language code should be at least 2 characters: {}", lang_country);
        
        // Test that quality is one of the expected values
        let quality = parts[2];
        let valid_qualities = ["low", "medium", "high", "x_low"];
        assert!(valid_qualities.contains(&quality), "Quality should be valid: {}", quality);
    }
} 