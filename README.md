# nofuzz_tuner

## Working with command line version

Run with: `cargo run`

This runs based on parameters set in config.yaml. 

## Working with webassembly version

Webassembly library is contained in folder nofuzz_tuner_lib. 

Combile it with `webassembly-build.sh` script. 

In a separate terminal, run up a simple python web server to serve index.html `python -m http.server`

Open browser to `http://localhost:8000`. This is a dummy implementation that passes in hardcoded frequencies to Rust. You can see output in the browser console. 