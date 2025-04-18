/* pitch‑worklet.ts
   Runs on the audio rendering thread.  It simply pipes raw
   Float32 samples back to the main thread; you still do your
   pitch math in JS/wasm exactly as before.
*/
class PitchWorkletProcessor extends AudioWorkletProcessor {
	// (We could accumulate 4096 samples here, but the main thread
	// already owns a circular buffer, so just forward each block.)
	process(inputs: Float32Array[][]) {
		const input = inputs[0];
		if (input && input[0]) {
			// Copy so we don’t ship the underlying ring‑buffer twice
			this.port.postMessage(new Float32Array(input[0]));
		}
		return true;          // keep processor alive
	}
}

registerProcessor('pitch-worklet', PitchWorkletProcessor);
