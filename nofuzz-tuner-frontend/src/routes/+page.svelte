<script lang="ts">
	import { env as PUBLIC } from '$env/dynamic/public';
	import { draw, fade } from 'svelte/transition';
	import workletURL from '$lib/audio/pitch-worklet.js?url'; 
	
	const buildVersion =
		PUBLIC.PUBLIC_BUILD_VERSION
		?? `dev-${new Date().toISOString()}`;
	

	/* Tooltip visibility */
	let open = false;

	/* Tuning selections */
	const tunings = [
		{ id: 'standard-e', label: 'Standard E' },
		{ id: 'flat-e',     label: 'Eb / Half-step Down' },
		{ id: 'drop-d',     label: 'Drop D' },
	];

	export let tuning: string = tunings[0].id;

	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';

	let YinPitchDetector: any;

	let canvas_container: HTMLDivElement | null = null;

	let canvas_static: HTMLCanvasElement | null = null;
	let ctx_static: CanvasRenderingContext2D | null = null;
	let canvas_dynamic: HTMLCanvasElement | null = null;
	let ctx_dynamic: CanvasRenderingContext2D | null = null;

	let audioContext: AudioContext | null = null;
	let workletNode: AudioWorkletNode | null = null;
	let input: MediaStreamAudioSourceNode | null = null;

	function resetCanvas() {
		if (!ctx_dynamic || !canvas_dynamic) {
			console.error('Canvas or context not found');
			return;
		}
		ctx_dynamic.clearRect(0, 0, canvas_dynamic.width, canvas_dynamic.height);
	}

	function drawScale() {
		if (!canvas_static || !ctx_static || !canvas_container) {
			console.error('Canvas or context not found');
			return;
		}
		const canvas = canvas_static;
		const ctx = ctx_static;
		// Set canvas dimensions

		const { clientWidth: w, clientHeight: h } = canvas_container;
		const DPR = window.devicePixelRatio || 1;
		const pixW = Math.floor(w * DPR);
		const pixH = Math.floor(h * DPR);

		const width = pixW;
		const height = pixH;

		// Draw the linear scale
		const startX = 0;
		const endX = width;
		const scaleY = height / 2;

		const centerX = (endX - startX) / 2;
		const drawScaleYMin = scaleY - (height * .20);
		const drawScaleYMax = scaleY + (height * .20);

		const lineWidth = 2 * DPR; //(height * .012);

		// draw center line
		ctx.beginPath();
		ctx.strokeStyle = '#FFFFFF';
		ctx.lineWidth = lineWidth;
		ctx.moveTo(centerX, drawScaleYMin); // Start above the main line
		ctx.lineTo(centerX, drawScaleYMax); // End below the main line
		ctx.stroke();

		// circle
		const radius = 40 * DPR;
		const centerY = drawScaleYMin - radius;
		ctx.beginPath();
		ctx.strokeStyle = '#FFFFFF';
		ctx.lineWidth = lineWidth*2;
		ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
		ctx.stroke();

		// Triangle (nudge) at the bottom of the circle
		const triHeight = 12 * DPR;           // how tall your nudge is
		const triWidth  = 12 * DPR;           // how wide its base is
		const baseY     = centerY + radius;  // sits flush on the circle’s bottom

		ctx.beginPath();
		ctx.moveTo(centerX - triWidth/2, baseY);      // bottom-left of circle
		ctx.lineTo(centerX + triWidth/2, baseY);      // bottom-right of circle
		ctx.lineTo(centerX,            baseY + triHeight); // pointy tip
		ctx.closePath();

		ctx.fillStyle = '#FFFFFF';
		ctx.fill();
		ctx.strokeStyle = '#FFFFFF';
		ctx.stroke();
	}
	
	// Function to draw the indicator at a specific value
	function drawIndicator(tuningTo:any | null, value:number) {
		if (!canvas_dynamic || !ctx_dynamic || !canvas_container) {
			console.error('Canvas or context not found');
			return;
		}

		const canvas = canvas_dynamic;
		const ctx = ctx_dynamic;

		const { clientWidth: w, clientHeight: h } = canvas_container;
		const DPR = window.devicePixelRatio || 1;
		const pixW = Math.floor(w * DPR);
		const pixH = Math.floor(h * DPR);

		// Set canvas dimensions
		const width = pixW;
		const height = pixH;

		const startX = 0; // Starting X position of the scale
		const endX = width; // Ending X position of the scale
		const scaleY = height / 2; // Vertical position of the scale
		const centerX = (endX - startX) / 2;
		const drawScaleYMin = scaleY - (height * .20);
		const radius = 40 * DPR;
		const centerY = drawScaleYMin - radius;

		// labels
		const noteLabel = tuningTo.note;
		// const label2 = `${tuningTo.freq} Hz`;

		const fontSize = 24 * DPR;
		const fontWeight = 'bold';
		ctx.font = `${fontWeight} ${fontSize}px Arial`;
		ctx.fillStyle = 'white';
		ctx.textAlign = 'center';
		//ctx.textBaseline = 'middle';

		const metrics = ctx.measureText(noteLabel);
		const ascent  = metrics.actualBoundingBoxAscent;
		const descent = metrics.actualBoundingBoxDescent;
		const yOffset = (ascent - descent) / 2;

		ctx.textBaseline = 'alphabetic';
		ctx.fillText(noteLabel, centerX, centerY + yOffset); // Label below the tick
		//ctx.fillText(label2, centerX, scaleY + (height * .30)); // Label below the tick


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
		
		const fontSizeLabel = 12 * DPR;
		ctx.font = `${fontSizeLabel}px Arial`;
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
		if (!canvas_container || !canvas_static || !canvas_dynamic || !ctx_static || !ctx_dynamic) {
			console.error("missing DOM refs");
			return;
		}

		const { clientWidth: w, clientHeight: h } = canvas_container;
		const DPR = window.devicePixelRatio || 1;
		const pixW = Math.floor(w * DPR);
		const pixH = Math.floor(h * DPR);

		if (canvas_static.width !== pixW || canvas_static.height !== pixH) {
			// redraw only when size changed
			canvas_static.width  = pixW;
			canvas_static.height = pixH;
			drawScale();
		}

		canvas_dynamic.width = pixW;
		canvas_dynamic.height = pixH;
		ctx_dynamic.clearRect(0, 0, pixW, pixH);

		// Testing
		const tuningTo = { note: 'E2', freq: 82.41 };
		drawIndicator(tuningTo, 84.31);
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
					//drawScale(tuningTo);
					drawIndicator(tuningTo, pitch.freq);
				}
			}
		};
	}
	let containerRO: ResizeObserver;

	// onMount logic
	onMount(async () => {
		canvas_container = document.getElementById('canvas_container') as HTMLDivElement;
		canvas_static = document.getElementById('canvas_static')  as HTMLCanvasElement;
		ctx_static = canvas_static?.getContext('2d') ?? null;
		canvas_dynamic = document.getElementById('canvas_dynamic') as HTMLCanvasElement;
		ctx_dynamic = canvas_dynamic?.getContext('2d') ?? null;

		if (!canvas_static || !ctx_static || !canvas_dynamic || !ctx_dynamic) {
			console.error('Canvas elements not found');
			return;
		}

		// this should work on mobile too
		resizeCanvas();
		containerRO = new ResizeObserver(() => resizeCanvas());
		containerRO.observe(canvas_container);

		// fall back on window resize/orientation just in case
		window.addEventListener('resize', resizeCanvas, { passive: true });

		// load WebAssembly + start microphone & pitch loop
		await loadWasm();
		await run();
	});

	// tidy up if the component unmounts 
	onDestroy(() => {
		workletNode?.disconnect();
		input?.disconnect();
		audioContext?.close();
	});
</script>

<svelte:head>
	<title>No-Fuzz Tuner</title>
	<meta name="description" content="Guitar Tuner - Browser-Based &amp; Free - No-Fuzz" />
</svelte:head>

<section>
    <div id="controls-container">
        <div id="controls">
			<label class="tuning-label">
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
	<div id="canvas_container">
		<canvas id="canvas_static"></canvas>
		<canvas id="canvas_dynamic"></canvas>
	</div>
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
	section {
		/* give it room to center inside */
		height: 100svh;

		display: flex;
		flex-direction: column;
		justify-content: center; /* center vertically */
		align-items: center;
	}

	#canvas_container {
		/* 80 % of the viewport width, 60 % of the viewport height */
		width: 80vw;
		height: 60svh;   /* ≈ “safe”, never pushes off-screen */
		position: relative;
	}

	@supports (height: 100dvh) {     /* Chrome / Safari ≥108, Firefox 113+ */
		#canvas_container {
			height: 60dvh;               /* tracks address-bar show/hide */
		}
	}

	/* Make both canvases fill—and overlap—the container */
	#canvas_container canvas {
		position: absolute;
		inset: 0; /* top:0; right:0; bottom:0; left:0; */
		width: 100%;
		height: 100%;
		display: block;
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

	#controls-container {
		position: fixed;
		bottom: calc(5% + env(safe-area-inset-bottom)); /* avoids the home bar */
		width: 100%;
		display: flex;
		justify-content: center;
	}
	
	#controls {
		padding: 5px;
		background-color: var(--bg);
	}
	
	.tuning-label {
		display: block;
		color: var(--fg);
		font-size: 0.9rem;
	}

	.tuning-select {
		margin-top: 0.25rem;
		width: 100%;
		text-align: center;
		font-size: 1rem;
		padding: 0.5rem 0.5rem;
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
