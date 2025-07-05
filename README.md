# pitch-tts

A fast, flexible text-to-speech (TTS) CLI and Rust library powered by [piper-rs](https://github.com/rhasspy/piper).

## Features
- **Dynamic voice selection** (auto-downloads models from HuggingFace)
- **Pitch shifting** (real-time or WAV export)
- **WAV export** for easy integration with Blender and other tools
- **Comprehensive CLI** with subcommands for listing voices, speaking, and exporting
- **Plans for phoneme extraction** (Rhubarb lip sync integration)

## Installation

1. **Clone the repo:**
   ```sh
   git clone https://github.com/yourusername/pitch-tts.git
   cd pitch-tts
   ```
2. **Build with Cargo:**
   ```sh
   cargo build --release
   ```

## Usage

### List available voices
```sh
cargo run -- list
```

### Speak text (with optional voice and pitch)
```sh
cargo run -- say "Hello world!" --voice en_GB-alba-medium --pitch 1.2
```
- If no text is given, defaults to a fun Scottish phrase with Alba.
- If no voice is given, defaults to Alba (en_GB-alba-medium).

### Export to WAV
```sh
cargo run -- export --voice en_GB-alba-medium --output hello.wav --text "Hello world!" --pitch 0.8
```

### Legacy/Quick Mode
```sh
cargo run -- --voice en_US-libritts_r-medium --text "Quick mode!"
```

## Library Usage

You can use `pitch_tts` as a Rust library for custom TTS workflows or Blender integration.

## Roadmap
- [ ] Rhubarb lip sync/phoneme extraction for animation
- [ ] More advanced pitch/timbre controls
- [ ] Voice quality auto-detection

## License
MIT 