
![Build & Deploy](https://github.com/jphollanti/nofuzz-tuner/actions/workflows/ci.yml/badge.svg)

# No-Fuzz Guitar Tuner

nofuzz-tuner is a free, no-nonsense online tuner for guitars (and ultimately for other instruments). It was created because a few years ago there just weren't good free, ad-free, no-strings-attached tuners available online. 

https://www.nofuzz.app/

For Developers and Tech Enthusiasts: This project is a showcase of what modern web technologies can do. It's interesting for anyone curious about Rust-WASM integration, audio processing in the browser, or building performant web apps with Svelte. Feel free to dive into the code, contribute, or fork it for your own projects.

## Tech Stack

This project is built to run smoothly in your browser without the need for any servers.

Rust + WebAssembly (WASM): The core audio processing (frequency detection) is written in Rust and compiled to WebAssembly for near-native speed. This lets client side do all the heavy lifting (signal analysis) and eliminating the need for a backend server. In other words, your browser handles everything – fast – so no internet connection is even required once the page is loaded, and no data leaves your device.


Svelte: The user interface is built with Svelte, a lightweight front-end framework. Svelte compiles UI components into highly efficient JavaScript, which means the app delivers a snappy, responsive experience. The combination of Rust/WASM for processing and Svelte for the UI results in a tuner that feels native and responsive in the browser.

## Setup and Running Locally

Prerequisites: You should have Rust (with Cargo), Node.js and npm, and wasm-pack installed on your system.

### Running the Command-Line Tuner (Rust backend)
If you want to run the tuner logic in a simple command-line mode (for testing or development purposes), you can run the Rust project directly with:

```
cargo run
```

This will launch the tuner in your terminal using settings from the config.yaml file. You might use this mode to feed in audio samples or debug the pitch detection in a controlled environment.

### Running the Web App (WASM + Svelte frontend)

For the full web experience (interactive tuner in the browser), you'll need to compile the Rust code to WebAssembly and run the Svelte frontend.

#### Quick one-liner option: 

You can use this convenience script in the root of the project. Just make sure to run step 2 below before running this:

```
./recompile-and-run.sh
```

This script will compile the Rust WASM module and then automatically start the Svelte dev server for you. It essentially wraps the following steps 1 and 3 (assumes you've already done step 2 at least once). This is handy for development – for example, every time you update the Rust code, just re-run this script to rebuild the WASM and reload the app.

#### Non one-liner option: 

1. Compile the Rust code to WASM: 

In the project root, run the provided build script:

```
./webassembly-build.sh
```

This uses wasm-pack to compile the Rust library (nofuzz_tuner_lib) into WebAssembly and generates the accompanying JavaScript glue code (placed in nofuzz_tuner_lib/pkg).

2. Install frontend dependencies:

Navigate to the Svelte frontend folder and install its npm dependencies (you only need to do this once):

```
cd nofuzz-tuner-frontend
npm install
```

3. Start the development server:

Still in the nofuzz-tuner-frontend folder, launch the Svelte app:

```
npm run dev
```

This will start a local web server (usually at http://localhost:5173 or as indicated in the console). Open that URL in your browser, allow microphone access, and you should see the tuner interface ready to detect notes.

## Frequency Detection Algorithms

I've tried couple different algorithms for pitch detection. 

McLeod Pitch Method (MPM): An algorithm that refines the basic autocorrelation approach to detect the fundamental frequency of a sound with high accuracy. MPM helps reduce common pitch-detection errors (like octave mistakes) by focusing on signal clarity and has proven effective for musical instrument tuning.

YIN algorithm: A popular algorithm for pitch detection that operates in the time domain and is known for its accuracy in single-pitch detection. Like MPM, the YIN algorithm is based on autocorrelation techniques but introduces clever tweaks to minimize errors and handle noise, making it well-suited for detecting the pitch of guitar notes and other monophonic sources.

In the future, I plan to explore additional algorithms such as pYIN (probabilistic YIN) – an enhanced version of YIN that could improve detection accuracy and stability even further. 

The current runner up is Yin as it performs well and compiles to WASM without problems. 

## Closing Note

Happy tuning! If you have any questions or suggestions, feel free to open an issue or contribute. 
