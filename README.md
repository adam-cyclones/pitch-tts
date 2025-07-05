# pitch-tts

A fast, flexible text-to-speech (TTS) CLI and Rust library powered by [piper-rs](https://github.com/rhasspy/piper). Features high-quality voice synthesis with real-time pitch shifting and support for 50+ voices across multiple languages.

## ✨ Features

- **🎯 Dynamic Voice Selection** - Auto-downloads models from HuggingFace with 50+ voices across 6 languages
- **🎵 Real-time Pitch Shifting** - Adjust pitch from 0.5x (octave down) to 2.0x (octave up) in real-time
- **💾 WAV Export** - Export synthesized speech to WAV files for integration with Blender, video editors, and other tools
- **🔧 Comprehensive CLI** - Intuitive subcommands for listing voices, speaking, and exporting
- **🌍 Multi-language Support** - English (US/UK), German, French, Spanish, Italian, and Russian voices
- **⚡ Fast Synthesis** - Built on piper-rs for efficient, high-quality speech generation
- **📚 Rust Library** - Use as a library for custom TTS workflows and Blender integration

## 🚀 Installation

### Prerequisites
- Rust 1.70+ and Cargo
- Internet connection (for downloading voice models)

### Build from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/pitch-tts.git
cd pitch-tts

# Build the release version
cargo build --release

# The binary will be available at target/release/pitch-tts
```

### First Run
On first run, pitch-tts will automatically download the default voice model (en_GB-alba-medium). Voice models are cached in the `models/` directory.

## 📖 Usage

### List Available Voices
```bash
# List all voices
cargo run -- list

# Group voices by language
cargo run -- list --by-language
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
# Export with default settings
cargo run -- export --voice en_GB-alba-medium --output hello.wav --text "Hello world!"

# Export with pitch shifting
cargo run -- export --voice en_US-libritts_r-medium --output high_pitch.wav --text "High pitched audio" --pitch 1.3

# Export for Blender integration
cargo run -- export --voice en_GB-alan-medium --output character_speech.wav --text "Character dialogue here" --pitch 1.0
```

### Legacy Mode (Quick Commands)
```bash
# Direct voice and text specification (legacy behavior)
cargo run -- --voice en_US-libritts_r-medium --text "Quick mode!"
```

## 🎵 Voice Examples

### English Voices
- **Scottish**: `en_GB-alba-medium` - Charming Scottish accent
- **British**: `en_GB-alan-medium` - Clear British English
- **US Male**: `en_US-ryan-medium` - Natural American male voice
- **US Female**: `en_US-amy-medium` - Clear American female voice

### International Voices
- **German**: `de_DE-thorsten-medium` - Natural German male voice
- **French**: `fr_FR-gilles-low` - French male voice
- **Spanish**: `es_ES-carlfm-x_low` - Spanish male voice
- **Italian**: `it_IT-riccardo-x_low` - Italian male voice
- **Russian**: `ru_RU-denis-medium` - Russian male voice

## 📚 Library Usage

Use `pitch_tts` as a Rust library for custom TTS workflows:

```rust
use pitch_tts::{synth_with_voice_config, pitch_shift, synth_to_wav_with_pitch};

// Synthesize speech
let samples = synth_with_voice_config(
    "Hello from the library!".to_string(), 
    "en_GB-alba-medium"
)?;

// Apply pitch shifting
let shifted_samples = pitch_shift(&samples, 1.2);

// Export to WAV
synth_to_wav_with_pitch(
    "Library test".to_string(),
    "en_US-amy-medium",
    "output.wav",
    0.9
)?;
```

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Run all tests (configured for single-threaded execution to prevent audio conflicts)
cargo test

# Run specific test file
cargo test --test cli_tests

# Run specific test
cargo test test_cli_say_output_validation

# Run with single thread (recommended for audio tests)
cargo test -- --test-threads=1
```

**Note**: Tests are configured to run single-threaded in `Cargo.toml` to prevent audio conflicts when multiple CLI tests play audio simultaneously.

### Test Categories
- **CLI Functionality**: Tests all command-line interface features
- **Audio Playback**: Validates actual audio synthesis and playback
- **Voice Management**: Tests voice listing and selection
- **Pitch Shifting**: Verifies pitch modification functionality
- **WAV Export**: Tests file export capabilities

## 🎯 Use Cases

### Content Creation
- Generate voice-overs for videos and podcasts
- Create character voices for games and animations
- Produce multilingual content with consistent quality

### Development & Testing
- Test audio applications and interfaces
- Generate sample audio for machine learning projects
- Create automated voice prompts for applications

### Blender Integration
- Export WAV files for lip-sync animation
- Generate character dialogue for 3D animations
- Create voice tracks for video projects

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
  - `slomo`: 0.7 (slow motion)
  - `deep`: 0.85 (deeper voice)
  - `child`: 1.1 (child-like voice)
  - `helium`: 1.5 (chipmunk/helium effect)

## 🛣️ Roadmap

- [ ] **Rhubarb Lip Sync Integration** - Phoneme extraction for animation
- [ ] **Advanced Audio Controls** - Tempo, timbre, and emotion modification
- [ ] **Voice Quality Auto-detection** - Automatic model quality selection
- [ ] **Batch Processing** - Process multiple text files at once
- [ ] **Streaming Support** - Real-time streaming audio output
- [ ] **Custom Voice Training** - Support for custom voice models

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup
```bash
git clone https://github.com/yourusername/pitch-tts.git
cd pitch-tts
cargo build
cargo test -- --test-threads=1
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [piper-rs](https://github.com/rhasspy/piper) - The core TTS engine
- [Rhasspy](https://rhasspy.readthedocs.io/) - Voice models and inspiration
- [HuggingFace](https://huggingface.co/) - Voice model hosting 