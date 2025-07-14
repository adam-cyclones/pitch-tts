use piper_rs::synth::PiperSpeechSynthesizer;
use std::fs;
use std::process::Command;
use std::path::Path;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use rubato::{FftFixedIn, Resampler};

use clap::ValueEnum;
use colored::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum LipsyncLevel {
    Low,
    High,
}

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

/// Return type for ARPAbet lookup: (phonemes, method)
type ArpabetResult = (Vec<String>, &'static str);

/// Use Ollama to get ARPAbet phonemes for a word not found in CMUdict or g2p-en
fn get_arpabet_from_ollama(word: &str, model: &str) -> Option<Vec<String>> {
    // List of valid ARPAbet phonemes (no stress markers)
    const ARPABET: &[&str] = &[
        "AA", "AE", "AH", "AO", "AW", "AY", "B", "CH", "D", "DH", "EH", "ER", "EY", "F", "G", "HH", "IH", "IY", "JH", "K", "L", "M", "N", "NG", "OW", "OY", "P", "R", "S", "SH", "T", "TH", "UH", "UW", "V", "W", "Y", "Z", "ZH",
        // With stress markers
        "AA0", "AA1", "AA2", "AE0", "AE1", "AE2", "AH0", "AH1", "AH2", "AO0", "AO1", "AO2", "AW0", "AW1", "AW2", "AY0", "AY1", "AY2", "EH0", "EH1", "EH2", "ER0", "ER1", "ER2", "EY0", "EY1", "EY2", "IH0", "IH1", "IH2", "IY0", "IY1", "IY2", "OW0", "OW1", "OW2", "OY0", "OY1", "OY2", "UH0", "UH1", "UH2", "UW0", "UW1", "UW2"
    ];
    let valid: std::collections::HashSet<&str> = ARPABET.iter().copied().collect();

    let prompt = format!(
        "Give only the ARPAbet phonemes for the word '{}'. Respond ONLY with the ARPAbet phonemes, space-separated, no explanation, no punctuation, no extra words.\nExample: hello => HH AH0 L OW1\nNow, {} =>",
        word, word
    );
    
    match std::process::Command::new("ollama")
        .arg("run")
        .arg(model)
        .arg(&prompt)
        .output() {
        Ok(output) if output.status.success() => {
            let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !response.is_empty() && !response.contains("error") && !response.contains("not found") {
                let all_tokens: Vec<String> = response.split_whitespace().map(|s| s.to_string()).collect();
                let filtered: Vec<String> = all_tokens.iter().filter(|s| valid.contains(s.as_str())).cloned().collect();
                if filtered.is_empty() {
                    println!("[ARPAbet] {} => {:?} (from Ollama/{}, but no valid ARPAbet tokens)", word, all_tokens, model);
                } else if filtered.len() != all_tokens.len() {
                    println!("[ARPAbet] {} => {:?} (filtered from {:?}, Ollama/{})", word, filtered, all_tokens, model);
                } else {
                    println!("[ARPAbet] {} => {:?} (from Ollama/{})", word, filtered, model);
                }
                if !filtered.is_empty() {
                    return Some(filtered);
                }
            }
        }
        _ => {}
    }
    None
}

// Global cache for CMUdict - loaded once and reused
static CMUDICT_CACHE: Lazy<HashMap<String, Vec<Vec<String>>>> = Lazy::new(|| {
    println!("[ARPAbet] Loading CMUdict into memory...");
    let project_root = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
    } else {
        let mut current = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        loop {
            if current.join("Cargo.toml").exists() {
                break current;
            }
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            }
        }
    };
    
    let extra_dir = project_root.join("extra");
    let dict_path = extra_dir.join("cmudict-0.7b.txt");

    if !extra_dir.exists() {
        let _ = std::fs::create_dir(&extra_dir);
    }
    if !dict_path.exists() {
        println!("[ARPAbet] cmudict-0.7b.txt not found, downloading to extra/...");
        let url = "https://raw.githubusercontent.com/Alexir/CMUdict/master/cmudict-0.7b";
        match std::process::Command::new("curl").arg("-L").arg("-o").arg(&dict_path).arg(url).status() {
            Ok(status) if status.success() => println!("[ARPAbet] Downloaded cmudict-0.7b.txt to extra/"),
            _ => {
                eprintln!("[ARPAbet] Failed to download cmudict-0.7b.txt. Please download it manually.");
                return HashMap::new();
            }
        }
    }
    
    // Simple CMUdict parser - reads the file directly and returns a HashMap
    let bytes = match std::fs::read(&dict_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("[ARPAbet] Failed to read cmudict file: {}", e);
            return HashMap::new();
        }
    };
    
    // Try to convert to UTF-8, with fallback to lossy conversion
    let content = String::from_utf8_lossy(&bytes);
    
    // Validate that this looks like a CMUdict file
    if !content.contains(";;; # CMUdict") {
        eprintln!("[ARPAbet] File does not appear to be a valid CMUdict file");
        return HashMap::new();
    }
    
    let mut dict = HashMap::new();
    
    for line in content.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with(";;;") {
            continue;
        }
        
        // Parse word entries: WORD PH1 PH2 PH3...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        
        let word = parts[0];
        let phonemes: Vec<String> = parts[1..].iter().map(|&s| s.to_string()).collect();
        
        // Handle multiple pronunciations (e.g., WORD(1), WORD(2))
        let base_word = if word.contains('(') {
            word.split('(').next().unwrap_or(word)
        } else {
            word
        };
        
        dict.entry(base_word.to_string()).or_insert_with(Vec::new).push(phonemes);
    }
    
    println!("[ARPAbet] Loaded {} words from CMUdict (cached)", dict.len());
    dict
});

/// Given a text, return a Vec<(Vec<String>, &str)> of ARPAbet phonemes and method for each word.
/// Uses CMUdict for known words, falls back to g2p-en, then Ollama for unknown words.
pub fn text_to_arpabet_with_method(text: &str, lipsync_with_llm: Option<&str>) -> Vec<ArpabetResult> {
    let dict = &*CMUDICT_CACHE;
    text.split_whitespace()
        .map(|word| {
            let word_upper = word.trim_matches(|c: char| !c.is_alphanumeric()).to_uppercase();
            if let Some(pronunciations) = dict.get(&word_upper) {
                if let Some(first_pronunciation) = pronunciations.first() {
                    println!("{} {} => {:?} (from {})", "[ARPAbet]".cyan(), word_upper, first_pronunciation, "cmudict".bold().green());
                    (first_pronunciation.clone(), "cmudict")
                } else {
                    println!("{} {} => [] (no pronunciations)", "[ARPAbet]".yellow(), word_upper);
                    (vec![], "cmudict")
                }
            } else if let Some(model) = lipsync_with_llm {
                if !model.trim().is_empty() {
                    if let Some(llm_phonemes) = get_arpabet_from_ollama(&word_upper, model) {
                        println!("{} {} => {:?} (from {})", "[ARPAbet]".cyan(), word_upper, llm_phonemes, "llm".bold().magenta());
                        (llm_phonemes, "llm")
                    } else {
                        println!("{} {} => [] (not found in CMUdict or Ollama/{})", "[ARPAbet]".red(), word_upper, model);
                        eprintln!("{} All fallbacks failed for '{}'. Make sure Ollama is installed and the '{}' model is available:", "[ARPAbet]".red(), word_upper, model);
                        eprintln!("  brew install ollama");
                        eprintln!("  ollama pull {} (first-time download may take several minutes)", model);
                        eprintln!("  We recommend using 'llama3.2' for best ARPAbet accuracy.");
                        (vec![], "user_manual")
                    }
                } else {
                    eprintln!("{} No phoneme data for '{}'. Rerun with --lipsync-with-llm <model> to enable LLM fallback.", "[ARPAbet]".red(), word);
                    (vec![], "user_manual")
                }
            } else {
                eprintln!("{} No phoneme data for '{}'. Rerun with --lipsync-with-llm <model> to enable LLM fallback.", "[ARPAbet]".red(), word);
                (vec![], "user_manual")
            }
        })
        .collect()
}

/// For backward compatibility: just return the phonemes (no method)
pub fn text_to_arpabet(text: &str, lipsync_with_llm: Option<&str>) -> Vec<Vec<String>> {
    text_to_arpabet_with_method(text, lipsync_with_llm).into_iter().map(|(p, _)| p).collect()
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
        println!("{} voice model...", voice.display_name.yellow());
        let output = Command::new("curl")
            .arg("-L").arg("-o").arg(&model_path).arg(&voice.model_path)
            .output()?;
        if !output.status.success() {
            return Err(format!("Failed to download {}: {}", voice.display_name, String::from_utf8_lossy(&output.stderr)).into());
        }
        println!("{}", "Successfully downloaded".green());
    }
    
    if !config_path.exists() {
        println!("{} config...", voice.display_name.yellow());
        let output = Command::new("curl")
            .arg("-L").arg("-o").arg(&config_path).arg(&voice.config_path)
            .output()?;
        if !output.status.success() {
            return Err(format!("Failed to download config for {}: {}", voice.display_name, String::from_utf8_lossy(&output.stderr)).into());
        }
        println!("{}", "Successfully downloaded config for".green());
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
    println!("{} file written to {} with pitch factor {} and tempo {}", "WAV".green(), output_path, pitch_factor, tempo);
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
    lipsync: LipsyncLevel,
    lipsync_json: Option<&str>,
    lipsync_with_llm: Option<&str>,
) {
    let pitch_factor = pitch.as_factor();
    let samples = match synth_with_voice_config(text.to_string(), voice) {
        Ok(samples) => samples,
        Err(e) => {
            eprintln!("{}", "Error:".red());
            eprintln!("{}", e);
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
        println!("{} file written to {} with pitch factor {} and tempo {}", "WAV".green(), wav_path, pitch_factor, tempo);
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
    if lipsync != LipsyncLevel::Low {
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
        run_whisperx_on_wav(wav_path, lipsync_json, lipsync == LipsyncLevel::High, text, lipsync_with_llm);
        if output_wav.is_none() {
            let _ = std::fs::remove_file(wav_path);
        }
    }
} 

/// Run WhisperX on a WAV file, optionally saving output JSON to a file or printing it.
pub fn run_whisperx_on_wav(wav_path: &str, output_json: Option<&str>, hi_fidelity: bool, text: &str, lipsync_with_llm: Option<&str>) {
    use std::env;
    use serde_json::Value;
    // Check for whisperx
    let whisperx_available = std::process::Command::new("whisperx")
        .arg("--help")
        .output()
        .is_ok();
    if !whisperx_available {
        eprintln!("\n{} Error: 'whisperx' executable not found in your PATH.", "[WhisperX]".red());
        eprintln!("{} To use the --lipsync flag, you must install WhisperX:", "[WhisperX]".red());
        eprintln!("  python3 -m pip install git+https://github.com/m-bain/whisperx.git");
        eprintln!("{} See: https://github.com/m-bain/whisperX\n", "[WhisperX]".red());
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
            eprintln!("{} Failed to change directory to {}: {}", "[WhisperX]".red(), dir, e);
            return;
        }
    }

    println!("{} Running whisperx on {}...", "[WhisperX]".cyan(), wav_filename);
    println!("{} Current working directory: {:?}", "[WhisperX]".cyan(), env::current_dir().unwrap_or_default());
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
            println!("{} Command stdout: {}", "[WhisperX]".cyan(), String::from_utf8_lossy(&result.stdout));
            println!("{} Command stderr: {}", "[WhisperX]".red(), String::from_utf8_lossy(&result.stderr));
            if result.status.success() {
                // WhisperX writes to <filename>.json (e.g., lipsync_test_phrase.json)
                let base = if let Some(stripped) = wav_filename.strip_suffix(".wav") {
                    stripped
                } else {
                    &wav_filename
                };
                println!("{} Base filename: {}", "[WhisperX]".cyan(), base);
                let whisperx_json_path = format!("{}.json", base);
                println!("{} Looking for output file: {}", "[WhisperX]".cyan(), whisperx_json_path);
                if !std::path::Path::new(&whisperx_json_path).exists() {
                    eprintln!("{} Output JSON not found: {}", "[WhisperX]".red(), whisperx_json_path);
                    // List files in current directory to see what WhisperX actually created
                    if let Ok(entries) = std::fs::read_dir(".") {
                        println!("{} Files in current directory:", "[WhisperX]".cyan());
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
                                    println!("{} Lipsync JSON renamed to {}", "[WhisperX]".cyan(), json_path);
                                },
                                Err(_e) => {
                                    if let Err(copy_err) = std::fs::copy(&whisperx_json_path, json_path) {
                                        eprintln!("{} Failed to copy WhisperX output: {}", "[WhisperX]".red(), copy_err);
                                    } else if let Err(remove_err) = std::fs::remove_file(&whisperx_json_path) {
                                        eprintln!("{} Failed to remove original WhisperX output: {}", "[WhisperX]".red(), remove_err);
                                    } else {
                                        println!("{} Lipsync JSON copied to {}", "[WhisperX]".cyan(), json_path);
                                    }
                                }
                            }
                        } else {
                            println!("{} Lipsync JSON written to {}", "[WhisperX]".cyan(), json_path);
                        }
                    }
                    None => {
                        println!("{} Lipsync JSON written to {}", "[WhisperX]".cyan(), whisperx_json_path);
                    }
                }
                // Hi-fidelity: add ARPAbet if requested
                if hi_fidelity {
                    if let Some(ref json_path) = json_filename {
                        if let Ok(mut json_value) = std::fs::read_to_string(json_path).and_then(|s| serde_json::from_str::<Value>(&s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))) {
                            // Get ARPAbet for each word
                            let arpabet_dict = text_to_arpabet_with_method(text, lipsync_with_llm);
                            
                            // Add phonemes to each word segment
                            if let Some(word_segments) = json_value.get_mut("word_segments") {
                                if let Some(word_segments_array) = word_segments.as_array_mut() {
                                    for (i, word_segment) in word_segments_array.iter_mut().enumerate() {
                                        if let Some(word_obj) = word_segment.as_object_mut() {
                                            if let Some(_word) = word_obj.get("word").and_then(|w| w.as_str()) {
                                                if let Some((phonemes, method)) = arpabet_dict.get(i) {
                                                    word_obj.insert("phonemes".to_string(), serde_json::to_value(phonemes).unwrap_or(Value::Null));
                                                    word_obj.insert("phoneme_method".to_string(), serde_json::to_value(method).unwrap_or(Value::Null));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            let _ = std::fs::write(json_path, serde_json::to_string_pretty(&json_value).unwrap());
                            println!("{} Added ARPAbet phonemes to word segments in {}", "[HiFidelity]".cyan(), json_path);
                        }
                    }
                }
            }
            else {
                eprintln!(
                    "{} Failed with status {}:\n{}",
                    "[WhisperX]".red(),
                    result.status,
                    String::from_utf8_lossy(&result.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("{} Failed to run WhisperX: {}", "[WhisperX]".red(), e);
        }
    }
    // Restore original directory if changed
    if let Some(ref orig) = restore_dir {
        let _ = env::set_current_dir(orig);
    }
} 