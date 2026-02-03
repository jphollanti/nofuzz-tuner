# CLAUDE.md - Development Guide for Claude Code

## Pre-Push Checklist

Always run these commands before pushing changes:

```bash
# 1. Format code
cargo fmt

# 2. Lint with clippy (warnings are errors)
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo test --package nofuzz_tuner_lib
```

## Project Structure

```
nofuzz-tuner/
├── src/main.rs              # CLI tuner (for testing/development)
├── nofuzz_tuner_lib/        # Core library (compiles to WASM)
│   └── src/lib.rs           # Pitch detection algorithms, filters
├── nofuzz-tuner-frontend/   # Svelte web frontend
│   └── src/routes/+page.svelte  # Main tuner UI
├── config.yaml              # CLI configuration
└── manual_testing/          # Test audio files
```

## Tech Stack

- **Rust + WebAssembly**: Core pitch detection compiled to WASM via wasm-pack
- **Svelte**: Frontend UI framework
- **cpal**: Audio input for CLI version
- **wasm-bindgen**: Rust-JS interop

## Key Algorithms

### YIN Pitch Detection (Primary)
- Autocorrelation-based pitch detection
- Per-string detectors with optimized parameters
- Post-processing: FFT refinement, averaging, clarity smoothing

### Signal Processing Pipeline
1. Biquad filter bank (highpass, notch filters for 50/60/100/120Hz, lowpass)
2. Optional AGC (Automatic Gain Control) for signal normalization
3. YIN algorithm for fundamental frequency detection
4. FFT refinement for sub-bin accuracy
5. Harmonic/octave error correction (for electric guitars)

## Development Commands

### Run CLI Tuner
```bash
cargo run
```

### Build WASM Module
```bash
./webassembly-build.sh
```

### Run Frontend Dev Server
```bash
cd nofuzz-tuner-frontend
npm install  # first time only
npm run dev
```

### Quick Rebuild and Run
```bash
./recompile-and-run.sh
```

## Instrument Presets

The tuner supports different instrument types with optimized detection:
- **Acoustic**: Standard settings
- **Electric (Clean)**: AGC + harmonic correction
- **Electric (Distorted)**: Full corrections (AGC, harmonic, octave)
- **Classical/Nylon**: AGC with gentler settings
- **Bass**: Extended low-frequency support
- **Extended Range**: 7/8-string guitars with very low frequency handling

## Tuning Support

Standard tunings plus:
- Drop tunings (Drop D, Drop A)
- D Standard, DADGAD, Open G
- 7-string and 8-string guitar tunings
- Bass (4-string, 5-string, Drop D)
- Ukulele (GCEA)

## Common Issues

### ALSA library not found (Linux)
```bash
sudo apt-get install -y libasound2-dev
```

### Clippy warnings
- Use `if let Some(x) = opt` instead of `if opt.is_some() { opt.unwrap() }`
- Use range contains: `(1.5..=4.0).contains(&x)` instead of `x >= 1.5 && x <= 4.0`
- Add `#[derive(Default)]` with `#[default]` attribute for enums
