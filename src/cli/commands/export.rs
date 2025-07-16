use text_to_face::{PitchArg, synthesize_and_handle};
use std::fs;
use std::path::Path;
use crate::LipsyncLevel;

pub fn handle_export(voice: &str, output: Option<&str>, text: &str, pitch: &PitchArg, tempo: f32, lipsync: LipsyncLevel, json_output: &str, lipsync_with_llm: Option<String>) {
    use std::path::PathBuf;
    let (wav_path, json_path): (PathBuf, PathBuf) = if let Some(path) = output {
        let p = Path::new(path);
        if p.extension().map(|e| e == "wav").unwrap_or(false) {
            // --output is a file path
            let wav = p.to_path_buf();
            let base = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let dir = p.parent().unwrap_or_else(|| Path::new("."));
            let json = dir.join(format!("{}.json", base));
            (wav, json)
        } else {
            // --output is a directory
            let dir = p;
            let filename = generate_filename_from_text(text);
            let base = Path::new(&filename).file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let wav = dir.join(&filename);
            let json = dir.join(format!("{}.json", base));
            (wav, json)
        }
    } else {
        // No output specified, use CWD
        let filename = generate_filename_from_text(text);
        let base = Path::new(&filename).file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let wav = Path::new(&filename).to_path_buf();
        let json = Path::new(&format!("{}.json", base)).to_path_buf();
        (wav, json)
    };
    // Ensure output directory exists
    if let Some(parent) = wav_path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create output directory: {}", e);
                return;
            }
        }
    }
    println!("Exporting voice: {} to {:?} (pitch: {}, tempo: {})", voice, wav_path, pitch.as_factor(), tempo);
    synthesize_and_handle(
        text,
        voice,
        pitch,
        tempo,
        Some(wav_path.to_str().unwrap()), // Output WAV file
        false, // Do not play audio
        lipsync,
        if lipsync != LipsyncLevel::Low { Some(json_path.to_str().unwrap()) } else { None },
        lipsync_with_llm.as_deref(),
    );
}

/// Clean a string for use as a folder name (alphanumeric and underscores only)
fn clean_for_folder(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

/// Generate a filename from text by taking the first few words and cleaning them
fn generate_filename_from_text(text: &str) -> String {
    // Take first 30 characters, clean them, and add .wav extension
    let cleaned: String = text
        .chars()
        .take(30)
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let words: Vec<&str> = cleaned.split_whitespace().take(5).collect();
    let filename = if words.is_empty() {
        "output".to_string()
    } else {
        words.join("_").to_lowercase()
    };
    // Add .wav extension
    format!("{}.wav", filename)
} 