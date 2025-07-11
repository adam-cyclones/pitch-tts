use clap::{Parser, Subcommand};
use commands::export::handle_export;
use commands::list::handle_list;
use commands::say::handle_say;
use pitch_tts::{synth_with_voice_config, PitchArg};
use rodio::buffer::SamplesBuffer;
use std::str::FromStr;


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

        /// Also run WhisperX and print lipsync JSON to terminal
        #[arg(long)]
        lipsync: bool,
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

        /// Also run WhisperX and output lipsync JSON
        #[arg(long)]
        lipsync: bool,

        /// Output JSON file for lipsync data (default: output.json, only used if --lipsync is set)
        #[arg(long, default_value = "output.json")]
        json_output: String,
    },
}

fn run_whisperx_on_wav(wav_path: &str, output_json: Option<&str>) {
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
    println!("[WhisperX] Running whisperx on {}...", wav_path);
    let whisperx_result = std::process::Command::new("whisperx")
        .arg(wav_path)
        .arg("--output_dir")
        .arg(".")
        .arg("--output_format")
        .arg("json")
        .arg("--compute_type")
        .arg("float32")
        .output();
    match whisperx_result {
        Ok(result) => {
            if !result.status.success() {
                eprintln!("WhisperX failed: {}", String::from_utf8_lossy(&result.stderr));
            } else {
                let whisperx_json = format!("{}.json", wav_path);
                if std::path::Path::new(&whisperx_json).exists() {
                    if let Some(json_path) = output_json {
                        if let Err(e) = std::fs::copy(&whisperx_json, json_path) {
                            eprintln!("Failed to copy WhisperX output: {}", e);
                        } else {
                            println!("[WhisperX] Lipsync JSON written to {}", json_path);
                        }
                    } else {
                        let json = std::fs::read_to_string(&whisperx_json).unwrap();
                        println!("[WhisperX] Lipsync JSON:\n{}", json);
                    }
                    let _ = std::fs::remove_file(&whisperx_json);
                } else {
                    eprintln!("WhisperX output JSON not found: {}", whisperx_json);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to run whisperx: {}", e);
        }
    }
}

mod commands {
    pub mod list;
    pub mod say;
    pub mod export;
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::List { by_language }) => handle_list(*by_language),
        Some(Commands::Say { voice, text, pitch, lipsync }) => handle_say(voice, text, pitch, *lipsync),
        Some(Commands::Export { voice, output, text, pitch, lipsync, json_output }) => handle_export(voice, output, text, pitch, *lipsync, json_output),
        None => {
            // Show help by default instead of playing audio
            if cli.voice.is_some() || cli.text.is_some() {
                // If voice or text is provided, play audio (legacy behavior)
                let voice_id = cli.voice.as_deref().unwrap_or("en_GB-alba-medium");
                let text = cli.text.as_deref().unwrap_or("Hello! I'm playing audio from memory directly with piper-rs.");
                println!("Using voice: {}", voice_id);
                let _pitch_factor = cli.pitch.as_ref().map(|p| p.as_factor()).unwrap_or(1.0);
                match synth_with_voice_config(text.to_string(), voice_id) {
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
                println!("    say      Synthesize speech and play it");
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