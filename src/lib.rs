use piper_rs::synth::PiperSpeechSynthesizer;
use std::fs;
use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone)]
pub struct Voice {
    pub id: String,
    pub display_name: String,
    pub language: String,
    pub quality: String,
    pub model_path: String,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phoneme {
    pub phoneme: String,
    pub start_time: f32,
    pub end_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LipSyncData {
    pub phonemes: Vec<Phoneme>,
    pub duration: f32,
    pub sample_rate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RhubarbOutput {
    mouth_cues: Vec<RhubarbMouthCue>,
    duration: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RhubarbMouthCue {
    value: String,
    start: f32,
    end: f32,
}

const HF_BASE: &str = "https://huggingface.co/rhasspy/piper-voices/resolve/main";

/// Pitch shift function using simple resampling
pub fn pitch_shift(samples: &[f32], pitch_factor: f32) -> Vec<f32> {
    if (pitch_factor - 1.0).abs() < 0.01 {
        return samples.to_vec(); // No shift needed
    }
    
    let original_length = samples.len();
    let new_length = (original_length as f32 / pitch_factor) as usize;
    let mut shifted = Vec::with_capacity(new_length);
    
    for i in 0..new_length {
        let pos = i as f32 * pitch_factor;
        let pos_floor = pos.floor() as usize;
        let pos_ceil = (pos.ceil() as usize).min(original_length - 1);
        let fraction = pos - pos.floor();
        
        if pos_floor < original_length {
            let sample = if pos_floor == pos_ceil {
                samples[pos_floor]
            } else {
                samples[pos_floor] * (1.0 - fraction) + samples[pos_ceil] * fraction
            };
            shifted.push(sample);
        }
    }
    
    shifted
}

/// Get all available voices
pub fn get_available_voices() -> Vec<Voice> {
    let mut voices = Vec::new();
    
    // Helper function to add a voice
    let mut add_voice = |id: &str, language: &str, quality: &str| {
        let display_name = format!("{} ({})", id.replace("_", " "), quality);
        
        // Parse the voice ID to get the correct path structure
        // Format: language_country-voice-quality (e.g., en_GB-alba-medium)
        let parts: Vec<&str> = id.split('-').collect();
        if parts.len() >= 3 {
            let lang_country = parts[0]; // e.g., "en_GB"
            let voice_name = parts[1];   // e.g., "alba"
            let quality = parts[2];      // e.g., "medium"
            
            let lang = &lang_country[..2]; // e.g., "en"
            let country = &lang_country[3..]; // e.g., "GB"
            
            let model_path = format!("{}/{}/{}_{}/{}/{}/{}.onnx", 
                HF_BASE, lang, lang, country, voice_name, quality, id);
            let config_path = format!("{}/{}/{}_{}/{}/{}/{}.onnx.json", 
                HF_BASE, lang, lang, country, voice_name, quality, id);
            
            voices.push(Voice {
                id: id.to_string(),
                display_name,
                language: language.to_string(),
                quality: quality.to_string(),
                model_path,
                config_path,
            });
        }
    };

    // English voices
    add_voice("en_GB-alba-medium", "Scottish English", "medium");
    add_voice("en_GB-alan-low", "British English", "low");
    add_voice("en_GB-alan-medium", "British English", "medium");
    add_voice("en_GB-aru-medium", "British English", "medium");
    add_voice("en_GB-cori-high", "British English", "high");
    add_voice("en_GB-cori-medium", "British English", "medium");
    add_voice("en_GB-jenny_dioco-medium", "British English", "medium");
    add_voice("en_GB-northern_english_male-medium", "Northern English", "medium");
    add_voice("en_GB-semaine-medium", "British English", "medium");
    add_voice("en_GB-southern_english_female-low", "Southern English", "low");
    add_voice("en_GB-vctk-medium", "British English", "medium");
    
    add_voice("en_US-amy-low", "US English", "low");
    add_voice("en_US-amy-medium", "US English", "medium");
    add_voice("en_US-arctic-medium", "US English", "medium");
    add_voice("en_US-danny-low", "US English", "low");
    add_voice("en_US-hfc_female-medium", "US English", "medium");
    add_voice("en_US-hfc_male-medium", "US English", "medium");
    add_voice("en_US-joe-medium", "US English", "medium");
    add_voice("en_US-kathleen-low", "US English", "low");
    add_voice("en_US-kristin-medium", "US English", "medium");
    add_voice("en_US-kusal-medium", "US English", "medium");
    add_voice("en_US-l2arctic-medium", "US English", "medium");
    add_voice("en_US-lessac-high", "US English", "high");
    add_voice("en_US-lessac-low", "US English", "low");
    add_voice("en_US-lessac-medium", "US English", "medium");
    add_voice("en_US-libritts-high", "US English", "high");
    add_voice("en_US-libritts_r-medium", "US English", "medium");
    add_voice("en_US-ljspeech-high", "US English", "high");
    add_voice("en_US-ljspeech-medium", "US English", "medium");
    add_voice("en_US-ryan-high", "US English", "high");
    add_voice("en_US-ryan-low", "US English", "low");
    add_voice("en_US-ryan-medium", "US English", "medium");
    add_voice("en_US-bryce-medium", "US English", "medium");
    add_voice("en_US-john-medium", "US English", "medium");
    add_voice("en_US-norman-medium", "US English", "medium");

    // German voices
    add_voice("de_DE-eva_k-x_low", "German", "x_low");
    add_voice("de_DE-karlsson-low", "German", "low");
    add_voice("de_DE-kerstin-low", "German", "low");
    add_voice("de_DE-mls-medium", "German", "medium");
    add_voice("de_DE-pavoque-low", "German", "low");
    add_voice("de_DE-ramona-low", "German", "low");
    add_voice("de_DE-thorsten-high", "German", "high");
    add_voice("de_DE-thorsten-low", "German", "low");
    add_voice("de_DE-thorsten-medium", "German", "medium");
    add_voice("de_DE-thorsten_emotional-medium", "German", "medium");

    // French voices
    add_voice("fr_FR-gilles-low", "French", "low");
    add_voice("fr_FR-mls-medium", "French", "medium");
    add_voice("fr_FR-mls_1840-low", "French", "low");
    add_voice("fr_FR-siwis-low", "French", "low");
    add_voice("fr_FR-siwis-medium", "French", "medium");
    add_voice("fr_FR-tom-medium", "French", "medium");
    add_voice("fr_FR-upmc-medium", "French", "medium");

    // Spanish voices
    add_voice("es_ES-carlfm-x_low", "Spanish", "x_low");
    add_voice("es_ES-davefx-medium", "Spanish", "medium");
    add_voice("es_ES-mls_10246-low", "Spanish", "low");
    add_voice("es_ES-mls_9972-low", "Spanish", "low");
    add_voice("es_ES-sharvard-medium", "Spanish", "medium");
    add_voice("es_MX-ald-medium", "Mexican Spanish", "medium");
    add_voice("es_MX-claude-high", "Mexican Spanish", "high");

    // Italian voices
    add_voice("it_IT-riccardo-x_low", "Italian", "x_low");
    add_voice("it_IT-paola-medium", "Italian", "medium");

    // Russian voices
    add_voice("ru_RU-denis-medium", "Russian", "medium");
    add_voice("ru_RU-dmitri-medium", "Russian", "medium");
    add_voice("ru_RU-irina-medium", "Russian", "medium");
    add_voice("ru_RU-ruslan-medium", "Russian", "medium");

    voices
}

/// Get voices grouped by language
pub fn get_voices_by_language() -> HashMap<String, Vec<Voice>> {
    let voices = get_available_voices();
    let mut by_language: HashMap<String, Vec<Voice>> = HashMap::new();
    
    for voice in voices {
        by_language.entry(voice.language.clone()).or_insert_with(Vec::new).push(voice);
    }
    
    by_language
}

/// Download voice model and config files
pub fn download_voice_files(voice: &Voice) -> Result<(String, String), Box<dyn std::error::Error>> {
    let models_dir = Path::new("models");
    if !models_dir.exists() {
        fs::create_dir(models_dir)?;
    }
    
    let model_filename = format!("{}.onnx", voice.id);
    let config_filename = format!("{}.onnx.json", voice.id);
    let model_path = models_dir.join(&model_filename);
    let config_path = models_dir.join(&config_filename);

    if !model_path.exists() {
        println!("Downloading {} voice model...", voice.display_name);
        let output = Command::new("curl")
            .arg("-L").arg("-o").arg(&model_path).arg(&voice.model_path)
            .output()?;
        if !output.status.success() {
            return Err(format!("Failed to download {}: {}", voice.display_name, String::from_utf8_lossy(&output.stderr)).into());
        }
        println!("Successfully downloaded {}", voice.display_name);
    }
    
    if !config_path.exists() {
        println!("Downloading {} config...", voice.display_name);
        let output = Command::new("curl")
            .arg("-L").arg("-o").arg(&config_path).arg(&voice.config_path)
            .output()?;
        if !output.status.success() {
            return Err(format!("Failed to download config for {}: {}", voice.display_name, String::from_utf8_lossy(&output.stderr)).into());
        }
        println!("Successfully downloaded config for {}", voice.display_name);
    }
    
    Ok((model_path.to_string_lossy().to_string(), config_path.to_string_lossy().to_string()))
}

/// Synthesize speech with a specific voice
pub fn synth_with_voice_config(text: String, voice_id: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let voices = get_available_voices();
    let voice = voices.iter()
        .find(|v| v.id == voice_id)
        .ok_or_else(|| {
            let available = voices.iter().map(|v| v.id.as_str()).collect::<Vec<_>>().join(", ");
            format!("Voice '{}' not found. Available voices: {}", voice_id, available)
        })?;
    
    let (_model_path, config_path) = download_voice_files(voice)?;
    let model = piper_rs::from_config_path(config_path.as_ref())?;
    let synth = PiperSpeechSynthesizer::new(model)?;
    
    let mut samples: Vec<f32> = Vec::new();
    let audio = synth.synthesize_parallel(text, None)?;
    for result in audio {
        samples.append(&mut result?.into_vec());
    }
    
    Ok(samples)
}

/// Synthesize speech to WAV file with pitch shifting
pub fn synth_to_wav_with_pitch(text: String, voice_id: &str, output_path: &str, pitch_factor: f32) -> Result<(), Box<dyn std::error::Error>> {
    // Get the raw audio samples
    let samples = synth_with_voice_config(text, voice_id)?;
    
    // Apply pitch shift if needed
    let processed_samples = pitch_shift(&samples, pitch_factor);
    
    // Write to WAV file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(output_path, spec)?;
    
    for sample in processed_samples {
        // Convert f32 to i16 and clamp to valid range
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16)?;
    }
    
    writer.finalize()?;
    println!("WAV file written to {} with pitch factor {}", output_path, pitch_factor);
    Ok(())
}

/// Check if any models exist in the models directory
pub fn has_any_models() -> bool {
    let models_dir = Path::new("models");
    models_dir.exists() && models_dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false)
}

/// Initialize default voice (downloads Alba if no models exist)
pub fn initialize_default_voice() -> Result<(), Box<dyn std::error::Error>> {
    if !has_any_models() {
        println!("No models found. Downloading Alba (default voice)...");
        synth_with_voice_config("Initializing voice system.".to_string(), "en_GB-alba-medium")?;
        println!("Alba voice downloaded successfully!");
    }
    Ok(())
} 

/// Check if Rhubarb lip-sync is available
pub fn has_lip_sync() -> bool {
    cfg!(feature = "lip-sync") && check_rhubarb_installed()
}

/// Check if Rhubarb executable is installed and accessible
fn check_rhubarb_installed() -> bool {
    Command::new("rhubarb")
        .arg("--version")
        .output()
        .is_ok()
}

/// Extract phonemes from audio file using Rhubarb executable
#[cfg(feature = "lip-sync")]
pub fn extract_phonemes_from_audio(audio_path: &str) -> Result<LipSyncData, Box<dyn std::error::Error>> {
    if !check_rhubarb_installed() {
        return Err("Rhubarb not found. Please install Rhubarb from https://github.com/DanielSWolf/rhubarb-lip-sync".into());
    }
    
    // Run Rhubarb to extract phonemes
    let output = Command::new("rhubarb")
        .arg("-f")
        .arg("json")
        .arg("-o")
        .arg("temp_rhubarb_output.json")
        .arg(audio_path)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Rhubarb failed: {}", stderr).into());
    }
    
    // Read the JSON output
    let json_content = fs::read_to_string("temp_rhubarb_output.json")?;
    let rhubarb_output: RhubarbOutput = serde_json::from_str(&json_content)?;
    
    // Clean up temporary file
    let _ = fs::remove_file("temp_rhubarb_output.json");
    
    // Convert to our format
    let mut phonemes = Vec::new();
    for mouth_cue in rhubarb_output.mouth_cues {
        phonemes.push(Phoneme {
            phoneme: mouth_cue.value,
            start_time: mouth_cue.start,
            end_time: mouth_cue.end,
        });
    }
    
    Ok(LipSyncData {
        phonemes,
        duration: rhubarb_output.duration,
        sample_rate: 22050, // Standard sample rate for our TTS
    })
}

/// Extract phonemes from text using Rhubarb (requires audio synthesis first)
#[cfg(feature = "lip-sync")]
pub fn extract_phonemes_from_text(text: &str, voice_id: &str) -> Result<LipSyncData, Box<dyn std::error::Error>> {
    // First synthesize the audio
    let samples = synth_with_voice_config(text.to_string(), voice_id)?;
    
    // Save to temporary WAV file
    let temp_wav = "temp_phoneme_extraction.wav";
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = hound::WavWriter::create(temp_wav, spec)?;
    for sample in samples {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16)?;
    }
    writer.finalize()?;
    
    // Extract phonemes from the temporary file
    let result = extract_phonemes_from_audio(temp_wav)?;
    
    // Clean up temporary file
    let _ = fs::remove_file(temp_wav);
    
    Ok(result)
}

/// Generate lip-sync data with WAV export
#[cfg(feature = "lip-sync")]
pub fn synth_with_lip_sync(text: String, voice_id: &str, wav_output: &str, lip_sync_output: &str, pitch_factor: f32) -> Result<(), Box<dyn std::error::Error>> {
    // Generate WAV with pitch shifting
    synth_to_wav_with_pitch(text.clone(), voice_id, wav_output, pitch_factor)?;
    
    // Extract phonemes from the generated WAV
    let lip_sync_data = extract_phonemes_from_audio(wav_output)?;
    
    // Save lip-sync data to JSON
    let lip_sync_json = serde_json::to_string_pretty(&lip_sync_data)?;
    fs::write(lip_sync_output, lip_sync_json)?;
    
    println!("Generated WAV: {}", wav_output);
    println!("Generated lip-sync data: {}", lip_sync_output);
    println!("Duration: {:.2}s, Phonemes: {}", lip_sync_data.duration, lip_sync_data.phonemes.len());
    
    Ok(())
}

#[cfg(not(feature = "lip-sync"))]
pub fn extract_phonemes_from_audio(_audio_path: &str) -> Result<LipSyncData, Box<dyn std::error::Error>> {
    Err("Lip-sync feature not enabled. Build with --features lip-sync".into())
}

#[cfg(not(feature = "lip-sync"))]
pub fn extract_phonemes_from_text(_text: &str, _voice_id: &str) -> Result<LipSyncData, Box<dyn std::error::Error>> {
    Err("Lip-sync feature not enabled. Build with --features lip-sync".into())
}

#[cfg(not(feature = "lip-sync"))]
pub fn synth_with_lip_sync(_text: String, _voice_id: &str, _wav_output: &str, _lip_sync_output: &str, _pitch_factor: f32) -> Result<(), Box<dyn std::error::Error>> {
    Err("Lip-sync feature not enabled. Build with --features lip-sync".into())
} 