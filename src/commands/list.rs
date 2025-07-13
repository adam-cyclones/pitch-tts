use text_to_face::{get_available_voices, get_voices_by_language};

pub fn handle_list(by_language: bool) {
    if by_language {
        println!("Available voices by language:");
        let by_language = get_voices_by_language();
        for (language, voices) in by_language.iter() {
            println!("\n{}:", language);
            for voice in voices {
                println!("  {} - {}", voice.id, voice.quality);
            }
        }
    } else {
        println!("Available voices:");
        let voices = get_available_voices();
        for voice in voices {
            println!("  {} - {} ({})", voice.id, voice.display_name, voice.language);
        }
    }
} 