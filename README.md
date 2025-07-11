# pitch-tts

A fast, flexible text-to-speech (TTS) CLI and Rust library powered by [piper-rs](https://github.com/rhasspy/piper). Features high-quality voice synthesis with real-time pitch shifting and support for 50+ voices across multiple languages.

## ✨ Features

- **🎯 Dynamic Voice Selection** - Auto-downloads models from HuggingFace with 50+ voices across 6 languages
- **🎵 Real-time Pitch Shifting** - Adjust pitch from 0.5x (octave down) to 2.0x (octave up) in real-time
- **💾 WAV Export** - Export synthesized speech to WAV files for integration with Blender, video editors, and other tools
- **🔧 Comprehensive CLI** - Intuitive subcommands for listing voices, speaking, exporting, and generating lipsync JSON with WhisperX
- **🌍 Multi-language Support** - English (US/UK), German, French, Spanish, Italian, and Russian voices
- **⚡ Fast Synthesis** - Built on piper-rs for efficient, high-quality speech generation
- **📚 Rust Library** - Use as a library for custom TTS workflows and Blender integration
- **🎬 WhisperX Lip-sync Support** - Generate animation-ready JSON with accurate word/phoneme timings (requires WhisperX)

## 🚀 Installation

### Prerequisites
- Rust 1.70+ and Cargo
- Internet connection (for downloading voice models)
- **For lipsync JSON:** [WhisperX](https://github.com/m-bain/whisperX) must be installed and available in your PATH

### Install WhisperX (for lipsync JSON)
WhisperX is a Python tool. Install it via pip:

```bash
pip install git+https://github.com/m-bain/whisperx.git
```

Or see the [WhisperX repo](https://github.com/m-bain/whisperX) for more details.

### Build from Source
```bash
git clone https://github.com/yourusername/pitch-tts.git
cd pitch-tts
cargo build --release
```

### First Run
On first run, pitch-tts will automatically download the default voice model (en_GB-alba-medium). Voice models are cached in the `models/` directory.

## 📖 Usage

### List Available Voices
```bash
cargo run -- list
```

### Speak Text
```bash
# Use defaults (Scottish Alba voice with fun phrase)
cargo run -- say

# Custom text with default voice
cargo run -- say "Hello, world!"

# Custom voice and text
cargo run -- say "Bonjour!" --voice fr_FR-gilles-low

# With pitch shifting (1.5x = higher pitch)
cargo run -- say "High pitched voice" --pitch 1.5

# With pitch preset (helium, child, deep, slomo)
cargo run -- say "Helium voice!" --pitch helium
cargo run -- say "Child voice!" --pitch child
cargo run -- say "Deep voice!" --pitch deep
cargo run -- say "Slow motion!" --pitch slomo

# Combine all options
cargo run -- say "Custom message" --voice en_US-amy-medium --pitch 0.8
```

### Export to WAV
```bash
cargo run -- export --voice en_GB-alba-medium --output hello.wav --text "Hello world!"
```

### Generate Lipsync JSON for Animation (WhisperX)
```bash
cargo run -- lipsync \
  --voice en_GB-alba-medium \
  --text "Hello, this is a test for WhisperX!" \
  --wav-output hello.wav \
  --json-output lipsync.json \
  --pitch 1.2
```
- This will create `hello.wav` and a WhisperX JSON file (`lipsync.json`) with word/phoneme timing data for animation.
- **Requires WhisperX to be installed and in your PATH.**

### Legacy Mode (Quick Commands)
```bash
cargo run -- --voice en_US-libritts_r-medium --text "Quick mode!"
```

## 📚 Library Usage

Use `pitch_tts` as a Rust library for custom TTS workflows:

```rust
use pitch_tts::synth_to_wav_with_pitch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    synth_to_wav_with_pitch(
        "Hello, this is a test!".to_string(),
        "en_GB-alba-medium",
        "hello.wav",
        1.0, // pitch factor
    )?;
    Ok(())
}
```

## 🔧 Configuration

### Voice Model Storage
Voice models are automatically downloaded and stored in the `models/` directory:
```
models/
├── en_GB-alba-medium.onnx
├── en_GB-alba-medium.onnx.json
├── en_US-amy-medium.onnx
└── ...
```

### Pitch Shifting
- **Range**: 0.5x to 2.0x (octave down to octave up)
- **Default**: 1.0x (no change)
- **Algorithm**: Linear interpolation resampling
- **Presets**:
  - `slomo`: 0.4 (slow motion)
  - `deep`: 0.85 (deeper voice)
  - `child`: 1.1 (child-like voice)
  - `helium`: 1.5 (chipmunk/helium effect)

### WhisperX JSON Output
- When using the `lipsync` command, the output JSON will contain word and phoneme timing data as produced by WhisperX. See the [WhisperX repo](https://github.com/m-bain/whisperX) for details on the format.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup
```bash
git clone https://github.com/yourusername/pitch-tts.git
cd pitch-tts
cargo build
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [piper-rs](https://github.com/rhasspy/piper) - The core TTS engine
- [Rhasspy](https://rhasspy.readthedocs.io/) - Voice models and inspiration
- [HuggingFace](https://huggingface.co/) - Voice model hosting
- [WhisperX](https://github.com/m-bain/whisperX) - Word/phoneme alignment for animation 