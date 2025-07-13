use clap::{Parser, Subcommand};
use commands::export::handle_export;
use commands::list::handle_list;
use commands::say::handle_say;
use text_to_face::{synth_with_voice_config, PitchArg};
use rodio::buffer::SamplesBuffer;
use std::str::FromStr;
use text_to_face::LipsyncLevel;


#[derive(Parser)]
#[command(name = "text-to-face")]
#[command(about = "A text-to-speech tool with pitch shifting, voice selection, and lipsync generation")]
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

        /// Tempo factor (1.0 = normal, 2.0 = twice as slow, 0.5 = twice as fast)
        #[arg(long, default_value = "1.0", help = "Tempo factor (1.0 = normal, 2.0 = slower, 0.5 = faster)")]
        tempo: f32,

        /// Lipsync level: low (default) or high (adds ARPAbet phonemes)
        #[arg(long, value_enum, default_value = "low")]
        lipsync: LipsyncLevel,
    },
    
    /// Export speech to WAV file
    Export {
        /// Text to synthesize (defaults to a fun Scottish phrase)
        #[arg(default_value = "Well hello there! I'm Alba, your Scottish friend. How about we go for a wee walk in the highlands? The weather is absolutely bonnie today!")]
        text: String,
        
        /// Voice ID to use (defaults to en_GB-alba-medium)
        #[arg(short, long, default_value = "en_GB-alba-medium")]
        voice: String,
        
        /// Output WAV file path (auto-generated from text if not provided, saved to output_/ directory with output_ prefix)
        #[arg(short, long)]
        output: Option<String>,
        
        /// Pitch factor or preset (e.g. 1.2, slomo, deep, child, helium)
        #[arg(short, long, value_parser = PitchArg::from_str, default_value = "1.0", help = "Pitch factor (0.5 = octave down, 2.0 = octave up) or preset (slomo, deep, child, helium)")]
        pitch: PitchArg,

        /// Tempo factor (1.0 = normal, 2.0 = twice as slow, 0.5 = twice as fast)
        #[arg(long, default_value = "1.0", help = "Tempo factor (1.0 = normal, 2.0 = slower, 0.5 = faster)")]
        tempo: f32,

        /// Lipsync level: low (default) or high (adds ARPAbet phonemes)
        #[arg(long, value_enum, default_value = "low")]
        lipsync: LipsyncLevel,

        /// Ollama model for ARPAbet phoneme generation (e.g., llama4, llama3.2, mistral)
        /// Note: First-time use will download the model, which may take several minutes
        /// Uses CMUdict for known words, falls back to Ollama for unknown words
        #[arg(long, value_name = "MODEL", default_value = "llama4")]
        lipsync_with_llm: String,

        /// Output JSON file for lipsync data (default: output.json, saved to output_/ directory with output_ prefix, only used if --lipsync is set)
        #[arg(long, default_value = "output.json")]
        json_output: String,
    },
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
        Some(Commands::Say { voice, text, pitch, tempo, lipsync }) => handle_say(voice, text, pitch, *tempo, *lipsync),
        Some(Commands::Export { voice, output, text, pitch, tempo, lipsync, json_output, lipsync_with_llm }) => handle_export(voice, output.as_deref(), text, pitch, *tempo, *lipsync, json_output, lipsync_with_llm),
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
                println!("text-to-face - A text-to-speech tool with pitch shifting, voice selection, and lipsync generation");
                println!();
                println!("USAGE:");
                println!("    text-to-face <SUBCOMMAND>");
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
                println!("For more information on a specific command, try 'text-to-face <COMMAND> --help'");
            }
        }
    }
}