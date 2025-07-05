use clap::{Parser, Subcommand};
use pitch_tts::{get_voices_by_language, synth_with_voice_config, synth_to_wav_with_pitch, initialize_default_voice};
use rodio::buffer::SamplesBuffer;

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
        
        /// Pitch factor (0.5 = octave down, 2.0 = octave up)
        #[arg(short, long, default_value = "1.0")]
        pitch: f32,
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
        
        /// Pitch factor (0.5 = octave down, 2.0 = octave up)
        #[arg(short, long, default_value = "1.0")]
        pitch: f32,
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
            println!("Playing voice: {} (pitch: {})", voice, pitch);
            match synth_with_voice_config(text.clone(), voice) {
                Ok(samples) => {
                    let processed_samples = pitch_tts::pitch_shift(&samples, *pitch);
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
            println!("Exporting voice: {} to {}", voice, output);
            match synth_to_wav_with_pitch(text.clone(), voice, output, *pitch) {
                Ok(_) => println!("Successfully exported to {}", output),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        
        None => {
            // Show help by default instead of playing audio
            if cli.voice.is_some() || cli.text.is_some() {
                // If voice or text is provided, play audio (legacy behavior)
                let voice_id = cli.voice.unwrap_or_else(|| "en_GB-alba-medium".to_string());
                let text = cli.text.unwrap_or_else(|| "Hello! I'm playing audio from memory directly with piper-rs.".to_string());
                
                println!("Using voice: {}", voice_id);
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