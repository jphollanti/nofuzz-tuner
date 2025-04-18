/* pitch‑worklet.js
   Runs on the audio rendering thread.  It simply feeds raw
   Float32 samples back to the main thread; you still do your
   pitch math in JS/wasm exactly as before.
*/
class PitchWorkletProcessor extends AudioWorkletProcessor {
    // Forward every render quantum to the main thread
    process(inputs) {
      const input = inputs[0];
      if (input && input[0]) {
        // Copy so we don’t ship the underlying ring‑buffer itself
        this.port.postMessage(new Float32Array(input[0]));
      }
      return true; // keep processor alive
    }
  }
  
  registerProcessor('pitch-worklet', PitchWorkletProcessor);