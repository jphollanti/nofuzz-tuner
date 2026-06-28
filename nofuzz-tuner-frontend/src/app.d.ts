// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
	class AudioWorkletProcessor {
		readonly port: MessagePort;
		process(inputs: Float32Array[][]): boolean;
	}

	function registerProcessor(
		name: string,
		processorCtor: typeof AudioWorkletProcessor
	): void;

	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
