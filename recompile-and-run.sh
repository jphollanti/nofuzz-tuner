pushd nofuzz_tuner_lib
wasm-pack build --release --target web
popd
rm -rf nofuzz-tuner-frontend/src/lib/no_fuzz_tuner
mkdir -p nofuzz-tuner-frontend/src/lib/no_fuzz_tuner/pkg
cp -r nofuzz_tuner_lib/pkg nofuzz-tuner-frontend/src/lib/no_fuzz_tuner

pushd nofuzz-tuner-frontend
npm run dev    
popd
