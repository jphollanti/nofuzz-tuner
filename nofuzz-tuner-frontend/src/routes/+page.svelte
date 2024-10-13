<script lang="ts">
   	import { onMount } from 'svelte';
    import { browser } from '$app/environment';

    let YinPitchDetector: any;

	let canvas: HTMLCanvasElement | null = null;
	let ctx: CanvasRenderingContext2D | null = null;
	let audioContext: AudioContext | null = null;
	let scriptProcessor: ScriptProcessorNode | null = null;
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

    async function loadWasm() {
        if (browser) {
			console.log('loading wasm');
            const module = await import('../lib/no_fuzz_tuner/pkg/nofuzz_tuner_lib.js');
            await module.default();
            YinPitchDetector = module.YinPitchDetector;
        }
    }

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

	async function run() {
		//await init();

		const detector = new YinPitchDetector(0.1, 60.0, 500.0, 44100);

		audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
		const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
		scriptProcessor = audioContext.createScriptProcessor(4096, 1, 1);
		input = audioContext.createMediaStreamSource(stream);
		input.connect(scriptProcessor);
		scriptProcessor.connect(audioContext.destination);

		scriptProcessor.onaudioprocess = function(event) {
			const inputBuffer = event.inputBuffer.getChannelData(0);
			const audioData = new Float64Array(inputBuffer);
			const pitch = detector.maybe_find_pitch_js(audioData);
			if (pitch) {
				const sd = find_string_and_distance(pitch);
				if (detectedString !== sd.string) {
					detectedString = sd.string;
				}
				resetCanvas();
				drawScale(detectedString);
				drawIndicator(detectedString, pitch);
			}
		};
	}
	
    onMount(() => {
        loadWasm();

		const startButton = document.getElementById('start') as HTMLButtonElement;
		const stopButton = document.getElementById('stop') as HTMLButtonElement;
		if (stopButton) {
			stopButton.disabled = true;
		}

		startButton.onclick = async function() {
			startButton.disabled = true;
			stopButton.disabled = false;
			await run();
		};

		stopButton.onclick = function() {
			startButton.disabled = false;
			stopButton.disabled = true;
			if (scriptProcessor && input && audioContext) {
				scriptProcessor.disconnect();
				input.disconnect();
				audioContext.close();
			}
		};

        canvas = document.getElementById('linearScale') as HTMLCanvasElement | null;
		if (!canvas) {
			console.error('Canvas element not found');
			return;
		}
        ctx = canvas.getContext('2d');

		// Resize the canvas when the window is loaded or resized
		window.addEventListener('load', resizeCanvas);
		window.addEventListener('resize', resizeCanvas);
		resizeCanvas();
    });

    function detectPitch(audioData: Float64Array) {
        if (YinPitchDetector) {
            const detector = new YinPitchDetector(0.1, 60.0, 500.0, 44100);
            return detector.maybe_find_pitch_js(audioData);
        }
        return null;
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
	/* Reset default margins and paddings */
	* {
		margin: 0;
		padding: 0;
		box-sizing: border-box;
	}
	html, body {
		margin: 0;
		padding: 0;
		height: 100%;
		background-color: #121212;
	}
	canvas {
		display: block; /* Removes unwanted scrollbars */
		width: 80%;
		height: 80%;
		margin: 0 10%;
		background-color: #1E1E1E;
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

	button#stop[disabled=disabled], button#stop:disabled,
	button#start[disabled=disabled], button#start:disabled {
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

	.welcome {
		display: block;
		position: relative;
		width: 100%;
		height: 0;
		padding: 0 0 calc(100% * 495 / 2048) 0;
	}

	.welcome img {
		position: absolute;
		width: 100%;
		height: 100%;
		top: 0;
		display: block;
	}
</style>
