# vibeband

Ambient sound generator for your terminal. Powered by [ElevenLabs](https://elevenlabs.io) sound generation API.

Play looping ambient soundscapes — coffee shops, rain, fireplaces, or anything you can describe — right from your terminal with a spectrum visualizer.

```
╭──────────────────────────────────────────────────────────────────╮
│                                                                  │
│  V I B E B A N D                                   ▶ Playing     │
│  ♫ ☕ Coffee Shop + 🌧 Rain                                      │
│                                                                  │
│  █████ ▇▇▇▇▇ ▅▅▅▅▅ █████ ▃▃▃▃▃ ▅▅▅▅▅ ▇▇▇▇▇ ▃▃▃▃▃ ▂▂▂▂▂ ▁▁▁▁▁ │
│  █████ █████ █████ █████ ▅▅▅▅▅ █████ █████ ▅▅▅▅▅ ▃▃▃▃▃ ▂▂▂▂▂ │
│  █████ █████ █████ █████ █████ █████ █████ █████ ▅▅▅▅▅ ▃▃▃▃▃ │
│                                                                  │
│  ▸ ☕ Coffee Shop ████████████████░░░░░░  80%                     │
│    🌧 Rain        ████████████████░░░░░░  80%                     │
│                                                                  │
│  VOL ██████████████████████░░░░░░░░░  70%                        │
│                                                                  │
│  [Spc]⏯  [↑↓]Vol [+-]Master [Tab]Layer [Q]Quit                  │
│                                                                  │
╰──────────────────────────────────────────────────────────────────╯
```
<img width="400" height="319" alt="CleanShot 2026-02-26 at 21 32 13@2x" src="https://github.com/user-attachments/assets/a19015d8-cb58-4f77-8c2e-c5f98ab696e5" />



## Install

### Homebrew (macOS/Linux)

```bash
brew tap daveliu/tap
brew install vibeband
```

### Cargo

```bash
cargo install vibeband
```

### From source

```bash
git clone https://github.com/daveliu/vibeband.git
cd vibeband
cargo build --release
```

## Setup

Requires an [ElevenLabs API key](https://elevenlabs.io):

```bash
export ELEVENLABS_API_KEY="your-key-here"
```

## Usage

```bash
# Play a single scene
vibeband cafe

# Mix multiple scenes
vibeband mix cafe rain fire

# Play a custom sound from a prompt
vibeband -c "lo-fi jazz piano with vinyl warmth"

# Mix presets with custom sounds
vibeband rain -c "soft piano melody" -c "vinyl crackle"
```

## Built-in Presets

| Scene | Emoji | Description |
|-------|-------|-------------|
| `cafe` | ☕ | Coffee shop ambiance |
| `rain` | 🌧 | Rain on a rooftop |
| `forest` | 🌲 | Forest with birdsong |
| `fire` | 🔥 | Crackling fireplace |
| `ocean` | 🌊 | Ocean waves on a beach |
| `thunder` | ⛈ | Rolling thunderstorm |
| `wind` | 💨 | Howling wind |
| `creek` | 🏞 | Babbling creek |
| `birds` | 🐦 | Morning birdsong |
| `night` | 🌙 | Crickets and nighttime |
| `train` | 🚂 | Train on tracks |
| `traffic` | 🚗 | City traffic |
| `library` | 📚 | Quiet library |
| `keyboard` | ⌨ | Mechanical keyboard |
| `synth` | 🎹 | Atmospheric synth pad |
| `guitar` | 🎸 | Ambient guitar |
| `drums` | 🥁 | Lo-fi hip-hop drums |

## Custom Presets

Save your own named presets and reuse them:

```bash
# Save a new preset
vibeband save myguitar "soft acoustic guitar fingerpicking with warm reverb"

# Use it like any built-in
vibeband myguitar

# Mix with other presets
vibeband mix rain myguitar

# Update the prompt
vibeband save myguitar "clean jazz guitar with soft chord voicings"

# Remove a preset
vibeband remove myguitar

# List all presets (built-in + custom)
vibeband list
```

Custom presets are stored in `~/.vibeband/presets.json`.

## Controls

| Key | Action |
|-----|--------|
| `Space` | Pause / Resume |
| `↑` `↓` | Adjust selected layer volume |
| `+` `-` | Adjust master volume |
| `Tab` | Switch selected layer |
| `q` | Quit |

## How It Works

1. **Sound generation** — Sends text prompts to ElevenLabs' sound generation API (`POST /v1/sound-generation`) with looping enabled
2. **Caching** — Generated audio is cached at `~/.vibeband/cache/` using SHA-256 hashes of the prompt. Second runs are instant
3. **Looping** — ElevenLabs generates loop-ready audio, rodio replays it seamlessly with `repeat_infinite()`
4. **Mixing** — Multiple rodio `Sink` instances on the same `OutputStream` mix automatically
5. **Visualizer** — Audio samples are tapped from the playback pipeline into a ring buffer, analyzed with FFT (2048-point, Hann window), and rendered as a 10-band spectrum with Unicode block characters

## Tech Stack

- [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) — TUI
- [rodio](https://github.com/RustAudio/rodio) — Audio playback and mixing
- [rustfft](https://github.com/ejmahler/RustFFT) — Spectrum analysis
- [reqwest](https://github.com/seanmonstar/reqwest) — HTTP client
- [tokio](https://github.com/tokio-rs/tokio) — Async runtime
- [clap](https://github.com/clap-rs/clap) — CLI parsing

## Author

[https://x.com/l_w_j](https://x.com/l_w_j)
