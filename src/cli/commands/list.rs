use text_to_face::{get_available_voices, get_voices_by_language, get_models_dir, Voice};
use serde_json;
use std::fs;

fn is_voice_installed(voice: &Voice) -> bool {
    let models_dir = get_models_dir();
    let model_path = models_dir.join(format!("{}.onnx", voice.id));
    let config_path = models_dir.join(format!("{}.onnx.json", voice.id));
    model_path.exists() && config_path.exists()
}

pub fn handle_list(by_language: bool, as_json: bool, installed: bool, not_installed: bool) {
    let all_voices = get_available_voices();
    let filtered_voices: Vec<Voice> = if installed {
        all_voices.iter().cloned().filter(|v| is_voice_installed(v)).collect()
    } else if not_installed {
        all_voices.iter().cloned().filter(|v| !is_voice_installed(v)).collect()
    } else {
        all_voices.clone()
    };

    if as_json {
        if by_language {
            // Group filtered voices by language
            use std::collections::HashMap;
            let mut by_lang: HashMap<String, Vec<Voice>> = HashMap::new();
            for v in &filtered_voices {
                by_lang.entry(v.language.clone()).or_default().push(v.clone());
            }
            println!("{}", serde_json::to_string_pretty(&by_lang).unwrap());
        } else {
            println!("{}", serde_json::to_string_pretty(&filtered_voices).unwrap());
        }
        return;
    }

    if by_language {
        println!("Available voices by language:");
        use std::collections::HashMap;
        let mut by_lang: HashMap<String, Vec<Voice>> = HashMap::new();
        for v in &filtered_voices {
            by_lang.entry(v.language.clone()).or_default().push(v.clone());
        }
        for (language, voices) in by_lang.iter() {
            println!("\n{}:", language);
            for voice in voices {
                println!("  {} - {}", voice.id, voice.quality);
            }
        }
    } else {
        if installed {
            println!("Installed voices:");
        } else if not_installed {
            println!("Not installed voices:");
        } else {
            println!("Available voices:");
        }
        for voice in &filtered_voices {
            println!("  {} - {} ({})", voice.id, voice.display_name, voice.language);
        }
    }
} 