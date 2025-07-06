use clap::{Parser, Subcommand, ValueEnum};
use std::str::FromStr;
use pitch_tts::{get_voices_by_language, synth_with_voice_config, synth_to_wav_with_pitch, initialize_default_voice};
use rodio::buffer::SamplesBuffer;

/// Pitch preset or custom value
#[derive(Clone, Debug)]
pub enum PitchArg {
    Value(f32),
    Preset(PitchPreset),
}

#[derive(Clone, Debug, ValueEnum)]
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
            PitchPreset::Child => 1.1,
            PitchPreset::Helium => 1.5,
        }
    }
}

impl FromStr for PitchArg {
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

#[derive(Parser)]
#[command(name = "pitch-tts")]
#[command(about = "A text-to-speech tool with pitch shifting and voice selection")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Voice ID to use (e.g., en_GB-alba-medium)
    #[arg(short, long)]
    voice: Option<String>,
    
    /// Text to synthesize
    #[arg(short, long)]
    text: Option<String>,
    
    /// Pitch factor or preset (e.g. 1.2, slomo, deep, child, helium)
    #[arg(long, value_parser = PitchArg::from_str, help = "Pitch factor (0.5 = octave down, 2.0 = octave up) or preset (slomo, deep, child, helium)")]
    pitch: Option<PitchArg>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available voices
    List {
        /// Group voices by language
        #[arg(short, long)]
        by_language: bool,
    },
    
    /// Synthesize speech and play it
    Say {
        /// Text to synthesize (defaults to a fun Scottish phrase)
        #[arg(default_value = "Well hello there! I'm Alba, your Scottish friend. How about we go for a wee walk in the highlands? The weather is absolutely bonnie today!")]
        text: String,
        
        /// Voice ID to use (defaults to en_GB-alba-medium)
        #[arg(short, long, default_value = "en_GB-alba-medium")]
        voice: String,
        
        /// Pitch factor or preset (e.g. 1.2, slomo, deep, child, helium)
        #[arg(short, long, value_parser = PitchArg::from_str, default_value = "1.0", help = "Pitch factor (0.5 = octave down, 2.0 = octave up) or preset (slomo, deep, child, helium)")]
        pitch: PitchArg,
    },
    
    /// Export speech to WAV file
    Export {
        /// Voice ID to use
        #[arg(short, long)]
        voice: String,
        
        /// Output WAV file path
        #[arg(short, long)]
        output: String,
        
        /// Text to synthesize
        #[arg(short, long)]
        text: String,
        
        /// Pitch factor or preset (e.g. 1.2, slomo, deep, child, helium)
        #[arg(short, long, value_parser = PitchArg::from_str, default_value = "1.0", help = "Pitch factor (0.5 = octave down, 2.0 = octave up) or preset (slomo, deep, child, helium)")]
        pitch: PitchArg,
    },
    
    /// Extract phonemes from audio file (requires lip-sync feature)
    #[cfg(feature = "lip-sync")]
    Phonemes {
        /// Input audio file (WAV format)
        #[arg(short, long)]
        input: String,
        
        /// Output JSON file for phoneme data
        #[arg(short, long, default_value = "phonemes.json")]
        output: String,
    },
    
    /// Generate lip-sync data with WAV export (requires lip-sync feature)
    #[cfg(feature = "lip-sync")]
    Lipsync {
        /// Voice ID to use
        #[arg(short, long)]
        voice: String,
        
        /// Text to synthesize
        #[arg(short, long)]
        text: String,
        
        /// Output WAV file path
        #[arg(short, long)]
        wav_output: String,
        
        /// Output JSON file for lip-sync data
        #[arg(short, long, default_value = "lipsync.json")]
        lipsync_output: String,
        
        /// Pitch factor or preset (e.g. 1.2, slomo, deep, child, helium)
        #[arg(short, long, value_parser = PitchArg::from_str, default_value = "1.0", help = "Pitch factor (0.5 = octave down, 2.0 = octave up) or preset (slomo, deep, child, helium)")]
        pitch: PitchArg,
    },
}

fn main() {
    let cli = Cli::parse();
    
    // Initialize default voice if needed
    if let Err(e) = initialize_default_voice() {
        eprintln!("Failed to initialize default voice: {}", e);
        return;
    }
    
    match &cli.command {
        Some(Commands::List { by_language }) => {
            if *by_language {
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
                let voices = pitch_tts::get_available_voices();
                for voice in voices {
                    println!("  {} - {} ({})", voice.id, voice.display_name, voice.language);
                }
            }
        }
        
        Some(Commands::Say { voice, text, pitch }) => {
            let pitch_factor = pitch.as_factor();
            println!("Playing voice: {} (pitch: {})", voice, pitch_factor);
            match synth_with_voice_config(text.clone(), voice) {
                Ok(samples) => {
                    let processed_samples = pitch_tts::pitch_shift(&samples, pitch_factor);
                    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                    let sink = rodio::Sink::try_new(&handle).unwrap();
                    let buf = SamplesBuffer::new(1, 22050, processed_samples);
                    sink.append(buf);
                    sink.sleep_until_end();
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        
        Some(Commands::Export { voice, output, text, pitch }) => {
            let pitch_factor = pitch.as_factor();
            println!("Exporting voice: {} to {} (pitch: {})", voice, output, pitch_factor);
            match synth_to_wav_with_pitch(text.clone(), voice, output, pitch_factor) {
                Ok(_) => println!("Successfully exported to {}", output),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        
        #[cfg(feature = "lip-sync")]
        Some(Commands::Phonemes { input, output }) => {
            use pitch_tts::extract_phonemes_from_audio;
            println!("Extracting phonemes from: {}", input);
            match extract_phonemes_from_audio(input) {
                Ok(lip_sync_data) => {
                    let json = serde_json::to_string_pretty(&lip_sync_data).unwrap();
                    std::fs::write(output, json).unwrap();
                    println!("Phonemes extracted to: {}", output);
                    println!("Duration: {:.2}s, Phonemes: {}", lip_sync_data.duration, lip_sync_data.phonemes.len());
                }
                Err(e) => eprintln!("Error extracting phonemes: {}", e),
            }
        }
        
        #[cfg(feature = "lip-sync")]
        Some(Commands::Lipsync { voice, text, wav_output, lipsync_output, pitch }) => {
            use pitch_tts::synth_with_lip_sync;
            let pitch_factor = pitch.as_factor();
            println!("Generating lip-sync data for: {} (pitch: {})", text, pitch_factor);
            match synth_with_lip_sync(text.clone(), voice, wav_output, lipsync_output, pitch_factor) {
                Ok(_) => println!("Lip-sync generation completed successfully!"),
                Err(e) => eprintln!("Error generating lip-sync data: {}", e),
            }
        }
        

        
        None => {
            // Show help by default instead of playing audio
            if cli.voice.is_some() || cli.text.is_some() {
                // If voice or text is provided, play audio (legacy behavior)
                let voice_id = cli.voice.unwrap_or_else(|| "en_GB-alba-medium".to_string());
                let text = cli.text.unwrap_or_else(|| "Hello! I'm playing audio from memory directly with piper-rs.".to_string());
                
                println!("Using voice: {}", voice_id);
                let _pitch_factor = cli.pitch.as_ref().map(|p| p.as_factor()).unwrap_or(1.0);
                match synth_with_voice_config(text, &voice_id) {
                    Ok(samples) => {
                        let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                        let sink = rodio::Sink::try_new(&handle).unwrap();
                        let buf = SamplesBuffer::new(1, 22050, samples);
                        sink.append(buf);
                        sink.sleep_until_end();
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else {
                // Show help by default
                println!("pitch-tts - A text-to-speech tool with pitch shifting and voice selection");
                println!();
                println!("USAGE:");
                println!("    pitch-tts <SUBCOMMAND>");
                println!();
                println!("SUBCOMMANDS:");
                println!("    list     List all available voices");
                println!("    play     Synthesize speech and play it");
                println!("    export   Export speech to WAV file");
                println!("    help     Print this message or the help of the given subcommand(s)");
                println!();
                println!("OPTIONS:");
                println!("    -h, --help       Print help");
                println!("    -V, --version    Print version");
                println!();
                println!("For more information on a specific command, try 'pitch-tts <COMMAND> --help'");
            }
        }
    }
}

/*
Usage examples:

1. List all voices:
   pitch-tts list
   pitch-tts list --by-language

2. Play speech:
   pitch-tts play --voice en_GB-alba-medium --text "Hello world!"
   pitch-tts play -v en_GB-alba-medium -t "Hello world!"

3. Export to WAV:
   pitch-tts export --voice en_GB-alba-medium --output test.wav --text "Hello world!" --pitch 1.2
   pitch-tts export -v en_GB-alba-medium -o test.wav -t "Hello world!" -p 1.2

4. Legacy behavior (still works):
   pitch-tts --voice en_US-libritts_r-medium --text "Custom text"

For Blender integration, use the pitch_tts library directly in your Rust code.
*/