<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
  
    /* --------------------------------------------------------------
       Public props
    --------------------------------------------------------------
       • `cents` (-30 … 30 or null) – tuning offset
       • `smoothingFactor` – 0‒1, higher = snappier, lower = smoother
       • `baseSpeed` – baseline orbit velocity when |cents| == MAX_CENTS
       • `speedVariance` – per‑particle speed wobble ± percentage
       • `minAlpha` / `maxAlpha` – opacity range (0‑1). Faster motion → alpha→minAlpha
       • `transparent` – if true, canvas background is transparent & trails off
    -------------------------------------------------------------- */
    export let cents: number | null = null;
    export let smoothingFactor: number = 0.08;       // ~0.05‒0.15 feels natural
    export let particleCount: number = 600;
    export let trailStrength: number = 0.06;         // ignored if transparent
    export let orbitRadius: number | null = null;
    export let baseSpeed: number = 0.025;
    export let speedVariance: number = 0.45;
    export let minAlpha: number = 0.1;               // fully transparent-ish
    export let maxAlpha: number = 0.6;               // fully opaque
    export let transparent: boolean = true;
    
    let themeMQ: MediaQueryList;

    let red = 255
    let green = 255
    let blue = 255
    /* Internal constant – full‑speed threshold */
    const MAX_CENTS = 30;
  
    /* Canvas plumbing */
    let canvas: HTMLCanvasElement;
    let ctx!: CanvasRenderingContext2D;
    let W = 0, H = 0, DPR = 1;
  
    /* Particle layout: [θ, radialOffset, personalAngVel, orbitFactor, speedMult] */
    const STRIDE = 5;
    let data!: Float32Array;
    const TWO_PI = Math.PI * 2;
    const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
  
    const ORBIT_VARIANCE_MIN = 0.8;
    const ORBIT_VARIANCE_MAX = 1.25;
  
    /* Smoothed cents value that drives the simulation */
    let smoothCents: number | null = null;
    let globalSpeedFactor = 0; // cached for draw()
  
    /* ------------------------------ resize ------------------------------ */
    function resize(): void {
      DPR = window.devicePixelRatio || 1;
      W   = canvas.clientWidth;
      H   = canvas.clientHeight;
      canvas.width  = W * DPR;
      canvas.height = H * DPR;
      ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
      if (orbitRadius === null) {
        orbitRadius = Math.min(W, H) * 0.35;
      }
    }
  
    /* ----------------------------- init -------------------------------- */
    function initParticles(): void {
      data = new Float32Array(particleCount * STRIDE);
      for (let i = 0; i < particleCount; ++i) {
        const p = i * STRIDE;
        data[p]     = Math.random() * TWO_PI;
        data[p + 1] = (Math.random() - 0.5) * 6;
        data[p + 2] = (Math.random() - 0.5) * 0.02;
        data[p + 3] = lerp(ORBIT_VARIANCE_MIN, ORBIT_VARIANCE_MAX, Math.random());
        data[p + 4] = 1 + (Math.random() * 2 - 1) * speedVariance;
      }
    }
  
    /* ------------------------ physics update --------------------------- */
    function update(): void {
      /* Smooth incoming cents */
      if (cents == null) {
        if (smoothCents == null) {
          smoothCents = 0;
        } else {
          smoothCents = lerp(smoothCents, 0, smoothingFactor);
        }
      } else {
        if (smoothCents == null) smoothCents = cents;
        smoothCents = lerp(smoothCents, cents, smoothingFactor);
      }
  
      const freeFloat = cents == null && Math.abs(smoothCents) < 0.5;
  
      const absCents = Math.abs(smoothCents ?? 0);
      const speedFactor = Math.min(absCents / MAX_CENTS, 1);
      const direction   = (smoothCents ?? 0) < 0 ? 1 : -1;
  
      globalSpeedFactor = speedFactor;
  
      for (let i = 0, p = 0; i < particleCount; ++i, p += STRIDE) {
        const radialIdx = p + 1;
        const velIdx    = p + 2;
        const multIdx   = p + 4;
  
        if (freeFloat) {
          // Calm random drift
          data[velIdx] += (Math.random() - 0.5) * 0.0005;
          data[velIdx] = Math.max(-0.003, Math.min(0.003, data[velIdx]));
          data[p] += data[velIdx];
          data[radialIdx] += (Math.random() - 0.5) * 0.05;
          continue;
        }
  
        // Tuning mode
        const particleBase = baseSpeed * data[multIdx];
        const personalTurn = data[velIdx] * 0.3;
        data[p] += speedFactor * particleBase * direction + personalTurn;
        data[velIdx] *= 0.985;
  
        const wobble = speedFactor * 0.3;
        data[radialIdx] = lerp(data[radialIdx], (Math.random() - 0.5) * wobble * 10, 0.05);
      }
    }
  
    /* ----------------------------- render ------------------------------ */
    function draw(): void {
      const freeFloatMode = cents == null && Math.abs(smoothCents ?? 0) < 0.5;
  
      // Clear or fade previous frame
      if (transparent || trailStrength === 0) {
        ctx.clearRect(0, 0, W, H);
      } else {
        ctx.fillStyle = `rgba(0,0,0,${1 - trailStrength})`;
        ctx.fillRect(0, 0, W, H);
      }
  
      const cx = W * 0.5;
      const cy = H * 0.5;
      const baseR = orbitRadius as number;
  
      const spread = cents == null ? 1 : globalSpeedFactor;
  
      // Ensure alpha bounds are sane
      const minA = Math.max(0, Math.min(1, Math.min(minAlpha, maxAlpha)));
      const maxA = Math.max(0, Math.min(1, Math.max(minAlpha, maxAlpha)));
      const alphaRange = maxA - minA;
  
      for (let i = 0, p = 0; i < particleCount; ++i, p += STRIDE) {
        const θ = data[p];
        const orbitFactor = data[p + 3];
        const blendedOrbit = 1 + spread * (orbitFactor - 1);
        const radialNoise  = data[p + 1] * spread;
        const r = baseR * blendedOrbit + radialNoise;
        const x = cx + Math.cos(θ) * r;
        const y = cy + Math.sin(θ) * r;
  
        // Speed → alpha mapping
        const angSpeed = freeFloatMode
          ? Math.abs(data[p + 2])
          : baseSpeed * data[p + 4] * globalSpeedFactor + Math.abs(data[p + 2] * 0.3);
  
        const normSpeed = Math.min(1, angSpeed / (baseSpeed * 1.2)); // 0..1
        const alpha = maxA - normSpeed * alphaRange;                 // map to range
        ctx.fillStyle = `rgba(${red},${green},${blue},${alpha.toFixed(3)})`;
        ctx.beginPath();
        ctx.arc(x, y, 1.4, 0, TWO_PI);
        ctx.fill();
      }
    }
  
    /* --------------------------- main loop ----------------------------- */
    let running = true;
    function loop(): void {
      if (!running) return;
      update();
      draw();
      requestAnimationFrame(loop);
    }
  
    /* -------------------------- lifecycle ------------------------------ */
    onMount(() => {
        themeMQ = window.matchMedia('(prefers-color-scheme: dark)');
		const handleThemeChange = () => {
			if (themeMQ.matches) {
                red = 255;
                green = 255;
                blue = 255;
            } else {
                red = 0;
                green = 0;
                blue = 0;
            }
		};
		themeMQ.addEventListener('change', handleThemeChange); // modern browsers
		if (!themeMQ.addEventListener) themeMQ.addListener(handleThemeChange); // legacy Safari / old Edge

        ctx = canvas.getContext('2d')!;
        resize();
        window.addEventListener('resize', resize);
        initParticles();
        requestAnimationFrame(loop);
    });
  
    onDestroy(() => {
      running = false;
      window.removeEventListener('resize', resize);
    });
  
    /* -------------------- reactive reallocations ----------------------- */
    $: if (data && particleCount * STRIDE !== data.length) initParticles();
  </script>
  
  <!-- Visual stage -->
  <canvas bind:this={canvas} class="w-full h-full block" style="background: transparent;"></canvas>
  
  <style>
    canvas {
      display: block;
      width: 80vw;
      height: 60svh;   /* ≈ “safe”, never pushes off-screen */
      position: relative;
      background: transparent;
    }
  </style>
  