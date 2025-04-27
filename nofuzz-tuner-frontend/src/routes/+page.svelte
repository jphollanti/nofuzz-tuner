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
	// driven by signed cents (–50 … +50).
	function drawIndicator(tuningTo: any, cents: number) {
		if (!canvas_dynamic || !ctx_dynamic || !canvas_container) return;

		// Helpers
		const { clientWidth: w, clientHeight: h } = canvas_container;
		const DPR = window.devicePixelRatio || 1;
		const W    = Math.floor(w * DPR);
		const H    = Math.floor(h * DPR);

		ctx_dynamic.clearRect(0, 0, W, H);
		const ctx = ctx_dynamic;

		const midY   = H / 2;                 // horizontal centre line
		const midX   = W / 2;                 // vertical mid
		const range  = 50;                    // ± 50 ¢ span
		const sign   = Math.sign(cents) || 1; // –1 for flat, +1 for sharp (treat 0 as +)
		const absC   = Math.abs(cents);
		const clampC = Math.min(range, absC);

		let indicatorX = midX + (clampC / range) * midX * sign;

		// Note label
		const fillStyle = absC <= 2 ? '#4CAF50' : 'white';
		ctx.font = `bold ${24 * DPR}px Arial`;
		ctx.fillStyle = fillStyle;
		ctx.textAlign = 'center';
		ctx.textBaseline = 'alphabetic';

		const metrics = ctx.measureText(tuningTo.note);
		const ascent  = metrics.actualBoundingBoxAscent;
		const descent = metrics.actualBoundingBoxDescent;
		const yOffset = (ascent - descent) / 2;

		const circleY = midY - H * 0.20 - 40 * DPR;
		ctx.fillText(tuningTo.note, midX, circleY + yOffset);

		// Colors for the indicator
		const colour =
			absC <= 2   ? '#4CAF50' :   // spot-on
			absC <= 10  ? '#FFEB3B' :   // close
						'#FF4C4C';    // off

		// Arrows
		// 1. Draw a single arrow pointing TOWARD the centre line
		// 2. Draw a second arrow if the pitch is more than ±20 ¢ off
		// 3. Don’t draw anything if the pitch is essentially perfect
		if (absC > 1) {
			const dx  = H * 0.03;
			const dy  = H * 0.03;

			// helper draws one arrow pointing TOWARD the centre line
			const drawArrow = (shift: number, fill: string) => {
				/*  shift is a positive distance
					sign =  +1 if sharp, -1 if flat
					We *subtract* sign so the apex flips sides               */
				const baseX = indicatorX - sign * shift;

				ctx.beginPath();
				ctx.moveTo(indicatorX, midY - dy);         // tip (near tick)
				ctx.lineTo(baseX,      midY);              // apex (points inward)
				ctx.lineTo(indicatorX, midY + dy);
				ctx.closePath();
				ctx.fillStyle = fill;
				ctx.fill();
			};

			drawArrow(dx, colour);

			// extra arrow beyond ±20 ¢
			if (absC > 20) {
				let origX = indicatorX;
				indicatorX -= sign * dx / 2; // shift the tip to the right
				drawArrow(dx + H * 0.003, '#FF5E5E');
				indicatorX = origX; // reset the tip
			}
		}

		// Vertical line
		ctx.beginPath();
		ctx.strokeStyle = colour;
		ctx.lineWidth   = 2 * DPR;
		ctx.moveTo(indicatorX, midY - H * 0.07);
		ctx.lineTo(indicatorX, midY + H * 0.07);
		ctx.stroke();

		// Display the cents value
		// const label = `${cents > 0 ? '+' : ''}${cents.toFixed(1)} ¢`;
		// ctx.font = `${12 * DPR}px Arial`;
		// ctx.fillStyle = 'white';

		// const boxW   = ctx.measureText(label).width + H * 0.03;
		// const labelX = Math.max(boxW / 2, Math.min(W - boxW / 2, indicatorX));

		// ctx.fillStyle = '#121212';
		// ctx.fillRect(labelX - boxW / 2, midY + H * 0.10, boxW, 30);

		// ctx.fillStyle = 'white';
		// ctx.textBaseline = 'middle';
		// ctx.fillText(label, labelX, midY + H * 0.13);
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
		drawIndicator(tuningTo, 10.0);
	}

	async function loadWasm() {
		if (!browser) return;
		const pkg = await import('../lib/no_fuzz_tuner/pkg/nofuzz_tuner_lib.js');
		await pkg.default();
		YinPitchDetector = pkg.YinPitchDetector;
	}
	async function unlockAudio(ctx: AudioContext | null) {
		if (!ctx) return;                 // guard
		if (ctx.state === 'suspended') {
			try { await ctx.resume(); } catch {/* ignore */}
		}

		/* ✨ Re-assign (even to the same object) to trigger an update */
		audioContext = ctx;
	}

	async function run() {
		audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();

		/* listen once for any tap/click/keydown */
		['click', 'touchstart', 'keydown'].forEach(evt =>
			window.addEventListener(evt, () => unlockAudio(audioContext!), { once: true, passive: true })
		);
		await audioContext.resume();
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
			/** signed cents between a measured freq and the target note freq */
			const centsDiff = (freq: number, target: number) =>
				1200 * Math.log2(freq / target);          // + = sharp, – = flat

			if (filled >= BLOCK) {
				filled = 0; // buffer ready
				const pitch = detector.maybe_find_pitch_js(buf, tuning);
				if (pitch) {
					const tuningTo = pitch.tuningTo;        // { note, freq }
					const cents =
						typeof pitch.cents === 'number'
						? pitch.cents                      // detector provides it
						: centsDiff(pitch.freq, tuningTo.freq);  // fallback

					resetCanvas();
					//console.log(pitch);
					drawIndicator(tuningTo, cents);
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
	{#if audioContext && audioContext.state === 'suspended'}
		<div
			class="start-overlay"
			role="button"
			tabindex="0"
			on:click={() => unlockAudio(audioContext)}
			on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && unlockAudio(audioContext)}
		>
			Tap or press ⏎ to start tuner
		</div>
	{/if}
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

	.start-overlay {
		position: fixed;
		inset: 0;
		display: grid;
		place-content: center;
		background:#000c;
		color:#fff;
		font: 1.2rem/1 system-ui;
		z-index: 10000;
	}

	/* 1. Reset browser chrome so we know what we’re styling */
	.tuning-select {
		/* preserve your look */
		font: inherit;
		padding: 0.45rem 2.2rem 0.45rem 0.6rem;  /* extra space on the right */
		color: var(--fg);
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 0.375rem;
		cursor: pointer;
		transition: border 0.2s, box-shadow 0.2s;

		text-align: right;          /* Chrome / Safari / Chromium Edge */
		text-align-last: right;     /* Edge-Legacy, some Blink builds  */
		padding-right: 2.2rem;      /* keep gap for the SVG arrow */

		/* kill native arrow everywhere */
		appearance: none;          /* modern */
		-webkit-appearance: none;  /* Safari */
	}
	/* Firefox still ignores text-align on <select>.
	Trick is: flip writing direction, then flip text back. */
	@supports (-moz-appearance: none) {
		.tuning-select {
			direction: rtl;
			text-align: left;
		}
	}

	/* Add a lightweight SVG arrow as a background image */
	.tuning-select {
		background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 12 8'%3E%3Cpath fill='%23aaa' d='M1 1l5 5 5-5'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 0.7rem center;
		background-size: 0.65rem;
	}

	/* focus & disabled tweaks */
	.tuning-select:focus {
		outline: none;
		border-color: var(--accent);
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 50%, transparent);
	}

	.tuning-select:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
</style>
