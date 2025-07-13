use piper_rs::synth::PiperSpeechSynthesizer;
use std::fs;
use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rubato::{FftFixedIn, Resampler};


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

#[derive(Clone, Debug)]
pub enum PitchArg {
    Value(f32),
    Preset(PitchPreset),
}

#[derive(Clone, Debug)]
pub enum PitchPreset {
    Slomo,
    Deep,
    Child,
    Helium,
}

impl PitchPreset {
    pub fn factor(&self) -> f32 {
        match self {
            PitchPreset::Slomo => 0.4,
            PitchPreset::Deep => 0.85,
            PitchPreset::Child => 1.2,
            PitchPreset::Helium => 1.5,
        }
    }
}

impl std::str::FromStr for PitchArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(val) = s.parse::<f32>() {
            Ok(PitchArg::Value(val))
        } else {
            match s.to_lowercase().as_str() {
                "slomo" => Ok(PitchArg::Preset(PitchPreset::Slomo)),
                "deep" => Ok(PitchArg::Preset(PitchPreset::Deep)),
                "child" => Ok(PitchArg::Preset(PitchPreset::Child)),
                "helium" => Ok(PitchArg::Preset(PitchPreset::Helium)),
                _ => Err(format!("Invalid pitch value or preset: {}", s)),
            }
        }
    }
}

impl PitchArg {
    pub fn as_factor(&self) -> f32 {
        match self {
            PitchArg::Value(v) => *v,
            PitchArg::Preset(p) => p.factor(),
        }
    }
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

/// High-quality pitch shift without speed change using SoX executable
pub fn true_pitch_shift(samples: &[f32], sample_rate: usize, pitch_factor: f32) -> Vec<f32> {
    if (pitch_factor - 1.0).abs() < 0.01 {
        return samples.to_vec();
    }
    
    // Create temporary input and output files
    let temp_input = "temp_input.wav";
    let temp_output = "temp_output.wav";
    
    // Write input samples to WAV file
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(temp_input, spec).expect("Failed to create temp WAV");
    for sample in samples {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize WAV");
    
    // Calculate pitch shift in cents (1200 cents per octave)
    let cents = 1200.0 * pitch_factor.log2();
    
    // Use sox executable to pitch shift while keeping tempo the same
    let output = Command::new("sox")
        .arg(temp_input)
        .arg(temp_output)
        .arg("pitch")
        .arg(&format!("{}", cents))
        .output()
        .expect("Failed to execute sox");
    
    if !output.status.success() {
        eprintln!("SoX error: {}", String::from_utf8_lossy(&output.stderr));
        // Clean up temp files
        let _ = std::fs::remove_file(temp_input);
        return samples.to_vec(); // Return original samples on error
    }
    
    // Read the processed audio back
    let reader = hound::WavReader::open(temp_output).expect("Failed to open output WAV");
    let samples: Vec<f32> = reader.into_samples::<i16>()
        .map(|s| s.expect("Failed to read sample") as f32 / 32767.0)
        .collect();
    
    // Clean up temp files
    let _ = std::fs::remove_file(temp_input);
    let _ = std::fs::remove_file(temp_output);
    
    samples
}

/// Time-stretch function using rubato (tempo_factor > 1.0 = slower, < 1.0 = faster)
pub fn time_stretch(samples: &[f32], sample_rate: usize, tempo_factor: f32) -> Vec<f32> {
    if (tempo_factor - 1.0).abs() < 0.01 {
        return samples.to_vec(); // No stretch needed
    }
    let channels = 1;
    let input_frame_length = 1024;
    let _output_frame_length = (input_frame_length as f32 * tempo_factor) as usize;
    let _fft_size = input_frame_length * 2;
    let mut resampler = FftFixedIn::<f32>::new(
        sample_rate, // sample_rate_input
        (sample_rate as f32 / tempo_factor) as usize, // sample_rate_output
        input_frame_length, // chunk_size_in
        1, // sub_chunks
        channels, // nbr_channels
    ).expect("Failed to create resampler");
    let mut output = Vec::new();
    let mut pos = 0;
    while pos < samples.len() {
        let end = (pos + input_frame_length).min(samples.len());
        let mut chunk = samples[pos..end].to_vec();
        if chunk.len() < input_frame_length {
            chunk.resize(input_frame_length, 0.0);
        }
        let input = vec![chunk];
        let result = resampler.process(&input, None).expect("Resample failed");
        output.extend_from_slice(&result[0]);
        pos += input_frame_length;
    }
    output
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

/// Synthesize speech to WAV file with pitch shifting and tempo adjustment
pub fn synth_to_wav_with_pitch(text: String, voice_id: &str, output_path: &str, pitch_factor: f32, tempo: f32) -> Result<(), Box<dyn std::error::Error>> {
    // Get the raw audio samples
    let samples = synth_with_voice_config(text, voice_id)?;
    // Apply pitch shift if needed
    let processed_samples = pitch_shift(&samples, pitch_factor);
    // Apply time stretch if needed
    let processed_samples = time_stretch(&processed_samples, 22050, tempo);
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
    println!("WAV file written to {} with pitch factor {} and tempo {}", output_path, pitch_factor, tempo);
    Ok(())
} 

/// Synthesize, process, and optionally export/play and lipsync.
/// - If `output_wav` is Some(path), writes to WAV.
/// - If `play_audio` is true, plays the audio.
/// - If `lipsync_json` is Some(path), runs WhisperX and saves JSON there; if None and lipsync is true, prints JSON.
pub fn synthesize_and_handle(
    text: &str,
    voice: &str,
    pitch: &PitchArg,
    tempo: f32,
    output_wav: Option<&str>,
    play_audio: bool,
    lipsync: bool,
    lipsync_json: Option<&str>,
) {
    let pitch_factor = pitch.as_factor();
    let samples = match synth_with_voice_config(text.to_string(), voice) {
        Ok(samples) => samples,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    // Use high-quality pitch shift
    let processed_samples = true_pitch_shift(&samples, 22050, pitch_factor);
    let processed_samples = time_stretch(&processed_samples, 22050, tempo);

    // Write to WAV if requested
    if let Some(wav_path) = output_wav {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(wav_path, spec).unwrap();
        for sample in &processed_samples {
            let sample_i16 = (*sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_sample(sample_i16).unwrap();
        }
        writer.finalize().unwrap();
        println!("WAV file written to {} with pitch factor {} and tempo {}", wav_path, pitch_factor, tempo);
    }

    // Play audio if requested
    if play_audio {
        if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                let buf = rodio::buffer::SamplesBuffer::new(1, 22050, processed_samples.as_slice());
                sink.append(buf);
                sink.sleep_until_end();
            }
        }
    }

    // Lipsync (WhisperX) if requested
    if lipsync {
        // Use the WAV file if it was just written, otherwise write a temp WAV
        let wav_path = if let Some(wav_path) = output_wav {
            wav_path
        } else {
            let temp_wav = "temp_lipsync.wav";
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
            temp_wav
        };
        // Call run_whisperx_on_wav (from main.rs)
        // This requires the function to be public and imported in the command modules
        run_whisperx_on_wav(wav_path, lipsync_json);
        // Clean up temp file if needed
        if output_wav.is_none() {
            let _ = std::fs::remove_file(wav_path);
        }
    }
} 

/// Run WhisperX on a WAV file, optionally saving output JSON to a file or printing it.
pub fn run_whisperx_on_wav(wav_path: &str, output_json: Option<&str>) {
    use std::env;
    // Check for whisperx
    let whisperx_available = std::process::Command::new("whisperx")
        .arg("--help")
        .output()
        .is_ok();
    if !whisperx_available {
        eprintln!("\n[WhisperX] Error: 'whisperx' executable not found in your PATH.");
        eprintln!("To use the --lipsync flag, you must install WhisperX:");
        eprintln!("  python3 -m pip install git+https://github.com/m-bain/whisperx.git");
        eprintln!("See: https://github.com/m-bain/whisperX\n");
        return;
    }

    // If output_json is provided, change to its directory
    let (run_dir, wav_filename, json_filename, restore_dir): (Option<String>, String, Option<String>, Option<String>) =
        if let Some(json_path) = output_json {
            let json_path = std::path::Path::new(json_path);
            let dir = json_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."));
            let json_filename = json_path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string());
            let wav_pathbuf = std::path::Path::new(wav_path);
            let wav_filename = wav_pathbuf.file_name().and_then(|n| n.to_str()).unwrap_or(wav_path).to_string();
            let orig_dir = env::current_dir().ok().map(|d| d.to_string_lossy().to_string());
            (Some(dir.to_string_lossy().to_string()), wav_filename, json_filename, orig_dir)
        } else {
            (None, wav_path.to_string(), None, None)
        };

    // Change directory if needed
    if let Some(ref dir) = run_dir {
        if let Err(e) = env::set_current_dir(dir) {
            eprintln!("Failed to change directory to {}: {}", dir, e);
            return;
        }
    }

    println!("[WhisperX] Running whisperx on {}...", wav_filename);
    println!("[WhisperX] Current working directory: {:?}", env::current_dir().unwrap_or_default());
    let whisperx_result = std::process::Command::new("whisperx")
        .arg(&wav_filename)
        .arg("--output_dir")
        .arg(".")
        .arg("--output_format")
        .arg("json")
        .arg("--compute_type")
        .arg("float32")
        .output();
    match whisperx_result {
        Ok(result) => {
            println!("[WhisperX] Command stdout: {}", String::from_utf8_lossy(&result.stdout));
            println!("[WhisperX] Command stderr: {}", String::from_utf8_lossy(&result.stderr));
            if result.status.success() {
                // WhisperX writes to <filename>.json (e.g., lipsync_test_phrase.json)
                let base = if let Some(stripped) = wav_filename.strip_suffix(".wav") {
                    stripped
                } else {
                    &wav_filename
                };
                println!("[WhisperX] Base filename: {}", base);
                let whisperx_json_path = format!("{}.json", base);
                println!("[WhisperX] Looking for output file: {}", whisperx_json_path);
                if !std::path::Path::new(&whisperx_json_path).exists() {
                    eprintln!("[WhisperX] Output JSON not found: {}", whisperx_json_path);
                    // List files in current directory to see what WhisperX actually created
                    if let Ok(entries) = std::fs::read_dir(".") {
                        println!("[WhisperX] Files in current directory:");
                        for entry in entries {
                            if let Ok(entry) = entry {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.ends_with(".json") {
                                        println!("  - {}", name);
                                    }
                                }
                            }
                        }
                    }
                    // Restore directory before returning
                    if let Some(ref orig) = restore_dir {
                        let _ = env::set_current_dir(orig);
                    }
                    return;
                }
                match json_filename {
                    Some(ref json_path) => {
                        // If the output file is not the expected name, rename it
                        if json_path != &whisperx_json_path {
                            match std::fs::rename(&whisperx_json_path, json_path) {
                                Ok(_) => {
                                    println!("[WhisperX] Lipsync JSON renamed to {}", json_path);
                                },
                                Err(e) => {
                                    if let Err(copy_err) = std::fs::copy(&whisperx_json_path, json_path) {
                                        eprintln!("Failed to copy WhisperX output: {}", copy_err);
                                    } else if let Err(remove_err) = std::fs::remove_file(&whisperx_json_path) {
                                        eprintln!("Failed to remove original WhisperX output: {}", remove_err);
                                    } else {
                                        println!("[WhisperX] Lipsync JSON copied to {}", json_path);
                                    }
                                }
                            }
                        } else {
                            println!("[WhisperX] Lipsync JSON written to {}", json_path);
                        }
                    }
                    None => {
                        println!("[WhisperX] Lipsync JSON written to {}", whisperx_json_path);
                    }
                }
            }
            else {
                eprintln!(
                    "[WhisperX] Failed with status {}:\n{}",
                    result.status,
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to run WhisperX: {}", e);
        }
    }
    // Restore original directory if changed
    if let Some(ref orig) = restore_dir {
        let _ = env::set_current_dir(orig);
    }
} 