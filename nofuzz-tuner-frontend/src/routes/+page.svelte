<script lang="ts">
   	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let YinPitchDetector: any;

	let canvas: HTMLCanvasElement | null = null;
	let ctx: CanvasRenderingContext2D | null = null;

	let audioContext: AudioContext | null = null;
	let workletNode: AudioWorkletNode | null = null;
	let input: MediaStreamAudioSourceNode | null = null;

	let detectedString: string | null = 'NAN';

	interface GuitarString {
		name: string;
		frequency: number;
		range?: { min: number; max: number };
	}
	const strings: GuitarString[] = [
		{ name: 'E2', frequency: 82.41 },
		{ name: 'A2', frequency: 110.00 },
		{ name: 'D3', frequency: 146.83 },
		{ name: 'G3', frequency: 196.00 },
		{ name: 'B3', frequency: 246.94 },
		{ name: 'E4', frequency: 329.63 }
	];
	// Add range to strings
	strings.forEach((s, i) => {
		s.range = {
			min: s.frequency - 10,
			max: s.frequency + 10
		};
	});

	function find_string_and_distance(pitch: number) {
		const strings = [
			{ name: 'E2', frequency: 82.41 },
			{ name: 'A2', frequency: 110.00 },
			{ name: 'D3', frequency: 146.83 },
			{ name: 'G3', frequency: 196.00 },
			{ name: 'B3', frequency: 246.94 },
			{ name: 'E4', frequency: 329.63 }
		];

		let minDistance = Infinity;
		let string = null;
		let frequency = null;

		for (const s of strings) {
			const distance = Math.abs(pitch - s.frequency);
			if (distance < minDistance) {
				minDistance = distance;
				string = s.name;
				frequency = s.frequency;
			}
		}

		return { frequency, distance: minDistance, string };
	}

	function resetCanvas() {
		if (!ctx || !canvas) {
			console.error('Canvas or context not found');
			return;
		}
		ctx.clearRect(0, 0, canvas.width, canvas.height);
	}

	function drawScale(scaleString:string | null) {
		if (!canvas || !ctx) {
			console.error('Canvas or context not found');
			return;
		}
		// Set canvas dimensions
		const width = canvas.width;
		const height = canvas.height;

		// Draw the linear scale
		const startX = 0;
		const endX = canvas.clientWidth;
		const scaleY = height / 2;

		const centerX = (endX - startX) / 2;
		const drawScaleYMin = scaleY - (height * .20);
		const drawScaleYMax = scaleY + (height * .20);

		// draw center line
		ctx.beginPath();
		ctx.strokeStyle = '#FFFFFF';
		ctx.lineWidth = (height * .012);
		ctx.moveTo(centerX, drawScaleYMin); // Start above the main line
		ctx.lineTo(centerX, drawScaleYMax); // End below the main line
		ctx.stroke();

		const label = "Tuning to string: " + scaleString;
		const targetString = strings.find(s => s.name === scaleString);
		const label2 = targetString ? `${targetString.frequency} Hz` : "N/A";

		ctx.font = '12px Arial';
		ctx.fillStyle = 'white';
		ctx.textAlign = 'center';
		ctx.fillText(label, centerX, scaleY - (height * .30)); // Label below the tick
		ctx.fillText(label2, centerX, scaleY + (height * .30)); // Label below the tick
	}
	
	// Function to draw the indicator at a specific value
	function drawIndicator(scaleString:string | null, value:number) {
		if (!canvas || !ctx) {
			console.error('Canvas or context not found');
			return;
		}

		// Set canvas dimensions
		const width = canvas.width;
		const height = canvas.height;

		const startX = 0; // Starting X position of the scale
		const endX = canvas.clientWidth; // Ending X position of the scale
		const scaleY = height / 2; // Vertical position of the scale
		const centerX = (endX - startX) / 2;

		// Calculate the X position of the indicator based on the value
		let string = strings.find(s => s.name === scaleString);
		if (!string || !string.range) return;


		let indicatorX = centerX;
		// map value to string.range and find x position
		if (value < string.frequency) {
			let xx = (value - string.range.min) / (string.frequency - string.range.min);
			if (xx < 0) {
				xx = 0;
			}
			indicatorX = centerX * xx;
		} else if (value > string.frequency) {
			let xx = (value - string.frequency) / (string.range.max - string.frequency);
			if (xx > 1) {
				xx = 1;
			}
			indicatorX = centerX + centerX * xx;
		}

		const dist = Math.abs(value - string.frequency);
		let color = '#4CAF50';
		if (dist > 5) {
			color = '#FF4C4C';
		} else if (dist > 2) {
			color = '#FFEB3B';
		}

		if (dist > 1) {
			let ax = indicatorX
			let ay = scaleY - (height * .03)
			let bx = indicatorX + (height * .03)
			let by = scaleY
			let cx = indicatorX
			let cy = scaleY + (height * .03)
			if (value > string.frequency) {
				bx = indicatorX - (height * .03)
			}

			ctx.beginPath();
			ctx.fillStyle = color;
			ctx.moveTo(ax, ay);
			ctx.lineTo(bx, by);
			ctx.lineTo(cx, cy);
			ctx.fill();

			if (dist > 5) {
				ax = indicatorX + (height * .015)
				ay = scaleY - (height * .03)
				bx = indicatorX + (height * .03) + (height * .015)
				by = scaleY
				cx = indicatorX + (height * .015)
				cy = scaleY + (height * .03)
				if (value > string.frequency) {
					ax = indicatorX - (height * .015)
					bx = indicatorX - (height * .03) - (height * .015)
					cx = indicatorX - (height * .015)
				}

				ctx.beginPath();
				ctx.fillStyle = '#FF5E5E';
				ctx.moveTo(ax, ay);
				ctx.lineTo(bx, by);
				ctx.lineTo(cx, cy);
				ctx.fill();
			}
		}

		// Draw a line connecting the indicator to the scale
		ctx.beginPath();
		ctx.moveTo(indicatorX, scaleY - (height * .07));
		ctx.lineTo(indicatorX, scaleY + (height * .07));
		ctx.strokeStyle = color;
		ctx.lineWidth = (height * .012);
		ctx.stroke();

		const label = value.toFixed(1) + " Hz";
		ctx.font = '12px Arial';
		ctx.fillStyle = 'white';
		ctx.textAlign = 'center';
		let labelX = indicatorX;
		if (labelX < 30) {
			labelX = 30;
		} else if (labelX > canvas.width - 30) {
			labelX = canvas.width - 30;
		}


		const tw = ctx.measureText(label).width + height * .03;
		//const th = parseInt(font, 10)
		ctx.fillStyle = '#121212';
		ctx.fillRect(labelX - (tw/2), scaleY + (height * .10), tw, 30);

		ctx.fillStyle = 'white';
		ctx.fillText(label, labelX, scaleY + (height * .13));

	}

	function resizeCanvas() {
		if (!canvas) {
			console.error('Canvas not found');
			return;
		}
		canvas.width = window.innerWidth * 0.8;
		canvas.height = window.innerHeight * 0.8;

		// Redraw or update your canvas content here if necessary
		//const context = canvas.getContext('2d');
		// Example: Fill the canvas with a color
		//context.fillStyle = '#f0f0f0';
		//context.fillRect(0, 0, canvas.width, canvas.height);

		// testing 
		drawScale('E2');
		drawIndicator('E2', 94.31);
	}

	async function loadWasm() {
		if (!browser) return;
		const pkg = await import('../lib/no_fuzz_tuner/pkg/nofuzz_tuner_lib.js');
		await pkg.default();
		YinPitchDetector = pkg.YinPitchDetector;
	}

	async function run() {
		audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
		await audioContext.audioWorklet.addModule(
			new URL('./pitch-worklet.ts', import.meta.url).href
		);
		const sr = audioContext.sampleRate; 
		// These values should come from config.yaml
		// or similar, but for now we hardcode them
		const threshold = 0.1;
        const freq_min = 60;
        const freq_max = 500;
		const detector = new YinPitchDetector(threshold, freq_min, freq_max, sr);

		// microphone
		const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
		input = audioContext.createMediaStreamSource(stream);

		// zero‑gain node so we don’t echo the mic to the speakers
		const silence = audioContext.createGain();
		silence.gain.value = 0;

		workletNode = new AudioWorkletNode(audioContext, 'pitch-worklet', {
			numberOfInputs: 1,
			channelCount: 1
		});

		// wiring: mic → worklet → (silent) destination
		input.connect(workletNode);
		workletNode.connect(silence).connect(audioContext.destination);

		const quantum = 128; // every AudioWorklet chunk
		const BLOCK   = 4096; // Yin frame size
		const buf     = new Float32Array(BLOCK);
		let   write   = 0;
		let   filled  = 0;

		workletNode.port.onmessage = ({ data }: MessageEvent<Float32Array>) => {
		const chunk = data; // 128 samples
		buf.set(chunk, write);
		write = (write + quantum) % BLOCK;
		filled += quantum;

		if (filled >= BLOCK) {
			filled = 0; // buffer ready
			const pitch = detector.maybe_find_pitch_js(buf);
			if (pitch) {
					const sd = find_string_and_distance(pitch.freq);
					if (detectedString !== sd.string) detectedString = sd.string;

					resetCanvas();
					drawScale(detectedString);
					drawIndicator(detectedString, pitch.freq);
				}
			}
		};
	}

	// onMount logic, resizeCanvas, draw helpers – unchanged
	onMount(() => {
		loadWasm();

		const startButton = document.getElementById('start') as HTMLButtonElement;
		const stopButton = document.getElementById('stop') as HTMLButtonElement;

		stopButton.disabled = true;

		startButton.onclick = async () => {
			startButton.disabled = true;
			stopButton.disabled = false;
			await run();
		};

		stopButton.onclick = () => {
			startButton.disabled = false;
			stopButton.disabled = true;
			workletNode?.disconnect();
			input?.disconnect();
			audioContext?.close();
		};

		canvas = document.getElementById('linearScale') as HTMLCanvasElement;
		ctx = canvas?.getContext('2d') ?? null;
		window.addEventListener('load', resizeCanvas);
		window.addEventListener('resize', resizeCanvas);
		resizeCanvas();
	});
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section>
	<h1>NoFuzz Tuner</h1>

    <div id="controls-container">
        <div id="controls">
            <button id="start">Start</button>
            <button id="stop">Stop</button>
        </div>
    </div>
    <canvas id="linearScale"></canvas>
</section>

<style>
	canvas {
		display: block; /* Removes unwanted scrollbars */
		width: 80%;
		height: 80%;
		margin: 0 10%;
	}
	#controls-container {
		position: fixed;
		width: 100%;
		bottom: 5%;
		display: flex;
		justify-content: center;
		align-items: center;
	}
	#controls {
		padding: 10px;
		background-color: rgba(99, 99, 99, 0.5);
		color: white;
	}
	button {
		background-color: #1F1F1F;
		color: #E0E0E0;
		border: none;
		padding: 10px 20px;
		cursor: pointer;
		width: 50%;
		display: block;
		float: left;
	}
	button#start {
		background-color: #4CAF50;
	}

	button#stop {
		background-color: #FF4C4C;
	}

	button#start:hover, button#stop:hover {
		background-color: #333333;
	}

	button#stop:disabled,
	button#start:disabled {
		background-color: #333333;
		cursor: not-allowed;
		color: #1E1E1E;
	}
	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 0.6;
	}

	h1 {
		width: 100%;
	}

</style>
