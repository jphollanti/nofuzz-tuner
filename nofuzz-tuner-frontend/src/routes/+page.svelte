<script lang="ts">
	import { env as PUBLIC } from '$env/dynamic/public';
	import { fade } from 'svelte/transition';
	import workletURL from '$lib/audio/pitch-worklet.js?url'; 
	
	const buildVersion =
		PUBLIC.PUBLIC_BUILD_VERSION
		?? `dev-${new Date().toISOString()}`;
	

	/* Tooltip visibility */
	let open = false;

	/* Tuning selections */
	const tunings = [
		{ id: 'standard-e', label: 'Standard E' },
		{ id: 'flat-e',     label: 'Eb / Half‑step Down' },
		{ id: 'drop-d',     label: 'Drop D' },
	];

	export let tuning: string = tunings[0].id;

	
   	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let YinPitchDetector: any;

	let canvas: HTMLCanvasElement | null = null;
	let ctx: CanvasRenderingContext2D | null = null;

	let audioContext: AudioContext | null = null;
	let workletNode: AudioWorkletNode | null = null;
	let input: MediaStreamAudioSourceNode | null = null;

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
	
	function resetCanvas() {
		if (!ctx || !canvas) {
			console.error('Canvas or context not found');
			return;
		}
		ctx.clearRect(0, 0, canvas.width, canvas.height);
	}

	function drawScale(tuningTo:any | null) {
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

		const label = "Tuning to string: " + tuningTo.note;
		const label2 = `${tuningTo.freq} Hz`;

		ctx.font = '12px Arial';
		ctx.fillStyle = 'white';
		ctx.textAlign = 'center';
		ctx.fillText(label, centerX, scaleY - (height * .30)); // Label below the tick
		ctx.fillText(label2, centerX, scaleY + (height * .30)); // Label below the tick
	}
	
	// Function to draw the indicator at a specific value
	function drawIndicator(tuningTo:any | null, value:number) {
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

		let indicatorX = centerX;
		let rangeMin = tuningTo.freq - 10;
		let rangeMax = tuningTo.freq + 10;
		// map value to string.range and find x position
		if (value < tuningTo.freq) {
			let xx = (value - rangeMin) / (tuningTo.freq - rangeMin);
			if (xx < 0) {
				xx = 0;
			}
			indicatorX = centerX * xx;
		} else if (value > tuningTo.freq) {
			let xx = (value - tuningTo.freq) / (rangeMax - tuningTo.freq);
			if (xx > 1) {
				xx = 1;
			}
			indicatorX = centerX + centerX * xx;
		}

		const dist = Math.abs(value - tuningTo.freq);
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
			if (value > tuningTo.freq) {
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
				if (value > tuningTo.freq) {
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
		let tuningTo = {
			note: 'E2',
			freq: 82.41,
		}
		drawScale(tuningTo);
		drawIndicator(tuningTo, 94.31);
	}

	async function loadWasm() {
		if (!browser) return;
		const pkg = await import('../lib/no_fuzz_tuner/pkg/nofuzz_tuner_lib.js');
		await pkg.default();
		YinPitchDetector = pkg.YinPitchDetector;
	}

	async function run() {		
		audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
		await audioContext.audioWorklet.addModule(workletURL);

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
				const pitch = detector.maybe_find_pitch_js(buf, tuning);
				if (pitch) {
					const tuningTo = pitch.tuningTo;
					resetCanvas();
					drawScale(tuningTo);
					drawIndicator(tuningTo, pitch.freq);
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

			<label class="tuning-label">
				<p>&nbsp;</p>
				Choose tuning
				<select
					class="tuning-select"
					bind:value={tuning}>
					{#each tunings as t}
						<option value={t.id}>{t.label}</option>
					{/each}
				</select>
			</label>
		</div>
    </div>
    <canvas id="linearScale"></canvas>
</section>

<button
	class="info-btn"
	on:mouseenter={() => (open = true)}
	on:mouseleave={() => (open = false)}
	on:focusin={() => (open = true)}
	on:focusout={() => (open = false)}
	aria-describedby="build-id">
	<!--  tiny info icon (Heroicons “information-circle”)  -->
	<svg class="icon" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
		<path fill-rule="evenodd"
			d="M18 10A8 8 0 11 2 10a8 8 0 0116 0zm-8-3a1.25 1.25 0 110-2.5 1.25 1.25 0 010 2.5zM9 8.75a1 1 0 012 0v6.5a1 1 0 11-2 0v-6.5z"
			clip-rule="evenodd" />
	</svg>

	{#if open}
		<div class="bubble" id="build-id" transition:fade>
			Build&nbsp;{buildVersion}
		</div>
	{/if}
</button>

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
	
	/* anchor button ----------------------------------- */
	.info-btn {
		position: fixed;
		right: 1rem;
		bottom: 1rem;
		z-index: 9999;

		/* strip native button chrome */
		background: transparent;
		border: none;
		padding: 0;

		/* shrink to icon size */
		display: inline-flex;
		align-items: center;
		justify-content: right;
		cursor: help;          /* little ❓ cursor   */
	}

	/* optional focus ring for a11y */
	.info-btn:focus-visible {
		outline: 2px solid var(--accent-500, #888);
		outline-offset: 2px;
	}

	/* icon --------------------------------------------- */
	.icon {
		width: 1.25rem;
		height: 1.25rem;
		color: var(--accent-500, #888);
		pointer-events: none;  /* keep tooltip trigger on button */
	}

	/* tooltip bubble ----------------------------------- */
	.bubble {
		position: absolute;
		bottom: 150%;
		right: 0;

		background: var(--tooltip-bg, #222);
		color: #fff;
		padding: 0.35rem 0.6rem;
		font: 0.72rem/1 monospace;
		border-radius: 0.3rem;
		white-space: nowrap;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
		pointer-events: none;
	}

	/* tuning select */
	/* ---------- Component styles ---------- */
	.tuning-label {
		display: block;
		color: var(--fg);
		font-size: 0.9rem;
	}

	.tuning-select {
		margin-top: 0.25rem;
		width: 100%;
		font-size: 1rem;
		padding: 0.45rem 0.6rem;
		color: var(--fg);
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 0.375rem;
		transition: border 0.2s, box-shadow 0.2s;
	}

	.tuning-select:focus {
		outline: none;
		border-color: var(--accent);
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 50%, transparent);
	}

	/* optional hover & disabled tweaks */
	.tuning-select:hover {
		border-color: color-mix(in srgb, var(--accent) 40%, var(--border));
	}
	.tuning-select:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
