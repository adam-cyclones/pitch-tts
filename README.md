# text-to-face

A comprehensive **animation pipeline tool** that converts text into speech and facial animation data. Built for character animators, game developers, and content creators who need to generate synchronized audio and facial expressions from text input.

## 🎬 Animation Pipeline Focus

**text-to-face** is specifically designed to create complete animation pipelines:

1. **Text Input** → **Speech Synthesis** → **Lip-sync Data** → **Animation-Ready Output**
2. **Perfect for**: Character animation, game dialogue, video content, virtual avatars, and interactive storytelling
3. **Outputs**: WAV audio files + JSON with precise word/phoneme timing + ARPAbet phonemes for facial rigging

## ✨ Features

- **🎯 Animation-Ready Output** - Generate WAV + JSON with word/phoneme timing for direct use in animation software
- **🎵 Real-time Pitch Shifting** - Adjust character voice characteristics (0.5x to 2.0x pitch range)
- **💾 Export Pipeline** - Save to organized folders with synchronized audio and animation data
- **🔧 Comprehensive CLI** - Intuitive commands for animation workflows
- **🌍 Multi-language Support** - 50+ voices across 6 languages for diverse character needs
- **⚡ Fast Synthesis** - Built on piper-rs for efficient, high-quality speech generation
- **📚 Rust Library** - Integrate into custom animation pipelines and game engines
- **🎬 WhisperX Lip-sync** - Generate animation-ready JSON with accurate word/phoneme timings
- **🤖 AI-Powered Phonemes** - Hybrid CMUdict + g2p-en + LLaMA 3.2 approach for accurate ARPAbet phonemes
- **🎭 Character Animation Ready** - ARPAbet phonemes embedded in JSON for facial rigging systems

## 🚀 Installation

### Prerequisites
- Rust 1.70+ and Cargo
- Internet connection (for downloading voice models)
- **For lipsync JSON:** [WhisperX](https://github.com/m-bain/whisperX) must be installed and available in your PATH
- **For ARPAbet phonemes:** [Ollama](https://ollama.ai/) with LLaMA 3.2 model (auto-downloaded on first use)
- **For fast fallback:** [g2p-en](https://github.com/Kyubyong/g2p) (Python package)

### Install WhisperX (for lipsync JSON)
WhisperX is a Python tool. Install it via pip:

```bash
pip install git+https://github.com/m-bain/whisperx.git
```

Or see the [WhisperX repo](https://github.com/m-bain/whisperX) for more details.

### Install g2p-en (for fast ARPAbet fallback)
```bash
pip install g2p-en
```

### Install Ollama (for ARPAbet phonemes)
```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Then pull the recommended model
ollama pull llama3.2
```

### Build from Source
```bash
git clone https://github.com/yourusername/text-to-face.git
cd text-to-face
cargo build --release
```

### First Run
On first run, text-to-face will automatically download the default voice model (en_GB-alba-medium). Voice models are cached in the `models/` directory.

## 📖 Usage

### Animation Pipeline Workflow

**Complete character dialogue generation:**
```bash
# Generate a complete animation-ready output
cargo run -- export \
  --voice en_GB-alba-medium \
  --text "Hello there! I'm excited to show you this animation pipeline." \
  --lipsync high \
  --lipsync-with-llm llama3.2

# This creates:
# - output_hello_there/hello_there.wav (audio file)
# - output_hello_there/hello_there.json (animation data with ARPAbet phonemes)
```

### List Available Voices (for character selection)
```bash
cargo run -- list
```

### Quick Character Dialogue Test
```bash
# Test a character voice before full export
cargo run -- say "This is how my character sounds!" --voice en_US-amy-medium --pitch 1.1
```

### Export Animation Assets
```bash
# Basic animation export
cargo run -- export --voice en_GB-alba-medium --output character_dialogue.wav --text "Character dialogue here!"

# High-fidelity animation export (includes ARPAbet phonemes for facial rigging)
cargo run -- export \
  --voice en_GB-alba-medium \
  --text "She sells seashells by the seashore." \
  --lipsync high \
  --lipsync-with-llm llama3.2

# Custom character voice with specific pitch
cargo run -- export \
  --voice en_US-amy-medium \
  --text "Custom character dialogue" \
  --pitch 0.9 \
  --lipsync high \
  --lipsync-with-llm llama3.2
```

### Legacy Mode (Quick Commands)
```bash
cargo run -- --voice en_US-libritts_r-medium --text "Quick mode!"
```

## 📚 Library Usage

Use `text_to_face` as a Rust library for custom animation pipelines:

```rust
use text_to_face::synth_to_wav_with_pitch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate animation assets programmatically
    synth_to_wav_with_pitch(
        "Character dialogue here!".to_string(),
        "en_GB-alba-medium",
        "character_audio.wav",
        1.0, // pitch factor
    )?;
    Ok(())
}
```

## 🔧 Configuration

### Animation Output Structure
Exports are organized in character-friendly folders:
```
output_character_dialogue/
├── character_dialogue.wav          # Audio file for animation
└── character_dialogue.json         # Animation data with timing + phonemes
```

### Voice Model Storage
Voice models are automatically downloaded and stored in the `models/` directory:
```
models/
├── en_GB-alba-medium.onnx
├── en_GB-alba-medium.onnx.json
├── en_US-amy-medium.onnx
└── ...
```

### Character Voice Customization
- **Pitch Range**: 0.5x to 2.0x (octave down to octave up)
- **Default**: 1.0x (no change)
- **Algorithm**: Linear interpolation resampling
- **Character Presets**:
  - `slomo`: 0.4 (slow motion character)
  - `deep`: 0.85 (deep-voiced character)
  - `child`: 1.1 (child character voice)
  - `helium`: 1.5 (comic character effect)

### ARPAbet Phoneme Generation (for facial animation)
- **Primary**: CMUdict for known words (fast, accurate)
- **Fallback**: g2p-en for unknown words (fast, rule-based)
- **Last Resort**: LLaMA 3.2 for truly novel words (with validation)
- **Validation**: Only valid ARPAbet phonemes are included in output
- **Animation Ready**: Phonemes are embedded in JSON for direct use in facial rigging systems
- **Models**: Configurable via `--lipsync-with-llm` (default: llama3.2)

### Animation JSON Output
- **Word Segments**: Precise timing for each word
- **ARPAbet Phonemes**: Accurate phoneme data for facial animation
- **WhisperX Integration**: Professional-grade word/phoneme alignment
- **Animation Software Compatible**: Ready for Blender, Maya, Unity, Unreal Engine, and other animation tools

## 🎭 Animation Pipeline Integration

### Typical Workflow:
1. **Script Input** → text-to-face generates audio + timing data
2. **Animation Software** → Import WAV + JSON for lip-sync
3. **Facial Rigging** → Use ARPAbet phonemes for mouth shapes
4. **Final Animation** → Synchronized character performance

### Supported Animation Software:
- **Blender**: Import WAV + use JSON for lip-sync automation
- **Maya**: Use JSON data for facial animation curves
- **Unity**: Import for real-time character dialogue
- **Unreal Engine**: Use for cinematic sequences
- **After Effects**: Import for video content creation

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup
```bash
git clone https://github.com/yourusername/text-to-face.git
cd text-to-face
cargo build
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [piper-rs](https://github.com/rhasspy/piper) - The core TTS engine
- [Rhasspy](https://rhasspy.readthedocs.io/) - Voice models and inspiration
- [HuggingFace](https://huggingface.co/) - Voice model hosting
- [WhisperX](https://github.com/m-bain/whisperX) - Word/phoneme alignment for animation
- [CMUdict](https://github.com/Alexir/CMUdict) - ARPAbet phoneme dictionary
- [Ollama](https://ollama.ai/) - Local LLM inference for phoneme generation 