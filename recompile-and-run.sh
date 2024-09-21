pushd nofuzz_tuner_lib
wasm-pack build --release --target web
popd
python -m http.server 