
![Build & Deploy](https://github.com/jphollanti/nofuzz-tuner/actions/workflows/ci.yml/badge.svg)

# nofuzz_tuner

## Working with command line version

Run with: `cargo run`

This runs based on parameters set in config.yaml. 

## Working with webassembly version

Webassembly library is contained in folder nofuzz_tuner_lib. 

Combile it with `webassembly-build.sh` script. 

Frontend is in folder nofuzz-tuner-frontend. Run `npm run dev` to start a local server. 

There is also a script `recompile-and-run.sh` that compiles the Rust library and runs the frontend. 
