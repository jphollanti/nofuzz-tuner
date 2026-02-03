<script lang="ts">
	import { env as PUBLIC } from '$env/dynamic/public';
	import { draw, fade } from 'svelte/transition';
	import workletURL from '$lib/audio/pitch-worklet.js?url';
	import PitchParticles from '$lib/PitchParticles.svelte';

  	let particleCents: number | null = null;   // update this from your tuner logic

	/* Data stuctures */
	class PitchDetector {
		detector: any;

		block: number;
		buf: Float32Array;
		quantum: number;

		tuning: any;

		// state
		write = 0;
		filled = 0;

		constructor(
			threshold: number, 
			freq_min: number, 
			freq_max: number, 
			sampleRate: number,  
			block: number, 
			filters: number,
			features: number,
			quantum: number,
			tuning: any = null, 
			averageBufferSize: number = 3,
			clarityAlpha: number = .4
		) {
			this.block = block;
			this.buf = new Float32Array(this.block);
			this.quantum = quantum;
			this.tuning = tuning;
			this.detector = new YinPitchDetector(
				threshold, 
				freq_min, 
				freq_max, 
				sampleRate, 
				block, 
				filters, 
				features, 
				averageBufferSize, 
				clarityAlpha);
		}

		add_string_filter(freq: number) {
			this.detector.add_string_filter(freq);
		}

		detect(chunk: Float32Array): any | null {
			if (!chunk) return null;

			this.buf.set(chunk, this.write);
			this.write = (this.write + this.quantum) % this.block;
			this.filled += this.quantum;

			if (this.filled >= this.block) {
				this.filled = 0;
				const start = performance.now();
				return this.detector.maybe_find_pitch_js(this.buf, this.tuning.id);
			}
			return null;
		}
	}

	// find the most common frequency in the window
	function mostCommon(arr: number[]): number {
		const counts: Record<number, number> = {};
		let maxNum: number = arr[0];
		let maxCount: number = 0;

		for (const num of arr) {
			counts[num] = (counts[num] || 0) + 1;
			if (counts[num] > maxCount) {
			maxNum = num;
			maxCount = counts[num];
			}
		}

		return maxNum;
	}

	function setBits(...positions: number[]): number {
		return positions.reduce((acc, bit) => acc | (1 << bit), 0);
	}

	class StringDetector extends PitchDetector {
		selected_note: number | null = null;
		window: number[] = []
		window_size: number = 4;

		// If the guitar is not tuned well, cents will be higher in 
		// general. The better tuned the guitar, the tighter cent control we can 
		// have. 
		cents: number[] = []
		cents_window_size: number = 10;
		cents_buffer: number = 10;
		
		constructor(threshold: number, freq_min: number, freq_max: number, sampleRate: number, tuning: any = null) {
			const filters = setBits(0, 1, 2, 3, 4, 5); // highpass, notch50, notch60, notch100, notch120, lowpass
			// const block = 4096 * fftBlockSizeMultiplier;
			const block = 16384 / 2; // TODO: needs more experimentation
			const quantum = 128;
			const features = 0; // no features
			super(
				threshold, 
				freq_min, 
				freq_max, 
				sampleRate, 
				block, 
				filters, 
				features, 
				quantum, 
				tuning);
		}

		push(chunk: Float32Array): number | null {
			const pitch = this.detect(chunk);
			if (pitch) {

				this.cents.push(Math.abs(pitch.tuningTo.cents));
				if (this.cents.length > this.cents_window_size) {
					this.cents.shift();
				}
				const avg_cents = this.cents.reduce((a, b) => a + b, 0) / this.cents.length;
				const allow_cents = avg_cents + this.cents_buffer;
				//console.log("cents:", pitch.tuningTo.cents, "allow_cents:", allow_cents);
				
				// We do this to exclude outliers from the string selection process.
				const is_within_bounds = Math.abs(pitch.tuningTo.cents) < allow_cents;

				if (is_within_bounds) {
					this.window.push(pitch.tuningTo.freq);
					if (this.window.length > this.window_size) {
						this.window.shift();
					}
					if (this.window.length > this.window_size / 2) {
						return mostCommon(this.window);
					}
				}
			}
			return null;
		}
	}
	
	type TuningPreset = {
		id: string;
		label: string;
		note_names: string[];
		freqs: number[];
		detectors: Map<number, PitchDetector>;
		stringDetector: StringDetector | null;
	};

	/* Instrument presets - different instruments need different processing */
	type InstrumentPresetType = 'acoustic' | 'electric-clean' | 'electric-distorted' | 'classical' | 'bass' | 'extended-range';

	const INSTRUMENT_PRESETS: { id: InstrumentPresetType; label: string; description: string }[] = [
		{ id: 'acoustic', label: 'Acoustic', description: 'Standard acoustic guitar' },
		{ id: 'electric-clean', label: 'Electric (Clean)', description: 'Electric guitar, clean tone' },
		{ id: 'electric-distorted', label: 'Electric (Distorted)', description: 'Electric guitar with distortion/overdrive' },
		{ id: 'classical', label: 'Classical/Nylon', description: 'Nylon string classical guitar' },
		{ id: 'bass', label: 'Bass', description: 'Electric or acoustic bass' },
		{ id: 'extended-range', label: 'Extended Range', description: '7/8 string guitars' },
	];

	/* Tuning selections */
	const TUNINGS: TuningPreset[] = [
		// Standard 6-string tunings
		{
			id: 'standard-e',
			label: 'Standard E',
			note_names: ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
			freqs: [82.41, 110.00, 146.83, 196.00, 246.94, 329.63],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'flat-e',
			label: 'Eb / Half-step Down',
			note_names: ['Eb2', 'Ab2', 'Db3', 'Gb3', 'Bb3', 'Eb4'],
			freqs: [77.78, 103.83, 138.59, 185.00, 233.08, 311.13],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'drop-d',
			label: 'Drop D',
			note_names: ['D2', 'A2', 'D3', 'G3', 'B3', 'E4'],
			freqs: [73.42, 110.00, 146.83, 196.00, 246.94, 329.63],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'd-standard',
			label: 'D Standard (Full step down)',
			note_names: ['D2', 'G2', 'C3', 'F3', 'A3', 'D4'],
			freqs: [73.42, 98.00, 130.81, 174.61, 220.00, 293.66],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'open-g',
			label: 'Open G',
			note_names: ['D2', 'G2', 'D3', 'G3', 'B3', 'D4'],
			freqs: [73.42, 98.00, 146.83, 196.00, 246.94, 293.66],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'dadgad',
			label: 'DADGAD',
			note_names: ['D2', 'A2', 'D3', 'G3', 'A3', 'D4'],
			freqs: [73.42, 110.00, 146.83, 196.00, 220.00, 293.66],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		// 7-string guitar tunings
		{
			id: '7-string-standard',
			label: '7-String Standard (B)',
			note_names: ['B1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
			freqs: [61.74, 82.41, 110.00, 146.83, 196.00, 246.94, 329.63],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: '7-string-drop-a',
			label: '7-String Drop A',
			note_names: ['A1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
			freqs: [55.00, 82.41, 110.00, 146.83, 196.00, 246.94, 329.63],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		// 8-string guitar tuning
		{
			id: '8-string-standard',
			label: '8-String Standard (F#)',
			note_names: ['F#1', 'B1', 'E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
			freqs: [46.25, 61.74, 82.41, 110.00, 146.83, 196.00, 246.94, 329.63],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		// Ukulele
		{
			id: 'ukulele-gcea',
			label: 'Ukulele GCEA',
			note_names: ['G4', 'C4', 'E4', 'A4'],
			freqs: [392.00, 261.63, 329.63, 440.00],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		// Bass tunings
		{
			id: 'bass-e',
			label: 'Bass 4-String EADG',
			note_names: ['E1', 'A1', 'D2', 'G2'],
			freqs: [41.20, 55.00, 73.42, 98.00],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'bass-5-string',
			label: 'Bass 5-String BEADG',
			note_names: ['B0', 'E1', 'A1', 'D2', 'G2'],
			freqs: [30.87, 41.20, 55.00, 73.42, 98.00],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
		{
			id: 'bass-drop-d',
			label: 'Bass Drop D',
			note_names: ['D1', 'A1', 'D2', 'G2'],
			freqs: [36.71, 55.00, 73.42, 98.00],
			detectors: new Map<number, PitchDetector>(),
			stringDetector: null
		},
	];
	
	// Variables
	const buildVersion =
		PUBLIC.PUBLIC_BUILD_VERSION
		?? `dev-${new Date().toISOString()}`;
	const isDev = buildVersion.startsWith('dev-');
	let devUpdateTimer: any = null;

	/* Tooltip visibility */
	let open = false;

	let sampleRate = -1;

	// Performance metrics
	// Budget = time between consecutive audio callbacks. Processing time should be < 50 % of the buffer duration to leave head‑room.
	// Todo: measure the buffer duration
	let algoPerformance_1 = -1;
	let algoPerformance_2 = -1;
	let drawPerformance = -1;

	// These values should come from config.yaml
	// or similar, but for now we hardcode them
	const threshold = 0.1;
	// const freq_min = 60;
	// const freq_max = 500;

	const stringFftRefine = false;
	const pitchFftRefine = true;

	// FFT refinement requires large block sizes.
	// TODO: the current value of 8 is large leading to slow
	// updates on UI. But it seems to be very accurate.
	const fftBlockSizeMultiplier = 8;

	export let tuning: string = TUNINGS[0].id;
	let instrumentPreset: InstrumentPresetType = 'acoustic';

	// Detection quality feedback
	let lastConfidence: number = 0;
	let lastRms: number = 0;

	// Get instrument-specific configuration
	function getInstrumentConfig(preset: InstrumentPresetType) {
		const configs: Record<InstrumentPresetType, {
			enableAgc: boolean;
			enableHarmonicCorrection: boolean;
			targetRms: number;
			blockMultiplier: number;
			extraFeatures: number; // bits 3,4,5 for AGC, harmonic, octave correction
		}> = {
			'acoustic': {
				enableAgc: false,
				enableHarmonicCorrection: false,
				targetRms: 0.1,
				blockMultiplier: 1.0,
				extraFeatures: 0b000000, // No extra features
			},
			'electric-clean': {
				enableAgc: true,
				enableHarmonicCorrection: true,
				targetRms: 0.1,
				blockMultiplier: 1.0,
				extraFeatures: 0b011000, // AGC + Harmonic correction
			},
			'electric-distorted': {
				enableAgc: true,
				enableHarmonicCorrection: true,
				targetRms: 0.15,
				blockMultiplier: 2.0,
				extraFeatures: 0b111000, // AGC + Harmonic + Octave correction
			},
			'classical': {
				enableAgc: true,
				enableHarmonicCorrection: false,
				targetRms: 0.08,
				blockMultiplier: 1.5,
				extraFeatures: 0b001000, // AGC only
			},
			'bass': {
				enableAgc: true,
				enableHarmonicCorrection: true,
				targetRms: 0.1,
				blockMultiplier: 2.0,
				extraFeatures: 0b111000, // All corrections
			},
			'extended-range': {
				enableAgc: true,
				enableHarmonicCorrection: true,
				targetRms: 0.1,
				blockMultiplier: 2.5,
				extraFeatures: 0b111000, // All corrections
			},
		};
		return configs[preset];
	}

	import { onMount, onDestroy } from 'svelte';
	import { browser, dev } from '$app/environment';

	let YinPitchDetector: any;
	// Todo: use this from Rust instead of setBits
	//let set_bits: any;

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
	
	const DPR = window.devicePixelRatio || 1;
	const radius = 200 * DPR;

	function drawScale() {
		if (!canvas_static || !ctx_static || !canvas_container) {
			console.error('Canvas or context not found');
			return;
		}
		const canvas = canvas_static;
		const ctx = ctx_static;
		// Set canvas dimensions

		const { clientWidth: w, clientHeight: h } = canvas_container;
		
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
		const scaleColour = getScaleColour();
		ctx.beginPath();
		ctx.strokeStyle = scaleColour;
		ctx.fillStyle = scaleColour;
		ctx.lineWidth = lineWidth;
		ctx.moveTo(centerX, drawScaleYMin); // Start above the main line
		ctx.lineTo(centerX, drawScaleYMax); // End below the main line
		ctx.stroke();

		// circle
		const centerY = drawScaleYMax + radius;
		ctx.beginPath();
		ctx.strokeStyle = scaleColour;
		ctx.lineWidth = lineWidth*2;
		ctx.arc(centerX, centerY, radius, Math.PI * 1.332, Math.PI * 1.668);
		//ctx.arc(centerX, centerY, radius, Math.PI, Math.PI * 2);
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

		ctx.fillStyle = scaleColour;
		ctx.fill();
		ctx.strokeStyle = scaleColour;
		ctx.stroke();
	}
	
	const getScaleColour = () =>
		getComputedStyle(document.documentElement)
			.getPropertyValue('--scale-fg').trim() || '#fff';

	
	let latestTuningTo = { note: "A" }; // whatever your app feeds in

	/**
	 * Call this whenever new pitch data arrives.
	 * The needle will glide from its current position
	 * to the new one instead of jumping.
	 */
	function updateIndicator(tuningTo:any, newCents:number) {
		latestTuningTo = tuningTo;

		// if the target changed, start / restart tween
		if (newCents !== targetCents) {
			startCents = currentCents;
			targetCents = newCents;
			tweenStart = performance.now();
			requestAnimationFrame(tick);
		}
	}
	// ------------- Animation state -------------
	let currentCents = 0;           // where the needle is right now
	let targetCents  = 0;           // where we want it to end up
	let startCents   = 0;           // value at the moment the tween starts
	let tweenStart   = 0;           // time (ms) tween started
	const TWEEN_MS   = 180;         // duration of one glide (adjust to taste)

	// simple ease-out (feels a bit “springy”, but cheap)
	const easeOutCubic = (t: number) => 1 - Math.pow(1 - t, 3);

	// ------------  Main animation loop ---------
	function tick(now: number) {
		// progress 0‥1
		const t = Math.min(1, (now - tweenStart) / TWEEN_MS);
		const eased = easeOutCubic(t);

		// lerp between the start value and the latest target
		const centsNow = startCents + (targetCents - startCents) * eased;

		drawIndicator(latestTuningTo, centsNow);   // <-- your original painter

		currentCents = centsNow;                   // keep state in sync

		if (t < 1) requestAnimationFrame(tick);    // still gliding
	}

	// Function to draw the indicator at a specific value
	// driven by signed cents (–50 … +50).
	function drawIndicator(tuningTo: any, cents: number) {
		if (!canvas_dynamic || !ctx_dynamic || !canvas_container) return;

		// Helpers
		const { clientWidth: w, clientHeight: h } = canvas_container;
		const DPR = window.devicePixelRatio || 1;
		const W = Math.floor(w * DPR);
		const H = Math.floor(h * DPR);

		ctx_dynamic.clearRect(0, 0, W, H);
		const ctx = ctx_dynamic;

		const midY   = H / 2;                 // horizontal centre line
		const midX   = W / 2;                 // vertical mid
		const range  = 50;                    // ± 50 ¢ span
		const sign   = Math.sign(cents) || 1; // –1 for flat, +1 for sharp (treat 0 as +)
		const absC   = Math.abs(cents);
		const clampC = Math.min(range, absC);

		// Note label
		const NOTE_COLOR = absC <= 2 ? '#4CAF50' : getScaleColour();
		const fillStyle = NOTE_COLOR;
		ctx.font = `bold ${24 * DPR}px Arial`;
		ctx.fillStyle = fillStyle;
		ctx.textAlign = 'center';
		ctx.textBaseline = 'alphabetic';

		const metrics = ctx.measureText(tuningTo.note);
		const ascent  = metrics.actualBoundingBoxAscent;
		const descent = metrics.actualBoundingBoxDescent;
		const yOffset = (ascent - descent) / 2;

		const circleY = midY + H * 0.25 + 40 * DPR;
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

			// draw to center instead
			let arrowX = midX
			let arrowY = midY + H * 0.25;

			const dx = H * 0.015;
			const dy = H * 0.015;

			// helper draws one arrow pointing TOWARD the centre line
			const drawArrow = (shift: number, fill: string) => {
				/*  shift is a positive distance
					sign =  +1 if sharp, -1 if flat
					We *subtract* sign so the apex flips sides               */
				const baseX = arrowX - sign * shift;

				ctx.beginPath();
				ctx.moveTo(arrowX, arrowY - dy);         // tip (near tick)
				ctx.lineTo(baseX, arrowY);              // apex (points inward)
				ctx.lineTo(arrowX, arrowY + dy);
				ctx.closePath();
				ctx.fillStyle = fill;
				ctx.fill();
			};

			drawArrow(dx, colour);

			// extra arrow beyond ±20 ¢
			if (absC > 20) {
				let origX = arrowX;
				arrowX -= sign * dx / 2; // shift the tip to the right
				drawArrow(dx + H * 0.003, '#FF5E5E');
				arrowX = origX; // reset the tip
			}
		}

		function drawNeedle(cents:number, centerX:number, centerY:number, length:number, colour:string) {
			// Clamp cents to the expected range
			cents = Math.max(-30, Math.min(30, cents));

			// Convert cents to angle in radians (max ±30° = ±π/6)
			const maxAngle = Math.PI / 6;
			const angle = (cents / 30) * maxAngle;

			ctx.save();

			ctx.translate(centerX, centerY); // Move origin to pivot point
			ctx.rotate(angle);               // Rotate needle based on tuning

			const lineWidth = 2 * DPR;

			ctx.beginPath();
			ctx.moveTo(0, -radius-lineWidth); // Needle tip
			ctx.lineTo(0, -length-radius); // Needle length upwards
			ctx.strokeStyle = colour;
			ctx.lineWidth = 2 * DPR;
			ctx.stroke();

			ctx.restore();
		}

		const pixH = Math.floor(h * DPR);
		const height = pixH;
		const scaleY = height / 2;
		const drawScaleYMax = scaleY + (height * .20);
		const drawScaleYMin = scaleY - (height * .20);
		const needleY = drawScaleYMax + radius;
		let length = drawScaleYMax - drawScaleYMin;
		drawNeedle(cents, midX, needleY, length, colour);
		
		const white = NOTE_COLOR;
		length = (drawScaleYMax - drawScaleYMin) * .9;
		drawNeedle(cents, midX, needleY, length, getScaleColour());
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
		particleCents = null;

		// Testing
		if (isDev) {
			// const tuningTo = { note: 'E2', freq: 82.41 };
			// particleCents = -10;
			// updateIndicator(tuningTo, particleCents);
			if (devUpdateTimer) {
				clearInterval(devUpdateTimer);
			}
			const tuningTo = { note: 'E2', freq: 82.41 };
			
			particleCents = -30;

			const STEP   = 10;
			const LIMIT  = 30;
			const PERIOD = 1000;

			let direction = 1;    // 1 = upward, -1 = downward triangle wave

			devUpdateTimer = setInterval(() => {
				particleCents = (particleCents ?? 0) + STEP * direction;
				updateIndicator(tuningTo, particleCents);
				// flip direction at the edges
				if (particleCents >=  LIMIT || particleCents <= -LIMIT) {
					direction *= -1;
				}
			}, PERIOD);

			// particleCents = 20;
			// updateIndicator(tuningTo, particleCents);
		}
	}

	async function loadWasm() {
		if (!browser) return;
		const pkg = await import('../lib/no_fuzz_tuner/pkg/nofuzz_tuner_lib.js');
		await pkg.default();
		YinPitchDetector = pkg.YinPitchDetector;
		// Todo: use this from Rust instead of setBits
		// set_bits = pkg.set_bits_js;

		// add all tunings to the package
		for (const tuning of TUNINGS) {
			const freqs = new Float64Array(tuning.freqs);
			pkg.add_tuning(
				tuning.id, 
				tuning.label,
				tuning.note_names,
				freqs);
			tuning.detectors = new Map<number, PitchDetector>();
		}

		const t2 = pkg.get_tunings();
		console.log('tunings', t2);
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

		sampleRate = audioContext.sampleRate; 

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

		const quantum = 128;
		let selected_freq = 82.41; // default: E2

		function resetDetector(detector: PitchDetector) {
			detector.write = 0;
			detector.filled = 0;
			detector.buf.fill(0);
			// detector.detector.reset(freq);
		}

		function resetDetectors(detectors: Map<number, PitchDetector>) {
			for (const [key, detector] of detectors.entries()) {
				resetDetector(detector);
			}
		}
		// Small helper:
		function nextPow2(x:number) { return 1 << (32 - Math.clz32(x - 1)); }

		// Block‑size chooser:
		function blockSize(freq:number, sampleRate:number, periods = 3, min = 1024, max = 8192) {
			const raw = periods * sampleRate / freq;          // cycles-based length
			const pow2 = nextPow2(raw);                       // round‑up
			return Math.max(min, Math.min(pow2, max));        // clamp to sane range
		}
		// ± windowSizeCents around `f0`
		function freqBounds(f0:number, windowSizeCents = 120) {          // ±120 ¢ = ±1.2 sem
			const ratio = Math.pow(2, windowSizeCents / 1200);     // cents → ratio
			return [f0 / ratio, f0 * ratio];
		}

		// console.log('generic settings')
		// console.log('- sample rate:', sampleRate);
		// console.log('- quantum:', quantum);

		// Build string specific detectors with instrument preset support
		function buildDetectors(preset: InstrumentPresetType) {
			const instConfig = getInstrumentConfig(preset);
			TUNINGS.forEach(tuning => {
			const freqs = tuning.freqs;
			let stringFilter = setBits(0, 5);
			console.log('-------------------------------------');
			console.log('settings for tuning', tuning.id);
			for (const freq of freqs) {
				// Rough table to determine block size.
				// Note		Freq (Hz)	Block Size @ 44.1 kHz
				// E2		82.41		8192 (≈186 ms window)
				// A2		110.00		6144
				// D3		146.83		4096
				// G3		196.00		3072
				// B3		246.94		2048
				// E4		329.63		2048 or even 1024
				// Block size with instrument-specific multiplier
				let bl = blockSize(freq, sampleRate) * fftBlockSizeMultiplier * instConfig.blockMultiplier;
				const [fMin, fMax] = freqBounds(freq, 120);
				// Base features + instrument-specific extras (bits 3,4,5)
				let features = setBits(0) | instConfig.extraFeatures;
				let avgBufferSize = 3;
				let alpha = 0.4;

				// G3 (196 Hz) - problematic string with rich harmonics
				if (Math.abs(freq - 196.00) < 1) {
					alpha = 0.15;
					features = setBits(0, 1, 2) | instConfig.extraFeatures;
					avgBufferSize = 7;
					bl = blockSize(freq, sampleRate) * fftBlockSizeMultiplier * instConfig.blockMultiplier * 2;
				}
				// Low E2 (82 Hz)
				else if (Math.abs(freq - 82.41) < 1) {
					features = setBits(0, 2) | instConfig.extraFeatures;
					bl = blockSize(freq, sampleRate) * fftBlockSizeMultiplier * instConfig.blockMultiplier * 2;
				}
				// D3 (147 Hz)
				else if (Math.abs(freq - 146.83) < 1) {
					features = setBits(0, 1, 2) | instConfig.extraFeatures;
					avgBufferSize = 5;
					bl = blockSize(freq, sampleRate) * fftBlockSizeMultiplier * instConfig.blockMultiplier * 2;
				}
				// Extended range: Very low frequencies (7/8 string, 5-string bass)
				else if (freq < 65) {
					features = setBits(0, 1, 2) | instConfig.extraFeatures;
					avgBufferSize = 5;
					bl = Math.max(16384, blockSize(freq, sampleRate) * fftBlockSizeMultiplier * instConfig.blockMultiplier * 2);
				}
				// Bass strings (E1=41Hz to G2=98Hz)
				else if (freq < 100) {
					features = setBits(0, 1, 2) | instConfig.extraFeatures;
					avgBufferSize = 4;
					bl = blockSize(freq, sampleRate) * fftBlockSizeMultiplier * instConfig.blockMultiplier * 1.5;
				}

				console.log('- string', tuning.note_names[freqs.indexOf(freq)], 'preset:', preset);
				console.log('  freq:', freq, 'block:', bl, 'features:', features.toString(2));

				const detector = new PitchDetector(
					threshold,
					fMin,
					fMax,
					sampleRate,
					bl,
					stringFilter,
					features,
					quantum,
					tuning,
					avgBufferSize,
					alpha);

				// Set expected frequency for octave correction
				detector.detector.set_expected_freq(freq);
				// Configure instrument-specific AGC
				if (instConfig.enableAgc) {
					detector.detector.set_agc(true, instConfig.targetRms);
				}
				// Configure harmonic correction
				if (instConfig.enableHarmonicCorrection) {
					detector.detector.set_harmonic_correction(true);
				}

				detector.add_string_filter(freq);
				tuning.detectors.set(freq, detector);
			}
			// String detector with extended range for low frequencies
				const minFreq = Math.min(...freqs);
				const maxFreq = Math.max(...freqs);
				const lowBuffer = minFreq < 50 ? 20 : 30;
				const sd_freq_min = Math.max(20, minFreq - lowBuffer);
				const sd_freq_max = maxFreq + 30;
				tuning.stringDetector = new StringDetector(threshold, sd_freq_min, sd_freq_max, sampleRate, tuning);
			});
		}

		// Initial build with current preset
		buildDetectors(instrumentPreset);

		let selectedTuning = tuning;
		let selectedPreset = instrumentPreset;
		let tuningObject = TUNINGS.find(t => t.id === tuning) || TUNINGS[0];

		let algo1_Array: number[] = [];
		let algo2_Array: number[] = [];
		let draw_Array: number[] = [];

		const TIMEOUT_MS = 3000;
		let lastSampleTime = performance.now();

		workletNode.port.onmessage = ({ data }: MessageEvent<Float32Array>) => {
			const chunk = data; // 128 samples

			// Check if tuning or instrument preset changed
			if (tuning !== selectedTuning || instrumentPreset !== selectedPreset) {
				console.log('Config changed - tuning:', tuning, 'preset:', instrumentPreset);
				// Rebuild detectors if preset changed
				if (instrumentPreset !== selectedPreset) {
					buildDetectors(instrumentPreset);
					selectedPreset = instrumentPreset;
				}
				const t2 = TUNINGS.find(t => t.id === tuning) || TUNINGS[0];
				resetDetectors(t2.detectors);
				selectedTuning = t2.id;
				tuningObject = t2;
				resetCanvas();
				particleCents = null;
				drawScale();
			}

			if (!tuningObject.stringDetector) {
				console.error('String detector not found');
				return;
			}

			if (lastSampleTime + TIMEOUT_MS < performance.now()) {
				// no audio for a while, reset the detector
				if (!isDev) {
					resetCanvas();
					particleCents = null;
				}
				lastSampleTime = performance.now();
			}

			let start = performance.now();
			const selectedString = tuningObject.stringDetector.push(chunk);
			if (selectedString) {
				algo1_Array.push(performance.now() - start);
				if (algo1_Array.length > 10) {
					algoPerformance_1 = algo1_Array.reduce((a, b) => a + b, 0) / algo1_Array.length;
					algo1_Array = [];
				}
				if (selectedString !== selected_freq) {
					resetDetectors(tuningObject.detectors);
				}
				selected_freq = selectedString;
				//console.log(selectedString);
			}

			const detector = tuningObject.detectors.get(selected_freq);
			if (detector) {
				start = performance.now();
				const pitch = detector.detect(chunk);
				if (pitch) {
					algo2_Array.push(performance.now() - start);
					if (algo2_Array.length > 10) {
						algoPerformance_2 = algo2_Array.reduce((a, b) => a + b, 0) / algo2_Array.length;
						algo2_Array = [];
					}

					const tuningTo = pitch.tuningTo;
					const cents = tuningTo.cents;

					// Capture confidence and RMS for quality feedback
					lastConfidence = pitch.confidence ?? 1.0;
					lastRms = pitch.rms ?? 0;

					start = performance.now();
					resetCanvas();
					particleCents = cents;
					updateIndicator(tuningTo, cents);
					draw_Array.push(performance.now() - start);
					if (draw_Array.length > 10) {
						drawPerformance = draw_Array.reduce((a, b) => a + b, 0) / draw_Array.length;
						draw_Array = [];
					}
					lastSampleTime = performance.now();
				}
			}
		};
	}

	let containerRO: ResizeObserver;
	let themeMQ: MediaQueryList;

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

		// watch container size changes
		containerRO = new ResizeObserver(() => resizeCanvas());
		containerRO.observe(canvas_container);
		window.addEventListener('resize', resizeCanvas, { passive: true }); // fall back on window resize/orientation just in case

		// watch OS light/dark toggle
		themeMQ = window.matchMedia('(prefers-color-scheme: dark)');
		const handleThemeChange = () => {
			drawScale(); // repaint static layer in new colours
			resetCanvas(); // clear dynamic canvas, wait for next audio frame to arrive and redraw
		};
		themeMQ.addEventListener('change', handleThemeChange); // modern browsers
		if (!themeMQ.addEventListener) themeMQ.addListener(handleThemeChange); // legacy Safari / old Edge

		// load WebAssembly + start microphone & pitch loop
		await loadWasm();
		await run();
	});

	// tidy up if the component unmounts 
	onDestroy(() => {
		workletNode?.disconnect();
		input?.disconnect();
		audioContext?.close();
		if (devUpdateTimer) {
			clearInterval(devUpdateTimer);
		}
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
					bind:value={instrumentPreset}
					title="Select your instrument type for optimized detection">
					{#each INSTRUMENT_PRESETS as p}
						<option value={p.id}>{p.label}</option>
					{/each}
				</select>
			</label>
			<label class="tuning-label">
				<select
					class="tuning-select"
					bind:value={tuning}>
					{#each TUNINGS as t}
						<option value={t.id}>{t.label}</option>
					{/each}
				</select>
			</label>
		</div>
    </div>
	<!-- Signal quality indicator -->
	{#if lastRms > 0}
		<div class="signal-indicator" class:weak={lastRms < 0.02} class:good={lastRms >= 0.02 && lastConfidence > 0.6}>
			{#if lastRms < 0.02}
				<span class="signal-text">Weak signal - play louder</span>
			{:else if lastConfidence < 0.5}
				<span class="signal-text">Unstable - let note ring</span>
			{/if}
		</div>
	{/if}
	<div id="canvas_container">
		<PitchParticles cents={particleCents} trailStrength={0} particleCount={50} transparent/>
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
			Build&nbsp;{buildVersion}<br><br>
			Preset: {instrumentPreset}<br>
			Sample rate: {sampleRate} Hz<br>
			Confidence: {(lastConfidence * 100).toFixed(0)}%<br>
			Signal (RMS): {(lastRms * 1000).toFixed(1)}<br><br>
			Performance:<br>
			 - String Detector: {Math.trunc(algoPerformance_1)} ms<br>
			 - Pitch Detector: {Math.trunc(algoPerformance_2)} ms<br>
			 - Draw: {Math.trunc(drawPerformance)} ms
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
		text-align: left;

		background: var(--bg);
		color: var(--fg);
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
		top: 1rem;
		right: 1rem;
		display: flex;
		justify-content: flex-end;
		z-index: 100;
	}

	#controls {
		padding: 5px;
		background-color: transparent;
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
		justify-content: flex-end;
	}

	/* Signal quality indicator */
	.signal-indicator {
		position: fixed;
		top: 1rem;
		left: 50%;
		transform: translateX(-50%);
		padding: 0.4rem 0.8rem;
		border-radius: 0.3rem;
		font-size: 0.8rem;
		z-index: 100;
		opacity: 0.9;
		transition: opacity 0.3s;
	}

	.signal-indicator.weak {
		background: rgba(255, 152, 0, 0.9);
		color: #000;
	}

	.signal-indicator.good {
		opacity: 0;
	}

	.signal-text {
		font-weight: 500;
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
		background:var(--bg);
		color: var(--fg);
		font: 1.2rem/1 system-ui;
		z-index: 10000;
	}

	/* 1. Reset browser chrome so we know what we’re styling */
	.tuning-select {
		color: var(--fg);                        /* text + caret colour */
		font: inherit;
		padding: 0.45rem 2.2rem 0.45rem 0.6rem;  /* room for the caret */
		background: transparent;
		border: none;
		cursor: pointer;
		transition: border 0.2s, box-shadow 0.2s;
		text-align: right;
		text-align-last: right;
		appearance: none;           /* hide native arrow */
		-webkit-appearance: none;   /* Safari */
		position: relative;         /* anchor for ::after */

		/* override any previous background-image: */
		background-image: var(--caret-svg);
		background-repeat: no-repeat;
		background-position: right 0.7rem center;
		background-size: 0.65rem;
	}
	/* Firefox still ignores text-align on <select>.
	Trick is: flip writing direction, then flip text back. */
	@supports (-moz-appearance: none) {
		.tuning-select {
			direction: rtl;
			text-align: left;
		}
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
