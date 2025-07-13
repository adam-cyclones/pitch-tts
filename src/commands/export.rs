use pitch_tts::{PitchArg, synthesize_and_handle};
use std::fs;
use std::path::Path;

pub fn handle_export(voice: &str, output: Option<&str>, text: &str, pitch: &PitchArg, tempo: f32, lipsync: bool, json_output: &str) {
    // Determine base name for folder (from text or custom filename)
    let (folder_base, wav_filename) = if let Some(path) = output {
        let filename = Path::new(path).file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("output.wav");
        let stem = Path::new(filename).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        (clean_for_folder(stem), filename.to_string())
    } else {
        let filename = generate_filename_from_text(text);
        let stem = Path::new(&filename).file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        (clean_for_folder(stem), filename)
    };
    let output_dir = format!("output_{}", folder_base);
    if !Path::new(&output_dir).exists() {
        if let Err(e) = fs::create_dir(&output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            return;
        }
    }
    let output_path = format!("{}/{}", output_dir, wav_filename);
    // JSON output filename
    let json_filename = Path::new(json_output).file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output.json");
    let json_output_path = if lipsync {
        format!("{}/{}", output_dir, json_filename)
    } else {
        json_output.to_string()
    };
    println!("Exporting voice: {} to {} (pitch: {}, tempo: {})", voice, output_path, pitch.as_factor(), tempo);
    synthesize_and_handle(
        text,
        voice,
        pitch,
        tempo,
        Some(&output_path), // Output WAV file
        false, // Do not play audio
        lipsync,
        if lipsync { Some(&json_output_path) } else { None }, // Save lipsync JSON if requested
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